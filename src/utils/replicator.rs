use std::fmt::{self, Display, Formatter};
use glob::glob;
use notify::{event::{CreateKind, RemoveKind}, EventKind};
use tracing::{error, debug};
use tokio::fs;
use std::error::Error;
use std::path::{PathBuf, Path};
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;
use minijinja::context;
use super::super::models::ENV;

use crate::models::Page;

#[derive(Debug, Clone)]
pub struct Replicator {
    pub origin: String,
    pub destination: String,
}

impl Replicator {
    pub fn new(origin: &str, destination: &str) -> Self {
        Self {
            origin: origin.to_string(),
            destination: destination.to_string(),
        }
    }

    pub fn get_absolute_destination(&self, path: &str) -> String {
        path.replace(&self.origin, &self.destination)
    }

    async fn generate_index(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        let mut metadata = Vec::new();
        let pattern = format!("{}/*.md", path.to_str().unwrap());
        for file in glob(&pattern)?{
            let source = &file?;
            metadata.push(Page::get_metadata(source.to_str().unwrap()).await?);
            self.replicate_file(source).await;
        }
        metadata.sort_by(|a, b| b.date.cmp(&a.date));
        let index = format!("{}/index.html", self.get_absolute_destination(path.to_str().unwrap()));
        let ctx = context!(
            files => metadata,
        );
        let template = ENV.get_template("index.html")?;
        let rendered = template.render(&ctx)?;
        tokio::fs::write(index, rendered).await?;
        Ok(())
    }

    async fn replicate_file(&self, path: &Path) {
        if path.extension().unwrap().to_str().unwrap().ends_with(".md") {
            let source = path.to_str().unwrap().to_string();
            let destination = self.get_absolute_destination(&source)
                .replace(".md", ".html");
            debug!("Going to generate {} from {}", destination, source);
            match Page::generate(&source, &destination).await {
                Ok(_) => {
                    debug!("Generated {} from {}", destination, source);
                },
                Err(err) => {
                    error!("Can not generate {} from {}. Error: {}", destination, source, err);
                    let mut err = err.as_ref();
                    while let Some(next_err) = err.source() {
                        error!("caused by: {:#}", next_err);
                        err = next_err;
                    }
                },
            }
        }

    }

    async fn replicate_folder(&self, path: &PathBuf) {
        debug!("Replicate folder: {:?}", path);
        let source = path.to_str().unwrap().to_string();
        let destination = self.get_absolute_destination(&source);
        match fs::create_dir(&destination).await{
            Ok(_) => {
                debug!("Created directory: {}", &destination);
                let mut directories = Vec::new();
                let mut pages = Vec::new();
                let mut entries = WalkDir::new(path);
                loop {
                    match entries.next().await {
                        Some(Ok(entry)) => {
                            if entry.path().is_dir() {
                                directories.push(entry.path());
                            }else if entry.path().is_file() && entry.path().ends_with(".md"){
                                match Page::read(entry.path().to_str().unwrap()).await {
                                    Ok(page) => pages.push(page),
                                    Err(e) => {
                                        error!("Cant read {}. {}", entry.path().to_str().unwrap(), e)
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => {
                            error!("error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }
                pages.sort_by(|a, b| b.metadata.date.cmp(&a.metadata.date));
                for page in pages {
                    page.generate(&destination).await;

                }
                for directory in directories{
                    let destination = self.get_absolute_destination(directory.to_str().unwrap());
                    match tokio::fs::create_dir_all(&destination).await {
                        Ok(()) => debug!("Created directory {:?}", &destination),
                        Err(err) => error!("Can create directory. {:?}. {}", &destination, err),
                    }
                    match self.generate_index(&directory).await {
                        Ok(()) => debug!("Generate index for {:?}", &destination),
                        Err(err) => error!("Can not generate index for {:?}. {}", &destination, err),
                    }
                }
            },
            Err(err) => {
                error!("Can not create directory: {}. {}", &destination, err);
            },
        }
    }

    pub async fn initial_replication(&self) {
        if let Ok(true) = tokio::fs::try_exists(&self.destination).await{
            match tokio::fs::remove_dir_all(&self.destination).await {
                Ok(()) => debug!("Delete main folder"),
                Err(err) => error!("Can not delete main folder: {}", err),
            }
        }
        self.replicate_folder(&PathBuf::from(&self.origin)).await;
    }


    pub async fn replicate(&self, event: notify::Event) -> Result<(), Box<dyn Error>>{
        match event.kind {
            EventKind::Create(create) => {
                match create {
                    CreateKind::File => {
                        for path in event.paths.iter() {
                            self.replicate_file(path).await;
                        }
                    },
                    CreateKind::Folder => {
                        for path in event.paths.iter() {
                            self.replicate_folder(path).await;
                        }
                    },
                    _ => {},
                }
            }
            EventKind::Modify(modify) => {
                debug!("Replicating {} to {}", self.origin, self.destination);
                debug!("Modify: {:?}", modify);
            }
            EventKind::Remove(delete) => {
                match delete {
                    RemoveKind::File => {
                        for path in event.paths.iter() {
                            println!("Deleting file: {}", self.get_absolute_destination(path.to_str().unwrap()));
                            let destination = self.get_absolute_destination(path.to_str().unwrap());
                            fs::remove_file(&destination).await?
                        }
                    },
                    RemoveKind::Folder => {
                        for path in event.paths.iter() {
                            println!("Deleting folder: {}", self.get_absolute_destination(path.to_str().unwrap()));
                            let destination = self.get_absolute_destination(path.to_str().unwrap());
                            fs::remove_dir_all(&destination).await?
                        }
                    },
                    _ => {},
                }
            }
            _ => {}
        }
    Ok(())
    }
}

impl Display for Replicator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Replicator from {} to {}", self.origin, self.destination)
    }
}
