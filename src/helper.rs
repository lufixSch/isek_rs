use chrono::{DateTime, Local, NaiveDateTime, NaiveTime, Utc, offset::LocalResult};
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
                    Some(dt) => format!(
                        "{}",
                        dt.with_timezone(&Local::now().timezone()).format(dt_fmt)
                    ),
                    None => format!("{} {}", date_time.format(dt_fmt), tzid.clone()),
                }
            }
        },
    }
}

// Creates timezone aware Datetime from ical datetime object
pub fn ical_datetime_to_chrono(dt: DatePerhapsTime) -> DateTime<Utc> {
    match dt {
        DatePerhapsTime::Date(dt) => dt.and_time(NaiveTime::default()).and_utc(),
        DatePerhapsTime::DateTime(dt) => match dt {
            CalendarDateTime::Floating(dt) => dt.and_utc(),
            CalendarDateTime::Utc(dt) => dt,
            CalendarDateTime::WithTimezone { date_time, tzid } => {
                match dt_with_timezone(date_time, &tzid) {
                    Some(dt) => dt.to_utc(),
                    None => date_time.and_utc(),
                }
            }
        },
    }
}

// Tries to convert a naive datetime into an timezone aware datetime based on a timezone ID
pub fn dt_with_timezone(dt: NaiveDateTime, tzid: &str) -> Option<DateTime<Tz>> {
    let tz: Tz = tzid.parse().ok()?;
    dt.and_local_timezone(tz).earliest()
}

pub fn calculate_index(priority: &u32, datetime: &DateTime<Utc>, now: &DateTime<Utc>) -> f64 {
    let diff = *datetime - now;

    (diff.as_seconds_f64() / 500e3).tanh() / 2.0 * 0.5 * (*priority as f64)
}
