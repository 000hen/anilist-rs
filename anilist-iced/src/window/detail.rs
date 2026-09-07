use anilist_core::anime::Anime;
use iced::{Element, widget::text};

use crate::state::Message;

pub fn view(anime: &Anime) -> Element<'_, Message> {
    text(format!("I GOT ANIME: {}", anime.name)).into()
}
