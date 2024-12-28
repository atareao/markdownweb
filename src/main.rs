mod models;
mod http;

use tracing_subscriber::{
    EnvFilter,
    layer::SubscriberExt,
    util::SubscriberInitExt,
};
use std::path::Path;
use notify::{Event, RecursiveMode, Result, Watcher};
use std::sync::mpsc;
use std::str::FromStr;
use std::env::var;


#[tokio::main]
async fn main(){
    tokio::spawn(async {
        let source = var("SOURCE").unwrap_or("/source".to_string());
        let (tx, rx) = mpsc::channel::<Result<Event>>();

        // Use recommended_watcher() to automatically select the best implementation
        // for your platform. The `EventHandler` passed to this constructor can be a
        // closure, a `std::sync::mpsc::Sender`, a `crossbeam_channel::Sender`, or
        // another type the trait is implemented for.
        let mut watcher = notify::recommended_watcher(tx).unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher.watch(Path::new(&source), RecursiveMode::Recursive).unwrap();
        // Block forever, printing out events as they come in
        for res in rx {
            match res {
                Ok(event) => println!("event: {:?}", event),
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    });

    let log_level: String = var("LOG_LEVEL").unwrap_or("debug".to_string());
    tracing_subscriber::registry()
        .with(EnvFilter::from_str(&log_level).unwrap())
        .with(tracing_subscriber::fmt::layer())
        .init();
    http::serve().await.unwrap();
}