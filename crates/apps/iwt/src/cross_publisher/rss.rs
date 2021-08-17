use async_trait::async_trait;
use rss::Channel;

pub struct ReqwestClient;

#[async_trait]
pub trait Client {
    /// Loads RSS feed from the given URL a parse it into a Channel
    