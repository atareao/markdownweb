use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    pub directory: String,
}

impl Config {
    pub fn new(directory: &str) -> Self {
        Self { directory: directory.to_string() }
    }
    
}
