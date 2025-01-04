use std::fmt::{self, Display, Formatter};
use notify::{event::{CreateKind, RemoveKind}, EventKind};
use tracing::{error, debug};
use tokio::fs;
use std::error::Error;
use std::env::var;
use std::path::PathBuf;
use async_walkdir::WalkDir;
use async_recursion::async_recursion;
use futures_lite::stream::StreamExt;

use crate::models::Page;

#[derive(Debug, Clone)]
pub struct Replicator {
    pub origin: String,
    pub destination: String,
}

impl Replicator {
    pub fn new(origin: String, destination: String) -> Self {
        Self {
            origin,
            destination,
        }
    }

    pub fn get_absolute_destination(&self, path: &str) -> String {
        path.replace(&self.origin, &self.destination)
    }

    async fn replicate_file(&self, path: &PathBuf) {
        if path.extension().unwrap() == "md" {
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
        }else{
            let source = path.to_str().unwrap().to_string();
            let destination = self.get_absolute_destination(&source);
            debug!("Copy from {} to {}", source, destination);
            match tokio::fs::copy(path, &destination).await{
                Ok(_) => {
                    debug!("Copied from {} to {}", source, destination);
                },
                Err(err) => {
                    error!("Can not copy from {} to {}. Error: {}", source, destination, err);
                },
            }
        }

    }

    #[async_recursion]
    async fn replicate_folder(&self, path: &PathBuf) {
        let source = path.to_str().unwrap().to_string();
        let destination = self.get_absolute_destination(&source);
        match fs::create_dir(&destination).await{
            Ok(_) => {
                debug!("Created directory: {}", &destination);
                let mut entries = WalkDir::new(path);
                loop {
                    match entries.next().await {
                        Some(Ok(entry)) => {
                            if entry.path().is_dir() {
                                println!("folder: {}", entry.path().display());
                                self.replicate_folder(&entry.path()).await;
                            } else if entry.path().is_file(){
                                println!("file: {}", entry.path().display());
                                self.replicate_file(&entry.path()).await;
                            }
                        },
                        Some(Err(e)) => {
                            eprintln!("error: {}", e);
                            break;
                        }
                        None => break,
                    }
                }

            },
            Err(err) => {
                error!("Can not create directory: {}. {}", &destination, err);
            },
        }
    }

    pub async fn initial_replication(&self) {
        let path: String = var("SOURCE").unwrap_or("/source".to_string());
        let mut entries = WalkDir::new(path);
        loop {
            match entries.next().await {
                Some(Ok(entry)) => {
                    if entry.path().is_dir() {
                        println!("folder: {}", entry.path().display());
                        self.replicate_folder(&entry.path()).await;
                    } else if entry.path().is_file(){
                        println!("file: {}", entry.path().display());
                        self.replicate_file(&entry.path()).await;
                    }
                },
                Some(Err(e)) => {
                    eprintln!("error: {}", e);
                    break;
                }
                None => break,
            }
        }
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
