use serde::{Deserialize, Serialize};

/// Configuration structure for the application
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IsekConfig {
    /// List of calendar configurations
    pub calendars: Vec<CalendarConfig>,
}

impl Default for IsekConfig {
    fn default() -> Self {
        // Create a default configuration with one VDIR calendar
        Self { calendars: vec![
            CalendarConfig::VDIR(VdirCalendarConfig {
                path: "/home/lukas/calendars/personal".into()
            })
        ] }
    }
}

/// Enum representing different types of calendar configurations
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CalendarConfig {
    /// VDIR calendar configuration variant
    VDIR(VdirCalendarConfig),
}

/// Configuration for a VDIR-based calendar
#[derive(Debug, Serialize,Deserialize,Clone)]
pub struct VdirCalendarConfig {
    /// Filesystem path to the calendar directory
    pub path: String,
}
