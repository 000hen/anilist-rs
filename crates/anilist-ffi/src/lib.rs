mod error;
mod model;
pub mod source;

#[cfg(feature = "http")]
pub mod http;

pub use error::*;
pub use model::*;

#[cfg(feature = "system-timezone")]
use anilist_core::get_current_week_order;

uniffi::setup_scaffolding!();

#[cfg(feature = "system-timezone")]
#[uniffi::export]
pub fn current_week_order() -> Vec<ScheduleDay> {
    get_current_week_order()
        .into_iter()
        .map(Into::into)
        .collect()
}

/// Order the week using the caller's local day, without reading the system clock.
#[uniffi::export]
pub fn week_order(today: Weekday) -> Vec<ScheduleDay> {
    anilist_core::get_week_order(today.into())
        .into_iter()
        .map(Into::into)
        .collect()
}
