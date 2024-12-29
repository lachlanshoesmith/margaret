use core::fmt;
use std::{collections::HashMap, error::Error};

use clap::Parser;
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;

use margaret::models::{
    blocks::Blocks,
    database::{DatabaseQueryResponse, Row},
    filters::{ColumnFilter, QueryFilter, RichTextColumnFilter},
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

#[derive(Debug)]
struct DatabaseCredentials {
    id: String,
    token: String,
}

#[derive(Debug)]
struct SimpleResponse {
    status: StatusCode,
    body: String,
}

impl fmt::Display for SimpleResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (status {})", self.body, self.status)
    }
}

#[derive(Debug)]
struct ErrorResponse {
    response: SimpleResponse,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.response)
    }
}

impl Error for ErrorResponse {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl SimpleResponse {
    async fn from_response(res: Response) -> Self {
        SimpleResponse {
            status: res.status(),
            body: res.text().await.unwrap(),
        }
    }
    // async fn from_error(err: reqwest::Error) -> Self {
    //     SimpleResponse {
    //         status: err.status().unwrap(),
    //         body: err.to_string(),
    //     }
    // }
}

async fn response_to_result(res: Response) -> Result<SimpleResponse, ErrorResponse> {
    let status_body = SimpleResponse::from_response(res).await;

    if status_body.status.is_success() {
        Ok(status_body)
    } else {
        Err(ErrorResponse {
            response: status_body,
        })
    }
}

async fn fetch_notion_database(
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
) -> Result<Vec<Blocks>, ErrorResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.notion.com/v1/databases/{}/query",
        credentials.id
    );
    let mut query_body = HashMap::new();
    query_body.insert(
        "filter",
        QueryFilter {
            property: &column.name,
            column_filter: ColumnFilter {
                rich_text: Some(RichTextColumnFilter {
                    is_not_empty: Some(true),
                    ..Default::default()
                }),
            },
        },
    );

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

    let blocks = query_column_values(&credentials, email_col.unwrap()).await?;
    let emails: Vec<String> = blocks.iter().map(|block| block.to_string()).collect();
    println!("{:#?}", emails);

    Ok(())
}
