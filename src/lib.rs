use std::{
    collections::{HashMap, HashSet},
    error::Error,
};

use models::{
    blocks::{Blocks, Relation},
    database::{Column, DatabaseCredentials, DatabaseQueryResponse, Row},
    filters::QueryFilter,
    responses::{response_to_result, ErrorResponse},
};
use reqwest::Client;
use serde_json::Value;

use futures::future::try_join_all;

pub mod models;

pub async fn follow_relations(token: &str, block: &Blocks) -> Result<Vec<Row>, Box<dyn Error>> {
    if let Blocks::Relation(relations) = block {
        Ok(try_join_all(relations.iter().map(|relation| async {
            let body: Value = follow_relation(token, relation).await?;
            Ok::<Row, Box<dyn Error>>(serde_json::from_value::<Row>(body)?)
        }))
        .await?)
    } else {
        Err("The block provided doesn't represent a relation.".into())
    }
}

async fn follow_relation(token: &str, relation: &Relation) -> Result<Value, Box<dyn Error>> {
    let credentials = DatabaseCredentials {
        id: relation.id.clone(),
        token: token.to_string(),
    };
    let res = get_page(&credentials).await?;
    Ok(res)
}

pub fn get_db_columns(db: &str) -> Result<Option<Vec<Column>>, Box<dyn Error>> {
    let body: Value = serde_json::from_str(db)?;
    let properties = body.get("properties");
    if properties.is_none() {
        return Ok(None);
    }
    let properties = properties.unwrap();
    Ok(Some(
        properties
            .as_object()
            .unwrap()
            .values()
            .map(|property| Column {
                id: property.get("id").unwrap().as_str().unwrap().to_string(),
                name: property.get("name").unwrap().as_str().unwrap().to_string(),
                column_type: property.get("type").unwrap().as_str().unwrap().to_string(),
            })
            .collect(),
    ))
}

pub fn row_to_cols(row: &Row) -> HashSet<Column> {
    let mut cols: HashSet<Column> = HashSet::new();
    let properties = row.properties.as_ref().unwrap();
    for (key, value) in properties.iter() {
        cols.insert(Column {
            id: key.to_string(),
            name: key.to_string(),
            column_type: value.cell_type.clone(),
        });
    }
    cols
}

pub async fn get_page(credentials: &DatabaseCredentials) -> Result<Value, ErrorResponse> {
    let client = Client::new();
    let url = format!("https://api.notion.com/v1/pages/{}/", credentials.id);

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", credentials.token))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await;

    let result = response_to_result(response.unwrap()).await?;
    let body = serde_json::from_str(&result.body).unwrap();
    Ok(body)
}

pub async fn query_column_values(
    credentials: &DatabaseCredentials,
    column: &Column,
    query: &QueryFilter,
) -> Result<Vec<Blocks>, ErrorResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.notion.com/v1/databases/{}/query",
        credentials.id
    );
    let mut query_body = HashMap::new();
    query_body.insert("filter", query);

    let response = client
        .post(url)
        .header("Authorization", format!("Bearer {}", credentials.token))
        .header("Notion-Version", "2022-06-28")
        .json(&query_body)
        .send()
        .await;

    let result = response_to_result(response.unwrap()).await?;
    let body: DatabaseQueryResponse = serde_json::from_str(&result.body).unwrap();
    let rows = body.results;
    let blocks: Vec<Blocks> = rows
      .iter()
      .map(|row: &Row| {
          let properties = row.properties.as_ref().unwrap();
          let cell = properties.get(&column.name).unwrap();
          cell.block
              .as_ref()
              .unwrap_or_else(|| {
                  panic!(
                      "Failed to get block in column '{}' of type '{}' - do I know how to handle that type?",
                      column.name,
                      column.column_type
                  )
              })
              .clone()
      })
      .collect();

    Ok(blocks)
}
