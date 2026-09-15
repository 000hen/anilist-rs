mod error;
mod model;
mod nextjs;

pub use error::*;
pub use model::*;
pub use nextjs::{NextJsError, deserialize_nextjs};

use std::{ffi::CString, os::raw::c_char, str::FromStr, sync::OnceLock};

use anilist_core::get_current_week_order;
use anilist_source::AnimeSource;
use anilist_youranimes::fetcher::YourAnimesFetcher;

uniffi::setup_scaffolding!();

static YOUR_ANIMES: OnceLock<YourAnimesFetcher> = OnceLock::new();

#[unsafe(no_mangle)]
pub fn version() -> *const c_char {
    let ver = env!("CARGO_PKG_VERSION");
    let cstr = CString::from_str(&ver).expect("Cannot format version into CString");
    cstr.as_ptr()
}

fn your_animes() -> &'static YourAnimesFetcher {
    YOUR_ANIMES.get_or_init(|| YourAnimesFetcher::new(reqwest::Client::new()))
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_list(year: u16, season: AnimeSeason) -> Result<Vec<Anime>, AnilistError> {
    let animes = your_animes()
        .list(year, season.into())
        .await
        .map_err(AnilistError::from)?;

    Ok(animes.into_iter().map(Anime::from).collect())
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_search(keyword: String) -> Result<Vec<Anime>, AnilistError> {
    let animes = your_animes()
        .search(&keyword)
        .await
        .map_err(AnilistError::from)?;

    Ok(animes.into_iter().map(Anime::from).collect())
}

#[uniffi::export(async_runtime = "tokio")]
pub async fn anilist_detail(id: String) -> Result<Anime, AnilistError> {
    let anime = your_animes()
        .detail(&id)
        .await
        .map_err(AnilistError::from)?;

    Ok(Anime::from(anime))
}

#[uniffi::export]
pub fn current_week_order() -> Vec<ScheduleDay> {
    get_current_week_order()
        .into_iter()
        .map(Into::into)
        .collect()
}
