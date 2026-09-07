use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::{AnimeSource, SourceError};
use chrono::Local;
use reqwest::{Client, StatusCode};
use scraper::{Html, Selector};
use serde_json::Value;

use crate::{errors::YourAnimesError, parser::AnimeInformation, system_timezone};

const BASE_URL: &str = "https://youranimes.tw/bangumi/";

const NEXTJS_SCRIPT_HANDLER: &str = "self.__next_f.push(";
const NEXTJS_SCRIPT_SUFFIX: &str = ")";
const MATCH_FORMAT: &str = "{\\\"animes\\\":[";

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

        let fragment = Html::parse_document(&content);
        let first_fetch =
            get_information_json(&fragment).ok_or(YourAnimesError::InvalidResponse {
                context: "Next.js anime data script was not found",
            })?;

        let cleared = json_clean_up(first_fetch)?;
        let parsed = serde_json::from_str::<Vec<AnimeInformation>>(&cleared).map_err(|source| {
            YourAnimesError::Json {
                context: "anime list",
                source,
            }
        })?;

        let reference_date = Local::now();
        let timezone = system_timezone();
        let naive_time = reference_date.date_naive();

        parsed
            .into_iter()
            .map(|data| {
                let anime_id = data.id.clone();
                data.into_anime(timezone, naive_time)
                    .map_err(|source| YourAnimesError::AnimeConversion { anime_id, source })
            })
            .collect()
    }
}

impl AnimeSource for YourAnimesFetcher {
    async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError> {
        self.fetch_list(year, season).await.map_err(Into::into)
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

fn get_information_json(body: &Html) -> Option<String> {
    let script = Selector::parse("script").expect("script should parse");

    body.select(&script).find_map(|element| {
        let data = element.inner_html();
        let content = data.strip_circumfix(NEXTJS_SCRIPT_HANDLER, NEXTJS_SCRIPT_SUFFIX)?;
        content.contains(MATCH_FORMAT).then(|| content.to_owned())
    })
}

fn json_clean_up(source: String) -> Result<String, YourAnimesError> {
    let raw_list = serde_json::from_str::<Value>(&source)
        .map_err(|source| YourAnimesError::Json {
            context: "Next.js data frame",
            source,
        })?
        .as_array()
        .ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js data frame is not an array",
        })?
        .get(1)
        .ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js data frame has no payload",
        })?
        .to_string();

    let (_, payload) = raw_list
        .split_once(':')
        .ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js data frame payload has no separator",
        })?;
    let cleaned = payload.replace("\\\"", "\"").replace("\\\\", "\\");
    let json_end = cleaned
        .len()
        .checked_sub(3)
        .ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js data frame payload is truncated",
        })?;
    let cleaned = cleaned
        .get(..json_end)
        .ok_or(YourAnimesError::InvalidResponse {
            context: "Next.js data frame payload ends at an invalid character boundary",
        })?;

    let inner = serde_json::from_str::<Value>(cleaned)
        .map_err(|source| YourAnimesError::Json {
            context: "embedded anime data",
            source,
        })?
        .as_array()
        .ok_or(YourAnimesError::InvalidResponse {
            context: "embedded anime data is not an array",
        })?
        .get(3)
        .ok_or(YourAnimesError::InvalidResponse {
            context: "embedded anime data has no properties object",
        })?
        .as_object()
        .ok_or(YourAnimesError::InvalidResponse {
            context: "embedded anime properties are not an object",
        })?
        .get("animes")
        .ok_or(YourAnimesError::InvalidResponse {
            context: "embedded anime properties have no anime list",
        })?
        .to_string();

    Ok(inner)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn malformed_data_frame_reports_its_parse_stage() {
        let error = json_clean_up("not JSON".to_owned()).unwrap_err();

        assert!(matches!(
            error,
            YourAnimesError::Json {
                context: "Next.js data frame",
                ..
            }
        ));
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

    #[test]
    fn truncated_data_frame_is_rejected_without_panicking() {
        let error = json_clean_up("[null, \":\"]".to_owned()).unwrap_err();

        assert!(matches!(
            error,
            YourAnimesError::InvalidResponse {
                context: "Next.js data frame payload is truncated"
            }
        ));
    }
}
