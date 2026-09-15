#[cfg(feature = "system-timezone")]
use chrono_tz::Tz;

mod errors;
#[cfg(feature = "http")]
pub mod fetcher;
mod format;
pub mod parser;

pub use errors::YourAnimesParseError;

const ID_PREFIX: &str = "youranimes";

#[cfg(feature = "system-timezone")]
pub fn system_timezone() -> Result<Tz, anilist_core::time::ZoneConversionError> {
    iana_time_zone::get_timezone()
        .ok()
        .and_then(|name| name.parse().ok())
        .ok_or(anilist_core::time::ZoneConversionError::UnknownTimeZone)
}

#[cfg(all(test, feature = "http"))]
mod tests {
    use anilist_core::season::AnimeSeason;
    use anilist_source::AnimeSource;
    #[cfg(feature = "system-timezone")]
    use chrono::{Datelike, Local};
    use reqwest::Client;

    use crate::fetcher::YourAnimesFetcher;

    fn create_source() -> YourAnimesFetcher {
        let client = Client::new();
        let source = YourAnimesFetcher::new(client);

        source
    }

    #[tokio::test]
    async fn test_fetching() {
        create_source()
            .list(2020, AnimeSeason::Summer)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "system-timezone")]
    async fn test_fetching_latest() {
        let local = Local::now();
        let season = AnimeSeason::try_from(local.month() as u8).unwrap();

        create_source()
            .list(local.year() as u16, season)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_fetching_detail() {
        let detail = create_source().detail("youranimes:1108").await.unwrap();
        assert_eq!(detail.name, "小林家的龍女僕S");
    }

    #[tokio::test]
    async fn test_fetching_search() {
        let detail = create_source().search("女僕").await.unwrap();
        println!("Results: {}", detail[0]);
    }
}
