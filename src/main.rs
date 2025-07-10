use std::io::stdout;

use color_eyre::eyre::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, enable_raw_mode},
};
use isek_rs::{App, MainView, View};
use ratatui::{DefaultTerminal, Terminal, prelude::CrosstermBackend};

/// Entry point of the application
fn main() -> Result<()> {
    // Initialize alternate screen and raw mode for terminal input handling
    execute!(stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    // Set up panic hook to restore terminal state on panic
    set_panic_hook();

    // Initialize color_eyre error reporting system
    color_eyre::install()?;

    //// Create and initialize the main application instance
    let mut app = App::new()?;

    //// Initialize ratatui and run the application
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let res = run(&mut terminal, &mut app);

    //// Restore terminal to its original state before exiting
    ratatui::restore();
    res
}

/// Sets a custom panic hook that restores the terminal before showing the panic message.
fn set_panic_hook() {
    let hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic_info| {
        // Make sure to restore the terminal state even if the program panics
        ratatui::restore();

        // Call original panic hook to display the error message
        hook(panic_info)
    }));
}

/// Main application loop that handles drawing and event handling.
fn run(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    // Create main view instance for rendering UI
    let view = MainView;

    // Application loop - continues until exit flag is set to true
    while !app.exit {
        // Draw the current application state in the terminal
        terminal.draw(|f| view.draw(app, f))?;

        // Handle user input and other events
        view.handle_events(app)?;
    }

    Ok(())
}
