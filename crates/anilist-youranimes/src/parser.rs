use std::collections::HashMap;

use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_nextjs::NextJsData;
use anilist_source::{
    AnimeParser, AnimeSource, HttpMethod, HttpRequest,
    error::{ParseError, SourceError},
};
use serde::Deserialize;

use crate::{ID_PREFIX, format::AnimeInformation};

const LIST_URL: &str = "https://youranimes.tw/bangumi/";
const DETAIL_URL: &str = "https://youranimes.tw/animes/";
const SEARCH_URL: &str = "https://youranimes.tw/api/v1/animes";

#[derive(Debug, Clone, Default)]
pub struct YourAnimeParser;

impl YourAnimeParser {
    pub const fn new() -> Self {
        Self
    }
}

impl AnimeParser for YourAnimeParser {
    fn source_id(&self) -> &'static str {
        ID_PREFIX
    }

    fn parse_list(&self, content: &str) -> Result<Vec<Anime>, ParseError> {
        let data = page(content)?;
        let animes: Vec<AnimeInformation> = data.deserialize("/3/animes").map_err(page_error)?;
        let vendors = match data.deserialize::<HashMap<String, String>>("/3/nameMap") {
            Ok(vendors) => vendors,
            Err(anilist_nextjs::ParseError::MissingPath { .. }) => HashMap::new(),
            Err(error) => return Err(page_error(error)),
        };

        Ok(animes
            .into_iter()
            .map(|anime| anime.into_anime(&vendors))
            .collect())
    }

    fn parse_detail(&self, content: &str) -> Result<Anime, ParseError> {
        let anime: AnimeInformation = page(content)?
            .deserialize("/1/3/anime")
            .map_err(page_error)?;
        Ok(anime.into_anime(&HashMap::new()))
    }

    /// Search responses only carry IDs; callers hydrate each one with a detail request.
    fn parse_search(&self, content: &str) -> Result<Vec<String>, ParseError> {
        #[derive(Deserialize)]
        struct SearchResponse {
            result: Vec<SearchItem>,
        }
        #[derive(Deserialize)]
        struct SearchItem {
            #[serde(rename = "_id")]
            id: String,
        }

        let response: SearchResponse =
            serde_json::from_str(content).map_err(|source| ParseError::Decode {
                context: "YourAnimes search response",
                source: Box::new(source),
            })?;

        Ok(response
            .result
            .into_iter()
            .map(|item| format!("{ID_PREFIX}:{}", item.id))
            .collect())
    }
}

impl AnimeSource for YourAnimeParser {
    fn list_request(&self, year: u16, season: AnimeSeason) -> Result<HttpRequest, SourceError> {
        let month = match season {
            AnimeSeason::Winter => "01",
            AnimeSeason::Spring => "04",
            AnimeSeason::Summer => "07",
            AnimeSeason::Fall => "10",
        };
        Ok(get(format!("{LIST_URL}{year}{month}")))
    }

    fn search_request(&self, keyword: &str) -> Result<HttpRequest, SourceError> {
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
        Ok(get(format!(
            "{SEARCH_URL}?tk={keyword}&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
        )))
    }

    fn detail_request(&self, id: &str) -> Result<HttpRequest, SourceError> {
        let id = id
            .strip_prefix(ID_PREFIX)
            .and_then(|id| id.strip_prefix(':'))
            .filter(|id| !id.is_empty() && id.bytes().all(|byte| byte.is_ascii_digit()))
            .ok_or(ParseError::InvalidResponse {
                context: "invalid YourAnimes anime id",
            })?;
        Ok(get(format!("{DETAIL_URL}{id}")))
    }
}

fn get(url: String) -> HttpRequest {
    HttpRequest {
        method: HttpMethod::Get,
        url,
        headers: Vec::new(),
        body: None,
    }
}

fn page(content: &str) -> Result<NextJsData, ParseError> {
    NextJsData::parse(content).map_err(page_error)
}

