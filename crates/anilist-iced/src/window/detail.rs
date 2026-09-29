use anilist_core::anime::Anime;
use iced::{
    Element, Font, Length,
    font::Weight,
    widget::{center, column, container, grid, mouse_area, scrollable, text},
};

use crate::{component::imager::Imager, state::Message};

pub fn view<'a>(imager: &'a Imager, anime: &'a Anime) -> Element<'a, Message> {
    let image = match anime.image.as_deref() {
        Some(image) => imager.view(&image).map(Message::Image),
        None => container("").into(),
    };

    let streaming = anime.streaming.iter().map(|stream| {
        mouse_area(imager.view(&stream.logo).map(Message::Image))
            .on_press(Message::OpenWeb(stream.url.clone()))
            .into()
    });

    scrollable(center(
        column![
            image,
            container(grid(streaming).fluid(96.0).spacing(8)).center_x(Length::Fill),
            text(&anime.name)
                .center()
                .width(Length::Fill)
                .size(36)
                .wrapping(text::Wrapping::Word)
                .font(Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }),
            text(&anime.description)
        ]
        .width(Length::Fill)
        .max_width(960)
        .spacing(16)
        .padding(16),
    ))
    .width(Length::Fill)
    .into()
}
