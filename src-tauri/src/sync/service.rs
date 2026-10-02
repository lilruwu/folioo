// sync::service — when sync runs, on one background worker.
//
// * Local changes are written ~2 s after the user stops editing (a push: only
//   the changed notes, no library scan), and once more when the app closes.
// * Other machines' changes are read by a full pass on startup, every 5
//   minutes, and on demand.
//
// One worker thread runs everything, so a push and a pass never overlap, and
// no Tauri command ever waits for the folder: they only update the schedule.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::Connection;
use serde::Serialize;

use super::engine::{self, SyncError, SyncReport};
use super::library::{self, LibraryError};
use crate::db;

/// What Settings shows about sync.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct SyncStatus {
    /// The library directory in use, or `None` when sync is off.
    pub folder: Option<String>,
    /// A push or pass is running right now.
    pub running: bool,
    /// When the last push or pass succeeded.
    #[serde(rename = "lastSyncMs")]
    pub last_sync_ms: Option<i64>,
    /// False when the last attempt found the folder unmounted or missing.
    pub available: bool,
    /// What went wrong last time, in words for the user. Cleared by success.
    pub error: Option<String>,
}

/// Something the UI should hear about.
#[derive(Debug, Clone, PartialEq)]
pub enum SyncEvent {
    Status(SyncStatus),
    /// Sync changed notes or tags here: the lists need reloading.
    LocalDataChanged,
}

#[derive(Debug, Clone, Copy)]
pub struct Timing {
    /// Quiet time after the last edit before local changes are written.
    pub debounce: Duration,
    /// Interval between full passes while the app is open.
    pub period: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            debounce: Duration::from_secs(2),
            period: Duration::from_secs(5 * 60),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Job {
    /// Write local changes only.
    Push,
    /// Full pass: read other machines' changes and write ours.
    Pass,
}

#[derive(Default)]
struct Schedule {
    /// When the user last changed something not yet written.
    last_change: Option<Instant>,
    pass_requested: bool,
    shutting_down: bool,
    /// The worker has taken a job and not finished it yet.
    busy: bool,
}

struct Shared {
    db: Arc<Mutex<Connection>>,
    schedule: Mutex<Schedule>,
    wake: Condvar,
    status: Mutex<SyncStatus>,
    /// Held for the duration of any push or pass.
    running: Mutex<()>,
    notify: Box<dyn Fn(SyncEvent) + Send + Sync>,
}

pub struct SyncService {
    shared: Arc<Shared>,
}

impl SyncService {
    /// Start the worker. A full pass runs straight away if a folder is set.
    pub fn start(
        db: Arc<Mutex<Connection>>,
        timing: Timing,
        notify: impl Fn(SyncEvent) + Send + Sync + 'static,
    ) -> Self {
        let (folder, last_sync_ms) = {
            let conn = db.lock().expect("database available at startup");
            (
                db::sync_folder(&conn).ok().flatten(),
                db::last_sync_ms(&conn).ok().flatten(),
            )
        };
        let shared = Arc::new(Shared {
            db,
            schedule: Mutex::new(Schedule {
                pass_requested: folder.is_some(),
                ..Schedule::default()
            }),
            wake: Condvar::new(),
            status: Mutex::new(SyncStatus {
                folder,
                last_sync_ms,
                available: true,
                ..SyncStatus::default()
            }),
            running: Mutex::new(()),
            notify: Box::new(notify),
        });

        let worker = Arc::clone(&shared);
        thread::Builder::new()
            .name("folioo-sync".into())
            .spawn(move || worker.run_worker(timing))
            .expect("could not start the sync worker");

        Self { shared }
    }

    /// Nothing scheduled and nothing running: every change made so far has been
    /// handled. Tests wait on this instead of guessing at timings.
    #[cfg(test)]
    fn settled(&self) -> bool {
        self.shared
            .schedule
            .lock()
            .map(|s| !s.busy && s.last_change.is_none() && !s.pass_requested)
            .unwrap_or(false)
    }

