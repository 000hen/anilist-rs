use anilist_nextjs::deserialize_nextjs;
use serde_json::{Value, json};

#[test]
fn returns_next_data_as_json_for_other_languages() {
    let html = r#"<script id="__NEXT_DATA__">{"props":{"title":"中文"},"buildId":"test"}</script>"#;
    let result = deserialize_nextjs(html.to_owned()).unwrap();
    let result: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(
        result,
        json!({
            "nextData": {"props": {"title": "中文"}, "buildId": "test"},
            "flight": []
        })
    );
}

#[test]
fn returns_all_flight_records_without_requiring_rust_types() {
    let html = format!(
        "<script>self.__next_f.push({})</script>",
        json!([1, "1:I[42,[],\"default\"]\na:{\"title\":\"中文\"}\nb:T3,文"])
    );
    let result: Value = serde_json::from_str(&deserialize_nextjs(html).unwrap()).unwrap();
    assert_eq!(
        result,
        json!({
            "nextData": null,
            "flight": [
                {"id": 1, "tag": "I", "value": [42, [], "default"]},
                {"id": 10, "tag": null, "value": {"title": "中文"}},
                {"id": 11, "tag": "T", "value": "文"}
            ]
        })
    );
}

#[test]
fn malformed_input_returns_a_descriptive_error() {
    let error = deserialize_nextjs("<html></html>".to_owned()).unwrap_err();
    assert!(error.to_string().contains("hydration data was not found"));
    let error = deserialize_nextjs(r#"<script>self.__next_f.push([1,42])</script>"#.to_owned())
        .unwrap_err();
    assert!(error.to_string().contains("payload must be a string"));
}
