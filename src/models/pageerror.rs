use std::{
    error::Error,
    fmt,
};

#[derive(Debug)]
pub struct PageError{
    message: String,
}

impl PageError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Error for PageError {}

impl fmt::Display for PageError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error: {}", self.message)
    }
}

