use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Table, TableState, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.size();

    match app.mode {
        InputMode::Help => draw_help(f, app, size),
        InputMode::EditingPopup | InputMode::ViewingPopup => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            draw_table(f, app, chunks[0]);
            draw_status(f, app, chunks[1]);
            draw_popup(f, app, size);
        }
        InputMode::ConfirmDeleteEntry => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            draw_table(f, app, chunks[0]);
            draw_status(f, app, chunks[1]);
            draw_confirm_delete_dialog(f, app, size);
        }
        InputMode::ConfirmClearEntries => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                .split(size);

            draw_table(f, app, chunks[0]);
            draw_status(f, app, chunks[1]);
            draw_confirm_clear_dialog(f, app, size);
        }
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
        
        let is_current_row = i == app.cursor.row;
        let active_cell_style = match app.mode {
            InputMode::Editing | InputMode::EditingPopup => Style::default()
                .fg(Color::Black)
                .bg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
            _ => Style::default()
                .fg(Color::Black)
                .bg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        };
        
        // Create cells with conditional styling
        let cell_data = vec![
            row_num,
            entry.task_number.clone(),
            entry.work_code.clone(),
            entry.time_entry.replace("\n", " "),
            entry.start_time.clone(),
            entry.end_time.clone(),
        ];
        
        let cells: Vec<Text> = cell_data.into_iter().enumerate().map(|(col_idx, content)| {
            // app.cursor.col is 1-indexed (1=Task Number, 2=Work Code, etc.)
            // cell_data array is 0-indexed (0=row number, 1=Task Number, etc.)
            // So app.cursor.col should equal col_idx for the active cell
            if is_current_row && app.cursor.col == col_idx {
                // Add text cursor when in editing mode, but NOT when popup is active
                let display_content = if matches!(app.mode, InputMode::Editing) && !matches!(app.mode, InputMode::EditingPopup | InputMode::ViewingPopup) {
                    // Insert cursor indicator at text_cursor position
                    let mut chars: Vec<char> = content.chars().collect();
                    if app.text_cursor <= chars.len() {
                        chars.insert(app.text_cursor, '|');
                    }
                    format!("[{}]", chars.into_iter().collect::<String>())
                } else {
                    format!("[{}]", content)
                };
                Text::styled(display_content, active_cell_style)
            } else {
                Text::raw(content)
            }
        }).collect();
        
        ratatui::widgets::Row::new(cells).bottom_margin(1)
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
        .block(Block::default().borders(Borders::ALL).title("Slothtime"));

    let mut state = TableState::default();
    state.select(Some(app.cursor.row));

    f.render_stateful_widget(table, area, &mut state);
}

fn draw_status(f: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        InputMode::Navigation => "Navigation",
        InputMode::Editing => "Editing",
        InputMode::EditingPopup => "Editing (Popup)",
        InputMode::ViewingPopup => "Viewing (Popup)",
        InputMode::Help => "Help",
        InputMode::ConfirmDeleteEntry => "Confirm Delete",
        InputMode::ConfirmClearEntries => "Confirm Clear",
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

    let status = if let Some(ref message) = app.status_message {
        // Show status message if available
        message.clone()
    } else if matches!(app.mode, InputMode::Editing) {
        format!(
            "Mode: {} | Editing {}: '{}' | Esc to exit, Tab to next cell",
            mode,
            col_name,
            current_value
        )
    } else {
        format!(
            "Mode: {} | Row: {} | Col: {} ({}) | i to edit, Ctrl+Y copy, Ctrl+S export, Ctrl+X clear, ? help | q quit",
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

fn draw_popup(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(80, 60, area);
    f.render_widget(Clear, popup_area);

    // Get task number for the title
    let task_number = if app.cursor.row < app.entries.len() {
        &app.entries[app.cursor.row].task_number
    } else {
        ""
    };
    let title = if task_number.is_empty() {
        "Edit Time Entry".to_string()
    } else {
        format!("{} - Edit Time Entry", task_number)
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL);
    let inner_area = block.inner(popup_area);
    f.render_widget(block, popup_area);

    if app.cursor.row < app.entries.len() {
        let text = app.entries[app.cursor.row].time_entry.clone();
        
        // Add cursor when in editing mode
        let display_text = if matches!(app.mode, InputMode::EditingPopup) {
            let mut chars: Vec<char> = text.chars().collect();
            if app.text_cursor <= chars.len() {
                chars.insert(app.text_cursor, '|');
            }
            chars.into_iter().collect::<String>()
        } else {
            text
        };
        
        let lines: Vec<Line> = display_text.lines().map(|l| Line::from(l)).collect();

        let paragraph = Paragraph::new(lines)
            .scroll((app.popup_scroll as u16, 0))
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, inner_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
fn draw_help(f: &mut Frame, _app: &App, area: Rect) {
    let help_text = r#"
Slothtime TUI - Help

Navigation Mode:
  i          - Enter edit mode (vim-style)
  dd         - Delete current entry (with confirmation)
  Tab        - Move to next column
  Shift+Tab  - Move to previous column
  Arrow Keys - Navigate up/down/left/right
  ?          - Show this help
  Ctrl+Y     - Copy current field to clipboard
  Ctrl+S     - Export to CSV
  Ctrl+X     - Clear all entries (with confirmation)
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

fn draw_confirm_delete_dialog(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);

    let task_info = if app.cursor.row < app.entries.len() {
        let entry = &app.entries[app.cursor.row];
        if entry.task_number.is_empty() {
            format!("Row {}", app.cursor.row + 1)
        } else {
            format!("Entry: {}", entry.task_number)
        }
    } else {
        "Entry".to_string()
    };

    let text = format!(
        "Delete {}?\n\nPress 'y' to confirm, 'n' or Esc to cancel.",
        task_info
    );

    let block = Block::default()
        .title("Confirm Delete")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, popup_area);
}

fn draw_confirm_clear_dialog(f: &mut Frame, _app: &App, area: Rect) {
    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);

    let text = "Clear all entries?\n\nThis will delete all time entries.\n\nPress 'y' to confirm, 'n' or Esc to cancel.";

    let block = Block::default()
        .title("Confirm Clear All")
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Red));

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true })
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(paragraph, popup_area);
}