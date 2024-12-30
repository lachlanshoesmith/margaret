use serde::Serialize;

#[derive(Debug, Serialize, Default)]
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

#[derive(Debug, Serialize, Default)]
pub struct CheckboxColumnFilter {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub does_not_equal: Option<bool>,
}

#[derive(Debug, Serialize, Default)]
pub struct ColumnFilter {
    pub property: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rich_text: Option<RichTextColumnFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkbox: Option<CheckboxColumnFilter>,
}

#[derive(Debug, Serialize)]
pub enum QueryFilter {
    #[serde(rename = "and")]
    And(Box<QueryFilter>, Box<QueryFilter>),
    #[serde(rename = "or")]
    Or(Box<QueryFilter>, Box<QueryFilter>),
    #[serde(untagged)]
    ColumnFilter(ColumnFilter),
}
