use crate::{
    catalog::{Catalog, current_year_season},
    pages::schedule::Event,
    services, windows,
};
use anilist_core::{ScheduleDay, anime::Anime, season::AnimeSeason};
use chrono::{Datelike, Local};
use windows_reactor::*;

pub struct Application {
    pub year: u16,
    pub season: AnimeSeason,
    pub catalog: Catalog,
    pub query: String,
    pub source_query: String,
    pub search_term: Option<String>,
    request: u64,
    pub day: Option<ScheduleDay>,
    pub loading: bool,
    pub error: Option<String>,
    pub width: f64,
}

pub enum Message {
    Loaded(u64, u16, AnimeSeason, Result<Vec<Anime>, String>),
    SourceQuery(String),
    SearchSource,
    Searched(u64, Result<Vec<Anime>, String>),
    Page(Event),
    Resized(f64),
}

impl Application {
    fn load(&mut self, context: &ComponentContext<Self>) {
        self.request += 1;
        let request = self.request;
        if self.search_term.take().is_some() {
            self.catalog = Catalog::new(Vec::new(), Local::now().weekday());
        }
        self.loading = true;
        self.error = None;
        let (year, season) = current_year_season();
        context.spawn_background_with_rejection(
            move |_| Message::Loaded(request, year, season, services::load_season(year, season)),
            Message::Loaded(request, year, season, Err("無法接收資料，請重試。".into())),
        );
    }
}

impl Component for Application {
    type Input = ();
    type Message = Message;
    fn create(_: &(), context: &ComponentContext<Self>) -> Self {
        let (year, season) = current_year_season();
        let mut app = Self {
            year,
            season,
            catalog: Catalog::new(Vec::new(), Local::now().weekday()),
            query: String::new(),
            source_query: String::new(),
            search_term: None,
            request: 0,
            day: None,
            loading: false,
            error: None,
            width: 1200.0,
        };
        app.load(context);
        app
    }
    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Loaded(request, year, season, result) => {
                if request != self.request {
                    return;
                }
                self.loading = false;
                match result {
                    Ok(animes) => {
                        (self.year, self.season) = (year, season);
                        self.catalog = Catalog::new(animes, Local::now().weekday());
                    }
                    Err(error) => self.error = Some(error),
                }
            }
            Message::SourceQuery(query) => self.source_query = query,
            Message::SearchSource => {
                let keyword = self.source_query.trim().to_owned();
                self.query.clear();
                self.day = None;
                if keyword.is_empty() {
                    self.load(context);
                    return;
                }
                self.request += 1;
                let request = self.request;
                self.search_term = Some(keyword.clone());
                self.catalog = Catalog::new(Vec::new(), Local::now().weekday());
                self.loading = true;
                self.error = None;
                context.spawn_background_with_rejection(
                    move |_| Message::Searched(request, services::search(&keyword)),
                    Message::Searched(request, Err("搜尋失敗，請重試。".into())),
                );
            }
            Message::Searched(request, result) => {
                if request != self.request {
                    return;
                }
                self.loading = false;
                match result {
                    Ok(animes) => self.catalog = Catalog::new(animes, Local::now().weekday()),
                    Err(error) => self.error = Some(error),
                }
            }
            Message::Page(event) => match event {
                Event::Refresh => {
                    if self.search_term.is_some() {
                        self.update(Message::SearchSource, context);
                    } else {
                        self.load(context);
                    }
                }
                Event::Search(query) => self.query = query,
                Event::Day(day) => self.day = day,
                Event::ClearFilters => {
                    self.query.clear();
                    self.day = None;
                }
                Event::Open(anime) => {
                    if !context
                        .open_window(View::component::<windows::detail::DetailWindow>(*anime))
                    {
                        self.error = Some("無法開啟詳細資料視窗，請再試一次。".into());
                    }
                }
            },
            Message::Resized(width) => self.width = width,
        }
    }
    fn view(&self, _: &(), context: &mut ViewContext<Self>) -> View {
        windows::main::view(self, context)
    }
}
