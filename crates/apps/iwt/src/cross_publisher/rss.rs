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
    use async_trait::async_trait;
    use reqwest::Url;
    use rss::{extension::ExtensionMap, Channel, GuidBuilder, Item};

    use crate::{cross_publisher::rss_item_ext::stubs::create_iwt_extension_map, social};

    use super::Client;

    pub struct StubRssClient {
        pub urls: Arc<Mutex<Vec<String>>>,
        items: HashMap<String, Vec<Item>>,
    }

    impl StubRssClient {
        pub fn new(items: &HashMap<String, Vec<Item>>) -> Self {
            Self {
                items: items.clone(),
                urls: Arc::default(),
            }
        }
    }

    pub fn gen_items(urls: &[&str]) -> HashMap<String, Vec<Item>> {
        gen_items_with_extension(
            urls,
            4,
            0,
            &create_iwt_extension_map(
                &[social::Network::Mastodon, social::Network::Twitter],
                None,
                &Vec::new(),
            ),
        )
    }

    pub fn gen_items_with_extension(
        urls: &[&str],
        count: usize,
        offset: usize,
        extensions: &ExtensionMap,
    ) -> HashMap<String, Vec<Item>> {
        let mut result = HashMap::new();
        for url in urls {
        