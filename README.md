# Slothtime TUI

A Terminal User Interface (TUI) application for time tracking, built with Rust and Ratatui. This is a simplified version of the original slothtime web app, designed for efficient time entry in the terminal.

## Features

- **Table-based Interface**: Navigate and edit time entries in a clean table format with row numbers and visual indicators
- **Keyboard Navigation**: Vim-style navigation with Tab, arrow keys, and mode switching
- **Data Persistence**: Automatic saving and loading of time entries between sessions
- **Auto-Row Creation**: Automatically adds new rows when completing entries
- **CSV Export**: Export your time entries to a configurable directory
- **Help Menu**: Built-in help showing all keybinds
- **Configurable Settings**: Customize export path and other options via `config.toml`

## Installation

### Prerequisites

- Rust (latest stable version)
- A terminal that supports ANSI escape codes

### Building

Clone the repository and build with Cargo:

```bash
git clone <repository-url>
cd slothtime-rs
cargo build --release
```

## Usage

Run the application:

```bash
cargo run
```

Or if built:

```bash
./target/release/slothtime-rs
```

### Navigation

- **Tab / Shift+Tab**: Move between columns
- **Arrow Keys**: Navigate up/down/left/right in the table
- **i**: Enter edit mode (vim-style)
- **Esc**: Exit edit mode
- **?**: Show help menu
- **Ctrl+S**: Export current entries to CSV
- **Ctrl+X**: Clear all entries
- **q**: Quit the application

### Editing

- **i**: Enter edit mode
- **Esc**: Exit edit mode
- **Tab**: Move to next column (stay in edit)
- **Enter**: Move to next row (stay in edit)
- Type to insert characters
- Backspace to delete characters

### Configuration

The app creates a `config.toml` file in the current directory on first run. You can modify it to change settings:

```toml
[export]
path = "~/Documents/slothtime_exports"
format = "csv"

[ui]
show_instructions = true
auto_save = true
```

## Export Format

Exports are saved as CSV files with the following format:

```csv
Row,Task Number,Work Code,Time Entry,Start Time,End Time,Task Time
1,PROJ-123,Development,Fixed login bug,09:00,10:30,01:30
```

Files are named `slothtime_YYYY-MM-DD.csv` and saved to the configured export directory.

## Time Format

- Supports HH:MM format (e.g., 09:30)
- Also accepts HHMM format (e.g., 0930)
- Automatically calculates task duration from start and end times

## Dependencies

- `ratatui`: For the terminal user interface
- `crossterm`: For terminal input/output handling
- `chrono`: For date and time operations
- `serde` & `toml`: For configuration file handling
- `csv`: For export functionality
- `anyhow`: For error handling

## Development

To contribute or modify the application:

1. Ensure you have Rust installed
2. Clone the repository
3. Make your changes
4. Test with `cargo test`
5. Build with `cargo build`

## License

[Add your license here]

## Acknowledgments

Inspired by the original slothtime web application. Built using the excellent Ratatui TUI framework.