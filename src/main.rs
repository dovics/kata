mod app;
mod constant;
mod kafka;
mod tabs;
mod theme;

use app::App;
use clap::Parser;
use color_eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::stdout;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Bootstrap servers
    #[arg(short, long)]
    brokers: String,

    /// Group id
    #[arg(short, long)]
    group: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    color_eyre::install()?;
    let terminal = ratatui::init();
    execute!(stdout(), EnterAlternateScreen).expect("failed to enter alternate screen");
    let app_result = App::new(args.brokers, args.group)?.run(terminal).await;
    execute!(stdout(), LeaveAlternateScreen).expect("failed to leave alternate screen");
    ratatui::restore();
    app_result
}
