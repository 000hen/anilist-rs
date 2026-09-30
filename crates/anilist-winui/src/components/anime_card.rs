use super::{
    artwork,
    typography::{heading, text},
};
use crate::catalog::schedule_text;
use anilist_core::anime::Anime;
use windows_reactor::*;

pub fn view(anime: &Anime, width: f64, on_open: Callback<()>) -> View {
    let providers = anime
        .streaming
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(" · ");
    Button::new()
        .style(ButtonStyle::Subtle)
        .automation_name(format!(
            "{}，{}，開啟詳細資料",
            anime.name,
            schedule_text(anime)
        ))
        .on_click(on_open)
        .content(
            Border::new()
                .width(width)
                .corner_radius(8.0)
                .background(ThemeBrush::CardBackground)
                .border_brush(ThemeBrush::CardStroke)
                .border_thickness(1.0)
                .content(
                    StackPanel::new().children((
                        artwork::view(anime.image.as_deref(), &anime.name, width * 1.35),
                        StackPanel::new().spacing(8.0).margin(12.0).children((
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
                            text(if providers.is_empty() {
                                "尚無串流平台資訊"
                            } else {
                                &providers
                            })
                            .max_lines(1)
                            .font_size(12.0)
                            .height(16.0),
                        )),
                    )),
                ),
        )
}
