use std::{error::Error, fmt};

#[cfg(feature = "http")]
use anilist_core::time::ZoneConversionError;
#[cfg(feature = "http")]
use anilist_source::error::{ParseError, SourceError};

#[derive(Debug)]
pub enum YourAnimesParseError {
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
    InvalidResponse {
        context: &'static str,
    },
    NextJs(anilist_nextjs::ParseError),
}

impl From<anilist_nextjs::ParseError> for YourAnimesParseError {
    fn from(source: anilist_nextjs::ParseError) -> Self {
        Self::NextJs(source)
    }
}

impl fmt::Display for YourAnimesParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json { context, source } => {
                write!(formatter, "invalid {context}: {source}")
            }
            Self::InvalidResponse { context } => formatter.write_str(context),
            Self::NextJs(source) => write!(formatter, "invalid YourAnimes page: {source}"),
        }
    }
}

impl Error for YourAnimesParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json { source, .. } => Some(source),
            Self::NextJs(source) => Some(source),
            Self::InvalidResponse { .. } => None,
        }
    }
}

#[cfg(feature = "http")]
#[derive(Debug)]
pub enum YourAnimesError {
    Timezone(ZoneConversionError),
    AnimeConversion {
        anime_id: String,
        source: ZoneConversionError,
    },
    Request {
        url: String,
        source: reqwest::Error,
    },
    UnexpectedStatus {
        url: String,
        status: reqwest::StatusCode,
    },
    ResponseBody {
        url: String,
        source: reqwest::Error,
    },
    Parse(YourAnimesParseError),
}

#[cfg(feature = "http")]
impl From<YourAnimesParseError> for YourAnimesError {
    fn from(source: YourAnimesParseError) -> Self {
        Self::Parse(source)
    }
}

#[cfg(feature = "http")]
impl fmt::Display for YourAnimesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Timezone(source) => write!(formatter, "cannot resolve system timezone: {source}"),
            Self::AnimeConversion { anime_id, source } => {
                write!(formatter, "failed to convert anime {anime_id}: {source}")
            }
            Self::Request { url, source } => write!(formatter, "request to {url} failed: {source}"),
            Self::UnexpectedStatus { url, status } => {
                write!(formatter, "unexpected HTTP status {status} from {url}")
            }
            Self::ResponseBody { url, source } => {
                write!(formatter, "failed to read response from {url}: {source}")
            }
            Self::Parse(source) => source.fmt(formatter),
        }
    }
}

#[cfg(feature = "http")]
impl Error for YourAnimesError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Timezone(source) | Self::AnimeConversion { source, .. } => Some(source),
            Self::Request { source, .. } | Self::ResponseBody { source, .. } => Some(source),
            Self::Parse(source) => Some(source),
            Self::UnexpectedStatus { .. } => None,
        }
    }
}

#[cfg(feature = "http")]
impl From<YourAnimesError> for SourceError {
    fn from(error: YourAnimesError) -> Self {
        match error {
            YourAnimesError::Request { .. }
            | YourAnimesError::UnexpectedStatus { .. }
            | YourAnimesError::ResponseBody { .. } => Self::Unavailable,

            YourAnimesError::Parse(source) => Self::Parse(source.into()),
            YourAnimesError::AnimeConversion { anime_id, source } => Self::AnimeConversion {
                anime_id,
                source: Box::new(source),
            },

            YourAnimesError::Timezone(source) => Self::TimezoneUnavailable { source },
        }
    }
}

impl From<YourAnimesParseError> for ParseError {
    fn from(error: YourAnimesParseError) -> Self {
        match error {
            YourAnimesParseError::Json { context, source } => ParseError::Decode {
                context,
                source: Box::new(source),
            },

            YourAnimesParseError::InvalidResponse { context } => {
                ParseError::InvalidFormat { context }
            }

            YourAnimesParseError::NextJs(source) => ParseError::Decode {
                context: source.context(),
                source: Box::new(source),
            },
        }
    }
}
