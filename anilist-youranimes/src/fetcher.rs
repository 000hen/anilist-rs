use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{AnimeSource, SourceError};
use reqwest::{Client, StatusCode};

use crate::{errors::YourAnimesError, parser::list::parse_list_source};

const BASE_URL: &str = "https://youranimes.tw/bangumi/";
const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

#[derive(Debug, Clone)]
pub struct YourAnimesFetcher {
    fetcher: Client,
}

impl YourAnimesFetcher {
    pub const fn new(client: Client) -> Self {
        Self { fetcher: client }
    }

    async fn fetch_list(
        &self,
        year: u16,
        season: AnimeSeason,
    ) -> Result<Vec<Anime>, YourAnimesError> {
        let url = parse_url(year, season);
        let response =
            self.fetcher
                .get(&url)
                .send()
                .await
                .map_err(|source| YourAnimesError::Request {
                    url: url.clone(),
                    source,
                })?;

        ensure_successful_status(&url, response.status())?;

        let content = response
            .text()
            .await
            .map_err(|source| YourAnimesError::ResponseBody { url, source })?;

        parse_list_source(&content)
    }
}

impl AnimeSource for YourAnimesFetcher {
    async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError> {
        self.fetch_list(year, season).await.map_err(Into::into)
    }

    async fn detail(&self, id: String) -> Result<Vec<Anime>, SourceError> {
        unimplemented!()
    }

    async fn search(&self, keyword: String) -> Result<Vec<Anime>, SourceError> {
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

    format!("{BASE_URL}{year}{month}")
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
