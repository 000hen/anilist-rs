use crate::{
    catalog::{Catalog, current_year_season},
    pages::schedule::Event,
    services, windows,
};
use anilist_core::{anime::Anime, season::AnimeSeason};
use chrono::{Datelike, Local};
use windows_reactor::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MainSurface {
    Schedule,
    Search,
}

pub struct Application {
    pub year: u16,
    pub season: AnimeSeason,
    pub catalog: Catalog,
    pub surface: MainSurface,
    pub source_query: String,
    pub search_term: Option<String>,
    pub search_results: Vec<Anime>,
    search_request: u64,
    pub search_loading: bool,
    pub search_error: Option<String>,
    request: u64,
    pub loading: bool,
    pub error: Option<String>,
    pub width: f64,
}

pub enum Message {
    Loaded(u64, Result<Vec<Anime>, String>),
    SourceQuery(String),
    SearchSource,
    SubmitSearch(String),
    Back,
    Searched(u64, Result<Vec<Anime>, String>),
    Page(Event),
    Resized(f64),
}

impl Application {
    fn finish_load(&mut self, request: u64, result: Result<Vec<Anime>, String>) {
        if request != self.request {
            return;
        }
        self.loading = false;
        match result {
            Ok(animes) => self.catalog = Catalog::new(animes, Local::now().weekday()),
            Err(error) => self.error = Some(error),
        }
    }

    fn finish_search(&mut self, request: u64, result: Result<Vec<Anime>, String>) {
        if request != self.search_request {
            return;
        }
        self.search_loading = false;
        match result {
            Ok(animes) => self.search_results = animes,
            Err(error) => self.search_error = Some(error),
        }
    }
    fn load(&mut self, context: &ComponentContext<Self>) {
        self.request += 1;
        let request = self.request;
        self.loading = true;
        self.error = None;
        let (year, season) = (self.year, self.season);
        context.spawn_background_with_rejection(
            move |_| Message::Loaded(request, services::load_season(year, season)),
            Message::Loaded(request, Err("無法接收資料，請重試。".into())),
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
            surface: MainSurface::Schedule,
            source_query: String::new(),
            search_term: None,
            search_results: Vec::new(),
            search_request: 0,
            search_loading: false,
            search_error: None,
            request: 0,
            loading: false,
            error: None,
            width: 1200.0,
        };
        app.load(context);
        app
    }

    fn update(&mut self, message: Message, context: &ComponentContext<Self>) {
        match message {
            Message::Loaded(request, result) => {
                self.finish_load(request, result);
            }
            Message::SourceQuery(query) => self.source_query = query,
            Message::Back => self.surface = MainSurface::Schedule,
            Message::SubmitSearch(query) => {
                self.source_query = query;
                self.update(Message::SearchSource, context);
            }
            Message::SearchSource => {
                self.surface = MainSurface::Search;
                let keyword = self.source_query.trim().to_owned();
                self.search_request += 1;
                self.search_results.clear();
                self.search_error = None;
                self.search_term = (!keyword.is_empty()).then(|| keyword.clone());
                self.search_loading = !keyword.is_empty();
                if keyword.is_empty() {
                    return;
                }
                let request = self.search_request;
                context.spawn_background_with_rejection(
                    move |_| Message::Searched(request, services::search(&keyword)),
                    Message::Searched(request, Err("搜尋失敗，請重試。".into())),
                );
            }
            Message::Searched(request, result) => {
                self.finish_search(request, result);
            }
            Message::Page(event) => match event {
                Event::Refresh => self.load(context),
                Event::Season(year, season) => {
                    if (year, season) != (self.year, self.season) {
                        (self.year, self.season) = (year, season);
                        self.catalog = Catalog::new(Vec::new(), Local::now().weekday());
                        self.load(context);
                    }
                }
                Event::Open(anime) => {
                    if !context
                        .open_window(View::component::<windows::detail::DetailWindow>(*anime))
                    {
                        let error = Some("無法開啟詳細資料視窗，請再試一次。".into());
                        match self.surface {
                            MainSurface::Schedule => self.error = error,
                            MainSurface::Search => self.search_error = error,
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn application() -> Application {
        Application {
            year: 2026,
            season: AnimeSeason::Summer,
            catalog: Catalog::new(vec![], chrono::Weekday::Wed),
            surface: MainSurface::Search,
            source_query: "new".into(),
            search_term: Some("new".into()),
            search_results: vec![],
            search_request: 2,
            search_loading: true,
            search_error: None,
            request: 1,
            loading: true,
            error: None,
            width: 1200.0,
        }
    }

    #[test]
    fn stale_search_completion_does_not_finish_newer_search() {
        let mut app = application();
        app.finish_search(1, Err("old failure".into()));
        assert!(app.search_loading);
        assert!(app.search_error.is_none());
        app.finish_search(2, Ok(vec![]));
        assert!(!app.search_loading);
        assert!(app.loading, "search must not finish the seasonal request");
    }

    #[test]
    fn search_completion_after_back_preserves_schedule_state() {
        let mut app = application();
        app.surface = MainSurface::Schedule;
        app.error = Some("season error".into());
        app.finish_search(2, Err("search error".into()));
        assert!(app.surface == MainSurface::Schedule);
        assert_eq!(app.error.as_deref(), Some("season error"));
        assert_eq!(app.search_error.as_deref(), Some("search error"));
        assert!(app.loading);
    }
    #[test]
    fn older_season_result_cannot_replace_the_selected_season() {
        let mut app = application();
        app.year = 2025;
        app.season = AnimeSeason::Winter;
        app.request = 3;
        app.finish_load(2, Err("previous season failed".into()));
        assert!(app.loading);
        assert!(app.error.is_none());
        app.finish_load(3, Ok(vec![]));
        assert!(!app.loading);
        assert_eq!((app.year, app.season), (2025, AnimeSeason::Winter));
    }
}
