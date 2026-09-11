use anilist_core::anime::Anime;
use chrono::Local;
use scraper::Html;

use crate::{
    errors::YourAnimesError,
    format::anime::AnimeInformation,
    parser::nextjs::{JsonPath, NextJsExtractor},
    system_timezone,
};

const MATCH_FORMAT: &str = "{\\\"anime\\\":";
const ANIME_LIST_PATH: &[JsonPath<'static>] = &[
    JsonPath::Index(1),
    JsonPath::Index(3),
    JsonPath::Key("anime"),
];

pub fn parse_detail(content: &str) -> Result<Anime, YourAnimesError> {
    let document = Html::parse_document(content);

    let parsed: AnimeInformation =
        NextJsExtractor::extract(&document, MATCH_FORMAT, ANIME_LIST_PATH)?;

    let timezone = system_timezone();
    let reference_date = Local::now().date_naive();

    let anime_id = parsed.id.clone();

    parsed
        .into_anime(timezone, reference_date)
        .map_err(|source| YourAnimesError::AnimeConversion { anime_id, source })
}
