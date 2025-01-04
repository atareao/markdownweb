use serde::{Serialize, Deserialize};
use std::error::Error;
use chrono::NaiveDate;
use tracing::debug;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub title: String,
    pub date: NaiveDate,
    pub excerpt: String,
    pub slug: String,
    pub vars: HashMap<String, String>,
    pub tags: Vec<String>,
    pub publicated: bool,
    pub template: String,
}

impl Metadata {
    pub fn validate(&self) -> Result<(), Box<dyn Error>>{
        debug!("Validating metadata: {:?}", self);
        if self.title.is_empty() {
            return Err("Title is required".into());
        }
        if self.excerpt.is_empty() {
            return Err("Excerpt is required".into());
        }
        if self.slug.is_empty() {
            return Err("Slug is required".into());
        }
        if self.template.is_empty() {
            return Err("Template is required".into());
        }
        Ok(())
    }
}
