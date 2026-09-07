#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minute(u16);

impl Minute {
    pub const ZERO: Self = Minute(0);
    pub const MAX: Self = Minute(1439);

    pub fn new(value: u16) -> Option<Self> {
        if (0..1440).contains(&value) {
            Some(Self(value))
        } else {
            None
        }
    }

    pub fn get(self) -> u16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_parse() {
        let noon = 12 * 60 as u16;
        let minute = Minute::new(noon).unwrap();

        assert_eq!(minute.get(), noon);
    }

    #[test]
    fn test_over_minute() {
        let over: u16 = 1440;
        let minute = Minute::new(over);

        assert_eq!(minute, None);
    }

    #[test]
    fn test_u16_max() {
        let max: u16 = u16::MAX;
        let minute = Minute::new(max);

        assert_eq!(minute, None);
    }
}
