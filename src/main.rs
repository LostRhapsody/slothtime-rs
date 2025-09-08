use std::io;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

mod app;
mod ui;
mod config;
mod time_entry;
mod export;

use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run
    let mut app = App::new()?;
    let res = app.run(&mut terminal);

    // cleanup
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}