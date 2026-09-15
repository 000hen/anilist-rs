use std::collections::{HashMap, HashSet};

use anilist_core::{
    anime::{Anime, AnimeSite, AnimeStreaming},
    minute::Minute,
    time::AnimeTime,
};
use chrono::Weekday;
use serde::{Deserialize, Deserializer};
use serde_json::Value;

use crate::ID_PREFIX;

const VENDOR_ICON_BASE_URL: &str = "https://d28s5ztqvkii64.cloudfront.net/images";

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeInformation {
    #[serde(rename = "_id")]
    pub id: String,

    #[serde(default)]
    pub adult_content: bool,
    pub adultstreaming: Vec<Streaming>,
    #[serde(default)]
    pub aliases: Vec<String>,
    pub ani_type: String,
    #[serde(default)]
    pub cast: Vec<Cast>,
    pub comment_count: i64,
    #[serde(default)]
    pub copyright: Option<String>,
    pub cover: String,

    #[serde(default, deserialize_with = "deserialize_weekday")]
    pub day_of_week: Option<Weekday>,

    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub duration_desc: Option<String>,
    pub episode: String,
    pub favorability: Favorability,
    #[serde(default)]
    pub jp_name: Option<String>,
    pub name: String,
    #[serde(default)]
    pub olinks: Vec<SocialLink>,
    #[serde(default)]
    pub season_item_date_modified: Option<String>,
    #[serde(default)]
    pub songs: Vec<Song>,
    #[serde(default)]
    pub staff: Vec<Staff>,
    pub status: String,
    pub streaming: Vec<Streaming>,
    #[serde(default)]
    pub studios: Vec<Studio>,
    #[serde(default)]
    pub tags: HashMap<String, i32>,

    #[serde(rename = "date", default, deserialize_with = "deserialize_time_in_day")]
    pub time_in_day: Option<Minute>,
}

