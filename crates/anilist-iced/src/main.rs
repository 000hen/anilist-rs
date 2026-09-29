#![windows_subsystem = "windows"]

use anilist_iced::state::{App, title, update, view, window_close_subscription};

fn main() -> iced::Result {
    iced::daemon(App::new, update, view)
        .title(title)
        .subscription(window_close_subscription)
        .run()
}
