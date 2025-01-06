use serde::{Serialize, Deserialize};
use tokio::fs::read_to_string;
use std::{process, fmt::{self, Display}};
use super::Site;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config{
    pub source: String,
    pub destination: String,
    pub assets: String,
    pub site: Site,
}

impl Display for Config{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "source: {}\ndestination: {}\nsite: {}",
            self.source,
            self.destination,
            self.site,
        )
    }
}

impl Config {
    pub async fn read_configuration() -> Config{
        let content = match read_to_string("config.yml")
            .await {
                Ok(value) => value,
                Err(e) => {
                    println!("Error with config file `config.yml`: {e}");
                    process::exit(0);
                }
            };
        match serde_yaml::from_str(&content){
            Ok(configuration) => configuration,
            Err(e) => {
                println!("Error with config file `config.yml`: {e}");
                process::exit(0);
            }
        }
    }
}
