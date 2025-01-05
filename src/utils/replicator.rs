use std::fmt::{self, Display, Formatter};
use notify::{event::{CreateKind, RemoveKind}, EventKind};
use tracing::{error, debug};
use tokio::fs;
use std::error::Error;
use std::path::PathBuf;
use async_walkdir::WalkDir;
use futures_lite::stream::StreamExt;

use crate::models::{Page, Index};

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

    async fn replicate_folder(&self, path: &PathBuf) {
        debug!("Replicate folder: {:?}", path);
        let source_folder = path.to_str().unwrap().to_string();
        let destination_folder = self.get_absolute_destination(&source_folder);
        let len = self.origin.len();
        let route = source_folder[len..].to_string();
        debug!("Route: {}", route);
        match fs::create_dir(&destination_folder).await{
            Ok(_) => {
                debug!("Created directory: {}", &destination_folder);
                let mut directories = Vec::new();
                let mut pages = Vec::new();
                let mut entries = WalkDir::new(path);
                loop {
                    match entries.next().await {
                        Some(Ok(entry)) => {
                            if entry.path().is_dir() {
                                directories.push(entry.path());
                            }else if entry.path().is_file() && entry.path().ends_with(".md"){
                                match Page::read(&route, entry.path().to_str().unwrap()).await {
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
                for page in pages.as_slice() {
                    match page.generate(&destination_folder).await{
                        Ok(_) => debug!("Generated page: {}", page.metadata.slug),
                        Err(e) => error!("Can not generate page: {}. {}", page.metadata.slug, e),
                    }
                }
                if self.origin == path.to_str().unwrap() {
                    debug!("Hola");
                }else{
                    debug!("Adios");
                }
                match Index::read(&route, &source_folder, &destination_folder, pages).await{
                    Ok(index) => {
                        match index.generate(&destination_folder).await {
                            Ok(_) => debug!("Generated index for {}", &destination_folder),
                            Err(err) => error!("Can not generate index for {}. {}", &destination_folder, err),
                        }
                    },
                    Err(err) => {
                        error!("Can not read index for {}. {}", &destination_folder, err);
                    },
                }
            },
            Err(err) => {
                error!("Can not create directory: {}. {}", &destination_folder, err);
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
                            //self.replicate_file(path).await;
                            debug!("Replicating {:?}. From {} to {}", path, self.origin, self.destination);
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
