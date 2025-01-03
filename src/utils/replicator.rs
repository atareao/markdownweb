use std::fmt::{self, Display, Formatter};
use notify::{event::{CreateKind, RemoveKind}, EventKind};
use tracing::{error, debug};
use tokio::fs;
use std::error::Error;

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

    pub fn get_absolute_destination(&self, path: String) -> String {
        path.replace(&self.origin, &self.destination)
    }

    pub async fn replicate(&self, event: notify::Event) -> Result<(), Box<dyn Error>>{
        match event.kind {
            EventKind::Create(create) => {
                match create {
                    CreateKind::File => {
                        for path in event.paths.iter() {
                            debug!("Deleting file: {}", self.get_absolute_destination(path.to_str().unwrap().to_string()));
                            let destination = self.get_absolute_destination(path.to_str().unwrap().to_string());
                            Page::generate(&self.origin, &destination).await?;
                        }
                    },
                    CreateKind::Folder => {
                        event.paths.iter().for_each(|path| {
                            println!("Replicating Folder {} to {}", path.display(), self.destination);
                            println!("Abs: {}", self.get_absolute_destination(path.to_str().unwrap().to_string()))
                        });
                    },
                    _ => {},
                }
            }
            EventKind::Modify(_) => {
                println!("Replicating {} to {}", self.origin, self.destination);
            }
            EventKind::Remove(delete) => {
                match delete {
                    RemoveKind::File => {
                        for path in event.paths.iter() {
                            println!("Deleting file: {}", self.get_absolute_destination(path.to_str().unwrap().to_string()));
                            let destination = self.get_absolute_destination(path.to_str().unwrap().to_string());
                            fs::remove_file(&destination).await?
                        }
                    },
                    RemoveKind::Folder => {
                        for path in event.paths.iter() {
                            println!("Deleting folder: {}", self.get_absolute_destination(path.to_str().unwrap().to_string()));
                            let destination = self.get_absolute_destination(path.to_str().unwrap().to_string());
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

async fn replicate_folder(origin: &str, destination: &str) -> Result<(), Error> {
    println!("Replicating Folder {} to {}", origin, destination);
    fs::create_dir(destination).await
}

async fn replicate_file(origin: &str, destination: &str) {
    debug!("Replicating File {} to {}", origin, destination);
}
