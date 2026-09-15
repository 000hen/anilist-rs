use anilist_core::anime::Anime;
use anilist_nextjs::NextJsData;

use crate::{errors::YourAnimesParseError, format::anime::AnimeInformation};

pub fn parse_detail(content: &str) -> Result<Anime, YourAnimesParseError> {
    let parsed: AnimeInformation = NextJsData::parse(content)?.deserialize("/1/3/anime")?;

    Ok(parsed.into_anime())
}
