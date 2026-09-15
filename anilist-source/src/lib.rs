use std::pin::Pin;

use anilist_core::{anime::Anime, season::AnimeSeason};

use crate::error::{ParseError, SourceError};

pub mod error;

pub type SourceFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, SourceError>> + Send + 'a>>;

pub trait AnimeSource: Send + Sync {
    fn source_id(&self) -> &'static str;
    fn list(&self, year: u16, season: AnimeSeason) -> SourceFuture<'_, Vec<Anime>>;
    fn search<'a>(&'a self, keyword: &'a str) -> SourceFuture<'a, Vec<Anime>>;
    fn detail<'a>(&'a self, id: &'a str) -> SourceFuture<'a, Anime>;
}

pub trait AnimeParser: Send + Sync {
    fn source_id(&self) -> &'static str;
    fn parse_list(&self, content: &str) -> Result<Vec<Anime>, ParseError>;
    fn parse_detail(&self, content: &str) -> Result<Anime, ParseError>;
    fn parse_search(&self, content: &str) -> Result<Vec<String>, ParseError>;
}
