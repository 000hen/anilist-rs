use std::{error::Error, fmt};

use anilist_core::{anime::Anime, season::AnimeSeason, time::ZoneConversionError};

#[derive(Debug)]
pub enum SourceError {
    Unavailable,
    InvalidResponse {
        context: &'static str,
    },
    AnimeConversion {
        anime_id: String,
        source: ZoneConversionError,
    },
}

impl fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => {
                write!(formatter, "the source is unavailable this time")
            }
            Self::InvalidResponse { context } => {
                write!(formatter, "invalid source response: {context}")
            }
            Self::AnimeConversion { anime_id, source } => {
                write!(formatter, "failed to convert anime {anime_id}: {source}")
            }
        }
    }
}

impl Error for SourceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::AnimeConversion { source, .. } => Some(source),
            Self::Unavailable { .. } | Self::InvalidResponse { .. } => None,
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait AnimeSource {
    async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError>;
    async fn search(&self, keyword: String) -> Result<Vec<Anime>, SourceError>;
    async fn detail(&self, id: String) -> Result<Vec<Anime>, SourceError>;
}
