mod models;
mod http;
mod utils;

use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use core::panic;
use std::path::Path;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::str::FromStr;
use std::env::var;
use tracing::{debug, error};

use utils::Replicator;


#[tokio::main]
async fn main(){

    let source = var("SOURCE").unwrap_or("/source".to_string());
    let destination = var("DESTINATION").unwrap_or("/destination".to_string());

    if !source.starts_with("/") || !destination.starts_with("/"){
        panic!("SOURCE and DESTINATION must be absolute paths")
    }
    let log_level: String = var("LOG_LEVEL").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let replicator = Replicator::new(&source, &destination);
    replicator.initial_replication().await;

    debug!("Starting server");
    tokio::spawn(async {
        server().await;
    });
    monitor(&source, &destination).await;
}

async fn server(){
    debug!("Starting server");
    match http::serve().await {
        Ok(()) => debug!("Server started"),
        Err(err) => {
            error!("Can start server: {}", err);
            let mut err = err.as_ref();
            while let Some(next_err) = err.source() {
                error!("caused by: {:#}", next_err);
                err = next_err;
            }
        },

    }
}

async fn monitor(source: &str, destination: &str){
    let replicator = Replicator::new(source, destination);
    //replicator.initial_replication().await;
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    // Use recommended_watcher() to automatically select the best implementation
    // for your platform. The `EventHandler` passed to this constructor can be a
    // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
    // another type the trait is implemented for.
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    debug!("Watching: {}", source);
    watcher.watch(Path::new(&source), RecursiveMode::Recursive).unwrap();
    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => {
                match replicator.replicate(event).await {
                    Ok(_) => {},
                    Err(err) => {
                        error!("Can not generate {} from {}. Error: {}", destination, source, err);
                        let mut err = err.as_ref();
                        while let Some(next_err) = err.source() {
                            error!("caused by: {:#}", next_err);
                            err = next_err;
                        }

                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
