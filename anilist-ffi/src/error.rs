use anilist_source::error::{ParseError, SourceError};

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum AnilistError {
    #[error("unsupported anime source: {source_id}")]
    UnsupportedSource { source_id: String },

    #[error("the source is unavailable")]
    SourceUnavailable,

    #[error("invalid source response: {detail}")]
    InvalidResponse { detail: String },

    #[error("system timezone is unavailable: {detail}")]
    TimezoneUnavailable { detail: String },

    #[error("failed to convert anime {anime_id}: {detail}")]
    AnimeConversion { anime_id: String, detail: String },
}

impl From<ParseError> for AnilistError {
    fn from(error: ParseError) -> Self {
        Self::InvalidResponse {
            detail: error.to_string(),
        }
    }
}

impl From<SourceError> for AnilistError {
    fn from(error: SourceError) -> Self {
        match error {
            SourceError::Unavailable => Self::SourceUnavailable,
            SourceError::Parse(source) => source.into(),
            SourceError::TimezoneUnavailable { source } => Self::TimezoneUnavailable {
                detail: source.to_string(),
            },

            SourceError::AnimeConversion { anime_id, source } => Self::AnimeConversion {
                anime_id,
                detail: source.to_string(),
            },
        }
    }
}
