use super::{
    artwork, metadata, streaming,
    typography::{heading, text},
};
use anilist_core::anime::Anime;
use windows_reactor::*;

pub fn view(anime: &Anime, width: f64) -> View {
    let genres = if anime.genres.is_empty() {
        View::empty()
    } else {
        text(
            anime
                .genres
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join(" · "),
        )
        .text_wrapping(TextWrapping::NoWrap)
        .text_trimming(TextTrimming::CharacterEllipsis)
        .max_lines(1)
        .font_size(12.0)
        .into()
    };

    StackPanel::new()
        .width(width)
        .spacing(8.0)
        .children((
            artwork::view(anime.image.as_deref(), &anime.name, width * 1.35),
            StackPanel::new()
                .margin(Thickness::new(8.0, 0.0, 8.0, 8.0))
                .spacing(4.0)
                .children((
                    StackPanel::new()
                        .orientation(Orientation::Horizontal)
                        .spacing(8.0)
                        .children((
                            text(metadata::card_time(anime))
                                .font_size(13.0)
                                .font_weight(FontWeight::SEMI_BOLD)
                                .vertical_alignment(VerticalAlignment::Center),
                            metadata::adult_badge(anime.is_adult),
                        )),
                    heading(&anime.name, 16.0).max_lines(2).height(44.0),
                    genres,
                    streaming::summary(&anime.streaming, width - 16.0),
                )),
        ))
        .into()
}
