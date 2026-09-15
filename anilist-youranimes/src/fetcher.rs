use std::time::Duration;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{AnimeParser, AnimeSource, error::SourceError};
use futures::{StreamExt, TryStreamExt, stream};
use futures_timer::Delay;
use reqwest::{Client, StatusCode};

use crate::{
    ID_PREFIX,
    errors::{YourAnimesError, YourAnimesParseError},
    parser::YourAnimeParser,
};

const LIST_URL: &str = "https://youranimes.tw/bangumi/";
const DETAIL_URL: &str = "https://youranimes.tw/animes/";
const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

const REQUEST_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CONCURRENT: usize = 6;

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher {
    fetcher: Client,
    parser: YourAnimeParser,
}

impl YourAnimesFetcher {
    pub const fn new(client: Client) -> Self {
        Self {
            fetcher: client,
            parser: YourAnimeParser::new(),
        }
    }

    async fn fetch_and_get_content(&self, url: &str) -> Result<String, YourAnimesError> {
        let response =
            self.fetcher
                .get(url)
                .send()
                .await
                .map_err(|source| YourAnimesError::Request {
                    url: url.to_owned(),
                    source,
                })?;

        ensure_successful_status(&url, response.status())?;

        let content = response
            .text()
            .await
            .map_err(|source| YourAnimesError::ResponseBody {
                url: url.to_owned(),
                source,
            })?;

        Ok(content)
    }

    async fn fetch_list(
        &self,
        year: u16,
        season: AnimeSeason,
    ) -> Result<Vec<Anime>, YourAnimesError> {
        let url = parse_url(year, season);
        let content = self.fetch_and_get_content(&url).await?;
        let mut animes = self.parser.parse_list(&content)?;
        localize(&mut animes)?;
        Ok(animes.to_vec())
    }

    async fn fetch_detail(&self, id: &str) -> Result<Anime, YourAnimesError> {
        let prefix = format!("{ID_PREFIX}:");
        let parsed_id = id
            .strip_prefix(&prefix)
            .ok_or(YourAnimesParseError::InvalidResponse {
                context: "Unexpected YourAnimes id.",
            })
            .map_err(YourAnimesError::from)?;

        let url = format!("{DETAIL_URL}{parsed_id}");
        let content = self.fetch_and_get_content(&url).await?;

        let mut anime = self.parser.parse_detail(&content)?;
        localize(std::slice::from_mut(&mut anime))?;
        Ok(anime)
    }

    async fn search(&self, keyword: &str) -> Result<Vec<Anime>, YourAnimesError> {
        let url = parse_search_url(keyword);
        let content = self.fetch_and_get_content(&url).await?;

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
            .try_collect::<Vec<_>>()
            .await
    }
}

impl AnimeSource for YourAnimesFetcher {
    async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError> {
        self.fetch_list(year, season).await.map_err(Into::into)
    }

    async fn detail(&self, id: &str) -> Result<Anime, SourceError> {
        self.fetch_detail(id).await.map_err(Into::into)
    }

    async fn search(&self, keyword: &str) -> Result<Vec<Anime>, SourceError> {
        self.search(keyword).await.map_err(Into::into)
    }
}

#[cfg(feature = "system-timezone")]
fn localize(animes: &mut [Anime]) -> Result<(), YourAnimesError> {
    let zone = crate::system_timezone().map_err(YourAnimesError::Timezone)?;
    let reference = chrono::Local::now().date_naive();
    localize_to(animes, zone, reference)
}

#[cfg(feature = "system-timezone")]
fn localize_to(
    animes: &mut [Anime],
    zone: chrono_tz::Tz,
    reference: chrono::NaiveDate,
) -> Result<(), YourAnimesError> {
    for anime in animes {
        if let Some(time) = anime.on_air_time.take() {
            anime.on_air_time = Some(time.to_zone(zone, reference).map_err(|source| {
                YourAnimesError::AnimeConversion {
                    anime_id: anime.id.clone(),
                    source,
                }
            })?);
        }
    }
    Ok(())
}

#[cfg(not(feature = "system-timezone"))]
fn localize(_animes: &mut [Anime]) -> Result<(), YourAnimesError> {
    Ok(())
}

fn parse_url(year: u16, season: AnimeSeason) -> String {
    let year = year.to_string();
    let month = match season {
        AnimeSeason::Winter => "01",
        AnimeSeason::Spring => "04",
        AnimeSeason::Summer => "07",
        AnimeSeason::Fall => "10",
    };

    format!("{LIST_URL}{year}{month}")
}

fn parse_search_url(keyword: &str) -> String {
    format!(
        "{API_BASE_URL}animes?tk={keyword}&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
    )
}

fn ensure_successful_status(url: &str, status: StatusCode) -> Result<(), YourAnimesError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(YourAnimesError::UnexpectedStatus {
            url: url.to_owned(),
            status,
        })
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

    #[test]
    fn unsuccessful_http_status_includes_the_url_and_status() {
        let error = ensure_successful_status("https://example.com/anime", StatusCode::BAD_GATEWAY)
            .unwrap_err();

        assert!(matches!(
            error,
            YourAnimesError::UnexpectedStatus { url, status }
                if url == "https://example.com/anime" && status == StatusCode::BAD_GATEWAY
        ));
    }
}
