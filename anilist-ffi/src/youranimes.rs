use crate::{AnilistError, Anime};

/// Parse a YourAnimes season page fetched by the caller.
#[uniffi::export]
pub fn your_animes_parse_list(content: String) -> Result<Vec<Anime>, AnilistError> {
    anilist_youranimes::parse_list(&content)
        .map(|items| items.into_iter().map(Anime::from).collect())
        .map_err(Into::into)
}

/// Parse a YourAnimes detail page fetched by the caller.
#[uniffi::export]
pub fn your_animes_parse_detail(content: String) -> Result<Anime, AnilistError> {
    anilist_youranimes::parse_detail(&content)
        .map(Anime::from)
        .map_err(Into::into)
}

/// Parse search JSON into source-prefixed IDs, in response order.
/// The caller fetches detail pages separately to obtain complete anime records.
#[uniffi::export]
pub fn your_animes_parse_search(content: String) -> Result<Vec<String>, AnilistError> {
    anilist_youranimes::parse_search(&content).map_err(Into::into)
}
