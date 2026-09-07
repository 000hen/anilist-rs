#![windows_subsystem = "windows"]

use anilist_iced::state::{App, update, view, window_close_subscription};

fn main() -> iced::Result {
    iced::daemon(App::new, update, view)
        .subscription(window_close_subscription)
        .run()
}
