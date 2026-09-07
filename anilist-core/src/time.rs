use std::{cmp::Ordering, error::Error, fmt};

use chrono::{
    Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, Weekday, offset::LocalResult,
};
use chrono_tz::Tz;

use crate::minute::Minute;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneConversionError {
    AmbiguousLocalTime,
    NonexistentLocalTime,
}

impl fmt::Display for ZoneConversionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AmbiguousLocalTime => formatter.write_str("local time is ambiguous"),
            Self::NonexistentLocalTime => formatter.write_str("local time does not exist"),
        }
    }
}

impl Error for ZoneConversionError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimeTime {
    pub week: Weekday,
    pub minute: Option<Minute>,
    pub zone: Tz,
}

impl AnimeTime {
    pub fn minute_of_week(&self) -> Option<u16> {
        self.minute
            .map(|minute| self.week.num_days_from_monday() as u16 * 1440 + minute.get())
    }

    pub fn to_zone(self, zone: Tz, reference: NaiveDate) -> Result<Self, ZoneConversionError> {
        let Some(minute) = self.minute else {
            return Ok(Self {
                week: self.week,
                minute: None,
                zone: zone,
            });
        };

        let monday = reference - Duration::days(reference.weekday().num_days_from_monday() as i64);
        let source_date = monday + Duration::days(self.week.num_days_from_monday() as i64);
        let source_time =
            NaiveTime::from_hms_opt((minute.get() / 60) as u32, (minute.get() % 60) as u32, 0)
                .expect("validate minute must produce a valid time");

        let source_naive = source_date.and_time(source_time);
        let source = match self.zone.from_local_datetime(&source_naive) {
            LocalResult::Single(value) => value,
            LocalResult::Ambiguous(_, _) => {
                return Err(ZoneConversionError::AmbiguousLocalTime);
            }
            LocalResult::None => {
                return Err(ZoneConversionError::NonexistentLocalTime);
            }
        };

        let target = source.with_timezone(&zone);
        let target_minute = Minute::new((target.hour() * 60 + target.minute()) as u16);

        Ok(Self {
            week: target.weekday(),
            minute: target_minute,
            zone: zone,
        })
    }

    pub fn airing_cmp(a: &Self, b: &Self) -> Ordering {
        (
            a.week.num_days_from_monday(),
            a.minute.unwrap_or(Minute::MAX),
        )
            .cmp(&(
                b.week.num_days_from_monday(),
                b.minute.unwrap_or(Minute::MAX),
            ))
    }
}

impl Ord for AnimeTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.week
            .num_days_from_monday()
            .cmp(&other.week.num_days_from_monday())
            .then_with(|| match (self.minute, other.minute) {
                (Some(a), Some(b)) => a.cmp(&b),
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (None, None) => Ordering::Equal,
            })
    }
}

impl PartialOrd for AnimeTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
