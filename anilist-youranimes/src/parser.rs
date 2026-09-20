use std::collections::HashMap;

use anilist_core::anime::Anime;
use anilist_core::season::AnimeSeason;
use anilist_nextjs::NextJsData;
use anilist_source::{
    AnimeParser, AnimeSource, HttpMethod, HttpRequest,
    error::{ParseError, SourceError},
};

use crate::{
    ID_PREFIX, YourAnimesParseError,
    format::{anime::AnimeInformation, search::SearchResult},
};

fn parse_detail(content: &str) -> Result<Anime, YourAnimesParseError> {
    let parsed: AnimeInformation = NextJsData::parse(content)?.deserialize("/1/3/anime")?;
    Ok(parsed.into_anime())
}

fn parse_list(content: &str) -> Result<Vec<Anime>, YourAnimesParseError> {
    let data = NextJsData::parse(content)?;
    let parsed: Vec<AnimeInformation> = data.deserialize("/3/animes")?;
    let vendors = match data.deserialize::<HashMap<String, String>>("/3/nameMap") {
        Ok(vendors) => vendors,
        Err(anilist_nextjs::ParseError::MissingPath { .. }) => HashMap::new(),
        Err(error) => return Err(error.into()),
    };

    Ok(parsed
        .into_iter()
        .map(|anime| anime.resolve_vendors(&vendors))
        .map(AnimeInformation::into_anime)
        .collect())
}

fn parse_search(content: &str) -> Result<Vec<String>, YourAnimesParseError> {
    let result = serde_json::from_str::<SearchResult>(content).map_err(|source| {
        YourAnimesParseError::Json {
            context: "YourAnimes search response",
            source,
        }
    })?;

    Ok(result
        .result
        .into_iter()
        .map(|item| format!("{ID_PREFIX}:{}", item.id))
        .collect())
}

#[derive(Debug, Clone)]
pub struct YourAnimeParser;

impl YourAnimeParser {
    pub const fn new() -> Self {
        Self {}
    }
}

impl AnimeParser for YourAnimeParser {
    fn source_id(&self) -> &'static str {
        ID_PREFIX
    }

    fn parse_detail(&self, content: &str) -> Result<Anime, ParseError> {
        parse_detail(content).map_err(Into::into)
    }

    fn parse_list(&self, content: &str) -> Result<Vec<Anime>, ParseError> {
        parse_list(content).map_err(Into::into)
    }

    fn parse_search(&self, content: &str) -> Result<Vec<String>, ParseError> {
        parse_search(content).map_err(Into::into)
    }
}

impl AnimeSource for YourAnimeParser {
    fn list_request(&self, year: u16, season: AnimeSeason) -> Result<HttpRequest, SourceError> {
        Ok(HttpRequest {
            method: HttpMethod::Get,
            url: list_url(year, season),
            headers: Vec::new(),
            body: None,
        })
    }

    fn search_request(&self, keyword: &str) -> Result<HttpRequest, SourceError> {
        Ok(HttpRequest {
            method: HttpMethod::Get,
            url: search_url(keyword),
            headers: Vec::new(),
            body: None,
        })
    }

    fn detail_request(&self, id: &str) -> Result<HttpRequest, SourceError> {
        let id = id
            .strip_prefix(&format!("{ID_PREFIX}:"))
            .ok_or(ParseError::InvalidResponse {
                context: "unexpected YourAnimes anime id",
            })?;

        if id.is_empty() || !id.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(ParseError::InvalidResponse {
                context: "invalid YourAnimes anime id",
            }
            .into());
        }
        Ok(HttpRequest {
            method: HttpMethod::Get,
            url: detail_url(id),
            headers: Vec::new(),
            body: None,
        })
    }
}

pub(crate) const LIST_URL: &str = "https://youranimes.tw/bangumi/";
pub(crate) const DETAIL_URL: &str = "https://youranimes.tw/animes/";
pub(crate) const API_BASE_URL: &str = "https://youranimes.tw/api/v1/";

pub(crate) fn list_url(year: u16, season: AnimeSeason) -> String {
    let month = match season {
        AnimeSeason::Winter => "01",
        AnimeSeason::Spring => "04",
        AnimeSeason::Summer => "07",
        AnimeSeason::Fall => "10",
    };

    format!("{LIST_URL}{year}{month}")
}

pub(crate) fn detail_url(id: &str) -> String {
    format!("{DETAIL_URL}{id}")
}

