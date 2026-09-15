mod error;
mod model;
pub mod parser;

#[cfg(feature = "http")]
pub mod http;

pub use error::*;
pub use model::*;

use std::os::raw::c_char;

#[cfg(feature = "system-timezone")]
use anilist_core::get_current_week_order;

uniffi::setup_scaffolding!();

#[unsafe(no_mangle)]
pub extern "C" fn version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

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
