use serde::{Serialize, Deserialize};
use super::{Telegram, Mastodon};
use std::fmt::{self, Display};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Site{
    pub url: String,
    pub language: String,
    pub language_direction: String,
    pub theme: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub avatar: String,
    pub email: String,
    pub telegram: Option<Telegram>,
    pub mastodon: Option<Mastodon>,
}

impl Display for Site{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "url: {}\title: {}\ndescription: {}",
            self.url,
            self.title,
            self.description,
        )
    }
}
