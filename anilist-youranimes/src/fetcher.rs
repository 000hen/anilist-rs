use std::time::Duration;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{
    AnimeParser, AnimeSource, SourceFuture,
    error::{ParseError, SourceError},
};
use futures::{StreamExt, TryStreamExt, stream};
use futures_timer::Delay;
use reqwest::{Client, StatusCode};

#[cfg(feature = "system-timezone")]
use crate::system_timezone;

use crate::{ID_PREFIX, parser::YourAnimeParser};

const LIST_URL: &str = "https://youranimes.tw/bangumi/";
const DETAIL_URL: &str = "https://youranimes.tw/animes/";
const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

const REQUEST_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CONCURRENT: usize = 6;

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher {
    client: Client,
    parser: YourAnimeParser,
}

impl YourAnimesFetcher {
    pub const fn new(client: Client) -> Self {
        Self {
            client,
            parser: YourAnimeParser::new(),
        }
    }

    async fn fetch_content(&self, url: &str) -> Result<String, SourceError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|_| SourceError::Unavailable)?;

        ensure_successful_status(response.status())?;

        response.text().await.map_err(|_| SourceError::Unavailable)
    }

    async fn fetch_list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError> {
        let url = list_url(year, season);
        let content = self.fetch_content(&url).await?;

        let mut animes = self.parser.parse_list(&content)?;

        localize(&mut animes)?;

        Ok(animes)
    }

    async fn fetch_detail(&self, id: &str) -> Result<Anime, SourceError> {
        let prefix = format!("{ID_PREFIX}:");

        let id = id
            .strip_prefix(&prefix)
            .ok_or(ParseError::InvalidResponse {
                context: "unexpected YourAnimes anime id",
            })?;

        let url = format!("{DETAIL_URL}{id}");
        let content = self.fetch_content(&url).await?;

        let mut anime = self.parser.parse_detail(&content)?;

        localize(std::slice::from_mut(&mut anime))?;

        Ok(anime)
    }

    async fn fetch_search(&self, keyword: &str) -> Result<Vec<Anime>, SourceError> {
        let url = search_url(keyword);
        let content = self.fetch_content(&url).await?;

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

impl AnimeSource for YourAnimesFetcher {
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
    let zone = system_timezone().map_err(|source| SourceError::TimezoneUnavailable { source })?;
    let reference = chrono::Local::now().date_naive();

    localize_to(animes, zone, reference)
}

#[cfg(feature = "system-timezone")]
fn localize_to(
    animes: &mut [Anime],
    zone: chrono_tz::Tz,
    reference: chrono::NaiveDate,
) -> Result<(), SourceError> {
    for anime in animes {
        let Some(time) = anime.on_air_time.take() else {
            continue;
        };

        anime.on_air_time =
            Some(
                time.to_zone(zone, reference)
                    .map_err(|source| SourceError::AnimeConversion {
                        anime_id: anime.id.clone(),
                        source,
                    })?,
            );
    }

    Ok(())
}

#[cfg(not(feature = "system-timezone"))]
fn localize(_animes: &mut [Anime]) -> Result<(), SourceError> {
    Ok(())
}

fn list_url(year: u16, season: AnimeSeason) -> String {
    let month = match season {
        AnimeSeason::Winter => "01",
        AnimeSeason::Spring => "04",
        AnimeSeason::Summer => "07",
        AnimeSeason::Fall => "10",
    };

    format!("{LIST_URL}{year}{month}")
}

fn search_url(keyword: &str) -> String {
    format!(
        "{API_BASE_URL}animes?tk={keyword}&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
    )
}

fn ensure_successful_status(status: StatusCode) -> Result<(), SourceError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(SourceError::Unavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "system-timezone")]
    fn localizes_parsed_schedules_after_parsing() {
        let body = r#"<script id="__NEXT_DATA__">["$","$L1",null,{"animes":[{"_id":"1108","adultstreaming":[],"aniType":"TV","commentCount":0,"cover":"cover","episode":"12","favorability":{"average":5.0,"counts":1},"name":"Anime","status":"finished","streaming":[],"dayOfWeek":1,"date":"2026-07-06 00:30"}]}]</script>"#;
        let mut animes = YourAnimeParser::new().parse_list(body).unwrap();
        assert_eq!(animes[0].on_air_time.as_ref().unwrap().zone, "Asia/Tokyo");
        localize_to(
            &mut animes,
            chrono_tz::Asia::Taipei,
            chrono::NaiveDate::from_ymd_opt(2026, 7, 6).unwrap(),
        )
        .unwrap();
        let time = animes[0].on_air_time.as_ref().unwrap();
        assert_eq!(time.zone, "Asia/Taipei");
        assert_eq!(time.week, chrono::Weekday::Sun);
        assert_eq!(time.minute.unwrap().get(), 23 * 60 + 30);
    }
}
