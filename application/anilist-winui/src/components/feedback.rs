use super::typography::{heading, text};
use windows_reactor::*;

pub fn error(message: Option<&str>) -> InfoBar {
    InfoBar::new()
        .is_open(message.is_some())
        .is_closable(false)
        .severity(InfoBarSeverity::Error)
        .title("無法完成操作")
        .message(message.unwrap_or_default())
}

pub fn empty(
    title: &str,
    description: &str,
    loading: bool,
    action: Option<(&str, Callback<()>)>,
) -> View {
    let action = match action {
        Some((label, on_click)) => Button::new()
            .style(ButtonStyle::Accent)
            .horizontal_alignment(HorizontalAlignment::Center)
            .on_click(on_click)
            .content(label),
        None => View::empty(),
    };
    let status: View = if loading {
        ProgressRing::new()
            .is_active(true)
            .width(40.0)
            .height(40.0)
            .into()
    } else {
        SymbolIcon::new()
            .symbol(Symbol::Video)
            .width(40.0)
            .height(40.0)
            .into()
    };
    StackPanel::new()
        .spacing(16.0)
        .max_width(400.0)
        .margin(24.0)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .children((status, heading(title, 20.0), text(description), action))
}
