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

#[derive(Debug)]
pub enum AppError {
    Crossbeam,
    TCPServer,
    EventRead,
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            AppError::Crossbeam => "Crossbeam",
            AppError::TCPServer => "TCPServer",
            AppError::EventRead => "EventRead",
        };
        write!(f, "{}", text)
    }
}
