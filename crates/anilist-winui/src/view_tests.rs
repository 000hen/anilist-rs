use crate::{
    catalog::Catalog,
    pages::{detail, schedule},
};
use anilist_core::{
    anime::{Anime, AnimeStreaming},
    season::AnimeSeason,
};
use chrono::Weekday;
use windows_reactor::{
    Callback, View,
    test::{Pump, RecordingRuntime},
};

fn anime() -> Anime {
    Anime {
        id: "fixture".into(),
        name: "A long animation title for layout checks".into(),
        description: "A synopsis that wraps in the narrow detail layout.".into(),
        on_air_time: None,
        is_adult: false,
        image: None,
        banner: None,
        cast: vec!["Actor".into()],
        genres: vec!["Adventure".into()],
        streaming: vec![AnimeStreaming {
            name: "Streaming platform".into(),
            url: "https://example.com/watch".into(),
            logo: "https://example.com/platform.webp".into(),
        }],
        site: vec![],
    }
}

#[test]
fn gallery_reconciles_across_window_sizes_and_empty_error_states() {
    let catalog = Catalog::new(vec![anime()], Weekday::Wed);
    let mut pump = Pump::new(RecordingRuntime::default());
    let render = |width, query, error, loading| {
        schedule::view(
            schedule::Page {
                year: 2026,
                season: AnimeSeason::Summer,
                catalog: &catalog,
                query,
                search_term: None,
                day: None,
                loading,
                error,
                width,
            },
            Callback::new(|_| {}),
        )
    };
    pump.mount_view(render(1200.0, "", None, false)).unwrap();
    pump.update_view(render(480.0, "", None, false)).unwrap();
    pump.update_view(render(800.0, "no results", None, false))
        .unwrap();
    pump.update_view(render(800.0, "", Some("offline"), false))
        .unwrap();
    pump.update_view(render(800.0, "", None, true)).unwrap();
}

#[test]
fn detail_reconciles_between_columns_and_stacked_layout() {
    let anime = anime();
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(detail::view(&anime, 980.0, None, Callback::new(|_| {})))
        .unwrap();
    pump.update_view(detail::view(
        &anime,
        480.0,
        Some("browser unavailable"),
        Callback::new(|_| {}),
    ))
    .unwrap();
    pump.update_view(View::empty()).unwrap();
}

#[test]
fn filters_expose_all_weekdays_without_a_dropdown() {
    use windows_reactor::test::{Command, PropertyId, PropertyValue};
    for width in [480.0, 1200.0] {
        let mut pump = Pump::new(RecordingRuntime::default());
        pump.mount_view(crate::components::gallery_filters::view(
            "",
            None,
            width,
            Callback::new(|_| {}),
        ))
        .unwrap();
        let labels: Vec<_> = pump
            .runtime()
            .commands()
            .iter()
            .flatten()
            .filter_map(|command| {
                if let Command::SetProperty {
                    property: PropertyId::TextBlockText,
                    value: PropertyValue::String(text),
                    ..
                } = command
                {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect();
        for label in [
            "播出星期",
            "全部",
            "星期一",
            "星期二",
            "星期三",
            "星期四",
            "星期五",
            "星期六",
            "星期日",
            "時間未定",
        ] {
            assert!(labels.contains(&label), "missing visible filter: {label}");
        }
    }
}

#[test]
fn source_search_and_local_filter_keep_independent_text() {
    use windows_reactor::test::{Command, PropertyId, PropertyValue};
    use windows_reactor::{ChildrenControl, StackPanel};
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(StackPanel::new().children((
        crate::components::title_search::view(
            "remote query",
            1200.0,
            Callback::new(|_| {}),
            Callback::new(|()| {}),
        ),
        crate::components::gallery_filters::view(
            "local filter",
            None,
            1200.0,
            Callback::new(|_| {}),
        ),
    )))
    .unwrap();
    for (property, expected) in [
        (PropertyId::AutoSuggestBoxText, "remote query"),
        (PropertyId::TextBoxText, "local filter"),
    ] {
        assert!(pump.runtime().commands().iter().flatten().any(|command| matches!(command,
            Command::SetProperty { property: actual, value: PropertyValue::String(value), .. } if *actual == property && value == expected)));
    }
}
