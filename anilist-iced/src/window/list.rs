use anilist_core::{ScheduleDay, get_current_week_order, season::AnimeSeason};
use chrono::{Datelike, Local};
use iced::{
    Alignment, Element, Length,
    widget::{center, column, container, grid, scrollable, stack, text},
};

use crate::{
    component::anime,
    state::{App, Message},
};

pub fn view(app: &App) -> Element<'_, Message> {
    stack![display_items(app)].into()
}

fn week_section<'a>(
    app: &'a App,
    week: ScheduleDay,
    anime_ids: &'a [String],
) -> Element<'a, Message> {
    let anime_grid = grid(
        anime_ids
            .iter()
            .filter_map(|id| app.state.animes.get(id))
            .map(|anime| anime::view(&app.image, anime, Message::Image, Message::Anime)),
    )
    .fluid(420.0)
    .spacing(10);

    column![
        container(center(
            column![
                text(week.to_string()).size(32),
                text(format!("共 {} 部", anime_ids.len())).size(16)
            ]
            .align_x(Alignment::Center)
            .spacing(4)
        ))
        .width(Length::Fill)
        .padding(4)
        .style(container::primary),
        anime_grid,
    ]
    .spacing(10)
    .into()
}

fn display_items(app: &App) -> Element<'_, Message> {
    let now = Local::now();
    let year = now.year();
    let season = AnimeSeason::try_from(now.month() as u8).expect("month should map to a season");

    let today = ScheduleDay::Weekday(now.weekday());
    let total = app.state.animes.len();
    let today_count = app.state.animes_week.get(&today).map_or(0, Vec::len);

    let streamable_count = app
        .state
        .animes
        .values()
        .filter(|anime| !anime.streaming.is_empty())
        .count();

    let header = column![
        text(format!("{year} {season}")).size(32),
        text(format!(
            "{total} 部動畫 · 今天 {today_count} 部 · {streamable_count} 部可觀看"
        ))
        .size(16),
    ]
    .width(Length::Fill)
    .spacing(4)
    .align_x(Alignment::Center)
    .padding(24);

    let sections = get_current_week_order().into_iter().filter_map(|week| {
        let animes = app.state.animes_week.get(&week)?;
        (!animes.is_empty()).then(|| week_section(app, week, animes))
    });

    scrollable(column![header, column(sections).spacing(20),])
        .width(Length::Fill)
        .into()
}
