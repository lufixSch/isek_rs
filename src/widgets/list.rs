use colors_transform::Color;
use eyre::ContextCompat;
use icalendar::Component;
use ratatui::{
    style::{self, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget},
};

use crate::{App, app::IsekCalendar, helper::format_ical_datetime};

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
        // Generate list items from the application's calendar data
        let items = state
            .calendars
            .iter()
            .flat_map(|cal| {
                cal.get_todos(Some(&state.display.sort), Some(&state.display.filter))
                    .iter()
                    .map(|t| {
                        Line::from(vec![
                            match t.get_completed() {
                                Some(_) => "[X] ",
                                None => "[ ] ",
                            }
                            .into(),
                            format!(" {} ", cal.name).bg(style::Color::Rgb(
                                cal.color.get_red() as u8,
                                cal.color.get_green() as u8,
                                cal.color.get_blue() as u8,
                            )),
                            " ".into(),
                            t.get_summary()
                                .wrap_err_with(|| {
                                    format!("No summary (e.g. title) for some ToDo in {}", cal.name)
                                })
                                .unwrap()
                                .into(),
                            match t.get_due() {
                                Some(dt) => format!(
                                    " {}",
                                    format_ical_datetime(
                                        dt,
                                        &state.display.date_format.date,
                                        &state.display.date_format.datetime
                                    )
                                )
                                .fg(style::Color::Blue),
                                None => "".into(),
                            },
                        ])
                    })
                    .collect::<Vec<Line>>()
            })
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
