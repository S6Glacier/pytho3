use async_trait::async_trait;
use rss::Channel;

pub struct ReqwestClient;

#[async_trait]
pub trait Client