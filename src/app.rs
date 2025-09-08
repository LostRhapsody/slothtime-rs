use std::io;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use crossterm::event;
use anyhow::Result;

use crate::time_entry::TimeEntry;
use crate::config::Config;
use crate::ui;
use std::fs;
use serde_json;

#[derive(Debug, Clone)]
pub enum InputMode {
    Navigation,
    Editing,
    EditingPopup,
    Help,
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
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let entries = Self::load_entries().unwrap_or_else(|_| vec![TimeEntry::new()]);
        Ok(Self {
            entries,
            cursor: Cursor::new(),
            mode: InputMode::Navigation,
            config,
            should_quit: false,
            popup_scroll: 0,
        })
    }

    fn load_entries() -> Result<Vec<TimeEntry>> {
        let content = fs::read_to_string("entries.json")?;
        let entries: Vec<TimeEntry> = serde_json::from_str(&content)?;
        Ok(entries)
    }

    fn save_entries(&self) -> Result<()> {
        let content = serde_json::to_string(&self.entries)?;
        fs::write("entries.json", content)?;
        Ok(())
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, self))?;
            if self.should_quit {
                self.save_entries().ok();
                break;
            }
            if event::poll(std::time::Duration::from_millis(100))? {
                if let event::Event::Key(key) = event::read()? {
                    self.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: event::KeyEvent) {
        match self.mode {
            InputMode::Navigation => match key.code {
                event::KeyCode::Char('q') => self.should_quit = true,
                event::KeyCode::Char('s') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    let _ = self.export();
                }
                event::KeyCode::Char('x') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                    self.clear_entries();
                }

                event::KeyCode::Char('i') => self.enter_edit(),
                event::KeyCode::Char('?') => self.mode = InputMode::Help,
                event::KeyCode::Tab => self.next_col(),
                event::KeyCode::BackTab => self.prev_col(),
                event::KeyCode::Up => self.prev_row(),
                event::KeyCode::Down => self.next_row(),
                event::KeyCode::Left => self.prev_col(),
                event::KeyCode::Right => self.next_col(),
                _ => {}
            },
            InputMode::Editing => match key.code {
                event::KeyCode::Esc => self.exit_edit(),
                event::KeyCode::Tab => self.next_col(),
                event::KeyCode::Enter => {
                    self.next_row();
                    // stay in edit
                }
                event::KeyCode::Char(c) => self.insert_char(c),
                event::KeyCode::Backspace => self.delete_char(),
                _ => {}
            },
            InputMode::EditingPopup => match key.code {
                event::KeyCode::Esc => {
                    self.mode = InputMode::Navigation;
                    self.popup_scroll = 0;
                }
                event::KeyCode::Tab => {
                    self.next_col();
                    self.popup_scroll = 0;
                }
                event::KeyCode::Enter => {
                    self.insert_char('\n');
                }
                event::KeyCode::Up => {
                    if self.popup_scroll > 0 {
                        self.popup_scroll -= 1;
                    }
                }
                event::KeyCode::Down => {
                    self.popup_scroll += 1;
                }
                event::KeyCode::Char(c) => self.insert_char(c),
                event::KeyCode::Backspace => self.delete_char(),
                _ => {}
            },
            InputMode::Help => {
                self.mode = InputMode::Navigation;
            }
        }
    }

    fn next_col(&mut self) {
        if self.cursor.col < 5 {
            self.cursor.col += 1;
        } else {
            self.cursor.col = 1;
        }
    }

    fn prev_col(&mut self) {
        if self.cursor.col > 1 {
            self.cursor.col -= 1;
        } else {
            self.cursor.col = 5;
        }
    }

    fn next_row(&mut self) {
        if self.cursor.row < self.entries.len() - 1 {
            self.cursor.row += 1;
        }
    }

    fn prev_row(&mut self) {
        if self.cursor.row > 0 {
            self.cursor.row -= 1;
        }
    }

    fn enter_edit(&mut self) {
        if self.cursor.col == 3 {
            self.mode = InputMode::EditingPopup;
        } else {
            self.mode = InputMode::Editing;
        }
    }

    fn exit_edit(&mut self) {
        self.mode = InputMode::Navigation;
        // auto-create new row if last and complete
        if self.cursor.row == self.entries.len() - 1 && self.entries[self.cursor.row].is_complete() {
            self.entries.push(TimeEntry::new());
        }
    }

    fn insert_char(&mut self, c: char) {
        if self.cursor.row >= self.entries.len() {
            return;
        }
        let entry = &mut self.entries[self.cursor.row];
        match self.cursor.col {
            1 => entry.task_number.push(c),
            2 => entry.work_code.push(c),
            3 => entry.time_entry.push(c),
            4 => entry.start_time.push(c),
            5 => entry.end_time.push(c),
            _ => {}
        }
    }

    fn delete_char(&mut self) {
        if self.cursor.row >= self.entries.len() {
            return;
        }
        let entry = &mut self.entries[self.cursor.row];
        match self.cursor.col {
            1 => { entry.task_number.pop(); }
            2 => { entry.work_code.pop(); }
            3 => { entry.time_entry.pop(); }
            4 => { entry.start_time.pop(); }
            5 => { entry.end_time.pop(); }
            _ => {}
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


}