
use crate::social::Network;
use async_trait::async_trait;
use rss::Item;

use super::{rss_item_ext::IwtRssExtension, syndicated_post::SyndicatedPost};

#[async_trait(?Send)]
pub trait Target {
    async fn publish<'a>(
        &self,
        post: &Item,
        extension: &IwtRssExtension,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>>;

    fn network(&self) -> Network;
}

#[cfg(test)]
pub mod stubs {
    use async_mutex::Mutex;
    use std::{fmt::Display, sync::Arc};

    use async_trait::async_trait;
    use rss::Item;

    use crate::cross_publisher::rss_item_ext::IwtRssExtension;
    use crate::cross_publisher::syndicated_post::SyndicatedPost;
    use crate::social::Network;

    use super::Target;

    pub struct StubTarget {
        pub social_network: Network,
        pub calls: Arc<Mutex<Vec<Item>>>,
    }

    impl StubTarget {
        pub fn new(social_network: Network) -> Self {
            Self {
                social_network,
                calls: Arc::default(),
            }
        }
    }

    #[async_trait(?Send)]
    impl Target for StubTarget {
        async fn publish<'a>(
            &self,
            post: &Item,
            _extension: &IwtRssExtension,
        ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
            let mut calls = self.calls.lock().await;
            let id = calls.len();
            calls.push(post.clone());
            Ok(SyndicatedPost::new(
                self.social_network.clone(),
                &id.to_string(),
                post,
            ))
        }

        fn network(&self) -> Network {
            self.social_network.clone()
        }
    }

    impl From<StubTarget> for Box<dyn Target> {
        fn from(stub_target: StubTarget) -> Self {
            Box::new(stub_target)
        }
    }

    #[derive(Debug)]
    pub struct TargetError;
    impl Display for TargetError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "RssClientError")
        }
    }

    impl std::error::Error for TargetError {}
