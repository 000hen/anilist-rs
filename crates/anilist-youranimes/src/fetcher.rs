use std::time::Duration;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{
    AnimeFetcher, AnimeParser, AnimeSource, HttpClient, SourceFuture, error::SourceError,
};
use futures::{StreamExt, TryStreamExt, stream};
use futures_timer::Delay;
use reqwest::Client;

#[cfg(feature = "system-timezone")]
use crate::system_timezone;

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

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHttpClient;

    impl HttpClient for FakeHttpClient {
        fn execute(&self, request: anilist_source::HttpRequest) -> SourceFuture<'_, String> {
            Box::pin(async move {
                assert_eq!(request.url, "https://youranimes.tw/bangumi/202607");
                assert_eq!(request.method, anilist_source::HttpMethod::Get);
                Ok(
                    r#"<script id="__NEXT_DATA__">["$","$L1",null,{"animes":[]}]</script>"#
                        .to_owned(),
                )
            })
        }
    }

    #[tokio::test]
    async fn fetcher_composes_source_and_independent_transport() {
        let result = YourAnimesFetcher::new(FakeHttpClient)
            .list(2026, AnimeSeason::Summer)
            .await
            .unwrap();
        assert!(result.is_empty());
    }

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
