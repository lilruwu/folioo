// sync — keeping the local database in step with a folder the user chose.
//
// Folioo never talks to a cloud provider. The user picks a folder and moves it
// between machines with a tool of their own (rclone, Syncthing, Nextcloud,
// Dropbox…); everything here is local file I/O against that folder.

pub mod engine;
pub mod library;
pub mod service;

pub use service::{SyncEvent, SyncService, SyncStatus, Timing};
