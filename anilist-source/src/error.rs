use std::error::Error;

use anilist_core::time::ZoneConversionError;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid source response: {context}")]
    InvalidResponse { context: &'static str },

    #[error("failed to decode source response: {context}")]
    Decode {
        context: &'static str,

        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("the source is unavailable")]
    Unavailable,

    #[error(transparent)]
    Parse(#[from] ParseError),

    #[error("system timezone is unavailable: {source}")]
    TimezoneUnavailable {
        #[source]
        source: ZoneConversionError,
    },

    #[error("failed to convert anime {anime_id}: {source}")]
    AnimeConversion {
        anime_id: String,

        #[source]
        source: ZoneConversionError,
    },
}
