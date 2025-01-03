use serde::{Serialize, Deserialize};
use gray_matter::{engine::YAML, Matter};
use tracing::{debug, error};
use std::error::Error;

use super::Metadata;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page{
    metadata: Metadata,
    pub content: String,
}

impl Page {
    pub async fn generate(source: &str, destination: &str) -> Result<(), Box<dyn Error>> {
        debug!("Filename: {}", source);
        if tokio::fs::try_exists(&destination).await? {
            debug!("File already exists");
            tokio::fs::remove_file(&destination).await?;
        }
        let data = tokio::fs::read_to_string(&source).await?;
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        let metadata: Metadata = result
            .data
            .ok_or("Can not read metadata")?
            .deserialize()?;
        debug!("Metadata: {:?}", metadata);
        metadata.validate()?;

        Ok(())
    }
}
