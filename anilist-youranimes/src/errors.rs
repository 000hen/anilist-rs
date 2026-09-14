use anilist_core::time::ZoneConversionError;
use anilist_source::SourceError;

#[derive(Debug)]
pub enum YourAnimesError {
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
    Json {
        context: &'static str,
        source: serde_json::Error,
    },
    InvalidResponse {
        context: &'static str,
    },
    NextJs(anilist_nextjs::ParseError),
    AnimeConversion {
        anime_id: String,
        source: ZoneConversionError,
    },
}

impl From<anilist_nextjs::ParseError> for YourAnimesError {
    fn from(source: anilist_nextjs::ParseError) -> Self {
        Self::NextJs(source)
    }
}

impl From<YourAnimesError> for SourceError {
    fn from(error: YourAnimesError) -> Self {
        println!("Got error while parsing: {:?}", error);
        match error {
            YourAnimesError::Request { .. }
            | YourAnimesError::UnexpectedStatus { .. }
            | YourAnimesError::ResponseBody { .. } => SourceError::Unavailable,

            YourAnimesError::Json { context, .. }
            | YourAnimesError::InvalidResponse { context } => {
                SourceError::InvalidResponse { context }
            }

            YourAnimesError::AnimeConversion { anime_id, source } => {
                SourceError::AnimeConversion { anime_id, source }
            }
            YourAnimesError::NextJs(source) => SourceError::InvalidResponse {
                context: source.context(),
            },
        }
    }
}
