use std::collections::HashMap;

use anilist_core::{ScheduleDay, anime::Anime, season::AnimeSeason};
use anilist_source::AnimeSource;
use anilist_youranimes::fetcher::YourAnimesFetcher;
use chrono::{Datelike, Local};
use iced::{
    Element, Size, Subscription, Task,
    widget::{center, container},
    window::{self, Settings, icon},
};
use reqwest::Client;

use crate::{
    component::{
        anime,
        imager::{Imager, ImagerMessage},
    },
    window::{detail, list},
};

static ICON_BYTES: &[u8] = include_bytes!("./image/icon.png");

#[derive(Debug, Clone)]
pub enum WindowType {
    MainWindow,
    AnimeDetailWindow { anime_id: String },
}

pub struct AppState {
    pub animes: HashMap<String, Anime>,
    pub animes_week: HashMap<ScheduleDay, Vec<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            animes: Default::default(),
            animes_week: Default::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateAnime(u16, AnimeSeason),
    AnimeInformation(Option<Vec<Anime>>),

    CloseWindow(window::Id),
    OpenWeb(String),
    OpenWindow {
        window_id: window::Id,
        window_type: WindowType,
    },

    Image(ImagerMessage),
    Anime(anime::Message),
}

pub struct App {
    main_window: window::Id,

    pub source: YourAnimesFetcher,
    pub image: Imager,
    pub state: AppState,

    windows: HashMap<window::Id, WindowType>,
}

fn create_window_settings() -> Settings {
    let icon = icon::from_file_data(ICON_BYTES, None).expect("failed to load app icon");
    Settings {
        icon: Some(icon),
        ..Default::default()
    }
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let client = Client::new();
        let source = YourAnimesFetcher::new(client.clone());
        let imager = Imager::new(client.clone());

        let now = Local::now();
        let year = now.year() as u16;
        let season =
            AnimeSeason::try_from(now.month() as u8).expect("month should map to a season");

        let (id, task) = window::open(create_window_settings());
        (
            Self {
                main_window: id,
                source: source,
                image: imager,
                windows: Default::default(),
                state: Default::default(),
            },
            Task::batch([
                task.map(move |id| Message::OpenWindow {
                    window_id: id,
                    window_type: WindowType::MainWindow,
                }),
                Task::done(Message::UpdateAnime(year, season)),
            ]),
        )
    }
}

pub fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::UpdateAnime(year, season) => {
            let source = app.source.clone();
            Task::perform(async move { source.list(year, season).await }, |data| {
                Message::AnimeInformation(data.ok())
            })
        }
        Message::AnimeInformation(data) => {
            if let Some(animes) = data {
                let mut items = animes;
                items.sort_by(|a, b| a.on_air_time.cmp(&b.on_air_time));

                app.state.animes = items
                    .into_iter()
                    .map(|anime| {
                        let week = anime
                            .on_air_time
                            .map(|d| ScheduleDay::Weekday(d.week))
                            .unwrap_or(ScheduleDay::Unknown);

                        app.state
                            .animes_week
                            .entry(week)
                            .or_default()
                            .push(anime.id.clone());

                        (anime.id.clone(), anime)
                    })
                    .collect();
            }
            Task::none()
        }
        Message::Image(message) => app.image.update(message).map(Message::Image),
        Message::Anime(anime::Message::Clicked(anime_id)) => {
            let (_, task) = window::open(Settings {
                size: Size::new(700.0, 900.0),
                ..create_window_settings()
            });

            task.map(move |id| Message::OpenWindow {
                window_id: id,
                window_type: WindowType::AnimeDetailWindow {
                    anime_id: anime_id.clone(),
                },
            })
        }

        Message::OpenWeb(url) => {
            let _ = open::that(url);
            Task::none()
        }

        Message::CloseWindow(id) => {
            app.windows.remove(&id);

            if id == app.main_window {
                iced::exit()
            } else {
                Task::none()
            }
        }

        Message::OpenWindow {
            window_id,
            window_type,
        } => {
            app.windows.insert(window_id, window_type);
            Task::none()
        }
    }
}

pub fn window_close_subscription(_: &App) -> Subscription<Message> {
    window::close_events().map(Message::CloseWindow)
}

pub fn view(app: &App, window_id: window::Id) -> Element<'_, Message> {
    if window_id == app.main_window {
        return list::view(app);
    }

    match app.windows.get(&window_id) {
        Some(WindowType::MainWindow) => list::view(app),
        Some(WindowType::AnimeDetailWindow { anime_id }) => match app.state.animes.get(anime_id) {
            Some(anime) => detail::view(&app.image, anime),
            None => center(container("Sorry, not found it")).into(),
        },
        None => container("Not implemented").into(),
    }
}

pub fn title(app: &App, window_id: window::Id) -> String {
    match app.windows.get(&window_id) {
        Some(WindowType::AnimeDetailWindow { anime_id }) => app
            .state
            .animes
            .get(anime_id)
            .and_then(|anime| Some(anime.name.clone()))
            .unwrap_or("Unknown".to_owned()),
        _ => "Anilist".to_owned(),
    }
}
