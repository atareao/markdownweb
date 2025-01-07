use serde::{Serialize, Deserialize};
use std::error::Error;
use chrono::{DateTime, FixedOffset};
use tracing::debug;
use std::collections::HashMap;
use slug::slugify;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Metadata {
    pub title: String,
    #[serde(default)]
    pub date: DateTime<FixedOffset>,
    #[serde(default)]
    pub excerpt: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub vars: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub publicated: bool,
    pub template: String,
}

impl Metadata {
    pub fn init(&mut self) {
        if self.slug.is_empty() {
            self.slug = slugify(&self.title);
        }
        if self.excerpt.is_empty() {
            self.excerpt = self.title.clone();
            self.excerpt.truncate(150);
        }
    }
    pub fn validate(&self) -> Result<(), Box<dyn Error>>{
        debug!("Validating metadata: {:?}", self);
        if self.title.is_empty() {
            return Err("Title is required".into());
        }
        if self.excerpt.is_empty() {
            return Err("Exceprt is required".into());
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