fn page_error(error: anilist_nextjs::ParseError) -> ParseError {
    ParseError::Decode {
        context: "YourAnimes page",
        source: Box::new(error),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Weekday;
    use serde_json::{Value, json};

    use super::*;

    fn anime() -> Value {
        json!({
            "_id": "1108",
            "name": "小林家的龍女僕S",
            "cover": "https://example.com/cover.webp",
            "description": "Quotes: \"hello\"\n中文",
            "dayOfWeek": null
        })
    }

    // Flight payloads are split across scripts in the middle of a record, as on the real site.
    fn html(row: Value) -> String {
        let payload = format!("1:I[42,[],\"default\"]\n2:{row}\n");
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

    fn list(animes: Value) -> Result<Vec<Anime>, ParseError> {
        YourAnimeParser.parse_list(&html(json!(["$", "$L1", null, {"animes": animes}])))
    }

    fn parse_one(fields: Value) -> Result<Anime, ParseError> {
        let mut anime = anime();
        anime
            .as_object_mut()
            .unwrap()
            .extend(fields.as_object().unwrap().clone());
        Ok(list(json!([anime]))?.remove(0))
    }

    #[test]
    fn list_and_detail_pages_convert_to_anime() {
        let listed = list(json!([anime()])).unwrap();
        let detail = YourAnimeParser
            .parse_detail(&html(json!([null, ["$", "$L1", null, {"anime": anime()}]])))
            .unwrap();

        for anime in [&listed[0], &detail] {
            assert_eq!(anime.id, "youranimes:1108");
            assert_eq!(anime.name, "小林家的龍女僕S");
            assert_eq!(anime.description, "Quotes: \"hello\"\n中文");
            assert_eq!(anime.on_air_time, None);
        }
    }

    #[test]
    fn invalid_pages_are_errors() {
        assert!(YourAnimeParser.parse_list("<html></html>").is_err());
        assert!(YourAnimeParser.parse_detail("<html></html>").is_err());
        assert!(list(json!([{"description": "missing required fields"}])).is_err());
    }

    #[test]
    fn schedule_keeps_tokyo_time() {
        let anime = parse_one(json!({"dayOfWeek": 0, "date": "2026-07-05 23:45"})).unwrap();
        let time = anime.on_air_time.unwrap();

        assert_eq!(time.week, Weekday::Sun);
        assert_eq!(time.minute.map(|minute| minute.get()), Some(23 * 60 + 45));
        assert_eq!(time.zone, "Asia/Tokyo");
    }

    #[test]
    fn weekdays_count_from_sunday_and_unknown_markers_are_unscheduled() {
        for (raw, expected) in [
            (json!(0), Some(Weekday::Sun)),
            (json!(6), Some(Weekday::Sat)),
            (json!("1"), Some(Weekday::Mon)),
            (json!(7), None),
            (json!(-1), None),
            (json!("未定"), None),
        ] {
            let anime = parse_one(json!({"dayOfWeek": raw, "date": "未定"})).unwrap();
            assert_eq!(anime.on_air_time.map(|time| time.week), expected, "{raw}");
        }
    }

    #[test]
    fn airing_time_must_be_within_a_day() {
        for (raw, expected) in [("00:00", 0), ("23:59", 23 * 60 + 59), ("12:30:00", 12 * 60 + 30)] {
            let anime = parse_one(json!({"dayOfWeek": 1, "date": format!("2026-07-06 {raw}")}));
            let minute = anime.unwrap().on_air_time.unwrap().minute.unwrap();
            assert_eq!(minute.get(), expected);
        }
        for raw in ["24:00", "23:60", "-1:00", "ab:cd", "70000:00"] {
            let anime = parse_one(json!({"dayOfWeek": 1, "date": format!("2026-07-06 {raw}")}));
            assert!(anime.is_err(), "{raw} should be rejected");
        }
    }

    #[test]
    fn streams_are_deduplicated_named_and_skip_missing_urls() {
        let stream = json!({"url": "https://example.com/watch", "vendor": "gamer"});
        let content = html(json!(["$", "$L1", null, {
            "animes": [{
                "_id": "1", "name": "Anime", "cover": "", "description": "",
                "streaming": [stream, {"vendor": "netflix"}], "adultstreaming": [stream],
                "cast": [{"name": "Actor", "role": "Lead"}],
                "olinks": [{"title": "Official", "url": "https://example.com"}, {"title": "Gone"}],
            }],
            "nameMap": {"gamer": "巴哈姆特動畫瘋"}
        }]));

        let anime = YourAnimeParser.parse_list(&content).unwrap().remove(0);

        assert!(anime.is_adult);
        assert_eq!(anime.cast, ["Actor"]);
        assert_eq!(anime.site.len(), 1);
        assert_eq!(anime.site[0].title, "Official");
        assert_eq!(anime.streaming.len(), 1);
        assert_eq!(anime.streaming[0].name, "巴哈姆特動畫瘋");
        assert_eq!(
            anime.streaming[0].logo,
            "https://d28s5ztqvkii64.cloudfront.net/images/gamer_icon.webp"
        );
    }

    #[test]
    fn search_returns_prefixed_ids_in_order() {
        let ids = YourAnimeParser
            .parse_search(r#"{"result":[{"_id":"1108"},{"_id":"42"}]}"#)
            .unwrap();

        assert_eq!(ids, ["youranimes:1108", "youranimes:42"]);
        assert!(YourAnimeParser.parse_search("not json").is_err());
    }

    #[test]
    fn requests_target_youranimes_urls() {
        let parser = YourAnimeParser;

        assert_eq!(
            parser.list_request(2026, AnimeSeason::Summer).unwrap().url,
            "https://youranimes.tw/bangumi/202607"
        );
        assert_eq!(
            parser.search_request("女僕").unwrap().url,
            "https://youranimes.tw/api/v1/animes?tk=%E5%A5%B3%E5%83%95&tags=&page=1&size=100&orderOption=-1&streaming=0&adult=1"
        );
        assert!(
            parser
                .search_request("a &b=1#")
                .unwrap()
                .url
                .contains("tk=a%20%26b%3D1%23&tags=")
        );
        assert_eq!(
            parser.detail_request("youranimes:1108").unwrap().url,
            "https://youranimes.tw/animes/1108"
        );
        for id in ["1108", "youranimes:", "youranimes:../42", "youranimes1108"] {
            assert!(parser.detail_request(id).is_err(), "{id}");
        }
    }
}
