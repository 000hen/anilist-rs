use crate::{
    catalog::schedule_text,
    components::{
        artwork, feedback, streaming,
        typography::{heading, text},
    },
};
use anilist_core::anime::Anime;
use windows_reactor::*;

pub fn view(anime: &Anime, width: f64, error: Option<&str>, on_open: Callback<String>) -> View {
    let compact = width < 760.0;
    let poster_width = if compact { 200.0 } else { 248.0 };
    let platforms = if anime.streaming.is_empty() {
        text("尚無串流平台資訊").into()
    } else {
        let columns = if width >= 900.0 { 2 } else { 1 };
        Grid::new()
            .columns(vec![GridLength::Star(1.0); columns])
            .rows(vec![
                GridLength::Auto;
                anime.streaming.len().div_ceil(columns)
            ])
            .column_spacing(8.0)
            .row_spacing(8.0)
            .keyed_children(anime.streaming.iter().enumerate().map(|(index, stream)| {
                KeyedView::new(
                    index,
                    Grid::new()
                        .grid_column((index % columns) as i32)
                        .grid_row((index / columns) as i32)
                        .children((streaming::button(stream, on_open.clone()),)),
                )
            }))
    };

    let sites = anime.site.iter().enumerate().map(|(index, site)| {
        let url = site.url.clone();
        let open = on_open.clone();
        KeyedView::new(
            index,
            Button::new()
                .style(ButtonStyle::Default)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .horizontal_content_alignment(HorizontalAlignment::Stretch)
                .automation_name(format!("在瀏覽器開啟 {}", site.title))
                .on_click(move || {
                    let _ = open.call(url.clone());
                })
                .content(
                    Grid::new()
                        .columns([GridLength::Pixel(20.0), GridLength::Star(1.0)])
                        .column_spacing(12.0)
                        .children((
                            SymbolIcon::new().symbol(Symbol::World),
                            text(&site.title).grid_column(1),
                        )),
                ),
        )
    });

    let poster = Border::new()
        .width(poster_width)
        .horizontal_alignment(if compact {
            HorizontalAlignment::Center
        } else {
            HorizontalAlignment::Left
        })
        .vertical_alignment(VerticalAlignment::Top)
        .content(artwork::view(
            anime.image.as_deref(),
            &anime.name,
            poster_width * 1.5,
        ));

    let description = StackPanel::new()
        .spacing(24.0)
        .grid_column(if compact { 0 } else { 1 })
        .grid_row(if compact { 1 } else { 0 })
        .children((
            StackPanel::new().spacing(12.0).children((
                heading(&anime.name, 28.0),
                text(format!(
                    "{}{}",
                    if anime.is_adult { "18+ · " } else { "" },
                    anime.genres.join(" · ")
                )),
                text(schedule_text(anime)).foreground(ThemeBrush::AccentText),
            )),
            section("線上觀看", platforms),
            feedback::error(error),
            section(
                "劇情簡介",
                text(if anime.description.is_empty() {
                    "尚無簡介"
                } else {
                    &anime.description
                })
                .into(),
            ),
            section(
                "配音員",
                text(if anime.cast.is_empty() {
                    "尚無配音資訊".into()
                } else {
                    anime.cast.join("、")
                })
                .into(),
            ),
            section(
                "相關網站",
                if anime.site.is_empty() {
                    text("尚無相關網站").into()
                } else {
                    StackPanel::new().spacing(8.0).keyed_children(sites)
                },
            ),
        ));

    ScrollViewer::new()
        .horizontal_scroll_bar_visibility(ScrollBarVisibility::Disabled)
        .content(
            Grid::new()
                .margin(if compact { 20.0 } else { 32.0 })
                .column_spacing(32.0)
                .row_spacing(24.0)
                .columns(if compact {
                    vec![GridLength::Star(1.0)]
                } else {
                    vec![GridLength::Pixel(poster_width), GridLength::Star(1.0)]
                })
                .rows(if compact {
                    vec![GridLength::Auto, GridLength::Auto]
                } else {
                    vec![GridLength::Auto]
                })
                .vertical_alignment(VerticalAlignment::Top)
                .children((poster, description)),
        )
}

fn section(title: &str, content: View) -> View {
    StackPanel::new()
        .spacing(12.0)
        .children((heading(title, 20.0), content))
}
