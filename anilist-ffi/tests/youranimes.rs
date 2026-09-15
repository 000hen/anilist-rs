use anilist::{
    AnilistError, your_animes_parse_detail, your_animes_parse_list, your_animes_parse_search,
};
use serde_json::json;

fn html(field: &str, list: bool) -> String {
    let anime = json!({
        "_id": "1108", "adultstreaming": [], "aniType": "TV",
        "commentCount": 0, "cover": "https://example.com/cover.webp",
        "episode": "12", "favorability": {"average": 5.0, "counts": 1},
        "name": "小林家的龍女僕S", "description": "中文 description",
        "status": "finished", "streaming": [], "dayOfWeek": 1,
        "date": "2026-07-06 00:30"
    });
    let value = if list { json!([anime]) } else { anime };
    let row = json!(["$", "$L1", null, {field: value}]);
    let row = if list { row } else { json!([null, row]) };
    format!(
        "<script>self.__next_f.push({})</script>",
        json!([1, format!("1:{row}\n")])
    )
}

#[test]
fn parses_response_bodies_without_an_async_runtime() {
    let list = your_animes_parse_list(html("animes", true)).unwrap();
    let detail = your_animes_parse_detail(html("anime", false)).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, "youranimes:1108");
    assert_eq!(detail.id, list[0].id);
    assert_eq!(detail.name, "小林家的龍女僕S");
    assert_eq!(detail.description, "中文 description");
    let time = detail.on_air_time.unwrap();
    assert_eq!(time.week, anilist::Weekday::Monday);
    assert_eq!(time.minute.unwrap().value, 30);
    assert_eq!(time.zone, "Asia/Tokyo");
}

#[test]
fn search_returns_ids_for_caller_owned_detail_requests() {
    assert_eq!(
        your_animes_parse_search(r#"{"result":[{"_id":"1108"},{"_id":"42"}]}"#.into()).unwrap(),
        ["youranimes:1108", "youranimes:42"]
    );
}

#[test]
fn invalid_bodies_return_ffi_errors() {
    assert!(matches!(
        your_animes_parse_list("<html/>".into()),
        Err(AnilistError::InvalidResponse { .. })
    ));
    assert!(matches!(
        your_animes_parse_detail("<html/>".into()),
        Err(AnilistError::InvalidResponse { .. })
    ));
    assert!(matches!(
        your_animes_parse_search("broken json".into()),
        Err(AnilistError::InvalidResponse { .. })
    ));
}

#[test]
fn orders_week_using_the_hosts_day_without_a_clock() {
    let days = anilist::week_order(anilist::Weekday::Saturday);
    assert_eq!(days.len(), 8);
    assert_eq!(
        days[0],
        anilist::ScheduleDay::Weekday {
            day: anilist::Weekday::Saturday
        }
    );
    assert_eq!(
        days[1],
        anilist::ScheduleDay::Weekday {
            day: anilist::Weekday::Sunday
        }
    );
    assert_eq!(days[7], anilist::ScheduleDay::Unknown);
}
