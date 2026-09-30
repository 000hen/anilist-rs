use windows_reactor::*;

pub fn visuals(width: f64, height: f64) -> WindowVisuals {
    WindowVisuals::new()
        .backdrop(WindowBackdrop::Acrylic)
        .client_size(width, height)
        .constraints(WindowConstraints {
            min_width: Some(480.0),
            min_height: Some(560.0),
            ..Default::default()
        })
}

pub fn frame(title: &str, subtitle: &str, content: View) -> View {
    frame_with_title_bar(
        TitleBar::new()
            .title(title)
            .subtitle(subtitle)
            .preferred_height(WindowTitleBarHeight::Standard)
            .into(),
        content,
    )
}

pub fn frame_with_title_bar(title_bar: View, content: View) -> View {
    Grid::new()
        .rows([GridLength::Auto, GridLength::Star(1.0)])
        .children((
            title_bar,
            Border::new()
                .grid_row(1)
                .margin(Thickness::new(12.0, 0.0, 12.0, 12.0))
                .corner_radius(8.0)
                .border_thickness(1.0)
                .border_brush(ThemeBrush::CardStroke)
                .background(ThemeBrush::CardBackground)
                .content(content),
        ))
}
