use async_trait::async_trait;
use rss::Channel;

pub struct ReqwestClient;

#[async_trait]
pub trait Client {
    /// Loads RSS feed from the given URL a parse it into a Channel
    async fn get_channel(&self, url: &str)
        -> Result<Channel, Box<dyn std::error::Error + 'static>>;
}

#[async_t