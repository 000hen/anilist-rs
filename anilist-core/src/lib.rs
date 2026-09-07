use std::fmt;

use chrono::{Datelike, Local, Weekday};

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

pub fn get_current_week_order() -> [ScheduleDay; 8] {
    let offset = Local::now().weekday().num_days_from_sunday() as usize;
    std::array::from_fn(|index| {
        if index < WEEK_LIST.len() {
            WEEK_LIST[(index + offset) % WEEK_LIST.len()]
        } else {
            ScheduleDay::Unknown
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_today_format() {
        let current_week = ScheduleDay::Weekday(Local::now().weekday());
        let ordered = get_current_week_order();

        assert_eq!(current_week, ordered[0]);
    }
}
