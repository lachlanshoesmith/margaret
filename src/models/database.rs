use std::{cmp, collections::HashMap, error::Error};

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::models::blocks::Blocks;
use crate::models::users::User;

use super::responses::{response_to_result, ErrorResponse, SimpleResponse};

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

#[derive(Debug)]
pub struct DatabaseCredentials {
    pub id: String,
    pub token: String,
}

pub async fn fetch_notion_database(
    credentials: &DatabaseCredentials,
) -> Result<SimpleResponse, ErrorResponse> {
    let client = Client::new();
    let url = format!("https://api.notion.com/v1/databases/{}", credentials.id);

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", credentials.token))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await;

    response_to_result(response.unwrap()).await
}

pub async fn follow_relation(
    token: &str,
    relation: &Relation,
) -> Result<SimpleResponse, Box<dyn Error>> {
    let credentials = DatabaseCredentials {
        id: relation.database_id.to_string(),
        token: token.to_string(),
    };
    let res = fetch_notion_database(&credentials).await?;
    Ok(res)
}

#[derive(Debug, Hash, PartialEq, cmp::Eq, Deserialize)]
#[allow(dead_code)]
pub struct Relation {
    pub database_id: String,
    pub synced_property_id: Option<String>,
    pub synced_property_name: Option<String>,
}

#[derive(Debug, Hash, PartialEq, cmp::Eq)]
#[allow(dead_code)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub column_type: String,
    pub relation: Option<Relation>,
}
