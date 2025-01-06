use serde::{Serialize, Deserialize};
use reqwest::Client;
use serde_json::json;
use tracing::{debug, error};
use super::Publisher;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Telegram{
    access_token: String,
    chat_id: String,
}

impl Telegram {
    fn prepare(text: &str) -> String{
        text.chars()
            .map(|c| match c {
                '"' => '\'',
                _   => c,
            })
            .collect()
    }
}

impl Publisher for Telegram{
    async fn post_text(&self, text: &str) -> Result<(), Box<dyn Error>>{
        debug!("Message to publish in Telegram: {}", text);
        let url = format!("https://api.telegram.org/bot{}/sendMessage",
            self.access_token);
        debug!("url  {}", url);
        let content = Self::prepare(text);
        debug!("content  {}", content);
        let message = json!({
            "chat_id": self.chat_id,
            "text": content,
            "parse_mode": "HTML",
        });
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await{
                Ok(response) => {
                    debug!("Mensaje envíado a Telegram: {}",
                        response.status().to_string());
                    Ok(())
                },
                Err(error) => {
                    error!("No he podido enviar el mensaje a Telegram: {}",
                        error.to_string());
                    Err(Box::new(error))
                },
            }
    }

    async fn post_audio(&self, text: &str, audio: &str) -> Result<(), Box<dyn std::error::Error>>{
        let url = format!("https://api.telegram.org/bot{}/sendAudio",
            self.access_token);
        debug!("url  {}", url);
        let content = Self::prepare(text);
        debug!("content  {}", content);
        let message = json!({
            "chat_id": self.chat_id,
            "audio": audio,
            "caption": content,
            "parse_mode": "HTML",
        });
        match Client::new()
            .post(url)
            .json(&message)
            .send()
            .await?
            .error_for_status()?
            .text()
            .await {
            Ok(response) => {
                debug!("Mensaje envíado a Telegram: {}", response);
                Ok(())
            }
            Err(e) => {
                error!("No he podido enviar el mensaje a Telegram: {}", e);
                Err(Box::new(e))
            }
        }
    }
}
