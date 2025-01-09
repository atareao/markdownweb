use serde::{Serialize, Deserialize};
use gray_matter::{engine::YAML, Matter};
use tracing::{
    error,
    debug
};
use std::path::PathBuf;
use std::collections::HashMap;
use std::error::Error;
use minijinja::context;
use comrak::{markdown_to_html, Options};
use super::{
    ENV,
    Metadata,
    Page,
    Site,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Index{
    pub index: Page,
    pub pages: Vec<Page>,
}

impl Index {
    pub async fn read(route: &PathBuf, source_folder: &PathBuf, destination_folder: &PathBuf, pages: Vec<Page>) -> Result<Self, Box<dyn Error>> {
        debug!("Route: {:?}", route);
        debug!("Source folder: {:?}", source_folder);
        debug!("Destination folder: {:?}", destination_folder);
        let source = PathBuf::new()
            .join(source_folder)
            .join("index.md");
        debug!("Reading index: {:?}", &source);
        if let Ok(true) = tokio::fs::try_exists(&source).await {
            debug!("File exists: {:?}", &source);
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
                index: Page {
                    route: route.to_path_buf(),
                    metadata,
                    content: markdown_to_html(&result.content, &Options::default()),
                },
                pages,
            })
        }else{
            debug!("File does not exist: {:?}", &source);
            let slug = destination_folder.file_name().unwrap().to_str().unwrap().to_string();
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
                index: Page {
                    route: route.to_path_buf(),
                    metadata,
                    content: "".to_string(),
                },
                pages,
            })
        }
    }

    pub async fn generate(&self, site: &Site, parent: &PathBuf) {
        debug!("--- Start generation {:?} - {:?}", &parent, &self.index.route);
        debug!("Parent: {:?}", parent);
        debug!("Generate {:?}index.html", parent);
        debug!("Route: {:?}", self.index.route);
        let destination_file = PathBuf::new()
            .join(parent)
            .join("index.html");
        debug!("Destination: {:?}", self.index.route);
        match tokio::fs::create_dir_all(&parent).await {
            Ok(_) => debug!("Created parent directory: {:?}", &parent),
            Err(e) => error!("Can not create parent directory: {:?}. {}", &parent, e),
        }
        if let Ok(true) = tokio::fs::try_exists(&destination_file).await {
            debug!("File exists. Overwriting {:?}", &destination_file);
            match tokio::fs::remove_file(&destination_file).await {
                Ok(_) => debug!("Removed file: {:?}", &destination_file),
                Err(e) => error!("Can not remove file: {:?}. {}", &destination_file, e),
            }
        }
        debug!("Pages: {:?}", self.pages);
        let ctx = context!(
            site => site,
            page => self.index,
            pages => self.pages,
        );
        debug!("Context: {:?}", ctx);
        match ENV.get_template(&self.index.metadata.template) {
            Ok(template) => {
                debug!("Template: {:?}", template);
                match template.render(&ctx) {
                    Ok(rendered) => {
                        match tokio::fs::write(&destination_file, &rendered).await {
                            Ok(_) => debug!("Generated index: {:?}", &destination_file),
                            Err(e) => error!("Can not generate index: {:?}. {}", &destination_file, e),
                        }
                    },
                    Err(e) => error!("Can not render template. {}", e),
                }
            },
            Err(e) => error!("Can not get template. {}", e),

        }
        debug!("--- End generation {:?} - {:?}", &parent, &self.index.route);
    }
}
