use ratatui::{
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget},
};

use crate::{App, app::IsekCalendar};

/// State for the todo list widget that tracks which calendars are displayed and list navigation state
pub struct ToDoListState {
    pub calendars: Vec<IsekCalendar>,
    pub list: ListState,
}

/// ToDo List Widget
#[derive(Default)]
pub struct ToDoList<'a> {
    block: Option<Block<'a>>,
}

impl<'a> ToDoList<'a> {
    /// Configure the block (border and title) for this widget
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for ToDoList<'_> {
    type State = App;

    fn render(
        self,
        area: ratatui::prelude::Rect,
        buf: &mut ratatui::prelude::Buffer,
        state: &mut Self::State,
    ) {
        let sort = Some(&state.display.sort);
        let filter = Some(&state.display.filter);

        // Generate list items from the application's calendar data
        let todos = state.calendars.get_todos(sort, filter);

        let items = todos
            .iter()
            .map(|t| Line::from(t.format(state)))
            .collect::<Vec<Line>>();

        // Configure and render the list widget
        let mut list = List::new(items);

        if let Some(block) = self.block {
            list = list.block(block)
        }

        list.highlight_style(Style::new().bold())
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .render(area, buf, &mut state.list_state);
    }
}
