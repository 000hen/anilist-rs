use anilist_source::error::{ParseError, SourceError};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum AnilistError {
    #[error("unsupported anime source: {source_id}")]
    UnsupportedSource { source_id: String },

    #[error("the source is unavailable")]
    SourceUnavailable,

    #[error("invalid source response: {message}")]
    InvalidResponse { message: String },

    #[error("system timezone is unavailable: {message}")]
    TimezoneUnavailable { message: String },

    #[error("failed to convert anime {anime_id}: {message}")]
    AnimeConversion { anime_id: String, message: String },
}

impl From<ParseError> for AnilistError {
    fn from(error: ParseError) -> Self {
        Self::InvalidResponse {
            message: error.to_string(),
        }
    }
}

#[cfg(feature = "http")]
impl From<SourceError> for AnilistError {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::Unavailable => Self::SourceUnavailable,
            SourceError::Parse(source) => source.into(),
            SourceError::TimezoneUnavailable { source } => Self::TimezoneUnavailable {
                message: source.to_string(),
            },

            SourceError::AnimeConversion { anime_id, source } => Self::AnimeConversion {
                anime_id,
                message: source.to_string(),
            },
        }
    }
}
