use std::fmt::Display;

use async_trait::async_trait;
use reqwest;

use super::permashort_link::PermashortCitation;

#[derive(Debug)]
pub struct ClientError {
    pub message: String,
}

impl From<reqwest::Error> for ClientError {
    fn from(e: reqwest::Error) -> Self {
        // TODO: better error handling
        ClientError {
            message: e.to_string(),
        }
    }
}

impl Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("UrlShortener Client Error: {}", self.message))
    }
}

impl std::error::Error for ClientError {}

#[async_trait(?Send)]
pub trait Client {
    async fn put_uri(&self, uri: &str) -> Result<PermashortCitation, ClientError>;
}

pub struct ReqwestClient {
    protocol: String,
    domain: String,
    base_uri: String,
    client: reqwest::Client,
}

impl ReqwestClient {
    #[m