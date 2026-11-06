use serde::de::DeserializeOwned;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Next.js hydration data was not found")]
    MissingData,
    #[error("Next.js target data was not found at {pointer}")]
    MissingPath { pointer: String },
    #[error("{context}: {source}")]
    Json {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("invalid Next.js Flight transport: {reason}")]
    Transport { reason: &'static str },
    #[error("invalid Next.js Flight record at byte {offset}: {reason}")]
    Record { offset: usize, reason: &'static str },
}

pub(crate) fn json<T: DeserializeOwned>(
    input: &str,
    context: &'static str,
) -> Result<T, ParseError> {
    serde_json::from_str(input).map_err(|source| ParseError::Json { context, source })
}
