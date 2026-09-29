use std::sync::Arc;

use anilist_source::AnimeSource;
#[cfg(feature = "youranimes")]
use anilist_youranimes::parser::YourAnimeParser;

use crate::{AnilistError, Anime, AnimeSeason, HttpRequest, SourceInfo};

#[derive(uniffi::Object)]
pub struct NativeAnimeSource {
    source: Box<dyn AnimeSource>,
}

#[uniffi::export]
impl NativeAnimeSource {
    #[uniffi::constructor]
    pub fn new(source_id: String) -> Result<Arc<Self>, AnilistError> {
        Ok(Arc::new(Self {
            source: create_source(&source_id)?,
        }))
    }

    pub fn source_id(&self) -> String {
        self.source.source_id().to_owned()
    }

    pub fn list_request(
        &self,
        year: u16,
        season: AnimeSeason,
    ) -> Result<HttpRequest, AnilistError> {
        self.source
            .list_request(year, season.into())
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn search_request(&self, keyword: String) -> Result<HttpRequest, AnilistError> {
        self.source
            .search_request(&keyword)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn detail_request(&self, id: String) -> Result<HttpRequest, AnilistError> {
        self.source
            .detail_request(&id)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn parse_list(&self, content: String) -> Result<Vec<Anime>, AnilistError> {
        self.source
            .parse_list(&content)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn parse_detail(&self, content: String) -> Result<Anime, AnilistError> {
        self.source
            .parse_detail(&content)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn parse_search(&self, content: String) -> Result<Vec<String>, AnilistError> {
        self.source.parse_search(&content).map_err(Into::into)
    }
}

#[uniffi::export]
pub fn available_sources() -> Vec<SourceInfo> {
    vec![
        #[cfg(feature = "youranimes")]
        SourceInfo {
            id: "youranimes".to_owned(),
            name: "YourAnimes".to_owned(),
        },
    ]
}

fn create_source(source_id: &str) -> Result<Box<dyn AnimeSource>, AnilistError> {
    match source_id {
        #[cfg(feature = "youranimes")]
        "youranimes" => Ok(Box::new(YourAnimeParser::new())),
        _ => Err(AnilistError::UnsupportedSource {
            source_id: source_id.to_owned(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "youranimes")]
    fn registry_exposes_the_compiled_source_and_its_request() {
        let source = NativeAnimeSource::new("youranimes".to_owned()).unwrap();
        assert_eq!(available_sources()[0].id, "youranimes");
        assert_eq!(
            source.list_request(2026, AnimeSeason::Fall).unwrap().url,
            "https://youranimes.tw/bangumi/202610"
        );
    }

    #[test]
    fn registry_rejects_unknown_sources() {
        assert!(NativeAnimeSource::new("unknown".to_owned()).is_err());
    }
}
