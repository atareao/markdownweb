use serde::{Serialize, Deserialize};
use gray_matter::{engine::YAML, Matter};
use minijinja::context;
use tracing::{debug, error};
use std::error::Error;

use super::{
    ENV,
    Metadata
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page{
    metadata: Metadata,
    pub content: String,
}

impl Page {
    pub async fn get_metadata(source: &str) -> Result<Metadata, Box<dyn Error>> {
        let data = tokio::fs::read_to_string(&source).await?;
        debug!("Data: {}", data);
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        debug!("Result: {:?}", result);
        Ok(result
            .data
            .ok_or("Can not read metadata")?
            .deserialize()?)
    }
    pub async fn generate(source: &str, destination: &str) -> Result<(), Box<dyn Error>> {
        debug!("Generate from {} to {}", source, destination);
        /*
        if tokio::fs::try_exists(&destination).await? {
            debug!("File already exists");
            tokio::fs::remove_file(&destination).await?;
        }
        */
        debug!("Going to read source file: {}", &source);
        let data = tokio::fs::read_to_string(&source).await?;
        debug!("Data: {}", data);
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        debug!("Result: {:?}", result);
        let metadata: Metadata = result
            .data
            .ok_or("Can not read metadata")?
            .deserialize()?;
        debug!("Metadata: {:?}", metadata);
        metadata.validate()?;
        let ctx = context!(
            title => metadata.title,
            date => metadata.date,
            excerpt => metadata.excerpt,
            vars => metadata.vars,
            tagas => metadata.tags,
        );
        let template = ENV.get_template(&metadata.template)?;
        let rendered = template.render(&ctx)?;
        tokio::fs::write(destination, rendered).await?;
        Ok(())
    }
}
