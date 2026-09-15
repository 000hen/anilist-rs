use anilist_source::error::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum YourAnimesParseError {
    #[error("invalid {context}")]
    Json {
        context: &'static str,

        #[source]
        source: serde_json::Error,
    },

    #[error("{context}")]
    InvalidResponse { context: &'static str },

    #[error("invalid YourAnimes page")]
    NextJs(#[from] anilist_nextjs::ParseError),
}

impl From<YourAnimesParseError> for ParseError {
    fn from(error: YourAnimesParseError) -> Self {
        match error {
            YourAnimesParseError::Json { context, source } => Self::Decode {
                context,
                source: Box::new(source),
            },

            YourAnimesParseError::InvalidResponse { context } => Self::InvalidResponse { context },

            YourAnimesParseError::NextJs(source) => Self::Decode {
                context: source.context(),
                source: Box::new(source),
            },
        }
    }
}
