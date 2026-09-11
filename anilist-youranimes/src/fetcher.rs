use std::time::Duration;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{AnimeSource, SourceError};
use futures::{StreamExt, TryStreamExt, stream};
use futures_timer::Delay;
use reqwest::{Client, StatusCode};

use crate::{
    ID_PREFIX,
    errors::YourAnimesError,
    format::search::SearchResult,
    parser::{detail::parse_detail, list::parse_list},
};

const LIST_URL: &str = "https://youranimes.tw/bangumi/";
const DETAIL_URL: &str = "https://youranimes.tw/animes/";
const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

const REQUEST_INTERVAL: Duration = Duration::from_millis(100);
const MAX_CONCURRENT: usize = 6;

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher {
    fetcher: Client,
}

impl YourAnimesFetcher {
    pub const fn new(client: Client) -> Self {
        Self { fetcher: client }
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
        parse_list(&content)
    }

    async fn fetch_detail(&self, id: &str) -> Result<Anime, YourAnimesError> {
        let prefix = format!("{ID_PREFIX}:");
        let parsed_id = id
            .strip_prefix(&prefix)
            .ok_or(YourAnimesError::InvalidResponse {
                context: "Unexpected YourAnimes id.",
            })?;

        let url = format!("{DETAIL_URL}{parsed_id}");
        let content = self.fetch_and_get_content(&url).await?;

        parse_detail(&content)
    }

    async fn search(&self, keyword: &str) -> Result<Vec<Anime>, YourAnimesError> {
        let url = parse_search_url(keyword);
        let content = self.fetch_and_get_content(&url).await?;

        let result = serde_json::from_str::<SearchResult>(&content).map_err(|source| {
            YourAnimesError::Json {
                context: "Next.js target data",
                source,
            }
        })?;

        stream::iter(result.result)
            .enumerate()
            .then(|(index, item)| async move {
                if index > 0 {
                    Delay::new(REQUEST_INTERVAL).await;
                }

                item
            })
            .map(|item| async move {
                let id = format!("{ID_PREFIX}:{}", item.id);
                self.fetch_detail(&id).await
            })
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
