use std::path::PathBuf;

use minijinja::{
    Environment,
    path_loader,
    Error,
    ErrorKind,
    State,
    value::{
        Kwargs,
        Value,
    },
};
use once_cell::sync::Lazy;
use chrono::{DateTime, FixedOffset};
use chrono_tz::Tz;
use tracing::debug;

mod metadata;
mod page;
mod config;
mod index;
mod site;
mod pageerror;
mod publishers;

pub use metadata::Metadata;
pub use page::Page;
pub use pageerror::create_page_error;
pub use index::Index;
pub use config::Config;
pub use site::Site;
pub use publishers::Mastodon;
pub use publishers::Telegram;


pub static ENV: Lazy<Environment<'static>> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    env.add_filter("striptags", striptags);
    env.add_filter("date", date);
    env.add_filter("truncate", truncate);
    env.add_filter("path", path);
    env.add_function("now", now);
    env
});

fn striptags(value: String) -> String {
    let mut data = String::new();
    let mut inside = false;
    // Step 1: loop over string chars.
    for c in value.chars() {
        // Step 2: detect markup start and end, and skip over markup chars.
        if c == '<' {
            inside = true;
            continue;
        }
        if c == '>' {
            inside = false;
            continue;
        }
        if !inside {
            // Step 3: push other characters to the result string.
            data.push(c);
        }
    }
    data
}

pub fn path(val: Value) -> Result<String, Error> {
        if val.is_undefined() || val.is_none() {
            return Ok(String::new());
        }

        let iter = val.try_iter().map_err(|err| {
            Error::new(
                ErrorKind::InvalidOperation,
                format!("cannot join value of type {}", val.kind()),
            )
            .with_source(err)
        })?;

        let mut path = PathBuf::new();
        for item in iter {
            if let Some(s) = item.as_str() {
                path.push(s)
            }
        }
        Ok(path.to_string_lossy().to_string())
    }

fn value_to_chrono_datetime(
    value: Value,
) -> Result<DateTime<FixedOffset>, Error> {
    match value.as_str(){
        Some(s) => match DateTime::parse_from_rfc3339(s){
            Ok(dt) => Ok(dt),
            Err(e) => Err(Error::new(
                ErrorKind::MissingArgument,
                e.to_string()
            )),
        },
        None => Err(Error::new(
            ErrorKind::MissingArgument,
            "Not a valid string"
        )),
    }
}

pub fn date(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let format = kwargs.get::<Option<&str>>("format")?;
    match kwargs.get::<Option<&str>>("timezone")?{
        Some(timezone) => {
            let tz: Tz = timezone.parse().unwrap();
            let datetime = value_to_chrono_datetime(value).unwrap().with_timezone(&tz);
            Ok(format!("{}", datetime.format(format.unwrap())))
        },
        None => {
            let datetime = value_to_chrono_datetime(value).unwrap();
            Ok(format!("{}", datetime.format(format.unwrap())))

        },
    }
}

pub fn truncate(_state: &State, value: Value, kwargs: Kwargs) -> Result<String, Error> {
    let length = kwargs.get::<Option<usize>>("length")?.unwrap();
    match value.as_str() {
        Some(s) => match value.as_str().unwrap().char_indices().nth(length) {
            None => Ok(s.to_string()),
            Some((idx, _)) => Ok(s[..idx].to_string()),
        },
        None => Err(Error::new(
            ErrorKind::MissingArgument,
            "Not a valid string"
        )),
    }
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
