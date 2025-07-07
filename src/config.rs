use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IsekConfig {
    pub calendars: Vec<CalendarConfig>,
}

impl Default for IsekConfig {
    fn default() -> Self {
        Self { calendars: vec![
            CalendarConfig::VDIR(VdirCalendarConfig {
                path: "/home/lukas/calendars/personal".into()
            })
        ] }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CalendarConfig {
    VDIR(VdirCalendarConfig),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VdirCalendarConfig {
    pub path: String,
}
