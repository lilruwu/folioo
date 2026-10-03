## ADDED Requirements

### Requirement: Choose A Sync Folder From Settings
The system SHALL present a "Sync" section in Settings. With no folder chosen, it SHALL offer
"Choose folder…", opening a native folder picker, together with a hint explaining that Folioo
keeps a copy of the notes in that folder and that the user can sync it between machines with a
tool such as rclone, Syncthing, Nextcloud or Dropbox. With a folder chosen, it SHALL show the
folder's path and let the user choose a different one.

#### Scenario: No folder chosen
- **WHEN** the user opens Settings with sync not set up
- **THEN** the Sync section offers "Choose folder…" and the hint naming tools that can sync the folder

#### Scenario: Folder chosen
- **WHEN** a sync folder is set
- **THEN** Settings shows its path and an option to choose a different folder

### Requirement: Report Sync State
The system SHALL display the current sync state in Settings: idle with the time of the last
successful sync, in progress, folder unavailable, or failed with a human-readable reason. A
problem SHALL be shown in Settings rather than raised as a modal interruption while the user is
writing.

#### Scenario: The folder is unavailable
- **WHEN** a background sync finds the sync folder unreachable or its library marker missing
- **THEN** Settings reports the folder as unavailable, with the time of the last successful sync, and the user is not interrupted

#### Scenario: A sync completes
- **WHEN** a sync finishes successfully
- **THEN** Settings shows the sync as idle with the just-completed time as the last successful sync

### Requirement: Trigger A Sync On Demand
The system SHALL offer a "Sync now" action, enabled only when a sync folder is set and no sync is
already running.

#### Scenario: Requesting a manual sync
- **WHEN** the user presses "Sync now" with a folder set and no sync running
- **THEN** a sync starts and the section switches to the in-progress state

#### Scenario: Requesting a sync while one runs
- **WHEN** a sync is already in progress
- **THEN** the "Sync now" action is disabled

### Requirement: Stop Syncing From Settings
The system SHALL offer a "Stop syncing" action when a folder is set. It SHALL ask for
confirmation, stating that local notes and the folder's contents are both kept.

#### Scenario: Stopping sync
- **WHEN** the user chooses "Stop syncing" and confirms
- **THEN** the folder is forgotten, syncing stops, and the section returns to offering "Choose folder…"
