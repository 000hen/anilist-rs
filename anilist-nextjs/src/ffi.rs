use crate::NextJsData;

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum NextJsError {
    #[error("{message}")]
    InvalidData { message: String },
}

/// Deserialize Next.js hydration scripts from HTML into a JSON string.
///
/// The result is `{"nextData": object_or_null, "flight": [...]}`. Each Flight
/// record contains `id` (number or null), `tag` (string or null), and `value`.
/// Callers can deserialize the JSON using their own language's JSON library.
#[uniffi::export]
pub fn deserialize_nextjs(content: String) -> Result<String, NextJsError> {
    let data = NextJsData::parse(&content).map_err(|error| NextJsError::InvalidData {
        message: error.to_string(),
    })?;
    serde_json::to_string(&data).map_err(|error| NextJsError::InvalidData {
        message: error.to_string(),
    })
}
