use std::io::stdout;

use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use isek_rs::{App, MainView, View};
use ratatui::{DefaultTerminal, Terminal, prelude::CrosstermBackend};

fn main() -> Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    set_panic_hook();

    color_eyre::install()?;

    let mut app = App::new()?;

    Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut terminal = ratatui::init();
    let res = run(&mut terminal, &mut app);

    ratatui::restore();
    res
}

// Gracefully exit program on panic
fn set_panic_hook() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        ratatui::restore();

        hook(panic_info)
    }))
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    let view = MainView;

    while !app.exit {
        terminal.draw(|f| view.draw(app, f))?;

        view.handle_events(app)?;
    }

    Ok(())
}
