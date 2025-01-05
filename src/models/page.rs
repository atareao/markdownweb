use serde::{Serialize, Deserialize};
use gray_matter::{engine::YAML, Matter};
use tracing::{
    error,
    debug
};
use std::error::Error;
use minijinja::context;
use super::{
    ENV,
    Metadata,
    PageError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page{
    pub route: String,
    pub metadata: Metadata,
    pub content: String,
}

impl Page {
    pub async fn read(route: &str, source: &str) -> Result<Self, Box<dyn Error>> {
        let data = tokio::fs::read_to_string(&source).await?;
        debug!("Data: {}", data);
        let matter = Matter::<YAML>::new();
        let result = matter.parse(&data);
        debug!("Result: {:?}", result);
        let mut metadata: Metadata = result
            .data
            .ok_or("Can not read metadata")?
            .deserialize()?;
        metadata.init();
        metadata.validate()?;
        Ok(Self {
            route: route.to_string(),
            metadata,
            content: result.content,
        })
    }

    pub async fn generate(&self, parent: &str) -> Result<(), Box<dyn Error>> {
        debug!("Generate {}/{}/index.html", parent, self.metadata.slug);
        let destination_folder = format!("{}/{}", parent, self.metadata.slug);
        let destination_file = format!("{}/index.html", destination_folder);
        if tokio::fs::try_exists(&destination_folder).await? {
            error!("Folder exists. There are documents with same slug {}", self.metadata.slug);
            return Err(Box::new(PageError::new("Folder exists. There are documents with same slug")));
        }
        if tokio::fs::try_exists(&destination_file).await? {
            debug!("File exists. Overwriting {}", &destination_file);
            tokio::fs::remove_file(&destination_file).await?;
        }
        let ctx = context!(
            title => self.metadata.title,
            date => self.metadata.date,
            excerpt => self.metadata.excerpt,
            vars => self.metadata.vars,
            tags => self.metadata.tags,
            content => self.content,
        );
        let template = ENV.get_template(&self.metadata.template)?;
        let rendered = template.render(&ctx)?;
        tokio::fs::write(&destination_file, rendered).await?;
        Ok(())
    }
}
