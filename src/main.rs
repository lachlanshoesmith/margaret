use std::{collections::HashMap, error::Error};

use clap::Parser;
use reqwest::Client;
use serde_json::Value;

use margaret::models::{
    blocks::Blocks,
    database::{fetch_notion_database, DatabaseCredentials, DatabaseQueryResponse, Row},
    filters::{CheckboxColumnFilter, ColumnFilter, QueryFilter, RichTextColumnFilter},
    responses::{response_to_result, ErrorResponse},
};

#[derive(Parser, Debug)]
struct Args {
    notion_db: String,
    integration_secret: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Column {
    id: String,
    name: String,
    column_type: String,
}

fn get_db_columns(db: &str) -> Result<Option<Vec<Column>>, Box<dyn Error>> {
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

async fn query_column_values(
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
            cell.block.as_ref().unwrap().clone()
        })
        .collect();

    Ok(blocks)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let credentials = DatabaseCredentials {
        id: args.notion_db,
        token: args.integration_secret,
    };
    let db = fetch_notion_database(&credentials).await?;

    let columns = get_db_columns(&db.body)?;

    if columns.is_none() {
        println!("No rows found");
        return Ok(());
    }

    let columns = columns.unwrap();

    let email_col = columns.iter().find(|col| col.name == "Email");
    if email_col.is_none() {
        println!("No email column found");
        return Ok(());
    }

    let query = QueryFilter::And(
        Box::new(QueryFilter::ColumnFilter(ColumnFilter {
            property: "Email".to_string(),
            rich_text: Some(RichTextColumnFilter {
                is_not_empty: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        })),
        Box::new(QueryFilter::ColumnFilter(ColumnFilter {
            property: "Interview?".to_string(),
            checkbox: Some(CheckboxColumnFilter {
                equals: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        })),
    );

    let blocks = query_column_values(&credentials, email_col.unwrap(), &query).await?;
    let emails: Vec<String> = blocks.iter().map(|block| block.to_string()).collect();
    println!("{:#?}", emails);

    Ok(())
}
