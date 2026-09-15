use anilist_core::anime::Anime;
use anilist_nextjs::NextJsData;

use crate::{errors::YourAnimesParseError, format::anime::AnimeInformation};

pub fn parse_list(content: &str) -> Result<Vec<Anime>, YourAnimesParseError> {
    let parsed: Vec<AnimeInformation> = NextJsData::parse(content)?.deserialize("/3/animes")?;

    Ok(parsed
        .into_iter()
        .map(AnimeInformation::into_anime)
        .collect())
}
