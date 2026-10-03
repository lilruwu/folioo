# media-embeds Specification

## Purpose
TBD - created by archiving change baseline-capture-existing-app. Update Purpose after archive.
## Requirements
### Requirement: Insert An Image From The File Picker
The system SHALL let the user pick an image file via a toolbar button, which opens a native
file picker restricted to image types, and embed the chosen file into the note.

#### Scenario: Inserting via the toolbar
- **WHEN** the user clicks the image toolbar button and selects an image file
- **THEN** the image is embedded into the note at the caret position

### Requirement: Insert An Image From Paste
The system SHALL embed an image found on the clipboard when the user pastes inside the note
body. It SHALL first check the DOM clipboard (files, then typed items) and, if no image is
found there, fall back to reading the system clipboard directly (needed because WebKitGTK often
doesn't expose pasted screenshots through the DOM paste event).

#### Scenario: Pasting a screenshot
- **WHEN** the user copies a screenshot to the system clipboard and pastes inside the note body
- **THEN** the image is embedded into the note even if the webview's paste event carries no image data

#### Scenario: Pasting text
- **WHEN** the clipboard holds plain text or HTML text (no image)
- **THEN** the paste is not intercepted and the webview's normal text paste behavior applies

### Requirement: Insert An Image By Drag And Drop
The system SHALL embed an image file dropped onto the note body, unless a to-do item is
currently being dragged for reordering (which takes precedence).

#### Scenario: Dropping an image file
- **WHEN** the user drags an image file from outside the app and drops it onto the note body
- **THEN** the image is embedded at the drop location's default insertion point

### Requirement: Downscale Large Images
The system SHALL downscale any embedded image whose largest dimension exceeds 1600px, so it fits
within 1600px on its longest side, re-encoding it as PNG (or JPEG at 0.85 quality for non-PNG
sources) before embedding it as a data URL.

#### Scenario: Embedding a large photo
- **WHEN** the user embeds an image larger than 1600px on its longest side
- **THEN** it is downscaled to fit 1600px before being embedded, keeping its aspect ratio

### Requirement: Insert A Hand-Drawn Sketch
The system SHALL provide a drawing canvas (pen colors including the current theme's accent color
plus five fixed colors, three stroke sizes, an eraser, undo up to 25 steps, and a clear-all
action) whose result is inserted into the note as a PNG image marked as a sketch.

#### Scenario: Drawing and inserting a sketch
- **WHEN** the user opens the draw tool, sketches something, and clicks "Insertar"
- **THEN** the sketch is embedded into the note as an image at the position the draw tool was opened from

### Requirement: Re-Edit An Existing Sketch
The system SHALL reopen the draw canvas, pre-painted with the existing image, when the user
clicks a previously inserted sketch, and SHALL update that same image in place when saved (not
insert a new one).

#### Scenario: Editing a previously inserted sketch
- **WHEN** the user clicks an image that was inserted via the draw tool
- **THEN** the draw canvas opens pre-loaded with that sketch, and saving replaces its pixels without creating a duplicate image

