use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeasonParseError {
    UnknownSeason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimeSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl TryFrom<u8> for AnimeSeason {
    type Error = SeasonParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1..=3 => Ok(AnimeSeason::Winter),
            4..=6 => Ok(AnimeSeason::Spring),
            7..=9 => Ok(AnimeSeason::Summer),
            10..=12 => Ok(AnimeSeason::Fall),
            _ => Err(SeasonParseError::UnknownSeason),
        }
    }
}

impl fmt::Display for AnimeSeason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            AnimeSeason::Fall => "秋季",
            AnimeSeason::Spring => "春季",
            AnimeSeason::Summer => "夏季",
            AnimeSeason::Winter => "冬季",
        };

        f.write_str(text)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::RangeInclusive;

    use super::*;

    fn test_and_expect(range: RangeInclusive<u8>, expect: AnimeSeason) {
        for month in range {
            let season = AnimeSeason::try_from(month).unwrap();
            assert_eq!(season, expect);
        }
    }

    #[test]
    fn test_normal_parse() {
        test_and_expect(1..=3, AnimeSeason::Winter);
        test_and_expect(4..=6, AnimeSeason::Spring);
        test_and_expect(7..=9, AnimeSeason::Summer);
        test_and_expect(10..=12, AnimeSeason::Fall);
    }

    #[test]
    fn test_out_of_range() {
        let month: u8 = 13;
        let season = AnimeSeason::try_from(month);

        assert_eq!(season, Err(SeasonParseError::UnknownSeason))
    }

    #[test]
    fn test_u8_max() {
        let max = u8::MAX;
        let season = AnimeSeason::try_from(max);

        assert_eq!(season, Err(SeasonParseError::UnknownSeason))
    }
}
