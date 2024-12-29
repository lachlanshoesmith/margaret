use core::fmt;

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct UserEmail {
    email: String,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct User {
    avatar_url: Option<String>,
    id: String,
    name: Option<String>,
    object: String,
    person: Option<UserEmail>,
    #[serde(rename = "type")]
    user_type: Option<String>,
}

impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
