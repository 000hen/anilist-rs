use serde_json::{Value, json};

use super::{detail::parse_detail, list::parse_list, search::parse_search};

fn anime() -> Value {
    json!({
        "_id": "1108", "adultstreaming": [], "aniType": "TV",
        "commentCount": 0, "cover": "https://example.com/cover.webp",
        "episode": "12", "favorability": {"average": 5.0, "counts": 1},
        "name": "小林家的龍女僕S", "description": "Quotes: \"hello\"\n中文",
        "status": "finished", "streaming": [], "dayOfWeek": null
    })
}

fn html(row: Value) -> String {
    let payload = format!("1:I[42,[],\"default\"]\n2:{row}\n");
    // Split in the middle of the serialized model; script boundaries are not
    // record boundaries. Include metadata before the target in the same stream.
    let split = payload.find("description").unwrap() + 5;
    [&payload[..split], &payload[split..]]
        .into_iter()
        .map(|chunk| {
            format!(
                "<script>self.__next_f.push({});</script>",
                json!([1, chunk])
            )
        })
        .collect()
}

#[test]
fn list_extracts_split_flight_model_and_converts_anime() {
    let result = parse_list(&html(json!(["$", "$L1", null, {"animes": [anime()]}]))).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].id, "youranimes:1108");
    assert_eq!(result[0].name, "小林家的龍女僕S");
    assert_eq!(result[0].description, "Quotes: \"hello\"\n中文");
}

#[test]
fn detail_extracts_split_flight_model_and_converts_anime() {
    let result =
        parse_detail(&html(json!([null, ["$", "$L1", null, {"anime": anime()}]]))).unwrap();
    assert_eq!(result.id, "youranimes:1108");
    assert_eq!(result.name, "小林家的龍女僕S");
    assert_eq!(result.description, "Quotes: \"hello\"\n中文");
}

#[test]
fn missing_hydration_and_wrong_anime_shape_return_errors() {
    assert!(parse_list("<html></html>").is_err());
    assert!(parse_detail("<html></html>").is_err());
    assert!(
        parse_list(&html(
            json!(["$", "$L1", null, {"animes": [{"description":"invalid"}]}])
        ))
        .is_err()
    );
}

#[test]
fn search_returns_prefixed_ids_without_detail_hydration() {
    let result = parse_search(r#"{"result":[{"_id":"1108"},{"_id":"42"}]}"#).unwrap();

    assert_eq!(result, ["youranimes:1108", "youranimes:42"]);
}

#[test]
fn search_parse_errors_identify_the_response_stage() {
    let error = parse_search("not json").unwrap_err();

    assert!(error.to_string().contains("YourAnimes search response"));
}
