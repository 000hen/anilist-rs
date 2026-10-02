use crate::{
    catalog::Catalog,
    pages::{detail, schedule, search},
};
use anilist_core::{
    anime::{Anime, AnimeStreaming},
    season::AnimeSeason,
};
use chrono::Weekday;
use windows_reactor::{
    Callback, EncodedImage, TitleBar, View,
    test::{Command, PropertyId, PropertyValue, Pump, RecordingRuntime},
};

#[test]
fn title_bar_icon_reconciles_replacement_and_removal() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(crate::windows::shell::title_bar("Anilist").into())
        .unwrap();
    let node = pump
        .runtime()
        .commands()
        .iter()
        .flatten()
        .find_map(|command| match command {
            Command::SetProperty {
                node,
                property: PropertyId::TitleBarIconSource,
                value: PropertyValue::EncodedImage(image),
            } => {
                assert!(image.as_bytes().starts_with(b"\x89PNG\r\n\x1a\n"));
                Some(*node)
            }
            _ => None,
        })
        .expect("icon must belong to the native TitleBar");

    let replacement = EncodedImage::from_static(b"replacement image bytes");
    pump.update_view(
        TitleBar::new()
            .title("Anilist")
            .icon_source_data(replacement.clone())
            .into(),
    )
    .unwrap();
    assert_eq!(
        pump.runtime()
            .node(node)
            .unwrap()
            .property(PropertyId::TitleBarIconSource),
        Some(&PropertyValue::EncodedImage(replacement)),
    );

    pump.update_view(TitleBar::new().title("Anilist").into())
        .unwrap();
    assert!(
        pump.runtime()
            .node(node)
            .unwrap()
            .property(PropertyId::TitleBarIconSource)
            .is_none()
    );
}
fn anime() -> Anime {
    Anime {
        id: "fixture".into(),
        name: "A long animation title for layout checks".into(),
        description: "A synopsis that wraps in the narrow detail layout.".into(),
        on_air_time: None,
        is_adult: true,
        image: None,
        banner: None,
        cast: vec!["Actor".into()],
        genres: vec!["Adventure".into(), "Fantasy".into(), "Drama".into()],
        streaming: vec![AnimeStreaming {
            name: "Streaming platform".into(),
            url: "https://example.com/watch".into(),
            logo: String::new(),
        }],
        site: vec![],
    }
}
fn labels(pump: &Pump<RecordingRuntime>) -> Vec<String> {
    pump.runtime()
        .commands()
        .iter()
        .flatten()
        .filter_map(|command| match command {
            Command::SetProperty {
                property: PropertyId::TextBlockText,
                value: PropertyValue::Str(text),
                ..
            } => Some(text.clone()),
            _ => None,
        })
        .collect()
}
#[test]
fn schedule_reconciles_sizes_and_loading_error_empty_states() {
    let catalog = Catalog::new(vec![anime()], Weekday::Wed);
    let empty = Catalog::new(vec![], Weekday::Wed);
    let mut pump = Pump::new(RecordingRuntime::default());
    for (index, (width, catalog, error, loading)) in [
        (1200.0, &catalog, None, false),
        (480.0, &catalog, None, false),
        (800.0, &empty, None, true),
        (800.0, &empty, Some("offline"), false),
        (800.0, &catalog, Some("offline"), false),
    ]
    .into_iter()
    .enumerate()
    {
        let view = schedule::view(
            schedule::Page {
                year: 2026,
                season: AnimeSeason::Summer,
                catalog,
                loading,
                error,
                width,
            },
            Callback::new(|_| {}),
        );
        if index == 0 {
            pump.mount_view(view).unwrap();
        } else {
            pump.update_view(view).unwrap();
        }
    }
    let labels = labels(&pump);
    assert!(labels.iter().any(|label| label == "時間未定"));
    assert!(
        !labels
            .iter()
            .any(|label| label.contains("篩選") || label.contains("部可觀看"))
    );
}
#[test]
fn search_reconciles_prompt_results_loading_empty_and_error() {
    let results = vec![anime()];
    let mut pump = Pump::new(RecordingRuntime::default());
    for (index, (term, results, loading, error)) in [
        (None, &[][..], false, None),
        (Some("show"), &[][..], true, None),
        (Some("show"), results.as_slice(), false, None),
        (Some("missing"), &[][..], false, None),
        (Some("offline"), &[][..], false, Some("offline")),
    ]
    .into_iter()
    .enumerate()
    {
        let view = search::view(
            search::Page {
                term,
                results,
                loading,
                error,
                width: 480.0,
            },
            Callback::new(|()| {}),
            Callback::new(|_| {}),
        );
        if index == 0 {
            pump.mount_view(view).unwrap();
        } else {
            pump.update_view(view).unwrap();
        }
    }
    let labels = labels(&pump);
    for expected in [
        "搜尋動畫",
        "正在搜尋動畫",
        "找不到符合的動畫",
        "暫時無法取得搜尋結果",
    ] {
        assert!(
            labels.iter().any(|label| label == expected),
            "missing {expected}"
        );
    }
}
#[test]
fn card_keeps_metadata_minimal_and_omits_missing_platforms() {
    let mut anime = anime();
    anime.streaming.clear();
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(crate::components::anime_card::view(&anime, 220.0))
        .unwrap();
    let labels = labels(&pump);
    assert!(labels.iter().any(|label| label == "18+"));
    assert!(
        labels
            .iter()
            .any(|label| label.contains("Adventure") && label.contains("Fantasy"))
    );
    assert!(
        !labels
            .iter()
            .any(|label| label.contains("Drama") || label.contains("尚無串流"))
    );
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
fn season_picker_browses_without_loading_and_commits_on_selection() {
    use std::{cell::RefCell, rc::Rc};
    use windows_reactor::test::{EventId, EventPayload, QueuedEvent};
    let chosen = Rc::new(RefCell::new(Vec::new()));
    let received = chosen.clone();
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(crate::components::season_picker::view(
        2026,
        AnimeSeason::Summer,
        Callback::new(move |selection| received.borrow_mut().push(selection)),
    ))
    .unwrap();
    let click = |pump: &mut Pump<RecordingRuntime>, name: &str| {
        let node = pump
            .runtime()
            .commands()
            .iter()
            .flatten()
            .rev()
            .find_map(|command| match command {
                Command::SetProperty {
                    node,
                    property: PropertyId::AutomationName,
                    value: PropertyValue::Str(value),
                } if value == name
                    && pump.event_revision(*node, EventId::ButtonClick).is_some() =>
                {
                    Some(*node)
                }
                _ => None,
            })
            .expect("button must be active");
        let revision = pump.event_revision(node, EventId::ButtonClick).unwrap();
        pump.queue_event(QueuedEvent::new(
            node,
            EventId::ButtonClick,
            revision,
            EventPayload::Unit,
        ));
        pump.dispatch_events().unwrap();
        pump.dispatch_components(32).unwrap();
    };
    click(&mut pump, "前一年");
    assert!(
        chosen.borrow().is_empty(),
        "browsing a year must not load a season"
    );
    click(&mut pump, "選擇 2025 冬季");
    assert_eq!(*chosen.borrow(), vec![(2025, AnimeSeason::Winter)]);
}

#[test]
fn wide_detail_content_has_a_maximum_width() {
    let mut pump = Pump::new(RecordingRuntime::default());
    pump.mount_view(detail::view(&anime(), 2400.0, None, Callback::new(|_| {})))
        .unwrap();
    assert!(pump.runtime().commands().iter().flatten().any(|command| matches!(command,
        Command::SetProperty { property: PropertyId::MaxWidth, value: PropertyValue::F64(width), .. } if *width == 1040.0)));
}
