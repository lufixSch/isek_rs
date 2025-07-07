use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use eyre::Result;
use ratatui::{
    style::Stylize, symbols::border, text::Line, widgets::{Block, Padding}, Frame
};

use crate::{App, widgets::ToDoList};

pub trait View {
    fn handle_events(&self, app: &mut App) -> Result<()>;

    fn draw(&self, app: &mut App, frame: &mut Frame);
}

#[derive(Default)]
pub struct MainView;

impl MainView {
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(app, key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, app: &mut App, frame: &mut Frame) {
        let title = Line::from(" ISEK ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::ROUNDED).padding(Padding::symmetric(2, 1))
        ;

        frame.render_stateful_widget(ToDoList::default().block(block), frame.area(), app);
    }
}
