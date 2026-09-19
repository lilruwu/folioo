## ADDED Requirements

### Requirement: Connect And Disconnect A Google Account From Configuración
The system SHALL present a "Sincronización" section in Configuración offering "Conectar con
Google" when no account is connected, and the connected account's email address plus a
"Desconectar" action when one is. Disconnecting SHALL ask for confirmation and SHALL state that
local notes are kept.

#### Scenario: No account connected
- **WHEN** the user opens Configuración with sync unconfigured
- **THEN** the Sincronización section offers "Conectar con Google" and explains that only files created by Folioo are accessed

#### Scenario: Account connected
- **WHEN** an account is connected
- **THEN** Configuración shows that account's email and a "Desconectar" action

### Requirement: Report Sync State
The system SHALL display the current sync state in Configuración: idle with the time of the last
successful sync, in progress, or failed with a human-readable reason. A failure SHALL be shown in
Configuración rather than raised as a modal interruption while the user is writing.

#### Scenario: Sync fails because the network is down
- **WHEN** a background sync fails to reach Google
- **THEN** Configuración reports the failure and the time of the last successful sync, and the user is not interrupted

#### Scenario: A sync completes
- **WHEN** a sync finishes successfully
- **THEN** Configuración shows the sync as idle with the just-completed time as the last successful sync

### Requirement: Trigger A Sync On Demand
The system SHALL offer a "Sincronizar ahora" action, enabled only when an account is connected
and no sync is already running.

#### Scenario: Requesting a manual sync
- **WHEN** the user presses "Sincronizar ahora" with an account connected and no sync running
- **THEN** a sync starts and the section switches to the in-progress state

#### Scenario: Requesting a sync while one runs
- **WHEN** a sync is already in progress
- **THEN** the "Sincronizar ahora" action is disabled
