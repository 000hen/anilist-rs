use anilist_source::SourceError;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum AnilistError {
    #[error("the source is unavailable")]
    SourceUnavailable,

    #[error("invalid source response: {context}")]
    InvalidResponse { context: String },

    #[error("failed to convert anime {anime_id}: {message}")]
    AnimeConversion { anime_id: String, message: String },
}

impl From<SourceError> for AnilistError {
    fn from(value: SourceError) -> Self {
        match value {
            SourceError::Unavailable => Self::SourceUnavailable,

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
