use std::sync::Arc;

use anilist_source::AnimeSource;

#[cfg(feature = "youranimes")]
use anilist_youranimes::fetcher::YourAnimesFetcher;

use crate::{AnilistError, Anime, AnimeSeason};

#[derive(uniffi::Object)]
pub struct NativeAnimeFetcher {
    fetcher: Box<dyn AnimeSource>,
}

#[uniffi::export]
impl NativeAnimeFetcher {
    #[uniffi::constructor]
    pub fn new(source_id: String) -> Result<Arc<Self>, AnilistError> {
        let fetcher = create_fetcher(&source_id)?;

        Ok(Arc::new(Self { fetcher }))
    }

    pub fn source_id(&self) -> String {
        self.fetcher.source_id().to_owned()
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, AnilistError> {
        self.fetcher
            .list(year, season.into())
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn search(&self, keyword: String) -> Result<Vec<Anime>, AnilistError> {
        self.fetcher
            .search(&keyword)
            .await
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    #[uniffi::method(async_runtime = "tokio")]
    pub async fn detail(&self, id: String) -> Result<Anime, AnilistError> {
        self.fetcher
            .detail(&id)
            .await
            .map(Into::into)
            .map_err(Into::into)
    }
}

fn create_fetcher(source_id: &str) -> Result<Box<dyn AnimeSource>, AnilistError> {
    match source_id {
        #[cfg(feature = "youranimes")]
        "youranimes" => Ok(Box::new(YourAnimesFetcher::new(reqwest::Client::new()))),

        _ => Err(AnilistError::UnsupportedSource {
            source_id: source_id.to_owned(),
        }),
    }
}
