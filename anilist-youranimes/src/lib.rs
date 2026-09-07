use chrono_tz::Tz;

mod errors;
pub mod fetcher;
mod parser;

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

    #[tokio::test]
    async fn test_fetching() {
        let client = Client::new();
        let source = YourAnimesFetcher::new(client);

        source.list(2020, AnimeSeason::Summer).await.unwrap();
    }

    #[tokio::test]
    async fn test_fetching_latest() {
        let local = Local::now();
        let client = Client::new();
        let source = YourAnimesFetcher::new(client);

        let season = AnimeSeason::try_from(local.month() as u8).unwrap();

        source.list(local.year() as u16, season).await.unwrap();
    }
}
