use anilist_nextjs::{NextJsData, ParseError};
use serde_json::{Value, json};

fn script(chunk: &str) -> String {
    format!(
        "<script>self.__next_f.push({});</script>",
        json!([1, chunk])
    )
}

#[test]
fn reassembles_chunks_and_finds_model_data_after_other_records() {
    let html = format!(
        "<script>(self.__next_f = self.__next_f || []).push([0]);</script>{}{}",
        script("1:I[42,[],\"default\"]\n2:[\"$\",\"$L1\",null,{\"ani"),
        script("mes\":[{\"id\":\"123\"}]}]\n")
    );
    let data = NextJsData::parse(&html).unwrap();
    let animes: Vec<Value> = data.deserialize("/3/animes").unwrap();
    assert_eq!(animes, vec![json!({"id": "123"})]);
    assert_eq!(data.flight[0].tag.as_deref(), Some("I"));
    assert_eq!(data.flight[1].id, Some(2));
}

#[test]
fn reads_detail_shape_and_preserves_escaped_content() {
    let row = json!([null, ["$", "$L1", null, {"anime": {"title": "中文 \"quote\" \\ slash\nself.__next_f.push("}}]]);
    let data = NextJsData::parse(&script(&format!("a:{row}\n"))).unwrap();
    let anime: Value = data.deserialize("/1/3/anime").unwrap();
    assert_eq!(
        anime["title"],
        "中文 \"quote\" \\ slash\nself.__next_f.push("
    );
}

#[test]
fn parses_next_data_and_json_pointer_escaping() {
    let data = NextJsData::parse(r#"<script id="__NEXT_DATA__" type="application/json">{"props":{"a/b":{"~key":7}},"buildId":"test"}</script>"#).unwrap();
    assert_eq!(data.deserialize::<u32>("/props/a~1b/~0key").unwrap(), 7);
    assert_eq!(data.next_data.unwrap()["buildId"], "test");
}

#[test]
fn reads_multiple_pushes_and_bootstrap_payload_in_one_script() {
    let data = NextJsData::parse(
        r#"<script>
        (self.__next_f=self.__next_f||[]).push([0]);
        self.__next_f.push([2,{"form":true}]);
        self.__next_f.push([1,"1:{\"first\":1}\n"]); self.__next_f.push([1,"2:{\"second\":2}"]);
    </script>"#,
    )
    .unwrap();
    assert_eq!(data.flight.len(), 2);
    assert_eq!(data.deserialize::<u32>("/second").unwrap(), 2);
}

#[test]
fn text_records_use_utf8_byte_lengths_and_can_contain_newlines() {
    let data = NextJsData::parse(&script("a:T7,中文\nb:{\"ok\":true}\n")).unwrap();
    assert_eq!(data.flight[0].value, "中文\n");
    assert!(data.deserialize::<bool>("/ok").unwrap());
}

#[test]
fn decodes_base64_transport_across_utf8_boundaries() {
    use base64::{Engine, engine::general_purpose::STANDARD};
    let bytes = "1:{\"title\":\"中文\"}\n".as_bytes();
    let html = [&bytes[..14], &bytes[14..]]
        .into_iter()
        .map(|part| {
            format!(
                "<script>self.__next_f.push({})</script>",
                json!([3, STANDARD.encode(part)])
            )
        })
        .collect::<String>();
    assert_eq!(
        NextJsData::parse(&html)
            .unwrap()
            .deserialize::<String>("/title")
            .unwrap(),
        "中文"
    );
}

#[test]
fn retains_hint_tags_and_unindexed_records() {
    let data = NextJsData::parse(&script(
        ":HL[\"/style.css\",\"style\"]\n1:D{\"name\":\"Page\"}\n2:null\n",
    ))
    .unwrap();
    assert_eq!(data.flight[0].id, None);
    assert_eq!(data.flight[0].tag.as_deref(), Some("HL"));
    assert_eq!(data.flight[1].tag.as_deref(), Some("D"));
    assert_eq!(data.flight[2].value, Value::Null);
}

#[test]
fn distinguishes_missing_data_from_bad_target_type() {
    assert!(matches!(
        NextJsData::parse("<html></html>"),
        Err(ParseError::MissingData)
    ));
    let data = NextJsData::parse(&script("1:{\"count\":\"many\"}\n")).unwrap();
    assert!(matches!(
        data.deserialize::<u32>("/missing"),
        Err(ParseError::MissingPath { .. })
    ));
    assert!(matches!(
        data.deserialize::<u32>("/count"),
        Err(ParseError::Json { .. })
    ));
}

#[test]
fn rejects_malformed_records_without_panicking() {
    for row in [
        "broken",
        "xyz:{}\n",
        "1:{bad}\n",
        "1:T8,short",
        "1:T2,中",
        "1:Tffffffffffffffffffffffff,x",
        "1:",
    ] {
        assert!(NextJsData::parse(&script(row)).is_err(), "accepted {row:?}");
    }
}

#[test]
fn rejects_invalid_transport_and_next_data() {
    for html in [
        r#"<script>self.__next_f.push([1,42])</script>"#,
        r#"<script>self.__next_f.push([3,"!"])</script>"#,
        r#"<script>self.__next_f.push([9,"x"])</script>"#,
        r#"<script>self.__next_f.push([1,"1:{}"]</script>"#,
        r#"<script id="__NEXT_DATA__">{bad}</script>"#,
    ] {
        assert!(NextJsData::parse(html).is_err(), "accepted {html}");
    }
}

#[test]
fn ignores_markers_inside_unrelated_scripts_and_metadata() {
    let html = format!(
        r#"<script>const example = 'self.__next_f.push([1,"bad"])';</script>{}"#,
        script("1:D{\"count\":99}\n2:{\"count\":7}\n")
    );
    assert_eq!(
        NextJsData::parse(&html)
            .unwrap()
            .deserialize::<u32>("/count")
            .unwrap(),
        7
    );
}

#[test]
fn text_length_can_start_with_uppercase_hex_and_be_zero() {
    let data = NextJsData::parse(&script("1:TA,01234567892:T0,3:true\n")).unwrap();
    assert_eq!(data.flight[0].value, "0123456789");
    assert_eq!(data.flight[1].value, "");
    assert_eq!(data.flight[2].value, true);
}

#[test]
fn bootstrap_resets_transport_and_duplicate_ids_preserve_wire_order() {
    let html = format!(
        "{}<script>(self.__next_f=self.__next_f||[]).push([0])</script>{}",
        script("discarded"),
        script("1:D{}\n1:{\"count\":7}\n")
    );
    let data = NextJsData::parse(&html).unwrap();
    assert_eq!(data.flight.len(), 2);
    assert_eq!(data.deserialize::<u32>("/count").unwrap(), 7);
}

#[test]
fn every_chunk_boundary_preserves_the_same_model() {
    let wire = "1:HL[\"/font.woff2\",\"font\"]\n2:{\"name\":\"中文\",\"quoted\":\"a\\\"b\"}\n";
    for split in (0..=wire.len()).filter(|&i| wire.is_char_boundary(i)) {
        let html = script(&wire[..split]) + &script(&wire[split..]);
        let data = NextJsData::parse(&html).unwrap();
        assert_eq!(
            data.deserialize::<String>("/name").unwrap(),
            "中文",
            "split at {split}"
        );
        assert_eq!(data.deserialize::<String>("/quoted").unwrap(), "a\"b");
    }
}
