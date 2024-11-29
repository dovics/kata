mod app;
mod kafka;
mod theme;
mod tabs;

use app::App;
use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use std::env;
use std::io::stdout;

fn main() -> Result<()> {
    let brokers = env::args()
        .nth(1)
        .unwrap_or("localhost:9092".to_string())
        .split(',')
        .map(|s| s.to_string())
        .collect();
    
    color_eyre::install()?;
    let terminal = ratatui::init();
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alternate screen");
    let app_result = App::new(brokers)?.run(terminal);
    execute!(stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");
    ratatui::restore();
    app_result
}
