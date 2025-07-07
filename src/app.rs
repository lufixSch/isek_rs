use std::{fs, path::Path};

use color_eyre::eyre::Result;
use colors_transform::Rgb;
use eyre::eyre;
use icalendar::Calendar;
use ratatui::widgets::ListState;

use crate::config::{CalendarConfig, IsekConfig};

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
    pub ical: Calendar,
}

impl IsekCalendar {
    /// Create a new calendar instance from configuration
    pub fn from_config(cfg: CalendarConfig) -> Result<Self> {
        match cfg {
            CalendarConfig::VDIR(ref config) => {
                // Read all .ics files in directory
                let dir_path = Path::new(&config.path);
                let entries = fs::read_dir(dir_path)?;
                let mut cal = Calendar::new();

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

/// Main application state and logic
#[derive(Debug, Default)]
pub struct App {
    /// Flag to indicate if the application should exit
    pub exit: bool,
    /// List of loaded calendars with their data
    pub calendars: Vec<IsekCalendar>,

    /// State for list navigation (selection)
    pub list_state: ListState,
}

impl App {
    /// Create and initialize a new application instance
    pub fn new() -> Result<Self> {
        // Load configuration from file
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

    /// Set the exit flag to true, which will terminate the application loop
    pub fn exit(&mut self) {
        self.exit = true
    }

    /// Clear list selection (used for escape key)
    pub fn escape(&mut self) {
        self.list_state.select(None);
    }
}
