use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    pub message: String,
    pub status: u16
}

impl Error {
    pub fn new(message: String, status: u16) -> Self {
        Self {
            message,
            status
        }
    }
}