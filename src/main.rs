mod app;
mod theme;
mod kafka;
mod normal;

use app::App;
use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{layout::Rect, TerminalOptions, Viewport};
use std::io::stdout;

fn main() -> Result<()> {
    let brokers = vec!["localhost:9092".to_string()];

    color_eyre::install()?;
    let viewport = Viewport::Fixed(Rect::new(0, 0, 81, 18));
    let terminal = ratatui::init_with_options(TerminalOptions { viewport });
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alternate screen");
    let app_result = App::new(brokers).run(terminal);
    execute!(stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");
    ratatui::restore();
    app_result
}