    pub fn status(&self) -> SyncStatus {
        self.shared.status.lock().map(|s| s.clone()).unwrap_or_default()
    }

    /// The user changed something; write it once they pause.
    pub fn local_change(&self) {
        if let Ok(mut s) = self.shared.schedule.lock() {
            s.last_change = Some(Instant::now());
        }
        self.shared.wake.notify_all();
    }

    /// "Sync now": a full pass as soon as the worker is free.
    pub fn sync_now(&self) {
        if let Ok(mut s) = self.shared.schedule.lock() {
            s.pass_requested = true;
        }
        self.shared.wake.notify_all();
    }

    /// Point sync at the folder the user chose: create a library there (or
    /// adopt the one it already holds) and start a first full pass.
    pub fn set_folder(&self, chosen: &Path) -> Result<SyncStatus, String> {
        let dir = library::resolve_library(chosen);
        let path = dir
            .to_str()
            .ok_or("Folioo can't use a folder whose path isn't valid text")?
            .to_string();
        library::init_library(&dir).map_err(|e| e.to_string())?;

        {
            // Never switch folders under a running pass.
            let _running = self.shared.running.lock().map_err(|_| "Sync is unavailable")?;
            let conn = self.shared.db.lock().map_err(|_| "The database is unavailable")?;
            if db::sync_folder(&conn).map_err(|e| e.to_string())?.as_deref() != Some(&path) {
                // What was agreed with another library says nothing about this one.
                db::clear_sync(&conn).map_err(|e| e.to_string())?;
                db::set_sync_folder(&conn, &path).map_err(|e| e.to_string())?;
            }
        }

        self.shared.update_status(|s| {
            *s = SyncStatus {
                folder: Some(path.clone()),
                available: true,
                ..SyncStatus::default()
            }
        });
        self.sync_now();
        Ok(self.status())
    }

    /// Stop syncing. Local notes and the folder's contents are left as they are.
    pub fn stop(&self) -> Result<SyncStatus, String> {
        {
            let _running = self.shared.running.lock().map_err(|_| "Sync is unavailable")?;
            let conn = self.shared.db.lock().map_err(|_| "The database is unavailable")?;
            db::clear_sync(&conn).map_err(|e| e.to_string())?;
        }
        if let Ok(mut s) = self.shared.schedule.lock() {
            s.last_change = None;
            s.pass_requested = false;
        }
        self.shared.update_status(|s| {
            *s = SyncStatus {
                available: true,
                ..SyncStatus::default()
            }
        });
        Ok(self.status())
    }

    /// Write anything still pending, now, and stop the worker. For app exit:
    /// waits for a push or pass already running, then pushes — milliseconds
    /// unless the folder is slow, and a no-op when nothing is pending.
    pub fn flush_and_stop(&self) {
        if let Ok(mut s) = self.shared.schedule.lock() {
            s.shutting_down = true;
            s.last_change = None;
        }
        self.shared.wake.notify_all();
        self.shared.run_job(Job::Push);
    }
}

impl Shared {
    fn update_status(&self, f: impl FnOnce(&mut SyncStatus)) {
        let snapshot = match self.status.lock() {
            Ok(mut s) => {
                f(&mut s);
                s.clone()
            }
            Err(_) => return,
        };
        (self.notify)(SyncEvent::Status(snapshot));
    }

    fn run_worker(self: Arc<Self>, timing: Timing) {
        let mut next_pass = Instant::now() + timing.period;
        loop {
            let job = {
                let Ok(mut s) = self.schedule.lock() else { return };
                loop {
                    if s.shutting_down {
                        return;
                    }
                    let now = Instant::now();
                    if s.pass_requested || now >= next_pass {
                        s.pass_requested = false;
                        // A full pass writes local changes too.
                        s.last_change = None;
                        s.busy = true;
                        break Job::Pass;
                    }
                    let push_at = s.last_change.map(|t| t + timing.debounce);
                    if push_at.is_some_and(|at| now >= at) {
                        s.last_change = None;
                        s.busy = true;
                        break Job::Push;
                    }
                    let until = push_at.map_or(next_pass, |at| at.min(next_pass));
                    let Ok((guard, _)) = self.wake.wait_timeout(s, until.saturating_duration_since(now)) else { return };
                    s = guard;
                }
            };
            if job == Job::Pass {
                next_pass = Instant::now() + timing.period;
            }
            self.run_job(job);
            if let Ok(mut s) = self.schedule.lock() {
                s.busy = false;
            }
        }
    }

