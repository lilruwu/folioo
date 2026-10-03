## ADDED Requirements

### Requirement: Inline Formatting
The system SHALL support bold, italic, and underline on the current selection, toggled either
from toolbar buttons or Ctrl+B / Ctrl+I / Ctrl+U, and SHALL reflect the active state of the
formatting under the caret in the toolbar.

#### Scenario: Toggling bold with a shortcut
- **WHEN** the user selects text and presses Ctrl+B
- **THEN** the selection becomes bold and the toolbar's "B" button shows as active while the caret remains inside it

### Requirement: Block Formatting
The system SHALL support setting the current line to Heading 1, Heading 2, Heading 3,
blockquote, or a code block from the toolbar, applied via direct DOM manipulation rather than
`execCommand("formatBlock")`. Clicking the active block's button again SHALL revert the line to
a normal paragraph.

#### Scenario: Applying a heading
- **WHEN** the user places the caret on a line and clicks "H1"
- **THEN** that line becomes an `<h1>` and the toolbar shows H1 as active

#### Scenario: Toggling a heading off
- **WHEN** the user clicks "H1" again while the caret is on an existing H1 line
- **THEN** the line reverts to a plain paragraph

### Requirement: Exit Heading Or Quote Style On Enter
The system SHALL drop back to a normal paragraph when Enter is pressed at the very end of an
otherwise-empty-after-caret heading (H1/H2/H3) or blockquote line, instead of continuing the
same style onto the new line.

#### Scenario: Pressing Enter at the end of a heading
- **WHEN** the caret is at the end of an H2 line with nothing after it and the user presses Enter
- **THEN** a new plain paragraph line is created below it, and typing continues as normal text

### Requirement: Markdown Shortcuts For Block Formatting
The system SHALL convert a line to the corresponding block format when the user types a
recognized Markdown prefix followed by a space at the very start of the line: `#`/`##`/`###` to
headings, `-`/`*` to a bulleted list, `1.` to a numbered list, `>` to a blockquote, and
`[]`/`[ ]` to a checklist item. The triggering prefix SHALL be removed.

#### Scenario: Typing a heading shortcut
- **WHEN** the user types `## ` at the start of an empty line
- **THEN** the `##` prefix is removed and the line becomes an `<h2>`

#### Scenario: Typing a checklist shortcut
- **WHEN** the user types `[] ` at the start of a line
- **THEN** the line becomes a to-do item

### Requirement: Inline Markdown Shortcuts
The system SHALL convert `**text**`, `*text*`, `_text_`, and `` `text` `` immediately preceding
the caret into bold, italic, italic, and code respectively when the user presses Space, as long
as the pattern is fully closed.

#### Scenario: Typing inline bold
- **WHEN** the user types `**important**` followed by a space
- **THEN** "important" becomes bold text and the markdown markers are removed

### Requirement: Undo And Redo History
The system SHALL maintain a snapshot-based undo/redo history of the note body (not the browser's
native undo), triggered by Ctrl+Z (undo) and Ctrl+Y or Ctrl+Shift+Z (redo). A new snapshot SHALL
be recorded whenever a save is committed. The history SHALL be capped (120 entries or ~8,000,000
characters of combined HTML, whichever comes first) while always retaining the current state
plus at least one prior step.

#### Scenario: Undoing an edit
- **WHEN** the user presses Ctrl+Z after making an edit
- **THEN** the note body reverts to its state before that edit, the caret position is restored, and the reverted state is saved

#### Scenario: Switching notes resets history
- **WHEN** the user selects a different note
- **THEN** the undo/redo history is reset to a single snapshot of that note's loaded content

### Requirement: Title Field Is Single-Line
The system SHALL treat the title as a single line: pressing Enter while focused in the title
SHALL move focus to the note body instead of inserting a line break.

#### Scenario: Pressing Enter in the title
- **WHEN** the user presses Enter while the caret is in the title field
- **THEN** no newline is inserted and focus moves to the note content area

### Requirement: Structural Normalization Of Loose Content
The system SHALL wrap any inline content or text nodes sitting directly under the editor root
(not inside a block element) into a `<div>` after edits, to prevent the webview from deleting
unrelated content when an adjacent list or block is removed.

#### Scenario: Loose text after a paste
- **WHEN** an edit leaves bare text nodes as direct children of the editor root
- **THEN** they are wrapped into a single `<div>` on the next save, preserving the caret position
