use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Message {
    pub username: User,
    pub message: String,
}

#[derive(Deserialize, Serialize)]
pub struct User(pub String);

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
