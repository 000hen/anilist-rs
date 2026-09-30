use crate::{components::typography::text, pages::schedule::Event};
use anilist_core::{ScheduleDay, get_current_week_order};
use windows_reactor::*;

pub fn view(
    query: &str,
    selected: Option<ScheduleDay>,
    width: f64,
    on_event: Callback<Event>,
) -> View {
    let filter = on_event.clone();
    let days = std::iter::once(None).chain(get_current_week_order().into_iter().map(Some));
    let columns = if width < 720.0 { 3 } else { 9 };
    let items = days
        .enumerate()
        .map(|(index, day)| {
            let event = on_event.clone();
            ToggleButton::new()
                .grid_column((index % columns) as i32)
                .grid_row((index / columns) as i32)
                .horizontal_alignment(HorizontalAlignment::Stretch)
                .is_checked(selected == day)
                .automation_name(day.map_or_else(|| "所有播出日".into(), day_label))
                .on_is_checked_changed(move |checked| {
                    let _ = event.call(Event::Day(if checked { day } else { None }));
                })
                .content(text(day.map_or_else(|| "全部".into(), day_label)))
        })
        .collect::<Vec<_>>();
    StackPanel::new()
        .spacing(12.0)
        .children((
            StackPanel::new().spacing(8.0).children((
                text("篩選目前顯示的動畫").font_weight(FontWeight::SEMI_BOLD),
                TextBox::new()
                    .text(query)
                    .placeholder_text("名稱、類型或配音員")
                    .automation_name("篩選目前顯示的動畫")
                    .automation_id("GalleryFilter")
                    .on_text_changed(move |value| {
                        let _ = filter.call(Event::Search(value));
                    }),
            )),
            StackPanel::new().spacing(8.0).children((
                text("播出星期").font_weight(FontWeight::SEMI_BOLD),
                Grid::new()
                    .column_spacing(8.0)
                    .row_spacing(8.0)
                    .columns(vec![GridLength::Star(1.0); columns])
                    .rows(vec![GridLength::Auto; items.len().div_ceil(columns)])
                    .keyed_children(
                        items
                            .into_iter()
                            .enumerate()
                            .map(|(i, view)| KeyedView::new(i.to_string(), view)),
                    ),
            )),
        ))
        .into()
}

fn day_label(day: ScheduleDay) -> String {
    match day {
        ScheduleDay::Unknown => "時間未定".into(),
        ScheduleDay::Weekday(day) => [
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
            "星期日",
        ][day.num_days_from_monday() as usize]
            .into(),
    }
}
