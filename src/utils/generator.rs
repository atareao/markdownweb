use std::fmt::{self, Display, Formatter};
use async_recursion::async_recursion;
use notify::{event::{CreateKind, RemoveKind}, EventKind};
use tracing::{error, debug};
use tokio::fs;
use std::error::Error;
use std::path::{Path, PathBuf};
use super::super::models::Config;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::super::models::{Page, Index, Site};

#[derive(Debug, Clone)]
pub struct Generator {
    pub site: Site,
    pub origin: PathBuf,
    pub destination: PathBuf,
}

impl Generator {
    pub async fn new(mutex_config: &Arc<Mutex<Config>>) -> Self {
        let config = mutex_config.lock().await;
        Self {
            site: config.site.clone(),
            origin: Path::new(&config.source).to_path_buf(),
            destination: Path::new(&config.destination).to_path_buf(),
        }
    }

    pub async fn replicate_folder(&self, path: &PathBuf) {
        generate_folder(&self.site, &self.origin, &self.destination, path, false).await;
    }

    pub async fn initial_replication(&self) {
        debug!("=============================");
        if let Ok(true) = tokio::fs::try_exists(&self.destination).await{
            match tokio::fs::remove_dir_all(&self.destination).await {
                Ok(()) => debug!("Delete main folder"),
                Err(err) => error!("Can not delete main folder: {}", err),
            }
        }
        match tokio::fs::create_dir_all(&self.destination).await {
            Ok(()) => debug!("Created destination folder {:?}", self.destination),
            Err(err) => error!("Can not create destination folder {:?}: {}", self.destination, err),
        }

        generate_folder(&self.site, &self.origin, &self.destination,&self.origin, true).await;
        debug!("=============================");
    }


    pub async fn replicate(&self, event: notify::Event) -> Result<(), Box<dyn Error>>{
        match event.kind {
            EventKind::Create(create) => {
                match create {
                    CreateKind::File => {
                        for path in event.paths.iter() {
                            //self.replicate_file(path).await;
                            debug!("Replicating {:?}. From {:?} to {:?}", path, self.origin, self.destination);
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
                debug!("Replicating {:?} to {:?}", self.origin, self.destination);
                debug!("Modify: {:?}", modify);
            }
            EventKind::Remove(delete) => {
                match delete {
                    RemoveKind::File => {
                        for path in event.paths.iter() {
                            let destination = self.get_absolute_destination(path);
                            debug!("Deleting file: {:?}", &destination);
                            fs::remove_file(&destination).await?
                        }
                    },
                    RemoveKind::Folder => {
                        for path in event.paths.iter() {
                            let destination = self.get_absolute_destination(path);
                            debug!("Deleting folder: {:?}", &destination);
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

    fn get_absolute_destination(&self, path: &Path) -> PathBuf {
        let relative = path.strip_prefix(&self.origin).unwrap();
        self.destination.join(relative)
    }
}

impl Display for Generator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Generator from {:?} to {:?}", self.origin, self.destination)
    }
}

#[async_recursion]
pub async fn generate_folder(site: &Site, main_source: &PathBuf, main_destination: &PathBuf, path: &PathBuf, recursive: bool) {
    debug!("Source folder: {:?}", path);
    let page_route = path.strip_prefix(main_source).unwrap().to_path_buf();
    debug!("Route: {:?}", page_route);
    let destination_folder = main_destination.join(&page_route);
    debug!("Destination folder: {:?}", &destination_folder);
    let mut directories = Vec::new();
    let mut pages = Vec::new();
    if let Ok(mut entries) = fs::read_dir(path).await{
        while let Ok(Some(entry)) = entries.next_entry().await {
            if entry.file_type().await.unwrap().is_dir() {
                directories.push(entry.path());
                if recursive {
                    generate_folder(site, main_source, main_destination, &entry.path(), recursive).await;
                }
            }else if entry.file_type().await.unwrap().is_file() && entry.path().extension().unwrap() == "md"{
                debug!("File: {:?}", entry.path());
                let wraped_page = Page::read(&page_route, &entry.path().to_path_buf()).await;
                if wraped_page.is_some() {
                    let page = wraped_page.unwrap();
                    page.generate(site, &destination_folder.to_path_buf()).await;
                    pages.push(page);
                }
            }
        }
        let index = Index::read(&page_route, path, &destination_folder, pages).await.unwrap();
        index.generate(site, &destination_folder).await;
    }
}
