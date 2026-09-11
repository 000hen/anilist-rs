use anilist_core::{ScheduleDay, get_current_week_order, season::AnimeSeason};
use chrono::{Datelike, Local};
use iced::{
    Element, Length,
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
        container(center(text(week.to_string()).size(24)))
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
    let season = AnimeSeason::try_from(now.month() as u8).expect("month should map to a season");

    let week = get_current_week_order();
    let items = week.iter().filter_map(|&week| {
        let animes = app.state.animes_week.get(&week)?;
        Some(week_section(app, week, animes))
    });

    scrollable(column![text(format!("季節: {}", season)), column(items)].spacing(10))
        .width(Length::Fill)
        .into()
}
