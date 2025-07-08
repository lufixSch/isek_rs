use std::{cmp::Ordering, collections::HashMap, fs, path::Path};

use chrono::Utc;
use color_eyre::eyre::Result;
use colors_transform::Rgb;
use eyre::eyre;
use icalendar::{Calendar, CalendarComponent, Component, Todo};
use ratatui::widgets::ListState;

use crate::config::{
    CalendarConfig, DisplayOptions, FilterConfig, IsekConfig, ShowDoneOptions, SortingConfig,
    SortingVariant,
};

#[derive(Debug)]
pub enum State {
    Normal,
    Interactive,
    ConfigSort,
    ConfigFilter,
}

#[derive(Debug)]
pub enum CalData {
    VDIR(HashMap<String, Calendar>),
}

/// Representation of a calendar with its configuration and data
#[derive(Debug)]
pub struct IsekCalendar {
    /// Configuration for this calendar
    pub config: CalendarConfig,
    /// Display name of the calendar
    pub name: String,
    /// Color associated with the calendar (used for display)
    pub color: Rgb,
    /// Parsed iCalendar data containing todos and events
    pub data: CalData,
}

impl IsekCalendar {
    /// Create a new calendar instance from configuration
    pub fn from_config(cfg: CalendarConfig) -> Result<Self> {
        match cfg {
            CalendarConfig::VDIR(ref config) => {
                // Read all .ics files in directory
                let dir_path = Path::new(&config.path);
                let entries = fs::read_dir(dir_path)?;
                let mut cal = HashMap::new();

                // Get calendar display name from file
                let name_path = dir_path.join("displayname");
                let name = fs::read_to_string(name_path)?;

                // Get and parse color configuration
                let color_path = dir_path.join("color");
                let color_string = fs::read_to_string(color_path)?;
                let color = Rgb::from_hex_str(&color_string).map_err(|err| {
                    eyre!(
                        "Unable to read color for calendar '{}': {}",
                        name,
                        err.message
                    )
                })?;

                // Process all .ics files in the directory
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
                                match contents.parse() {
                                    Ok(parsed_calendar) => {
                                        cal.insert(
                                            String::from(
                                                path.file_name().unwrap().to_str().unwrap(),
                                            ), // WARN: File name should always exist and be valid
                                            parsed_calendar,
                                        );
                                    }
                                    Err(e) => {
                                        eprintln!("Error parsing file {}: {}", path.display(), e)
                                    }
                                }
                            }
                            Err(e) => eprintln!("Error reading file {}: {}", path.display(), e),
                        }
                    }
                }

                Ok(Self {
                    name,
                    color,
                    config: cfg,
                    data: CalData::VDIR(cal),
                })
            }
        }
    }

    pub fn get_todos(
        &self,
        sort: Option<&SortingConfig>,
        filter: Option<&FilterConfig>,
    ) -> Vec<&Todo> {
        let mut todos: Vec<&Todo> = match &self.data {
            CalData::VDIR(cals) => cals
                .iter()
                .flat_map(|(_, cal)| {
                    cal.components
                        .iter()
                        .flat_map(|c| {
                            if let CalendarComponent::Todo(t) = c {
                                Some(t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<&Todo>>()
                })
                .collect(),
        };

        if let Some(filter) = filter {
            match filter.show_done {
                ShowDoneOptions::Hide => {
                    todos.retain(|t| t.get_completed().is_none());
                }
                ShowDoneOptions::Some => {
                    todos.retain(|t| {
                        let completed = t.get_completed();

                        if let Some(dt) = completed {
                            let diff = Utc::now() - dt;

                            diff.num_days() < filter.show_done_for as i64
                        } else {
                            true
                        }
                    });
                }
                ShowDoneOptions::Show => {}
            }
        }

        if let Some(sort) = sort {
            match sort.by {
                SortingVariant::Date => {
                    todos.sort_by(|a, b| {
                        let a_due = a.get_due();
                        let b_due = b.get_due();

                        if a.get_completed().is_some() {
                            if b.get_completed().is_some() {
                                return Ordering::Equal;
                            };

                            return Ordering::Greater;
                        }

                        if b.get_completed().is_some() {
                            return Ordering::Less;
                        }

                        if let Some(a_due) = a_due {
                            if let Some(b_due) = b_due {
                                return a_due.date_naive().cmp(&b_due.date_naive());
                            };

                            return Ordering::Less;
                        };

                        if b_due.is_none() {
                            return Ordering::Greater;
                        };

                        Ordering::Equal
                    });
                }
                SortingVariant::Priority => {
                    todos.sort_by(|a, b| {
                        let a_prio = a.get_priority().unwrap_or(10);
                        let b_prio = b.get_priority().unwrap_or(10);

                        if a.get_completed().is_some() {
                            if b.get_completed().is_some() {
                                return Ordering::Equal;
                            };

                            return Ordering::Greater;
                        }

                        if b.get_completed().is_some() {
                            return Ordering::Less;
                        }

                        a_prio.cmp(&b_prio)
                    });
                }
                _ => {}
            };

            if !sort.ascending {
                todos.reverse();
            }
        };

        todos
    }
}

/// Main application state and logic
#[derive(Debug)]
pub struct App {
    /// Flag to indicate if the application should exit
    pub exit: bool,

    /// Indicates the state, the application is in
    pub state: State,

    /// List of loaded calendars with their data
    pub calendars: Vec<IsekCalendar>,

    /// Settings defining how the data is displayed
    pub display: DisplayOptions,

    /// State for list navigation (selection)
    pub list_state: ListState,
}

impl App {
    /// Create and initialize a new application instance
    pub fn new() -> Result<Self> {
        // Load configuration from file
        let config: IsekConfig = confy::load("isek", "config")?;

        Ok(Self {
            exit: false,
            state: State::Normal,
            calendars: config
                .calendars
                .into_iter()
                .map(IsekCalendar::from_config)
                .collect::<Result<Vec<IsekCalendar>>>()?,
            display: config.display,
            list_state: ListState::default(),
        })
    }

    /// Switch to another application state
    pub fn switch_state(&mut self, state: State) {
        self.state = state
    }

    /// Set the exit flag to true, which will terminate the application loop
    pub fn exit(&mut self) {
        self.exit = true
    }

    /// Clear list selection (used for escape key)
    pub fn escape(&mut self) {
        self.list_state.select(None);
        self.state = State::Normal
    }

    pub fn configure_sort(&mut self, sort: SortingConfig) {
        self.display.sort = sort;
        self.escape();
    }

    pub fn configure_filter(&mut self, filter: FilterConfig) {
        self.display.filter = filter;
        self.escape();
    }
}
