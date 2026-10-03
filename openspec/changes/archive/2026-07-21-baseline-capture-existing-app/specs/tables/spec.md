## ADDED Requirements

### Requirement: Insert A Table
The system SHALL insert a 2x2 table after the current line when the toolbar's table button is
clicked, place the caret in its first cell, and add an empty line after the table so the caret
can move past it.

#### Scenario: Inserting a table
- **WHEN** the user clicks the table toolbar button
- **THEN** a 2-row, 2-column table is inserted after the current line with the caret in its first cell

### Requirement: Navigate Cells With Tab
The system SHALL move the caret to the next cell on Tab and the previous cell on Shift+Tab.
Pressing Tab in the last cell SHALL add a new row and move the caret into its first cell instead
of leaving the table.

#### Scenario: Tabbing past the last cell
- **WHEN** the caret is in the last cell of a table and the user presses Tab
- **THEN** a new row is appended and the caret moves to its first cell

### Requirement: Add Rows And Columns From In-Table Handles
The system SHALL show a "+" handle to append a row below the table and a "+" handle to append a
column to its right, visible only while the caret is inside that table.

#### Scenario: Adding a column
- **WHEN** the caret is in a table and the user clicks the column "+" handle
- **THEN** a new empty cell is appended to every row

### Requirement: Delete Rows And Columns From In-Table Handles
The system SHALL show a "−" handle above each column and beside each row to delete it. Deleting
the last remaining row or column SHALL delete the whole table instead of leaving an empty one.

#### Scenario: Deleting the only remaining row
- **WHEN** a table has a single row and the user clicks its row "−" handle
- **THEN** the entire table is removed

### Requirement: Align A Table Column
The system SHALL apply left/center/right text alignment to every cell in the caret's current
column when one of the alignment toolbar buttons (shown only while inside a table) is clicked.

#### Scenario: Center-aligning a column
- **WHEN** the caret is in a cell and the user clicks the center-align button
- **THEN** every cell in that column is set to center-aligned text

### Requirement: Paste Tabular Data Into A Table
The system SHALL, when pasting an HTML table or tab-separated text while the caret is in a
table cell and the pasted data spans more than one cell, distribute the values across cells
starting at the target cell, preserving their relative row/column positions and growing the
table with extra rows/columns if needed. Pasting a single value, or pasting outside a table,
SHALL insert flattened plain text instead.

#### Scenario: Pasting a spreadsheet selection into a cell
- **WHEN** the user pastes a 3x2 block of spreadsheet cells starting at a target cell
- **THEN** the table grows if necessary and the six values land in their corresponding cells relative to the target

### Requirement: Prevent Cross-Cell Backspace Merging
The system SHALL prevent Backspace at the start of a cell's content from merging into the
previous cell. If every cell in the table is empty when this happens, the whole table SHALL be
deleted instead.

#### Scenario: Backspacing at the start of an empty table
- **WHEN** every cell in a table is empty and the user presses Backspace at the start of a cell
- **THEN** the table is removed entirely rather than merging cells

### Requirement: Delete The Whole Table
The system SHALL provide a toolbar action, visible only while the caret is inside a table, that
removes the entire table.

#### Scenario: Deleting a table explicitly
- **WHEN** the caret is inside a table and the user clicks "Borrar tabla"
- **THEN** the table is removed and the caret moves to the adjacent line