    fn folder(&self) -> Option<PathBuf> {
        let conn = self.db.lock().ok()?;
        db::sync_folder(&conn).ok().flatten().map(PathBuf::from)
    }

    fn run_job(&self, job: Job) {
        let Ok(_running) = self.running.lock() else { return };
        let Some(folder) = self.folder() else { return };

        self.update_status(|s| s.running = true);
        let now = crate::now_ms();
        let outcome = match job {
            Job::Push => engine::push_pending(&self.db, &folder, now),
            Job::Pass => engine::run(&self.db, &folder, now),
        };
        self.finish(outcome, now);
    }

    fn finish(&self, outcome: Result<SyncReport, SyncError>, now: i64) {
        match outcome {
            Ok(report) => {
                if let Ok(conn) = self.db.lock() {
                    let _ = db::set_last_sync_ms(&conn, now);
                }
                let error = match report.errors.len() {
                    0 => None,
                    1 => Some(format!("1 item could not be synced: {}", report.errors[0])),
                    n => Some(format!("{n} items could not be synced; first: {}", report.errors[0])),
                };
                self.update_status(|s| {
                    s.running = false;
                    s.available = true;
                    s.last_sync_ms = Some(now);
                    s.error = error;
                });
                if report.changed_locally() {
                    (self.notify)(SyncEvent::LocalDataChanged);
                }
            }
            Err(e) => {
                let unavailable = matches!(e, SyncError::Library(LibraryError::Unavailable));
                let text = e.to_string();
                self.update_status(|s| {
                    s.running = false;
                    s.available = !unavailable;
                    s.error = Some(text);
                });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::library::{init_library, NoteFile};
    use crate::Note;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "folioo-service-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn database() -> Arc<Mutex<Connection>> {
        let conn = Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        db::insert_folder(&conn, "Personal", "#000", 1).unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn note(id: &str, title: &str) -> Note {
        Note {
            id: id.into(),
            title: title.into(),
            content: format!("<div>{title}</div>"),
            folder: "Personal".into(),
            favorite: false,
            created: "2026-10-01".into(),
            updated: "2026-10-01".into(),
            deleted_at: None,
        }
    }

    /// A service plus every event it sent.
    fn service(db: &Arc<Mutex<Connection>>, timing: Timing) -> (SyncService, Arc<Mutex<Vec<SyncEvent>>>) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&events);
        let svc = SyncService::start(Arc::clone(db), timing, move |e| sink.lock().unwrap().push(e));
        (svc, events)
    }

    fn fast() -> Timing {
        Timing {
            debounce: Duration::from_millis(50),
            period: Duration::from_secs(3600),
        }
    }

    /// Poll until `cond` holds, for up to 5 s.
    fn eventually(what: &str, cond: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if cond() {
                return;
            }
            thread::sleep(Duration::from_millis(10));
        }
        panic!("timed out waiting for: {what}");
    }

    fn idle(svc: &SyncService) -> bool {
        let s = svc.status();
        !s.running && s.last_sync_ms.is_some()
    }

    #[test]
    fn choosing_a_folder_creates_a_library_and_writes_existing_notes() {
        let db = database();
        db::insert_note(&db.lock().unwrap(), &note("n1", "Existing"), 10).unwrap();
        let (svc, _) = service(&db, fast());
        let chosen = TempDir::new();

        let status = svc.set_folder(&chosen.0).unwrap();

        let lib = chosen.0.join("Folioo");
        assert_eq!(status.folder.as_deref(), lib.to_str());
        eventually("the first pass writes the note", || lib.join("notes/n1.json").exists());
    }

