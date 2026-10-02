#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod app;
#[cfg(any(windows, test))]
mod catalog;
#[cfg(windows)]
mod components;
#[cfg(windows)]
mod pages;
#[cfg(windows)]
mod services;
#[cfg(all(windows, test))]
mod view_tests;
#[cfg(windows)]
mod windows;

#[cfg(windows)]
fn main() {
    windows_reactor::App::run_component::<app::Application>(())
        .expect("Unable to start Anilist. Windows App Runtime 2.4 or newer is required.");
}

#[cfg(not(windows))]
fn main() {
    eprintln!("anilist-winui requires Windows. Use anilist-iced on this platform.");
    std::process::exit(1);
}
