use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    pub metadata: Metadata,
    pub result: Vec<Result>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub order_option: i64,
    pub page: i64,
    pub size: i64,
    pub tk: String,
    pub total: i64,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    #[serde(rename = "_id")]
    pub id: String,
    pub ani_type: String,
    pub cover: String,
    pub episode: String,
    pub favorability: Favorability,
    pub is_favorite: bool,
    pub jp_date: String,
    pub jp_name: String,
    pub name: String,
    pub status: String,
    pub streaming: bool,
    pub studios: Vec<Studio>,
    pub tags: HashMap<String, u8>,
    pub tw_date: String,
    pub user_record: Value,
    pub play_total: Option<u32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Favorability {
    pub average: f64,
    pub counts: i64,
    pub dist: HashMap<String, u32>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Studio {
    #[serde(rename = "_id")]
    pub id: String,
    pub name: String,
}
