use std::error::Error;

use clap::Parser;
use reqwest::Client;
use serde_json::Value;

#[derive(Parser, Debug)]
struct Args {
    notion_db: String,
    integration_secret: String,
}

#[derive(Debug)]
struct Row {
    id: String,
    name: String,
    row_type: String,
}

async fn fetch_notion_database(database_id: &str, token: &str) -> Result<String, Box<dyn Error>> {
    let client = Client::new();
    let url = format!("https://api.notion.com/v1/databases/{database_id}");

    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Notion-Version", "2022-06-28")
        .send()
        .await?
        .text()
        .await?;

    Ok(response)
}

fn get_db_rows(db: &str) -> Result<Option<Vec<Row>>, Box<dyn Error>> {
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
            .map(|property| Row {
                id: property.get("id").unwrap().to_string(),
                name: property.get("name").unwrap().to_string(),
                row_type: property.get("type").unwrap().to_string(),
            })
            .collect(),
    ))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let db = fetch_notion_database(&args.notion_db, &args.integration_secret).await?;
    let rows = get_db_rows(&db)?;
    println!("{rows:#?}");
    Ok(())
}
