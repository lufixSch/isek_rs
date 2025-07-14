use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use eyre::Result;
use ratatui::{
    Frame,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::Line,
    widgets::{Block, Padding},
};

use crate::{
    App,
    app::State,
    config::{FilterConfig, SortingConfig, SortingVariant},
    widgets::{StatusBar, ToDoList},
};

/// Trait defining the interface for views in the application
pub trait View {
    /// Handle user input events
    fn handle_events(&self, app: &mut App) -> Result<()>;

    /// Render the view to the terminal frame
    fn draw(&self, app: &mut App, frame: &mut Frame);
}

/// Main application view that displays the todo list
#[derive(Default)]
pub struct MainView;

impl MainView {
    /// Handle key press events for navigation and quitting
    fn handle_key_event(&self, app: &mut App, key_event: KeyEvent) -> Result<()> {
        match app.state {
            State::Normal => match key_event.code {
                KeyCode::Char('q') => Ok(app.exit()),
                KeyCode::Char('j') => Ok(app.list_state.select_next()),
                KeyCode::Char('k') => Ok(app.list_state.select_previous()),
                KeyCode::Char('s') => Ok(app.switch_state(State::ConfigSort)),
                KeyCode::Char('f') => Ok(app.switch_state(State::ConfigFilter)),
                KeyCode::Char('x') => app.toggle_done(),
                KeyCode::Esc => Ok(app.escape()),
                _ => Ok(()),
            },
            State::ConfigSort => match key_event.code {
                KeyCode::Char('d') => app.configure_sort(SortingConfig {
                    by: SortingVariant::Date,
                    ascending: app.display.sort.ascending,
                    ignore_done: app.display.sort.ignore_done,
                }),
                KeyCode::Char('p') => app.configure_sort(SortingConfig {
                    by: SortingVariant::Priority,
                    ascending: app.display.sort.ascending,
                    ignore_done: app.display.sort.ignore_done,
                }),
                KeyCode::Char('i') => app.configure_sort(SortingConfig {
                    by: SortingVariant::Index,
                    ascending: app.display.sort.ascending,
                    ignore_done: app.display.sort.ignore_done,
                }),
                KeyCode::Char('a') => app.configure_sort(SortingConfig {
                    by: app.display.sort.by.clone(),
                    ascending: !app.display.sort.ascending,
                    ignore_done: app.display.sort.ignore_done,
                }),
                _ => Ok(app.escape()),
            },
            State::ConfigFilter => match key_event.code {
                KeyCode::Char('d') => app.configure_filter(FilterConfig {
                    show_done: app.display.filter.show_done.next(),
                    show_done_for: app.display.filter.show_done_for,
                }),
                _ => Ok(app.escape()),
            },
            _ => Ok(()),
        }
    }
}

impl View for MainView {
    fn handle_events(&self, app: &mut App) -> Result<()> {
        // Process keyboard events
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(app, key_event)?
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
            .padding(Padding::symmetric(1, 1));

        let area = frame.area();
        let status_bar_area = Rect::new(0, area.height - 1, area.width, 1);
        let main_area = Rect::new(0, 0, area.width, area.height - status_bar_area.height);

        // Render the todo list widget in the main frame area
        frame.render_stateful_widget(ToDoList::default().block(block), main_area, app);

        // Render status bar
        let keybinds = match app.state {
            State::Normal => vec![("q", "Exit"), ("s", "Sort"), ("f", "Filter"), ("x", "Toggle done")],
            State::ConfigSort => vec![
                ("d", "By Date"),
                ("p", "By Priority"),
                ("i", "By Index"),
                ("a", "Toggle Ascending"),
            ],
            State::ConfigFilter => {
                vec![("d", "Rotate show done")]
            }
            _ => vec![],
        };

        frame.render_widget(
            StatusBar::new(
                keybinds
                    .into_iter()
                    .map(|(a, b)| (String::from(a), String::from(b)))
                    .collect(),
            ),
            status_bar_area,
        );
    }
}
