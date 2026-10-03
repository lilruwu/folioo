## ADDED Requirements

### Requirement: Insert A To-Do Item
The system SHALL insert a new checklist item from the toolbar, placed immediately after the
current to-do item if the caret is inside one, or at the caret position otherwise, with focus
placed in the new item's text.

#### Scenario: Inserting the first to-do in a note
- **WHEN** the user clicks the checklist toolbar button with the caret on a normal line
- **THEN** a new, empty to-do item is inserted at the caret and focused

### Requirement: Toggle Completion
The system SHALL toggle a to-do item's completed state when its checkbox circle is clicked.

#### Scenario: Marking an item done
- **WHEN** the user clicks the circle next to an incomplete to-do item
- **THEN** the item is marked done and its state is saved

### Requirement: Continue Or Exit The Checklist On Enter
The system SHALL, on Enter inside a non-empty to-do item, insert a new empty to-do item
immediately after it and focus it. On Enter inside an empty to-do item, it SHALL instead replace
that item with a normal empty paragraph and move focus there, exiting the checklist.

#### Scenario: Continuing the list
- **WHEN** the user presses Enter at the end of a to-do item containing text
- **THEN** a new empty to-do item is created below it and focused

#### Scenario: Exiting the list from an empty item
- **WHEN** the user presses Enter on a to-do item with no text
- **THEN** that item becomes a normal paragraph and the checklist ends there

### Requirement: Revert Or Merge On Backspace
The system SHALL, on Backspace at the very start of a to-do item's text, convert that item back
into a normal paragraph. On Backspace at the very start of a normal line that directly follows a
to-do item, it SHALL instead merge that line's content into the end of the preceding to-do item.

#### Scenario: Backspacing at the start of a to-do
- **WHEN** the caret is at the very start of a to-do item's text and the user presses Backspace
- **THEN** the item becomes a normal paragraph, preserving its text

#### Scenario: Backspacing into a preceding to-do
- **WHEN** the caret is at the start of a plain line that comes right after a to-do item and the user presses Backspace
- **THEN** the line's content is appended to the to-do item's text and the plain line is removed

### Requirement: Reorder Items By Dragging
The system SHALL let the user reorder to-do items by dragging a dedicated handle on each item and
dropping it above or below another item, determined by which half of the target item the
pointer is over when dropped.

#### Scenario: Dragging an item below another
- **WHEN** the user drags a to-do item's handle and drops it on the lower half of another item
- **THEN** the dragged item is placed immediately after the target item

### Requirement: Markdown Shortcut For Checklists
The system SHALL convert a line to a checklist item when the user types `[]` or `[ ]` followed
by a space at the start of the line.

#### Scenario: Typing the checklist shortcut
- **WHEN** the user types `[] ` at the start of an empty line
- **THEN** the line becomes a to-do item
