use std::time::Duration;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{
    AnimeFetcher, AnimeParser, AnimeSource, HttpClient, SourceFuture, error::SourceError,
};
use futures::{StreamExt, TryStreamExt, stream};
use futures_timer::Delay;
use reqwest::Client;

use crate::{ID_PREFIX, parser::YourAnimeParser};

const REQUEST_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CONCURRENT: usize = 6;

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher<C = Client> {
    client: C,
    parser: YourAnimeParser,
}

impl<C: HttpClient> YourAnimesFetcher<C> {
    pub const fn new(client: C) -> Self {
        Self {
            client,
            parser: YourAnimeParser::new(),
        }
    }

    async fn fetch_list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError> {
        let content =
            HttpClient::execute(&self.client, self.parser.list_request(year, season)?).await?;

        let mut animes = self.parser.parse_list(&content)?;

        localize(&mut animes)?;

        Ok(animes)
    }

    async fn fetch_detail(&self, id: &str) -> Result<Anime, SourceError> {
        let content = HttpClient::execute(&self.client, self.parser.detail_request(id)?).await?;

        let mut anime = self.parser.parse_detail(&content)?;

        localize(std::slice::from_mut(&mut anime))?;

        Ok(anime)
    }

    async fn fetch_search(&self, keyword: &str) -> Result<Vec<Anime>, SourceError> {
        let content =
            HttpClient::execute(&self.client, self.parser.search_request(keyword)?).await?;

        let ids = self.parser.parse_search(&content)?;

        stream::iter(ids)
            .enumerate()
            .then(|(index, id)| async move {
                if index > 0 {
                    Delay::new(REQUEST_INTERVAL).await;
                }

                id
            })
            .map(|id| async move { self.fetch_detail(&id).await })
            .buffered(MAX_CONCURRENT)
            .try_collect()
            .await
    }
}

impl<C: HttpClient> AnimeFetcher for YourAnimesFetcher<C> {
    fn source_id(&self) -> &'static str {
        ID_PREFIX
    }

    fn list(&self, year: u16, season: AnimeSeason) -> SourceFuture<'_, Vec<Anime>> {
        Box::pin(async move { self.fetch_list(year, season).await })
    }

    fn search<'a>(&'a self, keyword: &'a str) -> SourceFuture<'a, Vec<Anime>> {
        Box::pin(async move { self.fetch_search(keyword).await })
    }

    fn detail<'a>(&'a self, id: &'a str) -> SourceFuture<'a, Anime> {
        Box::pin(async move { self.fetch_detail(id).await })
    }
}

#[cfg(feature = "system-timezone")]
fn localize(animes: &mut [Anime]) -> Result<(), SourceError> {
    let zone: chrono_tz::Tz = iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .ok_or(SourceError::TimezoneUnavailable {
            source: anilist_core::time::ZoneConversionError::UnknownTimeZone,
        })?;
    let reference = chrono::Local::now().date_naive();

    for anime in animes {
        let Some(time) = anime.on_air_time.take() else {
            continue;
        };
        let localized =
            time.to_zone(zone, reference)
                .map_err(|source| SourceError::AnimeConversion {
                    anime_id: anime.id.clone(),
                    source,
                })?;
        anime.on_air_time = Some(localized);
    }

    Ok(())
}

#[cfg(not(feature = "system-timezone"))]
fn localize(_animes: &mut [Anime]) -> Result<(), SourceError> {
    Ok(())
}
