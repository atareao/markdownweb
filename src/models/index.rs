use serde::{Serialize, Deserialize};
use gray_matter::{engine::YAML, Matter};
use tracing::{
    error,
    debug
};
use std::collections::HashMap;
use std::error::Error;
use minijinja::context;
use super::{
    ENV,
    Metadata,
    Page,
    PageError,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Index{
    pub route: String,
    pub metadata: Metadata,
    pub content: String,
    pub pages: Vec<Page>,
}

impl Index {
    pub async fn read(route: &str,source_folder: &str, destination_folder: &str, pages: Vec<Page>) -> Result<Self, Box<dyn Error>> {
        debug!("Route: {}", route);
        debug!("Source folder: {}", source_folder);
        debug!("Destination folder: {}", destination_folder);
        let source = format!("{}/index.md", source_folder);
        debug!("Reading index: {}", &source);
        if let Ok(true) = tokio::fs::try_exists(&source).await {
            debug!("File exists: {}", &source);
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
                pages,
            })
        }else{
            debug!("File does not exist: {}", &source);
            let slug = if let Some(striped) = destination_folder.strip_suffix("/"){
                striped.split("/").last().unwrap().to_string()
            }else{
                destination_folder.split("/").last().unwrap().to_string()
            };
            let title = slug.replace("-", " ");
            let metadata = Metadata {
                title: title.clone(),
                date: chrono::Utc::now().into(),
                excerpt: title.clone(),
                slug,
                vars: HashMap::new(),
                tags: Vec::new(),
                publicated: true,
                template: "index.html".to_string(),
            };
            metadata.validate()?;
            Ok(Self {
                route: route.to_string(),
                metadata,
                content: "".to_string(),
                pages,
            })
        }
    }

    pub async fn generate(&self, parent: &str) -> Result<(), Box<dyn Error>> {
        debug!("Parent: {}", parent);
        debug!("Generate {}/index.html", parent);
        let destination_folder = parent.to_string();
        let destination_file = format!("{}/index.html", destination_folder);
        if let Ok(true) = tokio::fs::try_exists(&destination_folder).await {
            error!("Folder exists. There are documents with same slug {}", self.metadata.slug);
            return Err(Box::new(PageError::new("Folder exists. There are documents with same slug")));
        }
        if let Ok(true) = tokio::fs::try_exists(&destination_file).await {
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
            pages => self.pages,
        );
        debug!("Context: {:?}", ctx);
        let template = ENV.get_template(&self.metadata.template)?;
        debug!("Template: {:?}", template);
        let rendered = template.render(&ctx)?;
        tokio::fs::write(&destination_file, rendered).await?;
        Ok(())
    }
}
