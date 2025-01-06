use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::json;
use tracing::{info, error};
use std::error::Error;
use super::Publisher;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Mastodon{
    instance: String,
    access_token: String,
}

impl Publisher for Mastodon{
    async fn post_text(&self, text: &str) -> Result<(), Box<dyn Error>>{
        let url = format!("https://{}/api/v1/statuses", self.instance);
        info!("{}", &url);
        let body = json!({"status": text});
        match Client::new()
            .post(&url)
            .json(&body)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .send()
            .await{
                Ok(response) => {
                    info!("Mensaje envíado a Mastodon: {}",
                        response.status().to_string());
                    Ok(())
                },
                Err(error) => {
                    error!("No he podido enviar el mensaje a Mastodon: {}",
                        error.to_string());
                    Err(Box::new(error))
                },
            }
    }
    async fn post_audio(&self, _text: &str, _audio: &str) -> Result<(), Box<dyn std::error::Error>> {
        Err("Not implemented".into())
    }
}

