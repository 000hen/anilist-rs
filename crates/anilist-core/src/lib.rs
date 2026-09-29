use std::fmt;

use chrono::Weekday;
#[cfg(feature = "clock")]
use chrono::{Datelike, Local};

pub mod anime;
pub mod minute;
pub mod season;
pub mod time;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScheduleDay {
    Weekday(Weekday),
    Unknown,
}

impl fmt::Display for ScheduleDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ScheduleDay::Weekday(day) => format!("{}", day),
            ScheduleDay::Unknown => "時間未定".to_owned(),
        };

        f.write_str(&text)
    }
}

const WEEK_LIST: [ScheduleDay; 7] = [
    ScheduleDay::Weekday(Weekday::Sun),
    ScheduleDay::Weekday(Weekday::Mon),
    ScheduleDay::Weekday(Weekday::Tue),
    ScheduleDay::Weekday(Weekday::Wed),
    ScheduleDay::Weekday(Weekday::Thu),
    ScheduleDay::Weekday(Weekday::Fri),
    ScheduleDay::Weekday(Weekday::Sat),
];

pub fn get_week_order(today: Weekday) -> [ScheduleDay; 8] {
    let offset = today.num_days_from_sunday() as usize;
    std::array::from_fn(|index| {
        if index < WEEK_LIST.len() {
            WEEK_LIST[(index + offset) % WEEK_LIST.len()]
        } else {
            ScheduleDay::Unknown
        }
    })
}

#[cfg(feature = "clock")]
pub fn get_current_week_order() -> [ScheduleDay; 8] {
    get_week_order(Local::now().weekday())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "clock")]
    fn test_today_format() {
        let current_week = ScheduleDay::Weekday(Local::now().weekday());
        let ordered = get_current_week_order();

        assert_eq!(current_week, ordered[0]);
    }

    #[test]
    fn week_order_uses_the_callers_day_and_keeps_unknown_last() {
        let order = get_week_order(Weekday::Sat);
        assert_eq!(order[0], ScheduleDay::Weekday(Weekday::Sat));
        assert_eq!(order[1], ScheduleDay::Weekday(Weekday::Sun));
        assert_eq!(order[6], ScheduleDay::Weekday(Weekday::Fri));
        assert_eq!(order[7], ScheduleDay::Unknown);
    }
}