impl AnimeInformation {
    pub fn into_anime(self) -> Anime {
        let on_air_time = self.day_of_week.map(|week| AnimeTime {
            week,
            minute: self.time_in_day,
            zone: "Asia/Tokyo".to_owned(),
        });

        let is_adult = self.adult_content || !self.adultstreaming.is_empty();
        let mut seen_streams = HashSet::new();
        let streaming = self
            .streaming
            .into_iter()
            .chain(self.adultstreaming)
            .filter(|stream| seen_streams.insert(stream.url.clone()))
            .map(|stream| AnimeStreaming {
                name: stream.vendor_local_name,
                url: stream.url,
                logo: format!("{VENDOR_ICON_BASE_URL}/{}_icon.webp", stream.vendor),
            })
            .collect();

        Anime {
            id: format!("{}:{}", ID_PREFIX, self.id),
            name: self.name,
            description: self.description,
            on_air_time,
            is_adult,
            image: Some(self.cover),
            banner: None,
            cast: self.cast.into_iter().map(|cast| cast.name).collect(),
            genres: self.tags.into_keys().collect(),
            streaming,
            site: self
                .olinks
                .into_iter()
                .map(|link| AnimeSite {
                    title: link.title,
                    url: link.url,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Cast {
    pub name: String,
    pub role: String,
    #[serde(default)]
    pub character: Option<String>,
    #[serde(default)]
    pub voice: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Favorability {
    pub average: f64,
    pub counts: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct SocialLink {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Song {
    pub credits: Vec<Credit>,
    pub title: String,
    #[serde(rename = "type")]
    pub kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Credit {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Staff {
    pub credits: Vec<CreditOwner>,
    pub role: String,
    #[serde(default)]
    pub job_title: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct CreditOwner {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Streaming {
    pub payment: Option<String>,
    pub title: String,
    pub url: String,
    pub vendor: String,
    pub vendor_local_name: String,
    pub has_zh_cn_subtitle: Option<bool>,
    pub has_zh_hk_subtitle: Option<bool>,
    pub ad_url: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingWire {
    #[serde(default)]
    payment: Option<String>,
    title: String,
    url: String,
    vendor: String,
    #[serde(default)]
    vendor_local_name: Option<String>,
    #[serde(default)]
    has_zh_cn_subtitle: Option<bool>,
    #[serde(default)]
    has_zh_hk_subtitle: Option<bool>,
    #[serde(default)]
    ad_url: Option<String>,
}

impl<'de> Deserialize<'de> for Streaming {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = StreamingWire::deserialize(deserializer)?;
        let vendor_local_name = wire
            .vendor_local_name
            .unwrap_or_else(|| wire.vendor.clone());

        Ok(Self {
            payment: wire.payment,
            title: wire.title,
            url: wire.url,
            vendor: wire.vendor,
            vendor_local_name,
            has_zh_cn_subtitle: wire.has_zh_cn_subtitle,
            has_zh_hk_subtitle: wire.has_zh_hk_subtitle,
            ad_url: wire.ad_url,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Studio {
    pub name: String,
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Option<Weekday>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    let raw = match value {
        Value::Number(number) => number.as_i64(),
        Value::String(value) => value.parse().ok(),
        _ => None,
    };

    Ok(match raw {
        Some(0) => Some(Weekday::Sun),
        Some(1) => Some(Weekday::Mon),
        Some(2) => Some(Weekday::Tue),
        Some(3) => Some(Weekday::Wed),
        Some(4) => Some(Weekday::Thu),
        Some(5) => Some(Weekday::Fri),
        Some(6) => Some(Weekday::Sat),
        _ => None,
    })
}

fn deserialize_time_in_day<'de, D>(deserializer: D) -> Result<Option<Minute>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    let Some(raw) = value.as_str() else {
        return Ok(None);
    };
    let Some(time) = raw.split_whitespace().nth(1) else {
        return Ok(None);
    };

    let mut parts = time.split(':');
    let Some(hour) = parts.next() else {
        return Ok(None);
    };
    let Some(minute) = parts.next() else {
        return Ok(None);
    };

    let hour: u16 = hour.parse().map_err(serde::de::Error::custom)?;
    let minute: u16 = minute.parse().map_err(serde::de::Error::custom)?;
    if hour > 23 || minute > 59 {
        return Err(serde::de::Error::custom("time is outside 00:00..23:59"));
    }
    let total = hour
        .checked_mul(60)
        .and_then(|hour| hour.checked_add(minute))
        .and_then(Minute::new)
        .ok_or_else(|| serde::de::Error::custom("time is outside 00:00..23:59"))?;

    Ok(Some(total))
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;

    fn anime_json(day_of_week: Value, date: Value) -> Value {
        json!({
            "_id": "anime-id",
            "adultstreaming": [{
                "title": "Prime",
                "url": "https://example.com/watch",
                "vendor": "prime"
            }],
            "aniType": "TV",
            "cast": [{ "name": "Actor", "role": "Lead" }],
            "commentCount": 0,
            "cover": "https://example.com/cover.webp",
            "cross": false,
            "dayOfWeek": day_of_week,
            "date": date,
            "episode": "1",
            "favorability": { "average": 5.0, "counts": 1 },
            "hasNews": false,
            "isFavorite": false,
            "name": "Anime",
            "newsDate": "",
            "olinks": [{ "title": "Official", "url": "https://example.com" }],
            "playDate": "",
            "playEps": 1,
            "playTotal": 12,
            "scheduleDate": "",
            "seriesStatus": "",
            "songs": [],
            "staff": [],
            "status": "",
            "streaming": [{
                "title": "Prime",
                "url": "https://example.com/watch",
                "vendor": "prime"
            }],
            "studios": [],
            "tags": { "Action": 1, "Fantasy": 2, "Comedy": 3 },
            "twAgent": "",
            "updatedTimestamp": ""
        })
    }

    #[test]
    fn deserializes_upstream_defaults_and_schedule() {
        let information: AnimeInformation =
            serde_json::from_value(anime_json(json!(0), json!("2026-07-05 23:45"))).unwrap();

        assert!(!information.adult_content);
        assert!(information.aliases.is_empty());
        assert_eq!(information.description, "");
        assert_eq!(information.day_of_week, Some(Weekday::Sun));
        assert_eq!(information.time_in_day.map(Minute::get), Some(23 * 60 + 45));
        assert_eq!(information.streaming[0].vendor_local_name, "prime");
    }

    #[test]
    fn treats_upstream_schedule_markers_as_unknown() {
        let information: AnimeInformation =
            serde_json::from_value(anime_json(json!("未定"), json!("未定"))).unwrap();

        assert_eq!(information.day_of_week, None);
        assert_eq!(information.time_in_day, None);
    }

    #[test]
    fn converts_to_domain_anime_and_removes_duplicate_streams() {
        let information: AnimeInformation =
            serde_json::from_value(anime_json(json!(1), json!("2026-07-06 12:30"))).unwrap();

        let anime = information.into_anime();

        assert_eq!(anime.id, "youranimes:anime-id");
        assert_eq!(anime.on_air_time.unwrap().week, Weekday::Mon);
        assert!(anime.is_adult);
        assert_eq!(anime.cast, ["Actor"]);
        let mut genres = anime.genres;
        genres.sort();
        assert_eq!(genres, ["Action", "Comedy", "Fantasy"]);
        assert_eq!(anime.streaming.len(), 1);
        assert_eq!(anime.streaming[0].name, "prime");
        assert_eq!(
            anime.streaming[0].logo,
            "https://d28s5ztqvkii64.cloudfront.net/images/prime_icon.webp"
        );
        assert_eq!(anime.site[0].title, "Official");
    }

    #[test]
    fn deserializes_every_upstream_weekday_and_rejects_non_weekdays() {
        let cases = [
            (json!(0), Some(Weekday::Sun)),
            (json!(1), Some(Weekday::Mon)),
            (json!(2), Some(Weekday::Tue)),
            (json!(3), Some(Weekday::Wed)),
            (json!(4), Some(Weekday::Thu)),
            (json!(5), Some(Weekday::Fri)),
            (json!(6), Some(Weekday::Sat)),
            (json!("1"), Some(Weekday::Mon)),
            (json!(7), None),
            (json!(-1), None),
            (Value::Null, None),
        ];

        for (raw, expected) in cases {
            let information: AnimeInformation =
                serde_json::from_value(anime_json(raw, Value::Null)).unwrap();
            assert_eq!(information.day_of_week, expected);
        }
    }

    #[test]
    fn preserves_tokyo_time_for_host_owned_conversion() {
        let information: AnimeInformation =
            serde_json::from_value(anime_json(json!(1), json!("2026-07-06 00:30"))).unwrap();

        let anime = information.into_anime();
        let time = anime.on_air_time.unwrap();

        assert_eq!(time.week, Weekday::Mon);
        assert_eq!(time.minute.map(Minute::get), Some(30));
        assert_eq!(time.zone, "Asia/Tokyo");
    }

    #[test]
    fn validates_time_boundaries() {
        for (raw, expected) in [("00:00", 0), ("23:59", 23 * 60 + 59)] {
            let information: AnimeInformation =
                serde_json::from_value(anime_json(json!(1), json!(format!("2026-07-06 {raw}"))))
                    .unwrap();
            assert_eq!(information.time_in_day.map(Minute::get), Some(expected));
        }

        for raw in ["24:00", "23:60", "22:60", "-1:00", "ab:cd", "70000:00"] {
            let result = serde_json::from_value::<AnimeInformation>(anime_json(
                json!(1),
                json!(format!("2026-07-06 {raw}")),
            ));
            assert!(result.is_err(), "{raw} should be rejected");
        }
    }
}
