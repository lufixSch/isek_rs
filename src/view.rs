use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use eyre::Result;
use ratatui::{
    style::Stylize, symbols::border, text::Line, widgets::{Block, Padding}, Frame
};

use crate::{App, widgets::ToDoList};

/// Trait defining the interface for views in the application
pub trait View {
    /// Handle user input events
    fn handle_events(&self, app: &mut App) -> Result<()>;

    /// Render the view to the terminal frame
    fn draw(&self, app: &mut App, frame: &mut Frame);
}

#[derive(Default)]
/// Main application view that displays the todo list
pub struct MainView;

impl MainView {
    /// Handle key press events for navigation and quitting
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => app.exit(),
            KeyCode::Char('j') => app.list_state.select_next(),
            KeyCode::Char('k') => app.list_state.select_previous(),
            KeyCode::Esc => app.escape(),
            _ => {}
        }
    }
}

impl View for MainView {
    fn handle_events(&self, app: &mut App) -> Result<()> {
        // Process keyboard events
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(app, key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, app: &mut App, frame: &mut Frame) {
        // Create title with bold styling
        let title = Line::from(" ISEK ".bold());

        // Configure the block border and padding for the todo list display
        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::ROUNDED)
            .padding(Padding::symmetric(2, 1))
        ;

        // Render the todo list widget in the main frame area
        frame.render_stateful_widget(ToDoList::default().block(block), frame.area(), app);
    }
}
