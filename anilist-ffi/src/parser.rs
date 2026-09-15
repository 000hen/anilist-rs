use crate::{AnilistError, Anime};

#[derive(uniffi::Object)]
pub struct NativeAnimeParser {
    parser: Box<dyn AnimeParser>,
}

#[uniffi::export]
impl NativeAnimeParser {
    #[uniffi::constructor]
    pub fn new(source_id: String) -> Result<Arc<Self>, AnilistError> {
        let parser = create_parser(&source_id)?;

        Ok(Arc::new(Self { parser }))
    }

    pub fn parse_list(&self, content: String) -> Result<Vec<Anime>, AnilistError> {
        self.parser
            .parse_list(&content)
            .map(|items| items.into_iter().map(Into::into).collect())
            .map_err(Into::into)
    }

    pub fn parse_detail(&self, content: String) -> Result<Anime, AnilistError> {
        self.parser
            .parse_detail(&content)
            .map(Into::into)
            .map_err(Into::into)
    }

    pub fn parse_search(&self, content: String) -> Result<Vec<String>, AnilistError> {
        self.parser.parse_search(&content).map_err(Into::into)
    }
}
