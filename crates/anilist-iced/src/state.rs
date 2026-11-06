use std::collections::HashMap;

use anilist_core::{ScheduleDay, anime::Anime, season::AnimeSeason};
use anilist_source::AnimeFetcher;
use anilist_youranimes::fetcher::YourAnimesFetcher;
use chrono::{Datelike, Local};
use iced::{
    Element, Size, Subscription, Task,
    widget::{center, container},
    window::{self, Settings, icon},
};
use reqwest::Client;

use crate::{
    component::imager::{Imager, ImagerMessage},
    window::{detail, list},
};

static ICON_BYTES: &[u8] = include_bytes!("./image/icon.ico");

#[derive(Default)]
pub struct AppState {
    pub animes: HashMap<String, Anime>,
    pub animes_week: HashMap<ScheduleDay, Vec<String>>,
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateAnime(u16, AnimeSeason),
    AnimeInformation(Option<Vec<Anime>>),

    CloseWindow(window::Id),
    OpenWeb(String),
    OpenDetail(String),
    DetailOpened {
        window_id: window::Id,
        anime_id: String,
    },

    Image(ImagerMessage),
}

pub struct App {
    main_window: window::Id,

    pub source: YourAnimesFetcher,
    pub image: Imager,
    pub state: AppState,

    detail_windows: HashMap<window::Id, String>,
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
        let imager = Imager::new(client);

        let now = Local::now();
        let year = now.year() as u16;
        let season =
            AnimeSeason::try_from(now.month() as u8).expect("month should map to a season");

        let (id, task) = window::open(create_window_settings());
        (
            Self {
                main_window: id,
                source,
                image: imager,
                state: AppState::default(),
                detail_windows: HashMap::new(),
            },
            Task::batch([task.discard(), Task::done(Message::UpdateAnime(year, season))]),
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
            if let Some(mut animes) = data {
                animes.sort_by(|a, b| a.on_air_time.cmp(&b.on_air_time));

                app.state.animes = animes
                    .into_iter()
                    .map(|anime| {
                        let week = anime
                            .on_air_time
                            .as_ref()
                            .map_or(ScheduleDay::Unknown, |time| ScheduleDay::Weekday(time.week));

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
        Message::OpenDetail(anime_id) => {
            let (_, task) = window::open(Settings {
                size: Size::new(700.0, 900.0),
                ..create_window_settings()
            });

            task.map(move |window_id| Message::DetailOpened {
                window_id,
                anime_id: anime_id.clone(),
            })
        }
        Message::DetailOpened {
            window_id,
            anime_id,
        } => {
            app.detail_windows.insert(window_id, anime_id);
            Task::none()
        }
        Message::OpenWeb(url) => {
            let _ = open::that(url);
            Task::none()
        }
        Message::CloseWindow(id) => {
            app.detail_windows.remove(&id);

            if id == app.main_window {
                iced::exit()
            } else {
                Task::none()
            }
        }
    }
}

pub fn window_close_subscription(_: &App) -> Subscription<Message> {
    window::close_events().map(Message::CloseWindow)
}

fn detail_anime(app: &App, window_id: window::Id) -> Option<&Anime> {
    app.detail_windows
        .get(&window_id)
        .and_then(|anime_id| app.state.animes.get(anime_id))
}

pub fn view(app: &App, window_id: window::Id) -> Element<'_, Message> {
    if window_id == app.main_window {
        return list::view(app);
    }

    match detail_anime(app, window_id) {
        Some(anime) => detail::view(&app.image, anime),
        None => center(container("找不到這部動畫")).into(),
    }
}

pub fn title(app: &App, window_id: window::Id) -> String {
    detail_anime(app, window_id).map_or_else(|| "Anilist".to_owned(), |anime| anime.name.clone())
}