    #[test]
    fn an_edit_is_written_shortly_after_typing_stops() {
        let db = database();
        db::insert_note(&db.lock().unwrap(), &note("n1", "Draft"), 10).unwrap();
        let (svc, _) = service(&db, fast());
        let chosen = TempDir::new();
        svc.set_folder(&chosen.0).unwrap();
        eventually("first pass", || idle(&svc));

        db::update_note(&db.lock().unwrap(), "n1", "Final", "<div>Final</div>", "2026-10-01", 20).unwrap();
        svc.local_change();

        let file = chosen.0.join("Folioo/notes/n1.json");
        eventually("the edit reaches the folder", || {
            fs::read_to_string(&file).is_ok_and(|raw| raw.contains("Final"))
        });
    }

    #[test]
    fn closing_writes_a_pending_edit_without_waiting() {
        let db = database();
        db::insert_note(&db.lock().unwrap(), &note("n1", "Draft"), 10).unwrap();
        // A debounce long enough that only the flush can be what writes it.
        let (svc, _) = service(
            &db,
            Timing {
                debounce: Duration::from_secs(3600),
                period: Duration::from_secs(3600),
            },
        );
        let chosen = TempDir::new();
        svc.set_folder(&chosen.0).unwrap();
        eventually("first pass", || idle(&svc));

        db::update_note(&db.lock().unwrap(), "n1", "Typed just now", "<div>x</div>", "2026-10-01", 20)
            .unwrap();
        svc.local_change();
        svc.flush_and_stop();

        let raw = fs::read_to_string(chosen.0.join("Folioo/notes/n1.json")).unwrap();
        assert!(raw.contains("Typed just now"), "{raw}");
    }

    #[test]
    fn sync_now_reads_another_machines_note_and_says_so() {
        let db = database();
        let (svc, events) = service(&db, fast());
        let chosen = TempDir::new();
        svc.set_folder(&chosen.0).unwrap();
        eventually("first pass", || idle(&svc));

        // Another machine drops a note into the shared library.
        let lib = init_library(&chosen.0.join("Folioo")).unwrap();
        lib.write_note(&NoteFile { note: note("n7", "From elsewhere"), updated_ms: 50 })
            .unwrap();
        svc.sync_now();

        eventually("the note arrives", || {
            db::get_note(&db.lock().unwrap(), "n7").unwrap().is_some()
        });
        eventually("the UI is told to reload", || {
            events.lock().unwrap().contains(&SyncEvent::LocalDataChanged)
        });
    }

    #[test]
    fn an_unmounted_folder_is_reported_and_nothing_is_lost() {
        let db = database();
        db::insert_note(&db.lock().unwrap(), &note("n1", "Precious"), 10).unwrap();
        let (svc, _) = service(&db, fast());
        let chosen = TempDir::new();
        svc.set_folder(&chosen.0).unwrap();
        eventually("first pass", || idle(&svc));

        // The mount goes away: the directory is still there, but empty.
        fs::remove_dir_all(chosen.0.join("Folioo")).unwrap();
        fs::create_dir(chosen.0.join("Folioo")).unwrap();
        svc.sync_now();

        eventually("the folder is reported unavailable", || !svc.status().available);
        assert!(svc.status().error.is_some());
        assert!(db::get_note(&db.lock().unwrap(), "n1").unwrap().is_some());
        assert_eq!(fs::read_dir(chosen.0.join("Folioo")).unwrap().count(), 0, "nothing recreated");
    }

    #[test]
    fn stopping_sync_leaves_notes_and_folder_alone() {
        let db = database();
        db::insert_note(&db.lock().unwrap(), &note("n1", "Mine"), 10).unwrap();
        let (svc, _) = service(&db, fast());
        let chosen = TempDir::new();
        svc.set_folder(&chosen.0).unwrap();
        eventually("first pass", || idle(&svc));

        let status = svc.stop().unwrap();
        db::update_note(&db.lock().unwrap(), "n1", "After stop", "<div>x</div>", "2026-10-01", 20).unwrap();
        svc.local_change();
        thread::sleep(Duration::from_millis(200));

        assert_eq!(status.folder, None);
        assert!(db::get_note(&db.lock().unwrap(), "n1").unwrap().is_some());
        let raw = fs::read_to_string(chosen.0.join("Folioo/notes/n1.json")).unwrap();
        assert!(!raw.contains("After stop"), "nothing is written once sync is off");
    }

