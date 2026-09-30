use windows_reactor::*;

pub fn view(query: &str, width: f64, on_change: Callback<String>, on_submit: Callback<()>) -> View {
    Grid::new()
        .width((width - 360.0).clamp(120.0, 560.0))
        .column_spacing(4.0)
        .columns([GridLength::Star(1.0), GridLength::Auto])
        .vertical_alignment(VerticalAlignment::Center)
        .children((
            AutoSuggestBox::new()
                .text(query)
                .placeholder_text("搜尋所有動畫")
                .automation_name("搜尋所有動畫")
                .automation_id("SourceSearch")
                .on_text_changed(on_change),
            Button::new()
                .grid_column(1)
                .automation_name("搜尋")
                .automation_id("SubmitSourceSearch")
                .on_click(on_submit)
                .content(SymbolIcon::new().symbol(Symbol::Find)),
        ))
        .into()
}
