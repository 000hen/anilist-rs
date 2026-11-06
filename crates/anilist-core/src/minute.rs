use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minute(u16);

impl Minute {
    pub fn new(value: u16) -> Option<Self> {
        (value < 1440).then_some(Self(value))
    }

    pub fn get(self) -> u16 {
        self.0
    }
}

impl fmt::Display for Minute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.0 / 60, self.0 % 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_minutes_within_a_day_only() {
        assert_eq!(Minute::new(1439).map(Minute::get), Some(1439));
        assert_eq!(Minute::new(1440), None);
    }

    #[test]
    fn displays_as_hours_and_minutes() {
        assert_eq!(Minute::new(9 * 60 + 5).unwrap().to_string(), "09:05");
    }
}
