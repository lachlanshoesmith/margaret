use core::fmt;

use serde::Deserialize;
use serde_json::Value;

use crate::models::users::User;

#[derive(Debug, Deserialize, Clone)]
pub enum Blocks {
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
    #[serde(rename = "number")]
    Number(f32),
    #[serde(rename = "relation")]
    Relation(Vec<RelationBlock>),
}

impl fmt::Display for Blocks {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match self {
            Blocks::RichText(texts) => texts
                .iter()
                .map(|text| text.plain_text.clone())
                .collect::<Vec<String>>()
                .join("\n"),
            Blocks::Checkbox(value) => value.to_string(),
            Blocks::Email(value) => value.to_string(),
            Blocks::Title(texts) => texts
                .iter()
                .map(|text| text.plain_text.clone())
                .collect::<Vec<String>>()
                .join("\n"),
            Blocks::MultiSelect(selections) => selections
                .iter()
                .map(|selection| selection.name.clone())
                .collect::<Vec<String>>()
                .join(", "),
            Blocks::CreatedBy(value) => value.to_string(),
            Blocks::CreatedTime(value) => value.to_string(),
            Blocks::Number(value) => value.to_string(),
            Blocks::Relation(ids) => ids
                .iter()
                .map(|ids| ids.id.clone())
                .collect::<Vec<String>>()
                .join(", "),
        };
        write!(f, "{}", value)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct RichTextAnnotations {
    bold: bool,
    code: bool,
    color: String,
    italic: bool,
    strikethrough: bool,
    underline: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RichText {
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

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Text {
    content: String,
    link: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub enum TextTypes {
    #[serde(rename = "text")]
    Text,
    #[serde(rename = "equation")]
    Equation,
    #[serde(rename = "mention")]
    Mention,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct MultiSelectSelection {
    color: String,
    id: String,
    name: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Expression {
    expression: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct RelationBlock {
    pub id: String,
}
