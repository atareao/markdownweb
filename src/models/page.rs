use super::{Metadata, Site, ENV};
use gray_matter::{engine::YAML, Matter};
use minijinja::context;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{debug, error};
use comrak::{markdown_to_html, Options};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Page {
    pub route: PathBuf,
    pub metadata: Metadata,
    pub content: String,
}

impl Page {
    pub async fn read(route: &Path, source: &PathBuf) -> Option<Self> {
        if let Ok(data) = tokio::fs::read_to_string(&source).await {
            let matter = Matter::<YAML>::new();
            let result = matter.parse(&data);
            if let Ok(serialized_data) = result.data.ok_or("Can not read metadata") {
                if let Ok(mut metadata) = serialized_data.deserialize::<Metadata>() {
                    metadata.init();
                    if let Ok(()) = metadata.validate() {
                        return Some(Self {
                            route: route.to_path_buf(),
                            metadata,
                            content: markdown_to_html(&result.content, &Options::default()),
                        });
                    } else {
                        error!("Can not validate metadata for {:?}", source);
                    }
                } else {
                    error!("Can not deserialize metadata for {:?}", source);
                }
            } else {
                error!("Can not read metadata for {:?}", source);
            }
        } else {
            error!("Can not read {:?}", source);
        }
        None
    }

    pub async fn generate(&self, site: &Site, parent: &PathBuf) {
        debug!(
            "--- Start generation {:?} - {}",
            &parent, &self.metadata.slug
        );
        debug!("Parent: {:?}", parent);
        debug!("Slug: {:?}", self.metadata.slug);
        let destination_folder = PathBuf::new().join(parent).join(&self.metadata.slug);
        debug!("Destination folder: {:?}", &destination_folder);
        let destination_file = PathBuf::new().join(&destination_folder).join("index.html");
        debug!("Destination file: {:?}", &destination_file);
        match tokio::fs::create_dir_all(&destination_folder).await {
            Ok(_) => {
                debug!("Created folder: {:?}", &destination_folder);
                if let Ok(true) = tokio::fs::try_exists(&destination_file).await {
                    debug!("File exists. Overwriting {:?}", &destination_file);
                    if let Ok(()) = tokio::fs::remove_file(&destination_file).await {
                        debug!("Removed file: {:?}", &destination_file);
                    } else {
                        error!("Can not remove file: {:?}", &destination_file);
                    }
                }
                let ctx = context!(
                    site => site,
                    page => self,
                );
                match ENV.get_template(&self.metadata.template) {
                    Ok(template) => {
                        match template.render(&ctx) {
                            Ok(rendered) => {
                                match tokio::fs::write(&destination_file, rendered).await {
                                    Ok(()) => debug!("Save {:?}", &destination_file),
                                    Err(e) => error!("Can not save {:?}. {}", &destination_file, e),
                                }
                            },
                            Err(e) => error!("Can not render {:?}. {}", &destination_file, e),
                        }
                    }
                    Err(e) => error!("Can not get template {:?}. {}", &self.metadata.template, e),
                };
            }
            Err(err) => {
                error!("Can not create folder: {:?}. {}", &destination_folder, err);
            }
        }
        debug!("--- End generation {:?} - {}", &parent, &self.metadata.slug);
    }
}
