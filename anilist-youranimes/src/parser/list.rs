use anilist_core::anime::Anime;
use anilist_nextjs::NextJsData;
use chrono::Local;

use crate::{errors::YourAnimesError, format::anime::AnimeInformation, system_timezone};

pub fn parse_list(content: &str) -> Result<Vec<Anime>, YourAnimesError> {
    let parsed: Vec<AnimeInformation> = NextJsData::parse(content)?.deserialize("/3/animes")?;

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
