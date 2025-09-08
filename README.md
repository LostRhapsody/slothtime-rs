# Slothtime TUI

A Terminal User Interface (TUI) application for time tracking, built with Rust and Ratatui. This is a simplified version of the original slothtime web app, designed for efficient time entry in the terminal.

[Original Slothtime Web App)[https://github.com/LostRhapsody/slothtime]

## Installation

### Prerequisites

- Rust (latest stable version)
- A terminal that supports ANSI escape codes

### Building

Clone the repository and build with Cargo:

```bash
git clone https://github.com/LostRhapsody/slothtime-rs
cd slothtime-rs
cargo build --release
# for windows
cargo build --release --target x86_64-pc-windows-gnu
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

## Help Menu

Type "?" to see a list of shortcuts and instructions.

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

Files are named `Month_dd_yyyy_slothtime.csv` and saved to the configured export directory.

## Time Format

- Supports HH:MM format (e.g., 09:30)
- Also accepts HHMM format (e.g., 0930)
- Automatically calculates task duration from start and end times

