use std::cmp::Ordering;

use chrono::Weekday;
#[cfg(feature = "timezone")]
use chrono::{Datelike, Duration, NaiveDate, NaiveTime, TimeZone, Timelike, offset::LocalResult};
#[cfg(feature = "timezone")]
use chrono_tz::Tz;

use crate::minute::Minute;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ZoneConversionError {
    #[error("unknown time zone")]
    UnknownTimeZone,

    #[error("local time is ambiguous")]
    AmbiguousLocalTime,

    #[error("local time does not exist")]
    NonexistentLocalTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnimeTime {
    pub week: Weekday,
    pub minute: Option<Minute>,
    /// IANA timezone name; keeping the identifier needs no timezone database.
    pub zone: String,
}

impl AnimeTime {
    pub fn minute_of_week(&self) -> Option<u16> {
        self.minute
            .map(|minute| self.week.num_days_from_monday() as u16 * 1440 + minute.get())
    }

    #[cfg(feature = "timezone")]
    pub fn to_zone(self, zone: Tz, reference: NaiveDate) -> Result<Self, ZoneConversionError> {
        let Some(minute) = self.minute else {
            return Ok(Self {
                week: self.week,
                minute: None,
                zone: zone.name().to_owned(),
            });
        };

        let monday = reference - Duration::days(reference.weekday().num_days_from_monday() as i64);
        let source_date = monday + Duration::days(self.week.num_days_from_monday() as i64);
        let source_time =
            NaiveTime::from_hms_opt((minute.get() / 60) as u32, (minute.get() % 60) as u32, 0)
                .expect("validate minute must produce a valid time");

        let source_naive = source_date.and_time(source_time);
        let source_zone: Tz = self
            .zone
            .parse()
            .map_err(|_| ZoneConversionError::UnknownTimeZone)?;
        let source = match source_zone.from_local_datetime(&source_naive) {
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
            zone: zone.name().to_owned(),
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

#[cfg(all(test, feature = "timezone"))]
mod tests {
    use chrono::{NaiveDate, Weekday};
    use chrono_tz::Asia::{Taipei, Tokyo};

    use super::{AnimeTime, ZoneConversionError};
    use crate::minute::Minute;

    fn anime_time(week: Weekday, minute: Option<u16>, zone: &str) -> AnimeTime {
        AnimeTime {
            week,
            minute: minute.and_then(Minute::new),
            zone: zone.to_owned(),
        }
    }

    #[test]
    fn converts_tokyo_monday_after_midnight_to_taipei_sunday() {
        let converted = anime_time(Weekday::Mon, Some(30), "Asia/Tokyo")
            .to_zone(Taipei, NaiveDate::from_ymd_opt(2026, 7, 6).unwrap())
            .unwrap();

        assert_eq!(converted.week, Weekday::Sun);
        assert_eq!(converted.minute.map(Minute::get), Some(23 * 60 + 30));
        assert_eq!(converted.zone, "Asia/Taipei");
    }

    #[test]
    fn rejects_nonexistent_new_york_dst_local_time() {
        let error = anime_time(Weekday::Sun, Some(2 * 60 + 30), "America/New_York")
            .to_zone(Tokyo, NaiveDate::from_ymd_opt(2026, 3, 8).unwrap())
            .unwrap_err();

        assert_eq!(error, ZoneConversionError::NonexistentLocalTime);
    }

    #[test]
    fn rejects_ambiguous_new_york_dst_local_time() {
        let error = anime_time(Weekday::Sun, Some(60 + 30), "America/New_York")
            .to_zone(Tokyo, NaiveDate::from_ymd_opt(2026, 11, 1).unwrap())
            .unwrap_err();

        assert_eq!(error, ZoneConversionError::AmbiguousLocalTime);
    }

    #[test]
    fn rejects_unknown_source_timezone() {
        let error = anime_time(Weekday::Mon, Some(30), "Unknown/Timezone")
            .to_zone(Tokyo, NaiveDate::from_ymd_opt(2026, 7, 6).unwrap())
            .unwrap_err();

        assert_eq!(error, ZoneConversionError::UnknownTimeZone);
    }

    #[test]
    fn retains_unknown_minutes_without_needing_a_source_timezone() {
        let converted = anime_time(Weekday::Mon, None, "Unknown/Timezone")
            .to_zone(Taipei, NaiveDate::from_ymd_opt(2026, 7, 6).unwrap())
            .unwrap();

        assert_eq!(converted.week, Weekday::Mon);
        assert_eq!(converted.minute, None);
        assert_eq!(converted.zone, "Asia/Taipei");
    }
}
