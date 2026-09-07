use std::collections::HashMap;

use iced::{
    Element, Length, Task,
    widget::{Sensor, center, container, image, text},
};
use reqwest::Client;

pub enum ImageStatus {
    Pending,
    Loaded(image::Handle),
    Failed,
}

#[derive(Debug, Clone)]
pub enum ImagerMessage {
    Load(String),
    Loaded {
        url: String,
        result: Result<image::Handle, ()>,
    },
}

pub struct Imager {
    client: Client,
    images: HashMap<String, ImageStatus>,
}

impl Imager {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            images: HashMap::new(),
        }
    }

    pub fn view<'a>(&'a self, url: &'a str) -> Element<'a, ImagerMessage> {
        match self.images.get(url) {
            Some(ImageStatus::Loaded(handle)) => image(handle.clone())
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),

            Some(ImageStatus::Pending) => Self::placeholder(),
            Some(ImageStatus::Failed) => center(container(text("?")))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),

            None => Sensor::new(Self::placeholder())
                .key_ref(url)
                .anticipate(200.0)
                .on_show(move |_| ImagerMessage::Load(url.to_owned()))
                .into(),
        }
    }

    pub fn update(&mut self, message: ImagerMessage) -> Task<ImagerMessage> {
        match message {
            ImagerMessage::Load(url) => {
                if self.images.contains_key(&url) {
                    return Task::none();
                }

                self.images.insert(url.clone(), ImageStatus::Pending);

                let client = self.client.clone();
                let request_url = url.clone();

                Task::perform(
                    async move {
                        let response = client
                            .get(request_url)
                            .send()
                            .await
                            .map_err(|_| ())?
                            .error_for_status()
                            .map_err(|_| ())?;

                        let bytes = response.bytes().await.map_err(|_| ())?;

                        Ok(image::Handle::from_bytes(bytes))
                    },
                    move |result| ImagerMessage::Loaded { url, result },
                )
            }

            ImagerMessage::Loaded { url, result } => {
                let status = match result {
                    Ok(handle) => ImageStatus::Loaded(handle),
                    Err(()) => ImageStatus::Failed,
                };

                self.images.insert(url, status);

                Task::none()
            }
        }
    }

    fn placeholder<'a>() -> Element<'a, ImagerMessage> {
        container(text(""))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
