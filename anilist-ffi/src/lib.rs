mod error;
mod model;
pub mod parser;

#[cfg(feature = "http")]
pub mod http;

pub use anilist_nextjs_ffi::{NextJsError, deserialize_nextjs};
pub use error::*;
pub use model::*;
anilist_nextjs_ffi::uniffi_reexport_scaffolding!();

use std::{ffi::CString, os::raw::c_char, str::FromStr};

#[cfg(feature = "system-timezone")]
use anilist_core::get_current_week_order;

uniffi::setup_scaffolding!();

#[unsafe(no_mangle)]
pub fn version() -> *const c_char {
    let ver = env!("CARGO_PKG_VERSION");
    let cstr = CString::from_str(&ver).expect("Cannot format version into CString");
    cstr.as_ptr()
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
