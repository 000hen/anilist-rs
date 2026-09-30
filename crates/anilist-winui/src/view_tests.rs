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
    Callback, View,
    test::{Command, PropertyId, PropertyValue, Pump, RecordingRuntime},
};
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
                query: "show",
                term,
                results,
                loading,
                error,
                width: 480.0,
            },
            Callback::new(|_| {}),
            Callback::new(|()| {}),
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