    // ── End to end: two machines sharing one folder ─────────────────────────

    /// One installation: its own database and sync service, as in the app.
    struct Machine {
        db: Arc<Mutex<Connection>>,
        svc: SyncService,
    }

    impl Machine {
        fn new(folder: &Path) -> Self {
            let db = database();
            let (svc, _) = service(&db, fast());
            svc.set_folder(folder).unwrap();
            let m = Self { db, svc };
            m.wait_idle();
            m
        }

        fn wait_idle(&self) {
            eventually("sync to finish", || idle(&self.svc));
        }

        /// A user action: change the database, then tell sync, as the
        /// commands in lib.rs do.
        fn act(&self, f: impl FnOnce(&Connection)) {
            f(&self.db.lock().unwrap());
            self.svc.local_change();
        }

        fn note(&self, id: &str) -> Option<Note> {
            db::get_note(&self.db.lock().unwrap(), id).unwrap()
        }

        fn tags(&self) -> Vec<String> {
            db::list_folders(&self.db.lock().unwrap()).unwrap().into_iter().map(|f| f.name).collect()
        }

        /// Wait until this machine's push has reached the folder, then have
        /// `other` read it with a full pass, and wait for `cond` on `other`.
        fn hand_over(&self, other: &Machine, what: &str, cond: impl Fn(&Machine) -> bool) {
            eventually("the push to be written", || self.svc.settled());
            other.svc.sync_now();
            eventually(what, || cond(other));
        }
    }

    #[test]
    fn two_machines_converge_through_one_folder() {
        let shared = TempDir::new();
        let a = Machine::new(&shared.0);
        let b = Machine::new(&shared.0);
        let personal = |m: &Machine| m.tags().contains(&"Personal".to_string());
        assert!(personal(&a) && personal(&b));

        // Create on A.
        a.act(|c| db::insert_note(c, &note("n1", "Shopping"), crate::now_ms()).unwrap());
        a.hand_over(&b, "B sees A's new note", |m| m.note("n1").is_some());

        // Edit on B.
        b.act(|c| {
            db::update_note(c, "n1", "Shopping list", "<div>milk</div>", "2026-10-02", crate::now_ms())
                .unwrap()
        });
        b.hand_over(&a, "A sees B's edit", |m| {
            m.note("n1").is_some_and(|n| n.title == "Shopping list")
        });

        // Trash on A, restore on B.
        a.act(|c| db::trash_note(c, "n1", "2026-10-02", crate::now_ms()).unwrap());
        a.hand_over(&b, "B sees it trashed", |m| {
            m.note("n1").is_some_and(|n| n.deleted_at.is_some())
        });
        b.act(|c| db::restore_note(c, "n1", crate::now_ms()).unwrap());
        b.hand_over(&a, "A sees it restored", |m| {
            m.note("n1").is_some_and(|n| n.deleted_at.is_none())
        });

        // A new tag and a retag on A.
        a.act(|c| {
            db::insert_folder(c, "Travel", "#2BB0A6", crate::now_ms()).unwrap();
            db::update_note_folder(c, "n1", "Travel", "2026-10-02", crate::now_ms()).unwrap();
        });
        a.hand_over(&b, "B sees the tag and the retag", |m| {
            m.tags().contains(&"Travel".to_string())
                && m.note("n1").is_some_and(|n| n.folder == "Travel")
        });

        // Purge on B.
        b.act(|c| db::purge_note(c, "n1", crate::now_ms()).unwrap());
        b.hand_over(&a, "A loses the purged note", |m| m.note("n1").is_none());

        // Tag deletion on A.
        a.act(|c| db::delete_folder(c, "Travel", "Personal", crate::now_ms()).unwrap());
        a.hand_over(&b, "B loses the deleted tag", |m| !m.tags().contains(&"Travel".to_string()));

        // And nothing comes back on later passes, on either side.
        for m in [&a, &b] {
            m.svc.sync_now();
            thread::sleep(Duration::from_millis(100));
            m.wait_idle();
        }
        for m in [&a, &b] {
            assert!(m.note("n1").is_none());
            assert_eq!(m.tags(), ["Personal"]);
        }
    }

