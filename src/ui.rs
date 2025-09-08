use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Table, TableState, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    match app.mode {
        InputMode::Help => draw_help(f, app, size),
        _ => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            draw_table(f, app, chunks[0]);
            draw_status(f, app, chunks[1]);
        }
    }
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    let header = ["#", "Task Number", "Work Code", "Time Entry", "Start Time", "End Time"];

    let rows: Vec<ratatui::widgets::Row> = app.entries.iter().enumerate().map(|(i, entry)| {
        let row_num = if i == app.cursor.row { ">>".to_string() } else { (i + 1).to_string() };
        ratatui::widgets::Row::new(vec![
            row_num,
            entry.task_number.clone(),
            entry.work_code.clone(),
            entry.time_entry.clone(),
            entry.start_time.clone(),
            entry.end_time.clone(),
        ]).bottom_margin(0)
    }).collect();

    let widths = [
        Constraint::Length(3),
        Constraint::Length(15),
        Constraint::Length(15),
        Constraint::Length(30),
        Constraint::Length(12),
        Constraint::Length(12),
    ];

    let table = Table::new(rows).widths(&widths)
        .header(
            ratatui::widgets::Row::new(header)
                .style(Style::default().fg(Color::Yellow))
                .bottom_margin(1),
        )
        .block(Block::default().borders(Borders::ALL).title("Slothtime"))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED).fg(Color::Black).bg(Color::White));

    let mut state = TableState::default();
    state.select(Some(app.cursor.row));

    f.render_stateful_widget(table, area, &mut state);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        InputMode::Navigation => "Navigation",
        InputMode::Editing => "Editing",
        InputMode::Help => "Help",
    };

    let col_name = match app.cursor.col {
        1 => "Task Number",
        2 => "Work Code",
        3 => "Time Entry",
        4 => "Start Time",
        5 => "End Time",
        _ => "",
    };

    let current_value = if app.cursor.row < app.entries.len() {
        let entry = &app.entries[app.cursor.row];
        match app.cursor.col {
            1 => entry.task_number.clone(),
            2 => entry.work_code.clone(),
            3 => entry.time_entry.clone(),
            4 => entry.start_time.clone(),
            5 => entry.end_time.clone(),
            _ => "".to_string(),
        }
    } else {
        "".to_string()
    };

    let status = if matches!(app.mode, InputMode::Editing) {
        format!(
            "Mode: {} | Editing {}: '{}' | Esc to exit, Tab to next cell",
            mode,
            col_name,
            current_value
        )
    } else {
        format!(
            "Mode: {} | Row: {} | Col: {} ({}) | i to edit, Ctrl+S export, Ctrl+X clear, ? help | q quit",
            mode,
            app.cursor.row + 1,
            app.cursor.col,
            col_name
        )
    };

    let paragraph = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

fn draw_help(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = r#"
Slothtime TUI - Help

Navigation Mode:
  i          - Enter edit mode (vim-style)
  Tab        - Move to next column
  Shift+Tab  - Move to previous column
  Arrow Keys - Navigate up/down/left/right
  ?          - Show this help
  Ctrl+S     - Export to CSV
  Ctrl+X     - Clear all entries
  q          - Quit

Edit Mode:
  Esc        - Exit edit mode
  Tab        - Move to next column (stay in edit)
  Enter      - Move to next row (stay in edit)
  Type       - Insert characters
  Backspace  - Delete characters

Press any key to return to navigation.
"#;

    let paragraph = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}