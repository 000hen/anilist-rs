use anilist_core::anime::Anime;
use iced::{
    Element, Font, Length, Padding,
    font::Weight,
    mouse::Interaction,
    widget::{center_x, column, container, mouse_area, text},
};

use crate::component::imager::{Imager, ImagerMessage};

#[derive(Debug, Clone)]
pub enum Message {
    Clicked(String),
}

pub fn view<'a, AppMessage>(
    imager: &'a Imager,
    anime: &'a Anime,
    on_image: impl Fn(ImagerMessage) -> AppMessage + Clone + 'a,
    on_press: impl Fn(Message) -> AppMessage + Clone + 'a,
) -> Element<'a, AppMessage>
where
    AppMessage: Clone + 'a,
{
    let image: Element<'a, AppMessage> = match anime.image.as_deref() {
        Some(url) => imager.view(url).map(on_image),
        None => container("").into(),
    };

    let minute = match anime.on_air_time.as_ref().and_then(|time| time.minute) {
        Some(time) => format!("@{}", time),
        None => "時間未定".to_owned(),
    };

    let content = column![
        image,
        center_x(
            container(text(minute))
                .style(container::rounded_box)
                .padding(Padding::from([2, 4]))
        )
        .width(Length::Fill),
        text(&anime.name)
            .center()
            .width(Length::Fill)
            .size(24)
            .wrapping(text::Wrapping::Word)
            .font(Font {
                weight: Weight::Bold,
                ..Font::DEFAULT
            }),
    ]
    .spacing(4);

    mouse_area(container(content).padding(8))
        .on_press(on_press(Message::Clicked(anime.id.clone())))
        .interaction(Interaction::Pointer)
        .into()
}
