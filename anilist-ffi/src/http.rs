use crate::{AnilistError, Anime, AnimeSeason};
use anilist_source::AnimeSource;
use anilist_youranimes::fetcher::YourAnimesFetcher;
use std::sync::OnceLock;

static YOUR_ANIMES: OnceLock<YourAnimesFetcher> = OnceLock::new();

fn your_animes() -> &'static YourAnimesFetcher {
    YOUR_ANIMES.get_or_init(|| YourAnimesFetcher::new(reqwest::Client::new()))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_list(year: u16, season: AnimeSeason) -> Result<Vec<Anime>, AnilistError> {
    let animes = your_animes()
        .list(year, season.into())
        .await
        .map_err(AnilistError::from)?;

    Ok(animes.into_iter().map(Anime::from).collect())
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_search(keyword: String) -> Result<Vec<Anime>, AnilistError> {
    let animes = your_animes()
        .search(&keyword)
        .await
        .map_err(AnilistError::from)?;

    Ok(animes.into_iter().map(Anime::from).collect())
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_detail(id: String) -> Result<Anime, AnilistError> {
    let anime = your_animes()
        .detail(&id)
        .await
        .map_err(AnilistError::from)?;

    Ok(Anime::from(anime))
}
