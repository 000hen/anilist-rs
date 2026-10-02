use super::typography::text;
use windows_reactor::*;

pub fn view(url: Option<&str>, name: &str, height: f64) -> View {
    View::component::<Artwork>(Input {
        url: url.map(str::to_owned),
        name: name.to_owned(),
        height,
    })
}

#[derive(Clone, PartialEq)]
struct Input {
    url: Option<String>,
    name: String,
    height: f64,
}
struct Artwork {
    failed: bool,
}

impl Component for Artwork {
    type Input = Input;
    type Message = ();
    fn create(_: &Input, _: &ComponentContext<Self>) -> Self {
        Self { failed: false }
    }
    fn input_changed(&mut self, _: &Input, _: &ComponentContext<Self>) {
        self.failed = false;
    }
    fn update(&mut self, _: (), _: &ComponentContext<Self>) {
        self.failed = true;
    }
    fn view(&self, input: &Input, context: &mut ViewContext<Self>) -> View {
        let image = input
            .url
            .as_deref()
            .filter(|_| !self.failed)
            .and_then(|url| Image::new().source(url).ok());
        let content = match image {
            Some(image) => image
                .stretch(Stretch::Uniform)
                .automation_name(format!("{} 海報", input.name))
                .on_failed(context.forward())
                .into(),
            None => StackPanel::new()
                .spacing(12.0)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center)
                .children((SymbolIcon::new().symbol(Symbol::Pictures), text("尚無圖片"))),
        };
        Border::new()
            .height(input.height)
            .background(ThemeBrush::CardBackground)
            .corner_radius(8.0)
            .content(content)
    }
}
