use crate::{
    catalog::Catalog,
    components::{
        anime_card, feedback,
        typography::{heading, text},
    },
};
use anilist_core::{ScheduleDay, anime::Anime, season::AnimeSeason};
use windows_reactor::*;

pub enum Event {
    Refresh,
    Search(String),
    Day(Option<ScheduleDay>),
    ClearFilters,
    Open(Box<Anime>),
}

pub struct Page<'a> {
    pub year: u16,
    pub season: AnimeSeason,
    pub catalog: &'a Catalog,
    pub query: &'a str,
    pub day: Option<ScheduleDay>,
    pub loading: bool,
    pub error: Option<&'a str>,
    pub width: f64,
    pub search_term: Option<&'a str>,
}

pub fn view(page: Page<'_>, on_event: Callback<Event>) -> View {
    let compact = page.width < 640.0;
    let inset = if compact { 16.0 } else { 32.0 };
    let on_refresh = on_event.clone();
    let toolbar = crate::components::gallery_filters::view(
        page.query,
        page.day,
        page.width,
        on_event.clone(),
    );
    let summary = &page.catalog.summary;
    let header = Grid::new()
        .column_spacing(16.0)
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .children((
            StackPanel::new().spacing(8.0).children((
                heading(
                    page.search_term.map_or_else(
                        || format!("{} {}動畫", page.year, page.season),
                        |term| format!("搜尋結果：{term}"),
                    ),
                    28.0,
                ),
                text(format!(
                    "{} 部動畫  ·  今天 {} 部播出  ·  {} 部可觀看",
                    summary.total, summary.today, summary.streamable
                )),
            )),
            Button::new()
                .grid_column(1)
                .vertical_alignment(VerticalAlignment::Center)
                .is_enabled(!page.loading)
                .automation_name("重新整理番表")
                .automation_id("RefreshSchedule")
                .on_click(move || {
                    let _ = on_refresh.call(Event::Refresh);
                })
                .content(
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .children((
                            SymbolIcon::new().symbol(Symbol::Sync),
                            if compact {
                                View::empty()
                            } else {
                                text("重新整理").into()
                            },
                        )),
                ),
        ));
    let animes = page
        .catalog
        .search(page.query)
        .into_iter()
        .filter(|anime| {
            page.day.is_none_or(|day| {
                day == anime
                    .on_air_time
                    .as_ref()
                    .map(|time| ScheduleDay::Weekday(time.week))
                    .unwrap_or(ScheduleDay::Unknown)
            })
        })
        .collect::<Vec<_>>();
    let result_count = animes.len();
    // Account for the shell border, page insets, scrollbar, item padding and gutter.
    let available = (page.width - 26.0 - inset * 2.0 - 16.0).max(200.0);
    let columns = (available / 224.0).floor().max(1.0);
    let cell_width = available / columns;
    let card_width = cell_width - 16.0;
    let content = if animes.is_empty() {
        let clear = on_event.clone();
        let retry = on_event.clone();
        if page.loading {
            feedback::empty(
                if page.search_term.is_some() {
                    "正在搜尋動畫"
                } else {
                    "正在載入本季動畫"
                },
                "正在取得播出時間與動畫資訊…",
                true,
                None,
            )
        } else if page.error.is_some() && page.catalog.summary.total == 0 {
            feedback::empty(
                if page.search_term.is_some() {
                    "暫時無法取得搜尋結果"
                } else {
                    "暫時無法取得番表"
                },
                "請檢查網路連線，然後再試一次。",
                false,
                Some((
                    "重試",
                    Callback::new(move |()| {
                        let _ = retry.call(Event::Refresh);
                    }),
                )),
            )
        } else if !page.query.trim().is_empty() || page.day.is_some() {
            feedback::empty(
                "找不到符合的動畫",
                "試試其他播出日或關鍵字。",
                false,
                Some((
                    "清除篩選",
                    Callback::new(move |()| {
                        let _ = clear.call(Event::ClearFilters);
                    }),
                )),
            )
        } else {
            feedback::empty(
                if page.search_term.is_some() {
                    "找不到符合的動畫"
                } else {
                    "本季尚無動畫資料"
                },
                "稍後重新整理，看看是否有新的番表。",
                false,
                Some((
                    "重新整理",
                    Callback::new(move |()| {
                        let _ = retry.call(Event::Refresh);
                    }),
                )),
            )
        }
    } else {
        let cards = animes.into_iter().map(|anime| {
            let open = on_event.clone();
            let selected = anime.clone();
            KeyedView::new(
                anime.id.clone(),
                GridViewItem::new()
                    .width(cell_width - 8.0)
                    .margin(Thickness::new(0.0, 0.0, 8.0, 8.0))
                    .content(anime_card::view(
                        anime,
                        card_width,
                        Callback::new(move |()| {
                            let _ = open.call(Event::Open(Box::new(selected.clone())));
                        }),
                    )),
            )
        });
        GridView::new()
            .automation_name("動畫海報集")
            .slots([SlotView::collection(GridViewSlot::Items, cards)])
    };

    Grid::new()
        .row_spacing(20.0)
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .children((
            StackPanel::new()
                .margin(Thickness::new(32.0, 32.0, 32.0, 8.0))
                .children((
                    header,
                    Grid::new().grid_row(1).children((toolbar,)),
                    StackPanel::new().grid_row(2).spacing(8.0).children((
                        feedback::error(page.error),
                        text(if page.loading {
                            "更新中…".into()
                        } else {
                            format!("{result_count} 部動畫 · 依播出日與時間排列 · 本機時區")
                        })
                        .font_size(12.0),
                    )),
                )),
            Grid::new()
                .grid_row(3)
                .margin(Thickness::xy(32.0, 0.0))
                .children((content,)),
        ))
}
