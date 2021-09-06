use async_trait::async_trait;
use rss::Channel;

pub struct ReqwestClient;

#[async_trait]
pub trait Client {
    /// Loads RSS feed from the given URL a parse it into a Channel
    async fn get_channel(&self, url: &str)
        -> Result<Channel, Box<dyn std::error::Error + 'static>>;
}

#[async_trait]
impl Client for ReqwestClient {
    async fn get_channel(
        &self,
        url: &str,
    ) -> Result<Channel, Box<dyn std::error::Error + 'static>> {
        let feed = reqwest::get(url).await?.bytes().await?;

        log::debug!("Response received from url: {}", url);

        let channel = Channel::read_from(&feed[..])?;

        log::debug!(
            "Successfully loaded channel \"{}\", with {} items",
            channel.title(),
            channel.items().len()
        );
        Ok(channel)
    }
}

pub mod tests {}

#[cfg(test)]
pub mod stubs {
    use std::{collections::HashMap, fmt::Display, sync::Arc};

    use async_mutex::Mutex;
    use as