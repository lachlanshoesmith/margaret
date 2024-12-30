use std::collections::HashMap;

use serde::Serialize;
use struct_iterable::Iterable;

pub fn get_filter_conditions() -> HashMap<String, String> {
    vec![
        (
            "contains".to_string(),
            "String or i32 (context dependent)".to_string(),
        ),
        (
            "does_not_contain".to_string(),
            "String or i32 (context dependent)".to_string(),
        ),
        ("is_empty".to_string(), "bool".to_string()),
        ("is_not_empty".to_string(), "bool".to_string()),
        ("starts_with".to_string(), "String".to_string()),
        ("ends_with".to_string(), "String".to_string()),
        (
            "equals".to_string(),
            "String, i32 or bool (context dependent)".to_string(),
        ),
        (
            "does_not_equal".to_string(),
            "String, i32, or bool (context dependent)".to_string(),
        ),
    ]
    .into_iter()
    .collect::<HashMap<String, String>>()
}

#[derive(Debug, Serialize, Default, Iterable)]
pub struct RichTextColumnFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub does_not_contain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_not_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starts_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_with: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub does_not_equal: Option<String>,
}

#[derive(Debug, Serialize, Default, Iterable)]
pub struct CheckboxColumnFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub does_not_equal: Option<bool>,
}

#[derive(Debug, Serialize, Default, Iterable)]
pub struct RelationColumnFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub does_not_contain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_empty: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_not_empty: Option<bool>,
}

#[derive(Debug, Serialize, Default)]
pub struct ColumnFilter {
    pub property: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rich_text: Option<RichTextColumnFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkbox: Option<CheckboxColumnFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation: Option<RelationColumnFilter>,
}

#[derive(Debug, Serialize)]
pub enum QueryFilter {
    #[serde(rename = "and")]
    And(Box<QueryFilter>, Box<QueryFilter>),
    #[serde(rename = "or")]
    Or(Box<QueryFilter>, Box<QueryFilter>),
    #[serde(untagged)]
    ColumnFilter(Box<ColumnFilter>),
}
