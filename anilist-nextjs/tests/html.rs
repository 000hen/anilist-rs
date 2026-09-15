use anilist_nextjs::NextJsData;
use serde_json::json;

#[test]
fn handles_attribute_case_quotes_entities_and_raw_script_text() {
    let html =
        r#"<SCRIPT data-note='a > b' ID=__NEXT_DAT&#65;__>{"title":"a &amp; b < c"}</SCRIPT>"#;
    assert_eq!(
        NextJsData::parse(html)
            .unwrap()
            .deserialize::<String>("/title")
            .unwrap(),
        "a &amp; b < c"
    );
}

#[test]
fn ignores_script_lookalikes_in_comments_attributes_and_raw_text() {
    let html = r#"
        <!-- <script id="__NEXT_DATA__">bad</script> -->
        <div data-example='<script id="__NEXT_DATA__">bad</script>'></div>
        <textarea><script id="__NEXT_DATA__">bad</script></textarea>
        <style><script id="__NEXT_DATA__">bad</script></style>
        <script src="external.js">self.__next_f.push([1,"bad"])</script>
        <script id="__NEXT_DATA__">{"ok":true}</script>
    "#;
    assert_eq!(
        NextJsData::parse(html).unwrap().next_data,
        Some(json!({"ok":true}))
    );
}

#[test]
fn preserves_script_data_state_and_does_not_decode_entities() {
    let wire = "1:{\"text\":\"<!-- <script>ignored </script> --> &amp;\"}\n";
    let html = format!("<script>self.__next_f.push({})</script>", json!([1, wire]));
    assert_eq!(
        NextJsData::parse(&html)
            .unwrap()
            .deserialize::<String>("/text")
            .unwrap(),
        "<!-- <script>ignored </script> --> &amp;"
    );
}

#[test]
fn parses_a_final_script_without_an_end_tag() {
    let html = r#"<script id="__NEXT_DATA__">{"ok":true}"#;
    assert_eq!(
        NextJsData::parse(html).unwrap().next_data,
        Some(json!({"ok":true}))
    );
}
