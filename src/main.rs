use std::{collections::HashMap, error::Error};

use clap::Parser;
use reqwest::Client;
use serde_json::Value;

#[derive(Parser, Debug)]
struct Args {
    notion_db: String,
    integration_secret: String,
}

async fn fetch_notion_database(database_id: &str, token: &str) -> Result<(), Box<dyn Error>> {
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

    let body: HashMap<&str, Value> = serde_json::from_str(&response).unwrap();

    println!("{body:?}");
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    fetch_notion_database(&args.notion_db, &args.integration_secret).await?;
    Ok(())
}
