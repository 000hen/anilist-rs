use std::error::Error;

use anilist_core::time::ZoneConversionError;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid response format: {context}")]
    InvalidFormat { context: &'static str },

    #[error("required data is missing: {context}")]
    MissingData { context: &'static str },

    #[error("invalid value for {field}: {value}")]
    InvalidValue { field: &'static str, value: String },

    #[error("failed to decode response: {context}")]
    Decode {
        context: &'static str,

        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("the source is unavailable this time")]
    Unavailable,

    #[error("failed to parse source response: {0}")]
    Parse(#[from] ParseError),

    #[error("system timezone is unavailable: {source}")]
    TimezoneUnavailable { source: ZoneConversionError },

    #[error("failed to convert anime {anime_id}: {source}")]
    AnimeConversion {
        anime_id: String,

        #[source]
        source: Box<dyn Error + Send + Sync>,
    },
}
