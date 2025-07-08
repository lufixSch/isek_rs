use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Configuration structure for the application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IsekConfig {
    /// List of calendar configurations
    pub calendars: Vec<CalendarConfig>,

    /// Default display Options
    pub display: DisplayOptions,
}

impl Default for IsekConfig {
    fn default() -> Self {
        // Create a default configuration with one VDIR calendar
        Self {
            calendars: vec![
                CalendarConfig::VDIR(VdirCalendarConfig {
                    path: "path/a".into(),
                }),
                CalendarConfig::VDIR(VdirCalendarConfig {
                    path: "path/b".into(),
                }),
            ],
            display: DisplayOptions {
                sort: SortingConfig {
                    by: SortingVariant::Priority,
                    ascending: true,
                    ignore_done: true,
                },
                filter: FilterConfig {
                    show_done: ShowDoneOptions::Hide,
                    show_done_for: 5,
                },
                date_format: DateFormatConfig {
                    date: "%Y-%m-%d".into(),
                    datetime: "%Y-%m-%d %H:%M".into(),
                },
            },
        }
    }
}

/// Enum representing different types of calendar configurations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CalendarConfig {
    /// VDIR calendar configuration variant
    VDIR(VdirCalendarConfig),
}

/// Configuration for a VDIR-based calendar
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VdirCalendarConfig {
    /// Filesystem path to the calendar directory
    pub path: String,
}

/// Structure representing the display configuration options
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayOptions {
    /// Sorting configuration used in the UI or data processing
    pub sort: SortingConfig,
    /// Date formatting settings for how dates and datetimes are displayed
    pub date_format: DateFormatConfig,

    pub filter: FilterConfig,
}

/// Enum representing different sorting variants for tasks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SortingVariant {
    /// Sort by date
    Date,
    /// Sort by priority
    Priority,
    /// Sort by ISEK index
    Index,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SortingConfig {
    pub by: SortingVariant,
    pub ascending: bool,
    pub ignore_done: bool,
}

/// Structure representing the date formatting options
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DateFormatConfig {
    /// Format string for date-only representation (e.g., "%Y-%m-%d")
    pub date: String,
    /// Format string for datetime representation (e.g., "%Y-%m-%d %H:%M")
    pub datetime: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ShowDoneOptions {
    Show,
    Some,
    Hide,
}

impl ShowDoneOptions {
    pub fn next(&self) -> Self {
        match self {
            ShowDoneOptions::Show => ShowDoneOptions::Some,
            ShowDoneOptions::Some => ShowDoneOptions::Hide,
            ShowDoneOptions::Hide => ShowDoneOptions::Show,
        }
    }
}

impl Display for ShowDoneOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShowDoneOptions::Show => write!(f, "Show"),
            ShowDoneOptions::Some => write!(f, "Some"),
            ShowDoneOptions::Hide => write!(f, "Hide"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FilterConfig {
    pub show_done: ShowDoneOptions,

    // Shows tasks done for less than x days if ShowDoneOptions::Some
    pub show_done_for: u64,
}
