use crate::components::{
    artwork, feedback,
    typography::{heading, text},
};
use crate::pages::schedule::day_label;
use anilist_core::{ScheduleDay, anime::Anime};
use windows_reactor::*;

pub struct Page<'a> {
    pub query: &'a str,
    pub term: Option<&'a str>,
    pub results: &'a [Anime],
    pub loading: bool,
    pub error: Option<&'a str>,
    pub width: f64,
}
pub fn view(
    page: Page<'_>,
    on_change: Callback<String>,
    on_submit: Callback<()>,
    on_back: Callback<()>,
    on_open: Callback<Box<Anime>>,
) -> View {
    let retry = on_submit.clone();
    let content = if page.results.is_empty() {
        let (title, description) = if page.loading {
            ("正在搜尋動畫", "正在取得搜尋結果…")
        } else if page.error.is_some() {
            ("暫時無法取得搜尋結果", "請檢查網路連線，然後再試一次。")
        } else if page.term.is_some() {
            ("找不到符合的動畫", "試試其他動畫名稱。")
        } else {
            ("搜尋動畫", "輸入動畫名稱，尋找想看的作品。")
        };
        feedback::empty(
            title,
            description,
            page.loading,
            (page.error.is_some() && !page.loading).then_some(("重試", retry)),
        )
    } else {
        ListView::new().selection_mode(ListViewSelectionMode::None).automation_name("動畫搜尋結果")
            .slots([SlotView::collection(ListViewSlot::Items, page.results.iter().map(|anime| {
                let selected = anime.clone(); let open = on_open.clone();
                KeyedView::new(anime.id.clone(), ListViewItem::new().content(
                    Button::new().style(ButtonStyle::Subtle).horizontal_alignment(HorizontalAlignment::Stretch)
                        .horizontal_content_alignment(HorizontalAlignment::Stretch)
                        .automation_name(format!("開啟 {} 的詳細資料", anime.name))
                        .on_click(move || { let _ = open.call(Box::new(selected.clone())); })
                        .content(Grid::new().columns([GridLength::Pixel(80.0), GridLength::Star(1.0)])
                            .column_spacing(16.0).margin(Thickness::xy(0.0, 8.0)).children((
                                artwork::view(anime.image.as_deref(), &anime.name, 108.0),
                                StackPanel::new().grid_column(1).spacing(8.0).vertical_alignment(VerticalAlignment::Center).children((
                                    heading(&anime.name, 16.0).max_lines(2),
                                    StackPanel::new().orientation(Orientation::Horizontal).spacing(8.0).children((
                                        if let Some(time) = &anime.on_air_time { text(day_label(ScheduleDay::Weekday(time.week))).into() } else { View::empty() },
                                        text(crate::components::metadata::card_time(anime)),
                                        crate::components::metadata::adult_badge(anime.is_adult),
                                    )),
                                )),
                            )))))
            }))])
    };
    Grid::new()
        .margin(if page.width < 640.0 { 16.0 } else { 32.0 })
        .row_spacing(20.0)
        .rows([
            GridLength::Auto,
            GridLength::Auto,
            GridLength::Auto,
            GridLength::Star(1.0),
        ])
        .children((
            StackPanel::new()
                .orientation(Orientation::Horizontal)
                .spacing(12.0)
                .children((
                    Button::new()
                        .style(ButtonStyle::Subtle)
                        .automation_name("返回番表")
                        .on_click(on_back)
                        .content(SymbolIcon::new().symbol(Symbol::Back))
                        .tooltip("返回番表"),
                    heading("搜尋動畫", 28.0),
                )),
            Grid::new()
                .grid_row(1)
                .columns([GridLength::Star(1.0), GridLength::Auto])
                .column_spacing(8.0)
                .children((
                    AutoSuggestBox::new()
                        .text(page.query)
                        .placeholder_text("輸入動畫名稱")
                        .automation_name("動畫名稱")
                        .automation_id("SourceSearch")
                        .on_text_changed(on_change),
                    Button::new()
                        .grid_column(1)
                        .automation_name("搜尋")
                        .automation_id("SubmitSourceSearch")
                        .on_click(on_submit)
                        .content(SymbolIcon::new().symbol(Symbol::Find)),
                )),
            StackPanel::new().grid_row(2).spacing(8.0).children((
                feedback::error(page.error),
                if let Some(term) = page.term {
                    text(format!("「{term}」的搜尋結果")).into()
                } else {
                    View::empty()
                },
            )),
            Grid::new().grid_row(3).children((content,)),
        ))
}
