use std::{ffi::CString, os::raw::c_char};

#[unsafe(no_mangle)]
pub extern "C" fn version() -> *const c_char {
    let version = env!("CARGO_PKG_VERSION");
    let cstr = CString::new(version).expect("Cannot create c_str for version");
    cstr.as_ptr()
}
