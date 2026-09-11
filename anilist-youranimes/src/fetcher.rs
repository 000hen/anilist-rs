use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{AnimeSource, SourceError};
use reqwest::{Client, StatusCode};
use scraper::Html;

use crate::{
    ID_PREFIX,
    errors::YourAnimesError,
    parser::{detail::parse_detail, list::parse_list},
};

const LIST_URL: &str = "https://youranimes.tw/bangumi/";
const DETAIL_URL: &str = "https://youranimes.tw/animes/";
const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher {
    fetcher: Client,
}

impl YourAnimesFetcher {
    pub const fn new(client: Client) -> Self {
        Self { fetcher: client }
    }

    async fn fetch_and_get_html(&self, url: &str) -> Result<String, YourAnimesError> {
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
        let content = self.fetch_and_get_html(&url).await?;
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
        let content = self.fetch_and_get_html(&url).await?;

        parse_detail(&content)
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
        unimplemented!()
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