    #[test]
    fn work_done_while_the_folder_is_unreachable_is_written_once_it_returns() {
        let shared = TempDir::new();
        let a = Machine::new(&shared.0);
        a.act(|c| db::insert_note(c, &note("n1", "Before"), crate::now_ms()).unwrap());
        eventually("the first note is written", || shared.0.join("Folioo/notes/n1.json").exists());
        a.wait_idle();

        // The drive holding the folder goes away.
        let away = shared.0.join("Folioo-unplugged");
        fs::rename(shared.0.join("Folioo"), &away).unwrap();
        a.act(|c| {
            db::insert_note(c, &note("n2", "Written offline"), crate::now_ms()).unwrap();
            db::update_note(c, "n1", "Edited offline", "<div>x</div>", "2026-10-02", crate::now_ms())
                .unwrap();
        });
        eventually("the folder is reported unavailable", || !a.svc.status().available);
        assert!(a.note("n2").is_some(), "the app keeps working locally");

        // It comes back.
        fs::rename(&away, shared.0.join("Folioo")).unwrap();
        a.svc.sync_now();

        let notes = shared.0.join("Folioo/notes");
        eventually("the offline work reaches the folder", || {
            notes.join("n2.json").exists()
                && fs::read_to_string(notes.join("n1.json")).is_ok_and(|r| r.contains("Edited offline"))
        });
        eventually("the status recovers", || a.svc.status().available);
    }

    #[test]
    fn joining_another_machines_library_merges_and_deletes_nothing() {
        let shared = TempDir::new();
        let a = Machine::new(&shared.0);
        a.act(|c| db::insert_note(c, &note("n1", "From A"), crate::now_ms()).unwrap());
        eventually("A's note is written", || shared.0.join("Folioo/notes/n1.json").exists());

        // B already has notes and a tag of its own before it ever syncs.
        let db_b = database();
        {
            let c = db_b.lock().unwrap();
            db::insert_note(&c, &note("n2", "From B"), 10).unwrap();
            db::insert_folder(&c, "Recipes", "#E86BB0", 10).unwrap();
        }
        let (svc_b, _) = service(&db_b, fast());
        svc_b.set_folder(&shared.0.join("Folioo")).unwrap();
        let b = Machine { db: db_b, svc: svc_b };
        b.wait_idle();
        b.hand_over(&a, "A gets B's note", |m| m.note("n2").is_some());

        for m in [&a, &b] {
            assert!(m.note("n1").is_some() && m.note("n2").is_some());
            assert!(m.tags().contains(&"Recipes".to_string()), "{:?}", m.tags());
        }
    }

    #[test]
    fn a_folder_already_holding_a_library_is_adopted() {
        let db = database();
        let (svc, _) = service(&db, fast());
        let chosen = TempDir::new();
        let lib = init_library(&chosen.0.join("Folioo")).unwrap();
        lib.write_note(&NoteFile { note: note("n5", "Theirs"), updated_ms: 5 }).unwrap();

        // The user picks the Folioo folder itself.
        let status = svc.set_folder(&chosen.0.join("Folioo")).unwrap();

        assert_eq!(status.folder.as_deref(), chosen.0.join("Folioo").to_str());
        eventually("the existing note is read", || {
            db::get_note(&db.lock().unwrap(), "n5").unwrap().is_some()
        });
        assert!(!chosen.0.join("Folioo/Folioo").exists(), "no nested library");
    }
}
