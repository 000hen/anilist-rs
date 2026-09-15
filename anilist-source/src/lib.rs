use anilist_core::{anime::Anime, season::AnimeSeason};

use crate::error::{ParseError, SourceError};

pub mod error;

#[allow(async_fn_in_trait)]
pub trait AnimeSource {
    async fn list(&self, year: u16, season: AnimeSeason) -> Result<Vec<Anime>, SourceError>;
    async fn search(&self, keyword: &str) -> Result<Vec<Anime>, SourceError>;
    async fn detail(&self, id: &str) -> Result<Anime, SourceError>;
}

pub trait AnimeParser {
    fn source_id(&self) -> &'static str;
    fn parse_list(&self, content: &str) -> Result<Vec<Anime>, ParseError>;
    fn parse_detail(&self, content: &str) -> Result<Anime, ParseError>;
    fn parse_search(&self, content: &str) -> Result<Vec<String>, ParseError>;
}
