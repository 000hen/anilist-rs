use super::{
    artwork,
    typography::{heading, text},
};
use crate::{catalog::schedule_text, components::streaming::summary};
use anilist_core::anime::Anime;
use windows_reactor::*;

pub fn view(anime: &Anime, width: f64, on_open: Callback<()>) -> View {
    Border::new()
        .width(width)
        .corner_radius(8.0)
        .border_brush(ThemeBrush::CardStroke)
        .capture_pointer_on_press(true)
        .on_pointer_released(move |_| {
            let _ = on_open.call(());
        })
        .content(
            StackPanel::new().margin(8.0).spacing(4.0).children((
                artwork::view(anime.image.as_deref(), &anime.name, width * 1.35),
                heading(&anime.name, 16.0).max_lines(2).height(44.0),
                text(schedule_text(anime))
                    .font_size(12.0)
                    .foreground(ThemeBrush::AccentText),
                text(if anime.is_adult {
                    format!("18+ · {}", anime.genres.join(" · "))
                } else {
                    anime.genres.join(" · ")
                })
                .max_lines(1)
                .font_size(12.0)
                .height(16.0),
                summary(&anime.streaming, width),
            )),
        )
        .into()
}
