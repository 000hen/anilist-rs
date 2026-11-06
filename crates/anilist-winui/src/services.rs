use anilist_core::{anime::Anime, season::AnimeSeason};
use anilist_source::AnimeFetcher;
use anilist_youranimes::fetcher::YourAnimesFetcher;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn load_season(year: u16, season: AnimeSeason) -> Result<Vec<Anime>, String> {
    let (runtime, fetcher) = fetcher()?;
    runtime
        .block_on(fetcher.list(year, season))
        .map_err(|e| e.to_string())
}

pub fn search(keyword: &str) -> Result<Vec<Anime>, String> {
    let (runtime, fetcher) = fetcher()?;
    runtime
        .block_on(fetcher.search(keyword))
        .map_err(|e| e.to_string())
}

fn fetcher() -> Result<(Runtime, YourAnimesFetcher), String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(45))
        .build()
        .map_err(|e| e.to_string())?;
    Ok((runtime, YourAnimesFetcher::new(client)))
}

pub fn open_web(url: &str) -> Result<(), String> {
    let parsed = reqwest::Url::parse(url).map_err(|e| e.to_string())?;
    if !matches!(parsed.scheme(), "https" | "http") || parsed.host_str().is_none() {
        return Err("僅支援 HTTP 或 HTTPS 網站連結。".into());
    }
    open::that_detached(parsed.as_str()).map_err(|e| e.to_string())
}
