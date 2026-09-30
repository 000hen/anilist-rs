use super::shell;
use crate::{pages, services};
use anilist_core::anime::Anime;
use windows_reactor::*;

pub struct DetailWindow {
    error: Option<String>,
    width: f64,
}
pub enum Message {
    Open(String),
    Opened(Result<(), String>),
    Resized(f64),
}

impl Component for DetailWindow {
    type Input = Anime;
    type Message = Message;
    fn create(_: &Anime, _: &ComponentContext<Self>) -> Self {
        Self {
            error: None,
            width: 980.0,
        }
    }
    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Open(url) => {
                self.error = None;
                context.spawn_background_with_rejection(
                    move |_| Message::Opened(services::open_web(&url)),
                    Message::Opened(Err("無法開啟瀏覽器，請重試。".into())),
                );
            }
            Message::Opened(result) => self.error = result.err(),
            Message::Resized(width) => self.width = width,
        }
    }
    fn view(&self, anime: &Anime, context: &mut ViewContext<Self>) -> View {
        context.window_title(format!("{} · Anilist", anime.name));
        context.window_visuals(shell::visuals(980.0, 860.0));
        context.on_window_size(context.callback(|size: WindowSize| Message::Resized(size.width)));
        shell::frame(
            "Anilist",
            "動畫詳細資料",
            pages::detail::view(
                anime,
                self.width,
                self.error.as_deref(),
                context.callback(Message::Open),
            ),
        )
    }
}
