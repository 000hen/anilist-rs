use anilist_core::anime::Anime;
use chrono::Local;
use scraper::Html;

use crate::{
    errors::YourAnimesError,
    format::list::AnimeInformation,
    parser::nextjs::{JsonPath, NextJsExtractor},
    system_timezone,
};

const MATCH_FORMAT: &str = "{\\\"animes\\\":[";
const ANIME_LIST_PATH: &[JsonPath<'static>] = &[JsonPath::Index(3), JsonPath::Key("animes")];

pub fn parse_list_source(content: &str) -> Result<Vec<Anime>, YourAnimesError> {
    let document = Html::parse_document(content);

    let parsed: Vec<AnimeInformation> =
        NextJsExtractor::extract(&document, MATCH_FORMAT, ANIME_LIST_PATH)?;

    let timezone = system_timezone();
    let reference_date = Local::now().date_naive();

    parsed
        .into_iter()
        .map(|data| {
            let anime_id = data.id.clone();

            data.into_anime(timezone, reference_date)
                .map_err(|source| YourAnimesError::AnimeConversion { anime_id, source })
        })
        .collect()
}
