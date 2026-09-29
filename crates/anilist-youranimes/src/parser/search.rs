use crate::{ID_PREFIX, errors::YourAnimesParseError, format::search::SearchResult};

/// Parses search response IDs. Search responses do not contain enough fields
/// to construct domain anime records; callers must hydrate each returned ID.
pub fn parse_search(content: &str) -> Result<Vec<String>, YourAnimesParseError> {
    let result = serde_json::from_str::<SearchResult>(content).map_err(|source| {
        YourAnimesParseError::Json {
            context: "YourAnimes search response",
            source,
        }
    })?;

    Ok(result
        .result
        .into_iter()
        .map(|item| format!("{ID_PREFIX}:{}", item.id))
        .collect())
}
