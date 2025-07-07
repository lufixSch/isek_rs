use std::io::stdout;

use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use isek_rs::App;
use ratatui::{prelude::CrosstermBackend, Terminal};

fn main() -> Result<()> {
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode();
    set_panic_hook();

    color_eyre::install()?;

    let mut app = App::new()?;

    Terminal::new(CrosstermBackend::new(stdout()));
    let mut terminal = ratatui::init();
    let res = app.tui(&mut terminal);

    ratatui::restore();
    res
}

// Gracefully exit program on panic
fn set_panic_hook() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = ratatui::restore();

        hook(panic_info)
    }))
}
