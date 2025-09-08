use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Table, TableState, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(size);

    draw_table(f, app, chunks[0]);
    draw_status(f, app, chunks[1]);
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    let header = ["Task Number", "Work Code", "Time Entry", "Start Time", "End Time"];

    let rows: Vec<ratatui::widgets::Row> = app.entries.iter().enumerate().map(|(_i, entry)| {
        ratatui::widgets::Row::new(vec![
            entry.task_number.clone(),
            entry.work_code.clone(),
            entry.time_entry.clone(),
            entry.start_time.clone(),
            entry.end_time.clone(),
        ])
    }).collect();

    let widths = [
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
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
        .highlight_symbol(">> ");

    let mut state = TableState::default();
    state.select(Some(app.cursor.row));

    f.render_stateful_widget(table, area, &mut state);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        InputMode::Navigation => "Navigation",
        InputMode::Editing => "Editing",
    };

    let status = format!(
        "Mode: {} | Row: {} | Col: {} | Press 'q' to quit",
        mode,
        app.cursor.row + 1,
        app.cursor.col + 1
    );

    let paragraph = Paragraph::new(status)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}