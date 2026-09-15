use anilist_nextjs::NextJsData;
use serde_json::json;

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
    let flight: Vec<_> = data
        .flight
        .into_iter()
        .map(|record| {
            json!({
                "id": record.id,
                "tag": record.tag,
                "value": record.value,
            })
        })
        .collect();
    Ok(json!({"nextData": data.next_data, "flight": flight}).to_string())
}
