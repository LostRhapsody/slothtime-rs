use std::io;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use crossterm::event;
use anyhow::Result;

use crate::time_entry::TimeEntry;
use crate::config::Config;
use crate::ui;

#[derive(Debug, Clone)]
pub enum InputMode {
    Navigation,
    Editing,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub row: usize,
    pub col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self { row: 0, col: 0 }
    }
}

pub struct App {
    pub entries: Vec<TimeEntry>,
    pub cursor: Cursor,
    pub mode: InputMode,
    pub config: Config,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let entries = vec![TimeEntry::new()];
        Ok(Self {
            entries,
            cursor: Cursor::new(),
            mode: InputMode::Navigation,
            config,
            should_quit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, self))?;
            if self.should_quit {
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
                event::KeyCode::Tab => self.next_col(),
                event::KeyCode::BackTab => self.prev_col(),
                event::KeyCode::Enter => self.enter_edit(),
                event::KeyCode::Up => self.prev_row(),
                event::KeyCode::Down => self.next_row(),
                event::KeyCode::Left => self.prev_col(),
                event::KeyCode::Right => self.next_col(),
                _ => {}
            },
            InputMode::Editing => match key.code {
                event::KeyCode::Enter => self.exit_edit(),
                event::KeyCode::Esc => self.exit_edit(),
                event::KeyCode::Char(c) => self.insert_char(c),
                event::KeyCode::Backspace => self.delete_char(),
                _ => {}
            },
        }
    }

    fn next_col(&mut self) {
        self.cursor.col = (self.cursor.col + 1) % 5;
    }

    fn prev_col(&mut self) {
        if self.cursor.col == 0 {
            self.cursor.col = 4;
        } else {
            self.cursor.col -= 1;
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
        self.mode = InputMode::Editing;
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
            0 => entry.task_number.push(c),
            1 => entry.work_code.push(c),
            2 => entry.time_entry.push(c),
            3 => entry.start_time.push(c),
            4 => entry.end_time.push(c),
            _ => {}
        }
    }

    fn delete_char(&mut self) {
        if self.cursor.row >= self.entries.len() {
            return;
        }
        let entry = &mut self.entries[self.cursor.row];
        match self.cursor.col {
            0 => { entry.task_number.pop(); }
            1 => { entry.work_code.pop(); }
            2 => { entry.time_entry.pop(); }
            3 => { entry.start_time.pop(); }
            4 => { entry.end_time.pop(); }
            _ => {}
        }
    }

    fn export(&self) -> Result<()> {
        crate::export::export_csv(&self.entries, &self.config)
    }
}