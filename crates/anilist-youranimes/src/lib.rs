#[cfg(feature = "http")]
pub mod fetcher;
mod format;
pub mod parser;

const ID_PREFIX: &str = "youranimes";

#[cfg(all(test, feature = "http", feature = "system-timezone"))]
mod live_tests {
    use anilist_core::season::AnimeSeason;
    use anilist_source::AnimeFetcher;
    use chrono::{Datelike, Local};

    use crate::fetcher::YourAnimesFetcher;

    fn fetcher() -> YourAnimesFetcher {
        YourAnimesFetcher::new(reqwest::Client::new())
    }

    #[tokio::test]
    #[ignore = "requires network access to youranimes.tw"]
    async fn lists_the_current_season() {
        let now = Local::now();
        let season = AnimeSeason::try_from(now.month() as u8).unwrap();
        let animes = fetcher().list(now.year() as u16, season).await.unwrap();
        assert!(!animes.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires network access to youranimes.tw"]
    async fn fetches_detail_and_search() {
        let detail = fetcher().detail("youranimes:1108").await.unwrap();
        assert_eq!(detail.name, "小林家的龍女僕S");

        let results = fetcher().search("女僕").await.unwrap();
        assert!(!results.is_empty());
    }
}
