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
    use super::*;

    #[test]
    fn maps_months_to_seasons() {
        let seasons: Vec<_> = (1..=12).map(|month| AnimeSeason::try_from(month).unwrap()).collect();
        assert_eq!(seasons[..3], [AnimeSeason::Winter; 3]);
        assert_eq!(seasons[3..6], [AnimeSeason::Spring; 3]);
        assert_eq!(seasons[6..9], [AnimeSeason::Summer; 3]);
        assert_eq!(seasons[9..], [AnimeSeason::Fall; 3]);
        assert_eq!(AnimeSeason::try_from(0), Err(SeasonParseError::UnknownSeason));
        assert_eq!(AnimeSeason::try_from(13), Err(SeasonParseError::UnknownSeason));
    }
}
