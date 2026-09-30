use windows_reactor::*;

pub fn view(
    query: &str,
    width: f64,
    on_change: Callback<String>,
    on_submit: Callback<String>,
) -> View {
    AutoSuggestBox::new()
        .text(query)
        .placeholder_text("搜尋動畫，按 Enter 搜尋")
        .width((width - 280.0).clamp(180.0, 480.0))
        .vertical_alignment(VerticalAlignment::Center)
        .automation_name("搜尋動畫")
        .on_text_changed(on_change)
        // .on_query_submitted(on_submit)
        .into()
}
