use core::fmt;
use std::{collections::HashMap, error::Error};

use clap::Parser;
use reqwest::{Client, Response, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Parser, Debug)]
struct Args {
    notion_db: String,
    integration_secret: String,
}

#[derive(Debug)]
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

#[derive(Debug, Serialize)]
struct RichTextColumnQuery {
    is_not_empty: bool,
}

#[derive(Debug, Serialize)]
struct SimpleColumnQuery<'a> {
    property: &'a str,
    rich_text: Option<RichTextColumnQuery>,
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
    async fn from_error(err: reqwest::Error) -> Self {
        SimpleResponse {
            status: err.status().unwrap(),
            body: err.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Cell {
    id: String,
    #[serde(rename = "type")]
    cell_type: String,
    #[serde(flatten)]
    block: Option<Blocks>,
}

#[derive(Debug, Deserialize)]
struct Text {
    content: String,
    link: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MultiSelectSelection {
    color: String,
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct UserEmail {
    email: String,
}

#[derive(Debug, Deserialize)]
struct User {
    avatar_url: Option<String>,
    id: String,
    name: Option<String>,
    object: String,
    person: Option<UserEmail>,
    #[serde(rename = "type")]
    user_type: Option<String>,
}

#[derive(Debug, Deserialize)]
enum Blocks {
    #[serde(rename = "rich_text")]
    RichText(Vec<RichText>),
    #[serde(rename = "checkbox")]
    Checkbox(bool),
    #[serde(rename = "email")]
    Email(String),
    #[serde(rename = "title")]
    Title(Vec<RichText>),
    #[serde(rename = "multi_select")]
    MultiSelect(Vec<MultiSelectSelection>),
    #[serde(rename = "created_by")]
    CreatedBy(User),
    #[serde(rename = "created_time")]
    CreatedTime(String),
}

#[derive(Debug, Deserialize)]
enum TextTypes {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "equation")]
    Equation,
    #[serde(rename = "mention")]
    Mention,
}

#[derive(Debug, Deserialize)]
struct Expression {
    expression: String,
}

#[derive(Debug, Deserialize)]
struct RichTextAnnotations {
    bold: bool,
    code: bool,
    color: String,
    italic: bool,
    strikethrough: bool,
    underline: bool,
}

#[derive(Debug, Deserialize)]
struct RichText {
    #[serde(rename = "type")]
    block_type: TextTypes,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<Text>,
    #[serde(skip_serializing_if = "Option::is_none")]
    // TODO: handle mentions
    mention: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    equation: Option<Expression>,
    annotations: RichTextAnnotations,
    plain_text: String,
    href: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Row {
    archived: bool,
    cover: Option<Value>,
    created_by: User,
    created_time: String,
    icon: Option<Value>,
    id: String,
    in_trash: bool,
    last_edited_by: Value,
    last_edited_time: String,
    object: String,
    parent: Option<Value>,
    properties: Option<HashMap<String, Cell>>,
    url: String,
}

#[derive(Debug, Deserialize)]
struct DatabaseQueryResponse {
    object: String,
    results: Vec<Row>,
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

async fn query_rows_by_column(
    credentials: &DatabaseCredentials,
    column: &Column,
) -> Result<Vec<Row>, ErrorResponse> {
    let client = Client::new();
    let url = format!(
        "https://api.notion.com/v1/databases/{}/query",
        credentials.id
    );
    let mut query_body = HashMap::new();
    query_body.insert(
        "filter",
        SimpleColumnQuery {
            property: &column.name,
            rich_text: Some(RichTextColumnQuery { is_not_empty: true }),
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

    Ok(rows)
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

    let row = query_rows_by_column(&credentials, email_col.unwrap()).await?;
    println!("Rows: {:#?}", row);
    Ok(())
}
