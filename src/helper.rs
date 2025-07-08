use chrono::{DateTime, Local, NaiveDateTime, offset::LocalResult};
use chrono_tz::Tz;
use icalendar::{CalendarDateTime, DatePerhapsTime};

/// Formats provided datetime object
pub fn format_ical_datetime(dt: DatePerhapsTime, date_fmt: &str, dt_fmt: &str) -> String {
    match dt {
        DatePerhapsTime::Date(dt) => {
            format!("{}", dt.format(date_fmt))
        }
        DatePerhapsTime::DateTime(dt) => match dt {
            CalendarDateTime::Floating(dt) => {
                format!("{}", dt.format(dt_fmt))
            }
            CalendarDateTime::Utc(dt) => {
                format!(
                    "{}",
                    dt.with_timezone(&Local::now().timezone()).format(dt_fmt)
                )
            }
            CalendarDateTime::WithTimezone { date_time, tzid } => {
                match dt_with_timezone(date_time, &tzid) {
                    Ok(dt) => format!(
                        "{}",
                        dt.with_timezone(&Local::now().timezone()).format(dt_fmt)
                    ),
                    Err(dt) => format!("{} {}", dt.format(dt_fmt), tzid.clone()),
                }
            }
        },
    }
}

// Tries 
pub fn dt_with_timezone(dt: NaiveDateTime, tzid: &str) -> Result<DateTime<Tz>, NaiveDateTime> {
    let tz: Tz = tzid.parse().map_err(|_| dt)?;

    if let LocalResult::Single(dt) = dt.and_local_timezone(tz) {
        Ok(dt)
    } else {
        Err(dt)
    }
}
