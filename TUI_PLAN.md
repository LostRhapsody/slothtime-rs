# Slothtime TUI - Implementation Plan

## Overview
Create a Terminal User Interface (TUI) version of slothtime using Rust and Ratatui. This will be a simplified version focused on core time tracking functionality.

## Core Requirements
- Table-style interface with columns: Task Number, Work Code, Time Entry, Start Time, End Time
- Tab navigation between columns
- Auto-create new row when finishing the last entry
- Export current day's entries to configurable folder
- Config file for export path and other settings

## Technical Stack
- **Language**: Rust
- **TUI Framework**: Ratatui (formerly tui-rs)
- **Terminal Backend**: Crossterm
- **Config Format**: TOML (using `toml` crate)
- **Time Handling**: Chrono
- **Serialization**: Serde

## Project Structure
```
slothtime-tui/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── app.rs          # Main application state and logic
│   ├── ui.rs           # UI rendering and widgets
│   ├── config.rs       # Configuration file handling
│   ├── time_entry.rs   # Time entry data structures
│   └── export.rs       # Export functionality
├── config.toml         # Default configuration file
└── README.md
```

## Implementation Steps

### Step 1: Project Setup
1. Create new Rust project: `cargo new slothtime-tui`
2. Add dependencies to `Cargo.toml`:
   ```toml
   [dependencies]
   ratatui = "0.24"
   crossterm = "0.27"
   chrono = { version = "0.4", features = ["serde"] }
   serde = { version = "1.0", features = ["derive"] }
   serde_json = "1.0"
   toml = "0.8"
   anyhow = "1.0"
   ```
3. Set up basic main.rs with terminal initialization

### Step 2: Data Structures
1. Create `time_entry.rs`:
   - `TimeEntry` struct with fields: task_number, work_code, time_entry, start_time, end_time
   - Implement serialization/deserialization
   - Add methods for time calculation and validation

2. Create `config.rs`:
   - `Config` struct with export_path and other settings
   - Load/save from TOML file
   - Default configuration creation

### Step 3: Application State
1. Create `app.rs`:
   - `App` struct containing:
     - Vec<TimeEntry> for current entries
     - Current cursor position (row, column)
     - Current input mode (navigation vs editing)
     - Config instance
   - Implement methods:
     - `new()` - initialize app
     - `tick()` - handle time updates
     - `quit()` - cleanup and exit
     - Navigation methods (move_cursor, next_row, etc.)
     - Editing methods (insert_char, delete_char, etc.)
     - Auto-row creation logic

### Step 4: UI Components
1. Create `ui.rs`:
   - Main layout function
   - Table widget for time entries
   - Input field rendering
   - Status bar with current mode and instructions
   - Implement cursor positioning and highlighting

2. Table Structure:
   ```
   ┌─────────────┬─────────────┬─────────────────┬─────────────┬─────────────┐
   │ Task Number │ Work Code  │ Time Entry      │ Start Time  │ End Time    │
   ├─────────────┼─────────────┼─────────────────┼─────────────┼─────────────┤
   │             │             │                 │             │             │
   │             │             │                 │             │             │
   │             │             │                 │             │             │
   └─────────────┴─────────────┴─────────────────┴─────────────┴─────────────┘
   ```

### Step 5: Input Handling
1. Implement key event handling in main loop:
   - **Tab/Shift+Tab**: Navigate between columns
   - **Enter**: Move to next row or create new row
   - **Arrow Keys**: Navigate within table
   - **Character Keys**: Input text in current cell
   - **Backspace/Delete**: Edit text
   - **Ctrl+S**: Save/export
   - **Ctrl+Q**: Quit
   - **Ctrl+N**: New row

2. Input modes:
   - **Navigation**: Move cursor around table
   - **Editing**: Modify cell content
   - **Command**: Special operations (export, quit)

### Step 6: Auto-Row Creation Logic
1. When user finishes editing the last row (End Time field):
   - Validate that required fields are filled
   - Automatically create new empty row
   - Move cursor to first column of new row
   - Save current state

### Step 7: Export Functionality
1. Create `export.rs`:
   - Export to CSV format matching original web app
   - Use current date for filename
   - Write to configured export directory
   - Handle file creation errors gracefully

2. Export format:
   ```csv
   Row,Task Number,Work Code,Time Entry,Start Time,End Time,Task Time
   1,PROJ-123,Development,Fixed login bug,09:00,10:30,01:30
   2,PROJ-124,Testing,Test new features,10:45,12:00,01:15
   ```

### Step 8: Configuration Management
1. Default config file (`config.toml`):
   ```toml
   [export]
   path = "~/Documents/slothtime_exports"
   format = "csv"
   
   [ui]
   show_instructions = true
   auto_save = true
   ```

2. Config loading:
   - Check for config file in standard locations
   - Create default if not exists
   - Validate paths and create directories if needed

### Step 9: Time Handling
1. Use Chrono for time parsing and formatting
2. Support multiple input formats:
   - HH:MM (e.g., 09:30)
   - HHMM (e.g., 0930)
   - Auto-format to HH:MM
3. Calculate task time automatically from start/end times
4. Handle 12-hour vs 24-hour format preferences

### Step 10: Error Handling and Validation
1. Input validation for time fields
2. File system error handling for export
3. Graceful handling of malformed config files
4. User-friendly error messages in status bar

### Step 11: Testing and Refinement
1. Unit tests for data structures and logic
2. Integration tests for export functionality
3. Manual testing of all features
4. Performance optimization for large datasets

## Key Features to Implement

### Navigation
- Tab between columns
- Arrow keys for cell navigation
- Enter to move to next logical field
- Home/End for row navigation

### Editing
- Direct cell editing
- Auto-completion for work codes
- Time format auto-correction
- Undo/redo functionality (optional)

### Data Persistence
- Auto-save on changes
- Load previous session on startup
- Backup mechanism for data safety

### Export Options
- CSV export (primary)
- JSON export (secondary)
- Configurable filename format
- Date-based organization

## Development Milestones

1. **MVP**: Basic table display with navigation
2. **Core Functionality**: Full editing and auto-row creation
3. **Export**: Working CSV export to configured location
4. **Polish**: Error handling, validation, and user experience improvements
5. **Extras**: Additional export formats, themes, advanced features

## Potential Challenges

1. **Terminal Compatibility**: Ensure works across different terminal emulators
2. **Input Handling**: Complex key combinations and special keys
3. **Time Parsing**: Robust parsing of various time input formats
4. **File System**: Cross-platform path handling for export
5. **Performance**: Efficient rendering for large numbers of entries

## Success Criteria

- ✅ Table navigation with Tab/Enter
- ✅ Auto-row creation when finishing last entry
- ✅ CSV export to configurable directory
- ✅ Time format validation and auto-correction
- ✅ Configurable export path via TOML file
- ✅ Clean, intuitive TUI interface
- ✅ Cross-platform compatibility (Linux/macOS/Windows)

## Additional Considerations

- **Accessibility**: Keyboard-only navigation
- **Performance**: Handle 100+ entries smoothly
- **Data Safety**: Prevent data loss on crashes
- **User Experience**: Clear instructions and feedback
- **Extensibility**: Easy to add new features later

This plan provides a comprehensive roadmap for building the slothtime TUI application with all requested core functionality.