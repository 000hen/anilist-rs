use super::typography::text;
use anilist_core::anime::AnimeStreaming;
use windows_reactor::*;

/// Compact platform identities; the complete list is available in the detail window.
pub fn summary(streams: &[AnimeStreaming], width: f64) -> View {
    if streams.is_empty() {
        return View::empty();
    }
    let capacity = ((width - 24.0) / 24.0).floor().clamp(1.0, 3.0) as usize;
    let icons = streams
        .iter()
        .take(capacity)
        .enumerate()
        .map(|(index, stream)| {
            KeyedView::new(
                index,
                Border::new()
                    .width(18.0)
                    .height(18.0)
                    .content(logo(stream, 18.0))
                    .tooltip(&stream.name),
            )
        });
    StackPanel::new()
        .orientation(Orientation::Horizontal)
        .spacing(6.0)
        .height(18.0)
        .children((
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(6.0)
                .keyed_children(icons),
            if streams.len() > capacity {
                text(format!("+{}", streams.len() - capacity))
                    .font_size(12.0)
                    .vertical_alignment(VerticalAlignment::Center)
                    .into()
            } else {
                View::empty()
            },
        ))
}

pub fn button(stream: &AnimeStreaming, on_open: Callback<String>) -> View {
    let url = stream.url.clone();
    Button::new()
        .horizontal_alignment(HorizontalAlignment::Stretch)
        .horizontal_content_alignment(HorizontalAlignment::Stretch)
        .automation_name(format!("在 {} 觀看（開啟瀏覽器）", stream.name))
        .on_click(move || {
            let _ = on_open.call(url.clone());
        })
        .content(
            Grid::new()
                .columns([GridLength::Pixel(32.0), GridLength::Star(1.0)])
                .column_spacing(12.0)
                .margin(Thickness::xy(4.0, 6.0))
                .children((
                    logo(stream, 32.0),
                    text(&stream.name)
                        .grid_column(1)
                        .vertical_alignment(VerticalAlignment::Center),
                )),
        )
}

fn logo(stream: &AnimeStreaming, size: f64) -> View {
    View::component::<PlatformLogo>(LogoInput {
        url: stream.logo.clone(),
        name: stream.name.clone(),
        size,
    })
}

#[derive(Clone, PartialEq)]
struct LogoInput {
    url: String,
    name: String,
    size: f64,
}
struct PlatformLogo {
    failed: bool,
}
impl Component for PlatformLogo {
    type Input = LogoInput;
    type Message = ();
    fn create(_: &LogoInput, _: &ComponentContext<Self>) -> Self {
        Self { failed: false }
    }
    fn input_changed(&mut self, _: &LogoInput, _: &ComponentContext<Self>) {
        self.failed = false;
    }
    fn update(&mut self, _: (), _: &ComponentContext<Self>) {
        self.failed = true;
    }
    fn view(&self, input: &LogoInput, context: &mut ViewContext<Self>) -> View {
        let image = (!self.failed && !input.url.is_empty())
            .then(|| Image::new().source(&input.url).ok())
            .flatten();
        let content: View = match image {
            Some(image) => image
                .stretch(Stretch::Uniform)
                .automation_name(format!("{} 圖示", input.name))
                .on_failed(context.forward())
                .into(),
            None => SymbolIcon::new()
                .symbol(Symbol::Play)
                .automation_name(&input.name)
                .into(),
        };
        Border::new()
            .width(input.size)
            .height(input.size)
            .corner_radius(4.0)
            .content(content)
    }
}
