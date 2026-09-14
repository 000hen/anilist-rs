use anilist_core::anime::Anime;
use anilist_nextjs::NextJsData;
use chrono::Local;

use crate::{errors::YourAnimesError, format::anime::AnimeInformation, system_timezone};

pub fn parse_detail(content: &str) -> Result<Anime, YourAnimesError> {
    let parsed: AnimeInformation = NextJsData::parse(content)?.deserialize("/1/3/anime")?;

    let timezone = system_timezone();
    let reference_date = Local::now().date_naive();

    let anime_id = parsed.id.clone();

    parsed
        .into_anime(timezone, reference_date)
        .map_err(|source| YourAnimesError::AnimeConversion { anime_id, source })
}