pub(crate) fn search_url(keyword: &str) -> String {
    // Encode the value, not the entire URL, so keywords cannot introduce query parameters.
    let keyword: String = keyword
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                char::from(byte).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    format!(
        "{API_BASE_URL}animes?tk={keyword}&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn anime() -> Value {
        json!({
            "_id": "1108", "adultstreaming": [], "aniType": "TV",
            "commentCount": 0, "cover": "https://example.com/cover.webp",
            "episode": "12", "favorability": {"average": 5.0, "counts": 1},
            "name": "小林家的龍女僕S", "description": "Quotes: \"hello\"\n中文",
            "status": "finished", "streaming": [], "dayOfWeek": null
        })
    }

    fn html(row: Value) -> String {
        let payload = format!("1:I[42,[],\"default\"]\n2:{row}\n");
        // Split in the middle of the serialized model; script boundaries are not
        // record boundaries. Include metadata before the target in the same stream.
        let split = payload.find("description").unwrap() + 5;
        [&payload[..split], &payload[split..]]
            .into_iter()
            .map(|chunk| {
                format!(
                    "<script>self.__next_f.push({});</script>",
                    json!([1, chunk])
                )
            })
            .collect()
    }

    #[test]
    fn list_extracts_split_flight_model_and_converts_anime() {
        let result = parse_list(&html(json!(["$", "$L1", null, {"animes": [anime()]}]))).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, "youranimes:1108");
        assert_eq!(result[0].name, "小林家的龍女僕S");
        assert_eq!(result[0].description, "Quotes: \"hello\"\n中文");
    }

    #[test]
    fn detail_extracts_split_flight_model_and_converts_anime() {
        let result =
            parse_detail(&html(json!([null, ["$", "$L1", null, {"anime": anime()}]]))).unwrap();
        assert_eq!(result.id, "youranimes:1108");
        assert_eq!(result.name, "小林家的龍女僕S");
        assert_eq!(result.description, "Quotes: \"hello\"\n中文");
    }

    #[test]
    fn missing_hydration_and_wrong_anime_shape_return_errors() {
        assert!(parse_list("<html></html>").is_err());
        assert!(parse_detail("<html></html>").is_err());
        assert!(
            parse_list(&html(
                json!(["$", "$L1", null, {"animes": [{"description":"invalid"}]}])
            ))
            .is_err()
        );
    }

    #[test]
    fn search_returns_prefixed_ids_without_detail_hydration() {
        let result = parse_search(r#"{"result":[{"_id":"1108"},{"_id":"42"}]}"#).unwrap();

        assert_eq!(result, ["youranimes:1108", "youranimes:42"]);
    }

    #[test]
    fn search_parse_errors_identify_the_response_stage() {
        let error = parse_search("not json").unwrap_err();

        assert!(error.to_string().contains("YourAnimes search response"));
    }

    #[test]
    fn client_requests_keep_youranimes_protocol_details_in_rust() {
        let parser = YourAnimeParser::new();

        assert_eq!(
            parser.list_request(2026, AnimeSeason::Summer).unwrap().url,
            "https://youranimes.tw/bangumi/202607"
        );
        assert_eq!(
            parser.search_request("女僕").unwrap().url,
            "https://youranimes.tw/api/v1/animes?tk=%E5%A5%B3%E5%83%95&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
        );
        assert_eq!(
            parser.detail_request("youranimes:1108").unwrap().url,
            "https://youranimes.tw/animes/1108"
        );
        assert!(parser.detail_request("1108").is_err());
        assert!(parser.detail_request("youranimes:").is_err());
        assert!(parser.detail_request("youranimes:../42").is_err());
        assert!(
            parser
                .search_request("a &b=1#中")
                .unwrap()
                .url
                .contains("tk=a%20%26b%3D1%23%E4%B8%AD&tags=")
        );
    }

    #[test]
    fn list_resolves_streaming_vendor_names() {
        let mut anime = anime();

        anime["streaming"] = json!([
            {
                "title": "Bahamut",
                "url": "https://ani.gamer.com.tw/animeVideo.php?sn=1",
                "vendor": "gamer"
            }
        ]);

        let content = html(json!([
            "$",
            "$L1",
            null,
            {
                "animes": [anime],
                "nameMap": {
                    "gamer": "巴哈姆特動畫瘋"
                }
            }
        ]));

        let result = parse_list(&content).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].streaming.len(), 1);
        assert_eq!(result[0].streaming[0].name, "巴哈姆特動畫瘋");
        assert_eq!(
            result[0].streaming[0].logo,
            "https://d28s5ztqvkii64.cloudfront.net/images/gamer_icon.webp"
        );
    }
}
