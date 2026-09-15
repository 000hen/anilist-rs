#[cfg(feature = "http")]
use anilist_source::SourceError;
use anilist_youranimes::YourAnimesParseError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum AnilistError {
    #[error("the source is unavailable")]
    SourceUnavailable,

    #[error("system timezone is unavailable: {message}")]
    TimezoneUnavailable { message: String },

    #[error("invalid source response: {context}")]
    InvalidResponse { context: String },

    #[error("failed to convert anime {anime_id}: {message}")]
    AnimeConversion { anime_id: String, message: String },
}

#[cfg(feature = "http")]
impl From<SourceError> for AnilistError {
    fn from(value: SourceError) -> Self {
        match value {
            SourceError::Unavailable => Self::SourceUnavailable,
            SourceError::TimezoneUnavailable { source } => Self::TimezoneUnavailable {
                message: source.to_string(),
            },

            SourceError::InvalidResponse { context } => Self::InvalidResponse {
                context: context.to_owned(),
            },

            SourceError::AnimeConversion { anime_id, source } => Self::AnimeConversion {
                anime_id,
                message: source.to_string(),
            },
        }
    }
}

impl From<YourAnimesParseError> for AnilistError {
    fn from(value: YourAnimesParseError) -> Self {
        Self::InvalidResponse {
            context: value.to_string(),
        }
    }
}

#[cfg(all(test, feature = "http"))]
mod tests {
    use super::*;

    #[test]
    fn timezone_lookup_failure_is_not_a_malformed_response() {
        let error = AnilistError::from(SourceError::TimezoneUnavailable {
            source: anilist_core::time::ZoneConversionError::UnknownTimeZone,
        });
        assert!(matches!(error, AnilistError::TimezoneUnavailable { .. }));
        assert!(error.to_string().contains("unknown IANA timezone"));
    }
}
