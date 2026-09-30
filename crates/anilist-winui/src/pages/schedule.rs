use crate::{
    catalog::Catalog,
    components::{
        anime_card, feedback, season_picker,
        typography::{heading, text},
    },
};
use anilist_core::{ScheduleDay, anime::Anime, season::AnimeSeason};
use chrono::{Datelike, Local};
use windows_reactor::*;

pub enum Event {
    Refresh,
    Season(u16, AnimeSeason),
    Open(Box<Anime>),
}

pub struct Page<'a> {
    pub year: u16,
    pub season: AnimeSeason,
    pub catalog: &'a Catalog,
    pub loading: bool,
    pub error: Option<&'a str>,
    pub width: f64,
}

pub fn day_label(day: ScheduleDay) -> &'static str {
    match day {
        ScheduleDay::Unknown => "時間未定",
        ScheduleDay::Weekday(day) => [
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
            "星期日",
        ][day.num_days_from_monday() as usize],
    }
}

pub fn view(page: Page<'_>, on_event: Callback<Event>) -> View {
    let inset = if page.width < 640.0 { 16.0 } else { 32.0 };
    let refresh = on_event.clone();
    let select = on_event.clone();

    let header = Grid::new()
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .children((
            season_picker::view(
                page.year,
                page.season,
                Callback::new(move |(year, season)| {
                    let _ = select.call(Event::Season(year, season));
                }),
            ),
            StackPanel::new()
                .grid_column(1)
                .orientation(Orientation::Horizontal)
                .spacing(8.0)
                .children((Button::new()
                    .style(ButtonStyle::Subtle)
                    .automation_name("重新整理番表")
                    .automation_id("RefreshSchedule")
                    .is_enabled(!page.loading)
                    .on_click(move || {
                        let _ = refresh.call(Event::Refresh);
                    })
                    .content(SymbolIcon::new().symbol(Symbol::Sync))
                    .tooltip("重新整理番表"),)),
        ));

    let content = if page.catalog.sections.is_empty() {
        let retry = on_event.clone();
        feedback::empty(
            if page.loading {
                "正在載入本季動畫"
            } else if page.error.is_some() {
                "暫時無法取得番表"
            } else {
                "本季尚無動畫資料"
            },
            if page.loading {
                "正在取得播出時間與動畫資訊…"
            } else {
                "稍後重新整理，看看是否有新的番表。"
            },
            page.loading,
            (!page.loading).then(|| {
                (
                    "重新整理",
                    Callback::new(move |()| {
                        let _ = retry.call(Event::Refresh);
                    }),
                )
            }),
        )
    } else {
        // One scrolling surface; the pinned Reactor API has no grouped collection layout.
        let available = (page.width - inset * 2.0 - 20.0).max(200.0);
        let columns = (available / 208.0).floor().max(1.0) as usize;
        let card_width = (available - (columns - 1) as f64 * 16.0) / columns as f64 - 26.0;
        ScrollViewer::new()
            .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
            .content(StackPanel::new().spacing(32.0).keyed_children(
                page.catalog.sections.iter().map(|section| {
                    let today = section.day == ScheduleDay::Weekday(Local::now().weekday());
                    let cards = section.animes.iter().enumerate().map(|(index, anime)| {
                        let open = on_event.clone();
                        let selected = anime.clone();
                        KeyedView::new(
                            anime.id.clone(),
                            Button::new()
                                .style(ButtonStyle::Subtle)
                                .grid_column((index % columns) as i32)
                                .grid_row((index / columns) as i32)
                                .horizontal_alignment(HorizontalAlignment::Stretch)
                                .vertical_alignment(VerticalAlignment::Top)
                                .horizontal_content_alignment(HorizontalAlignment::Stretch)
                                .automation_name(format!("開啟 {} 的詳細資料", anime.name))
                                .on_click(move || {
                                    let _ = open.call(Event::Open(Box::new(selected.clone())));
                                })
                                .content(anime_card::view(anime, card_width)),
                        )
                    });
                    KeyedView::new(
                        day_label(section.day),
                        StackPanel::new().spacing(12.0).children((
                            Grid::new()
                                .columns([GridLength::Star(1.0), GridLength::Auto])
                                .children((
                                    StackPanel::new()
                                        .orientation(Orientation::Horizontal)
                                        .spacing(8.0)
                                        .children((
                                            if today {
                                                text("今天")
                                                    .foreground(ThemeBrush::AccentText)
                                                    .into()
                                            } else {
                                                View::empty()
                                            },
                                            heading(day_label(section.day), 20.0),
                                        )),
                                    text(format!("{} 部", section.animes.len())).grid_column(1),
                                )),
                            Grid::new()
                                .columns(vec![GridLength::Star(1.0); columns])
                                .rows(vec![
                                    GridLength::Auto;
                                    section.animes.len().div_ceil(columns)
                                ])
                                .column_spacing(16.0)
                                .row_spacing(16.0)
                                .keyed_children(cards),
                        )),
                    )
                }),
            ))
    };

    Grid::new()
        .margin(Thickness::new(inset, 24.0, inset, 16.0))
        .row_spacing(20.0)
        .rows([GridLength::Auto, GridLength::Auto, GridLength::Star(1.0)])
        .children((
            header,
            StackPanel::new().grid_row(1).children((
                feedback::error(page.error),
                if page.loading && !page.catalog.sections.is_empty() {
                    text("更新中…").into()
                } else {
                    View::empty()
                },
            )),
            Grid::new().grid_row(2).children((content,)),
        ))
}
