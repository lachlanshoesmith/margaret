use std::{collections::HashMap, error::Error};

use models::{
    blocks::{Blocks, Relation},
    database::{fetch_notion_database, Column, DatabaseCredentials, DatabaseQueryResponse, Row},
    filters::QueryFilter,
    responses::{response_to_result, ErrorResponse},
};
use reqwest::Client;
use serde_json::Value;

pub mod models;

pub async fn follow_relations(token: &str, block: &Blocks) -> Result<(), Box<dyn Error>> {
    if let Blocks::Relation(relations) = block {
        for relation in relations {
            let cols = follow_relation(token, relation).await?;
            println!("{:#?}", cols);
        }
        Ok(())
    } else {
        Err("The block provided doesn't represent a relation.".into())
    }
}

async fn follow_relation(
    token: &str,
    relation: &Relation,
) -> Result<Option<Vec<Column>>, Box<dyn Error>> {
    let credentials = DatabaseCredentials {
        id: relation.id.clone(),
        token: token.to_string(),
    };
    let db = fetch_notion_database(&credentials).await;
    let cols = get_db_columns(&db.unwrap().body)?;
    Ok(cols)
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
