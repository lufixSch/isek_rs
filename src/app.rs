use std::{fs, path::Path};

use color_eyre::eyre::Result;
use colors_transform::{Color, Rgb};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use eyre::{eyre, ContextCompat};
use icalendar::{Calendar, CalendarComponent, Component};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::Rect,
    style::{self, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{Block, List, Widget},
};

use crate::config::{CalendarConfig, IsekConfig};

#[derive(Debug)]
pub struct IsekCalendar {
    pub config: CalendarConfig,
    pub name: String,
    pub color: Rgb,
    pub ical: Calendar,
}

impl IsekCalendar {
    pub fn from_config(cfg: CalendarConfig) -> Result<Self> {
        match cfg {
            CalendarConfig::VDIR(ref config) => {
                // Read all .ics files in directory
                let dir_path = Path::new(&config.path);
                let entries = fs::read_dir(dir_path)?;
                let mut cal = Calendar::new();

                let name_path = dir_path.join("displayname");
                let name = fs::read_to_string(name_path)?;

                let color_path = dir_path.join("color");
                let color_string = fs::read_to_string(color_path)?;
                let color = Rgb::from_hex_str(&color_string).map_err(|err| {
                    eyre!(
                        "Unable to read color for calendar '{}': {}",
                        name,
                        err.message
                    )
                })?;

                for entry in entries.flatten() {
                    let path = entry.path();

                    // Check if it is a file and has .ics extension
                    if path.is_file()
                        && path
                            .extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("ics"))
                    {
                        match fs::read_to_string(&path) {
                            Ok(contents) => {
                                // Parse iCalendar content
                                let mut parsed_calendar: Calendar = contents.parse().unwrap();
                                cal.append(&mut parsed_calendar);
                            }
                            Err(e) => eprintln!("Error reading file {}: {}", path.display(), e),
                        }
                    }
                }

                Ok(Self {
                    name,
                    color,
                    config: cfg,
                    ical: cal,
                })
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    calendars: Vec<IsekCalendar>,
}

impl App {
    pub fn new() -> Result<Self> {
        let config: IsekConfig = confy::load("isek", "config")?;

        Ok(Self {
            calendars: config
                .calendars
                .into_iter()
                .map(IsekCalendar::from_config)
                .collect::<Result<Vec<IsekCalendar>>>()?,
            ..Default::default()
        })
    }

    pub fn tui(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;

            self.handle_events()?;
        }

        Ok(())
    }

    pub fn exit(&mut self) {
        self.exit = true
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from(" ISEK ".bold());

        let block = Block::bordered()
            .title(title.centered())
            .border_set(border::ROUNDED);

        let items = self
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
                            "- ".into(),
                            format!(" {} ", cal.name).bg(style::Color::Rgb(
                                cal.color.get_red() as u8,
                                cal.color.get_green() as u8,
                                cal.color.get_blue() as u8,
                            )),
                            " ".into(),
                            t.get_summary().wrap_err_with(|| format!("No summary (e.g. title) for some ToDo in {}", cal.name)).unwrap().into(),
                        ])
                    })
                    .collect::<Vec<Line>>()
            })
            .collect::<Vec<Line>>();

        List::new(items)
            .block(block)
            .highlight_style(Style::new().bold())
            .highlight_symbol(">")
            .repeat_highlight_symbol(true)
            .render(area, buf);
    }
}
