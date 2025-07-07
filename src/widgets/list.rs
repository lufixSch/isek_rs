use colors_transform::Color;
use eyre::ContextCompat;
use icalendar::{CalendarComponent, Component};
use ratatui::{
    style::{self, Style, Stylize},
    text::Line,
    widgets::{Block, List, ListState, StatefulWidget},
};

use crate::{App, app::IsekCalendar};

pub struct ToDoListState {
    pub calendars: Vec<IsekCalendar>,
    pub list: ListState,
}

#[derive(Default)]
pub struct ToDoList<'a> {
    block: Option<Block<'a>>,
}

impl<'a> ToDoList<'a> {
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
        let items = state
            .calendars
            .iter()
            .flat_map(|cal| {
                cal.ical
                    .components
                    .iter()
                    .flat_map(|c| {
                        if let CalendarComponent::Todo(t) = c {
                            Some(t)
                        } else {
                            None
                        }
                    })
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
                                Some(dt) => format!(" {}", dt.date_naive().format("%Y-%m-%d"))
                                    .fg(style::Color::Blue),
                                None => "".into(),
                            },
                        ])
                    })
                    .collect::<Vec<Line>>()
            })
            .collect::<Vec<Line>>();

        let mut list = List::new(items);

        if let Some(block) = self.block {
            list = list.block(block)
        }

        list.highlight_style(Style::new().bold())
            .highlight_symbol("> ")
            .repeat_highlight_symbol(true)
            .render(area, buf, &mut state.list_state);
    }
}
