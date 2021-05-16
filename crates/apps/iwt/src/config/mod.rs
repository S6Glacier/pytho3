
use std::fs;

use oauth2::{AccessToken, ClientId};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub rss: Rss,
    pub db: DB,
    pub twitter: Twitter,
    pub mastodon: Mastodon,
    pub url_shortener: UrlShortener,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Rss {
    pub urls: Vec<String>,
}