use chrono_tz::Tz;

mod errors;
pub mod fetcher;
mod format;
mod parser;

const ID_PREFIX: &str = "youranimes";

pub fn system_timezone() -> Tz {
    iana_time_zone::get_timezone()
        .expect("system timezone should be available")
        .parse()
        .expect("system timezone should be a valid IANA timezone")
}

#[cfg(test)]
mod tests {
    use anilist_core::season::AnimeSeason;
    use anilist_source::AnimeSource;
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
