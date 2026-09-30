// sync — keeping the local database in step with a folder the user chose.
//
// Folioo never talks to a cloud provider. The user picks a folder and moves it
// between machines with a tool of their own (rclone, Syncthing, Nextcloud,
// Dropbox…); everything here is local file I/O against that folder.

// Nothing calls into this module yet: the engine and the Tauri commands that
// drive it land in later phases of the folder-sync change. Until then the whole
// surface reads as dead code.
#![allow(dead_code)]

pub mod engine;
pub mod library;
