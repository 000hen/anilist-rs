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

/// Links without a URL appear on some older seasons; they cannot be opened and are skipped.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimeInformation {
    #[serde(rename = "_id")]
    id: String,
    name: String,
    cover: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    adult_content: bool,
    #[serde(default)]
    adultstreaming: Vec<Streaming>,
    #[serde(default)]
    streaming: Vec<Streaming>,
    #[serde(default)]
    cast: Vec<Cast>,
    #[serde(default)]
    olinks: Vec<SocialLink>,
    #[serde(default)]
    tags: HashMap<String, i32>,
    #[serde(default, deserialize_with = "deserialize_weekday")]
    day_of_week: Option<Weekday>,
    #[serde(rename = "date", default, deserialize_with = "deserialize_time_in_day")]
    time_in_day: Option<Minute>,
}

#[derive(Debug, Deserialize)]
struct Cast {
    name: String,
}

#[derive(Debug, Deserialize)]
struct SocialLink {
    title: String,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Streaming {
    #[serde(default)]
    url: Option<String>,
    vendor: String,
    #[serde(default)]
    vendor_local_name: Option<String>,
}

impl AnimeInformation {
    /// `vendors` maps vendor IDs to display names; it is only present on season pages.
    pub fn into_anime(self, vendors: &HashMap<String, String>) -> Anime {
        let on_air_time = self.day_of_week.map(|week| AnimeTime {
            week,
            minute: self.time_in_day,
            zone: "Asia/Tokyo".to_owned(),
        });

        let is_adult = self.adult_content || !self.adultstreaming.is_empty();
        let mut seen_urls = HashSet::new();
        let streaming = self
            .streaming
            .into_iter()
            .chain(self.adultstreaming)
            .filter_map(|stream| {
                let url = stream.url?;
                seen_urls.insert(url.clone()).then(|| AnimeStreaming {
                    name: vendors
                        .get(&stream.vendor)
                        .cloned()
                        .or(stream.vendor_local_name)
                        .unwrap_or_else(|| stream.vendor.clone()),
                    logo: format!("{VENDOR_ICON_BASE_URL}/{}_icon.webp", stream.vendor),
                    url,
                })
            })
            .collect();

        Anime {
            id: format!("{ID_PREFIX}:{}", self.id),
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
                .filter_map(|link| {
                    Some(AnimeSite {
                        url: link.url?,
                        title: link.title,
                    })
                })
                .collect(),
        }
    }
}

fn deserialize_weekday<'de, D>(deserializer: D) -> Result<Option<Weekday>, D::Error>
where
    D: Deserializer<'de>,
{
    // Upstream counts from Sunday and uses strings such as "未定" for unknown days.
    let raw = match Value::deserialize(deserializer)? {
        Value::Number(number) => number.as_u64(),
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
    // Upstream uses "YYYY-MM-DD HH:MM", or a placeholder such as "未定".
    let value = Value::deserialize(deserializer)?;
    let mut parts = value
        .as_str()
        .and_then(|raw| raw.split_whitespace().nth(1))
        .into_iter()
        .flat_map(|time| time.split(':'));
    let (Some(hour), Some(minute)) = (parts.next(), parts.next()) else {
        return Ok(None);
    };

    let hour: u16 = hour.parse().map_err(serde::de::Error::custom)?;
    let minute: u16 = minute.parse().map_err(serde::de::Error::custom)?;
    if hour > 23 || minute > 59 {
        return Err(serde::de::Error::custom("time is outside 00:00..23:59"));
    }

    Ok(Minute::new(hour * 60 + minute))
}
