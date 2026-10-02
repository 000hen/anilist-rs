use super::typography::text;
use anilist_core::anime::Anime;
use windows_reactor::*;

pub fn card_time(anime: &Anime) -> String {
    anime
        .on_air_time
        .as_ref()
        .and_then(|time| time.minute)
        .map(|minute| format!("@{minute}"))
        .unwrap_or_else(|| "時間未定".to_owned())
}

pub fn adult_badge(is_adult: bool) -> View {
    if !is_adult {
        return View::empty();
    }
    Border::new()
        .corner_radius(4.0)
        .border_thickness(1.0)
        .border_brush(ThemeBrush::CardStroke)
        .padding(Thickness::xy(6.0, 2.0))
        .vertical_alignment(VerticalAlignment::Center)
        .content(text("18+").font_size(12.0))
        .into()
}
