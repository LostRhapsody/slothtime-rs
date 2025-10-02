use anyhow::Result;
use arboard::Clipboard;
use crossterm::event::{self, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::time::{Duration, Instant};

use crate::config::Config;
use crate::time_entry::TimeEntry;
use crate::ui;
use serde_json;
use std::fs;

#[derive(Debug, Clone)]
pub enum InputMode {
    Navigation,
    Editing,
    EditingPopup,
    ViewingPopup,
    Help,
    ConfirmDeleteEntry,
    ConfirmClearEntries,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 1 } // start at first editable column
    }
}

pub struct App {
    pub entries: Vec<TimeEntry>,
    pub cursor: Cursor,
    pub mode: InputMode,
    pub config: Config,
    pub should_quit: bool,
    pub popup_scroll: usize,
    pub text_cursor: usize,   // Position within the current text field
    pub pending_delete: bool, // Track if first 'd' was pressed for 'dd' command
    pub status_message: Option<String>, // Temporary status message
    pub message_timer: Option<std::time::Instant>, // Timer for status message
    pub last_save_time: Instant, // Track when we last saved
    pub auto_save_interval: Duration, // How often to auto-save
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let entries = Self::load_entries().unwrap_or_else(|_| vec![TimeEntry::new()]);
        let mut app = Self {
            entries,
            cursor: Cursor::new(),
            mode: InputMode::Navigation,
            config,
            should_quit: false,
            popup_scroll: 0,
            text_cursor: 0,
            pending_delete: false,
            status_message: None,
            message_timer: None,
            last_save_time: Instant::now(),
            auto_save_interval: Duration::from_secs(30), // Auto-save every 30 seconds
        };
        // Initialize mode based on starting column
        app.update_mode_for_column();
        Ok(app)
    }

    fn load_entries() -> Result<Vec<TimeEntry>> {
        // Get home dir/ location for entries file
        let home_dir = dirs::home_dir().unwrap();
        let config_dir = home_dir.join(".slothtime");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).unwrap();
        }
        let file = config_dir.join("entries.json");
        let content = fs::read_to_string(file)?;
        let entries: Vec<TimeEntry> = serde_json::from_str(&content)?;
        Ok(entries)
    }

    fn save_entries(&self) -> Result<()> {
        let content = serde_json::to_string(&self.entries)?;
        // Get home dir/ location for entries file
        let home_dir = dirs::home_dir().unwrap();
        let config_dir = home_dir.join(".slothtime");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).unwrap();
        }
        let file = config_dir.join("entries.json");
        
        // Create backup before saving
        self.create_backup(&config_dir)?;
        
        // Save the main file
        fs::write(&file, content)?;
        Ok(())
    }

    fn create_backup(&self, config_dir: &std::path::Path) -> Result<()> {
        let content = serde_json::to_string(&self.entries)?;
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = config_dir.join(format!("entries_backup_{}.json", timestamp));
        fs::write(backup_file, content)?;
        
        // Keep only the last 10 backups to avoid disk space issues
        self.cleanup_old_backups(config_dir)?;
        
        Ok(())
    }

    fn cleanup_old_backups(&self, config_dir: &std::path::Path) -> Result<()> {
        let mut backup_files: Vec<_> = fs::read_dir(config_dir)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("entries_backup_") && name.ends_with(".json"))
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time (newest first)
        backup_files.sort_by(|a, b| {
            b.metadata().unwrap().modified().unwrap()
                .cmp(&a.metadata().unwrap().modified().unwrap())
        });

        // Keep only the 10 most recent backups
        for backup in backup_files.iter().skip(10) {
            let _ = fs::remove_file(&backup.path());
        }

        Ok(())
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            self.update_message_timer();
            self.check_auto_save();
            terminal.draw(|f| ui::draw(f, self))?;
            if self.should_quit {
                self.save_entries().ok();
                break;
            }
            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    // Only handle key press events, ignore key release events
                    // This fixes double input on Windows
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key);
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            InputMode::Navigation => match key.code {
                event::KeyCode::Char('q') => self.should_quit = true,
                event::KeyCode::Char('s')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    let _ = self.export();
                }
                event::KeyCode::Char('x')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    self.mode = InputMode::ConfirmClearEntries;
                }
                event::KeyCode::Char('y')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    self.copy_current_field();
                }
                event::KeyCode::Char('d') => {
                    if self.pending_delete {
                        // Second 'd' - show confirmation
                        self.mode = InputMode::ConfirmDeleteEntry;
                        self.pending_delete = false;
                    } else {
                        // First 'd' - set pending
                        self.pending_delete = true;
                    }
                }

                event::KeyCode::Char('i') => {
                    self.pending_delete = false;
                    self.enter_edit();
                }
                event::KeyCode::Char('?') => {
                    self.pending_delete = false;
                    self.mode = InputMode::Help;
                }
                event::KeyCode::Tab => {
                    self.pending_delete = false;
                    self.next_col();
                }
                event::KeyCode::BackTab => {
                    self.pending_delete = false;
                    self.prev_col();
                }
                event::KeyCode::Up => {
                    self.pending_delete = false;
                    self.prev_row();
                }
                event::KeyCode::Down => {
                    self.pending_delete = false;
                    self.next_row();
                }
                event::KeyCode::Left => {
                    self.pending_delete = false;
                    self.prev_col();
                }
                event::KeyCode::Right => {
                    self.pending_delete = false;
                    self.next_col();
                }
                _ => {
                    // Reset pending delete on any other key
                    self.pending_delete = false;
                }
            },
            InputMode::Editing => match key.code {
                event::KeyCode::Esc => self.exit_edit(),
                event::KeyCode::Tab => self.next_col(),
                event::KeyCode::BackTab => self.prev_col(),
                event::KeyCode::Enter => {
                    self.next_col();
                    // stay in edit mode
                }
                event::KeyCode::Left => {
                    if self.text_cursor > 0 {
                        self.text_cursor -= 1;
                    }
                }
                event::KeyCode::Right => {
                    let max_len = self.get_current_field_length();
                    if self.text_cursor < max_len {
                        self.text_cursor += 1;
                    }
                }
                event::KeyCode::Home => self.text_cursor = 0,
                event::KeyCode::End => {
                    self.text_cursor = self.get_current_field_length();
                }
                event::KeyCode::Char(c) => self.insert_char(c),
                event::KeyCode::Backspace => self.delete_char(),
                _ => {}
            },
            InputMode::ViewingPopup => match key.code {
                event::KeyCode::Char('i') => self.enter_edit(),
                event::KeyCode::Char('y')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    self.copy_current_field();
                }
                event::KeyCode::Tab => self.next_col(),
                event::KeyCode::BackTab => self.prev_col(),
                event::KeyCode::Up => {
                    if self.popup_scroll > 0 {
                        self.popup_scroll -= 1;
                    }
                }
                event::KeyCode::Down => {
                    self.popup_scroll += 1;
                }
                event::KeyCode::Left => self.prev_col(),
                event::KeyCode::Right => self.next_col(),
                _ => {}
            },
            InputMode::EditingPopup => match key.code {
                event::KeyCode::Esc => {
                    // Exit edit mode but stay in popup view
                    self.mode = InputMode::ViewingPopup;
                }
                event::KeyCode::Tab => {
                    self.next_col();
                    self.popup_scroll = 0;
                }
                event::KeyCode::BackTab => {
                    self.prev_col();
                    self.popup_scroll = 0;
                }
                event::KeyCode::Char('y')
                    if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                {
                    self.copy_current_field();
                }
                event::KeyCode::Enter => {
                    self.insert_char('\n');
                }
                event::KeyCode::Up => {
                    self.move_cursor_up_in_text();
                }
                event::KeyCode::Down => {
                    self.move_cursor_down_in_text();
                }
                event::KeyCode::Left => {
                    if self.text_cursor > 0 {
                        self.text_cursor -= 1;
                    }
                }
                event::KeyCode::Right => {
                    let max_len = self.get_current_field_length();
                    if self.text_cursor < max_len {
                        self.text_cursor += 1;
                    }
                }
                event::KeyCode::Home => self.text_cursor = 0,
                event::KeyCode::End => {
                    self.text_cursor = self.get_current_field_length();
                }
                event::KeyCode::Char(c) => self.insert_char(c),
                event::KeyCode::Backspace => self.delete_char(),
                _ => {}
            },
            InputMode::Help => {
                self.mode = InputMode::Navigation;
            }
            InputMode::ConfirmDeleteEntry => match key.code {
                event::KeyCode::Char('y') | event::KeyCode::Char('Y') => {
                    self.delete_current_entry();
                    self.mode = InputMode::Navigation;
                }
                event::KeyCode::Char('n') | event::KeyCode::Char('N') | event::KeyCode::Esc => {
                    self.mode = InputMode::Navigation;
                }
                _ => {}
            },
            InputMode::ConfirmClearEntries => match key.code {
                event::KeyCode::Char('y') | event::KeyCode::Char('Y') => {
                    self.clear_entries();
                    self.mode = InputMode::Navigation;
                }
                event::KeyCode::Char('n') | event::KeyCode::Char('N') | event::KeyCode::Esc => {
                    self.mode = InputMode::Navigation;
                }
                _ => {}
            },
        }
    }

    fn next_col(&mut self) {
        if self.cursor.col < 5 {
            self.cursor.col += 1;
        } else {
            // When on last column (End Time), move to next row, first column
            self.cursor.col = 1;
            self.next_row();
        }
        self.update_mode_for_column();
    }

    fn prev_col(&mut self) {
        if self.cursor.col > 1 {
            self.cursor.col -= 1;
        } else {
            // When on first column (Task Number), go to previous row's last column (End Time)
            self.cursor.col = 5;
            if self.cursor.row > 0 {
                self.cursor.row -= 1;
            }
        }
        self.update_mode_for_column();
    }

    fn next_row(&mut self) {
        if self.cursor.row < self.entries.len() - 1 {
            self.cursor.row += 1;
        } else {
            // Auto-create new row if at the end and current row is complete
            if self.entries[self.cursor.row].is_complete() {
                self.entries.push(TimeEntry::new());
                self.cursor.row += 1;
            }
        }
        self.update_mode_for_column();
    }

    fn prev_row(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
        self.update_mode_for_column();
    }

    fn update_mode_for_column(&mut self) {
        // Auto-show popup when on Time Entry column (3), auto-hide when not
        if self.cursor.col == 3 {
            // Convert to popup mode while preserving edit state
            match self.mode {
                InputMode::Navigation => self.mode = InputMode::ViewingPopup,
                InputMode::Editing => self.mode = InputMode::EditingPopup,
                _ => {} // Already in popup mode or other mode
            }
        } else {
            // Convert back to regular mode while preserving edit state
            match self.mode {
                InputMode::ViewingPopup => self.mode = InputMode::Navigation,
                InputMode::EditingPopup => self.mode = InputMode::Editing,
                _ => {} // Already in regular mode or other mode
            }
            self.popup_scroll = 0;
        }
        // Update text cursor position when switching cells
        self.update_text_cursor();
    }

    fn update_text_cursor(&mut self) {
        // Set text cursor to end of current field
        if self.cursor.row < self.entries.len() {
            let entry = &self.entries[self.cursor.row];
            let text_length = match self.cursor.col {
                1 => entry.task_number.len(),
                2 => entry.work_code.len(),
                3 => entry.time_entry.len(),
                4 => entry.start_time.len(),
                5 => entry.end_time.len(),
                _ => 0,
            };
            self.text_cursor = text_length;
        }
    }

    fn get_current_field_length(&self) -> usize {
        if self.cursor.row < self.entries.len() {
            let entry = &self.entries[self.cursor.row];
            match self.cursor.col {
                1 => entry.task_number.len(),
                2 => entry.work_code.len(),
                3 => entry.time_entry.len(),
                4 => entry.start_time.len(),
                5 => entry.end_time.len(),
                _ => 0,
            }
        } else {
            0
        }
    }

    fn move_cursor_up_in_text(&mut self) {
        if self.cursor.col != 3 || self.cursor.row >= self.entries.len() {
            return;
        }

        let text = self.entries[self.cursor.row].time_entry.clone();
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return;
        }

        // Find current line and position within that line
        let mut char_count = 0;
        let mut current_line = 0;
        let mut pos_in_line = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            if char_count + line.len() >= self.text_cursor {
                current_line = line_idx;
                pos_in_line = self.text_cursor - char_count;
                break;
            }
            char_count += line.len() + 1; // +1 for newline
        }

        // Move to previous line if possible
        if current_line > 0 {
            let prev_line = lines[current_line - 1];
            let new_pos_in_line = pos_in_line.min(prev_line.len());

            // Calculate new cursor position
            let mut new_cursor = 0;
            for i in 0..(current_line - 1) {
                new_cursor += lines[i].len() + 1;
            }
            new_cursor += new_pos_in_line;

            self.text_cursor = new_cursor;
        }
    }

    fn move_cursor_down_in_text(&mut self) {
        if self.cursor.col != 3 || self.cursor.row >= self.entries.len() {
            return;
        }

        let text = self.entries[self.cursor.row].time_entry.clone();
        let lines: Vec<&str> = text.lines().collect();

        if lines.is_empty() {
            return;
        }

        // Find current line and position within that line
        let mut char_count = 0;
        let mut current_line = 0;
        let mut pos_in_line = 0;

        for (line_idx, line) in lines.iter().enumerate() {
            if char_count + line.len() >= self.text_cursor {
                current_line = line_idx;
                pos_in_line = self.text_cursor - char_count;
                break;
            }
            char_count += line.len() + 1; // +1 for newline
        }

        // Move to next line if possible
        if current_line < lines.len() - 1 {
            let next_line = lines[current_line + 1];
            let new_pos_in_line = pos_in_line.min(next_line.len());

            // Calculate new cursor position
            let mut new_cursor = 0;
            for i in 0..=current_line {
                new_cursor += lines[i].len() + 1;
            }
            new_cursor += new_pos_in_line;

            self.text_cursor = new_cursor;
        }
        // Note: Removed auto-scroll call to avoid borrow issues for now
    }

    fn enter_edit(&mut self) {
        match self.mode {
            InputMode::ViewingPopup => {
                self.mode = InputMode::EditingPopup;
            }
            _ => {
                if self.cursor.col == 3 {
                    self.mode = InputMode::EditingPopup;
                } else {
                    self.mode = InputMode::Editing;
                }
            }
        }
        // Update text cursor when entering edit mode
        self.update_text_cursor();
    }

    fn exit_edit(&mut self) {
        self.mode = InputMode::Navigation;
        // auto-create new row if last and complete
        if self.cursor.row == self.entries.len() - 1 && self.entries[self.cursor.row].is_complete()
        {
            self.entries.push(TimeEntry::new());
        }
        // Save when exiting edit mode
        self.auto_save();
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor.row >= self.entries.len() {
            return;
        }
        let entry = &mut self.entries[self.cursor.row];
        let field = match self.cursor.col {
            1 => &mut entry.task_number,
            2 => &mut entry.work_code,
            3 => &mut entry.time_entry,
            4 => &mut entry.start_time,
            5 => &mut entry.end_time,
            _ => return,
        };

        // Insert character at cursor position
        if self.text_cursor <= field.len() {
            field.insert(self.text_cursor, c);
            self.text_cursor += 1;
            // Auto-save after each character insertion
            self.auto_save();
        }
    }

    fn delete_char(&mut self) {
        if self.cursor.row >= self.entries.len() || self.text_cursor == 0 {
            return;
        }
        let entry = &mut self.entries[self.cursor.row];
        let field = match self.cursor.col {
            1 => &mut entry.task_number,
            2 => &mut entry.work_code,
            3 => &mut entry.time_entry,
            4 => &mut entry.start_time,
            5 => &mut entry.end_time,
            _ => return,
        };

        // Delete character before cursor position
        if self.text_cursor > 0 && self.text_cursor <= field.len() {
            field.remove(self.text_cursor - 1);
            self.text_cursor -= 1;
            // Auto-save after each character deletion
            self.auto_save();
        }
    }

    fn export(&self) -> Result<()> {
        crate::export::export_csv(&self.entries, &self.config)?;
        self.save_entries()
    }

    fn clear_entries(&mut self) {
        self.entries = vec![TimeEntry::new()];
        self.cursor = Cursor::new();
        let _ = self.save_entries();
    }

    fn delete_current_entry(&mut self) {
        if self.entries.len() <= 1 {
            // Don't delete the last entry, just clear it
            self.entries[0] = TimeEntry::new();
            self.cursor = Cursor::new();
        } else {
            // Remove current entry
            self.entries.remove(self.cursor.row);

            // Adjust cursor position if needed
            if self.cursor.row >= self.entries.len() {
                self.cursor.row = self.entries.len() - 1;
            }
        }

        // Reset cursor column and update mode
        self.cursor.col = 1;
        self.update_mode_for_column();
        let _ = self.save_entries();
    }

    fn show_message(&mut self, msg: &str) {
        self.status_message = Some(msg.to_string());
        self.message_timer = Some(std::time::Instant::now());
    }

    fn update_message_timer(&mut self) {
        if let Some(timer) = self.message_timer {
            if timer.elapsed().as_secs() >= 3 {
                self.status_message = None;
                self.message_timer = None;
            }
        }
    }

    fn check_auto_save(&mut self) {
        if self.last_save_time.elapsed() >= self.auto_save_interval {
            if let Err(e) = self.save_entries() {
                self.show_message(&format!("Auto-save failed: {}", e));
            } else {
                self.last_save_time = Instant::now();
            }
        }
    }

    fn auto_save(&mut self) {
        if let Err(e) = self.save_entries() {
            self.show_message(&format!("Save failed: {}", e));
        } else {
            self.last_save_time = Instant::now();
        }
    }

    fn copy_current_field(&mut self) {
        if self.cursor.row >= self.entries.len() {
            self.show_message("No entry to copy from");
            return;
        }

        let entry = &self.entries[self.cursor.row];
        let (field_content, field_name) = match self.cursor.col {
            1 => (&entry.task_number, "Task Number"),
            2 => (&entry.work_code, "Work Code"),
            3 => (&entry.time_entry, "Time Entry"),
            4 => (&entry.start_time, "Start Time"),
            5 => (&entry.end_time, "End Time"),
            _ => {
                self.show_message("Invalid field");
                return;
            }
        };

        if field_content.is_empty() {
            self.show_message(&format!("{} is empty", field_name));
            return;
        }

        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(field_content) {
                Ok(()) => {
                    self.show_message(&format!("{} copied to clipboard!", field_name));
                }
                Err(_) => {
                    self.show_message("Failed to copy to clipboard");
                }
            },
            Err(_) => {
                self.show_message("Could not access clipboard");
            }
        }
    }
}
