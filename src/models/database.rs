use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::models::blocks::Blocks;
use crate::models::users::User;

#[derive(Debug, Deserialize)]
pub struct Cell {
    pub id: String,
    #[serde(rename = "type")]
    pub cell_type: String,
    #[serde(flatten)]
    pub block: Option<Blocks>,
}

#[derive(Debug, Deserialize)]
pub struct Row {
    pub archived: bool,
    pub cover: Option<Value>,
    pub created_by: User,
    pub created_time: String,
    pub icon: Option<Value>,
    pub id: String,
    pub in_trash: bool,
    pub last_edited_by: Value,
    pub last_edited_time: String,
    pub object: String,
    pub parent: Option<Value>,
    pub properties: Option<HashMap<String, Cell>>,
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseQueryResponse {
    pub object: String,
    pub results: Vec<Row>,
}
