
use crate::social::Network;
use async_trait::async_trait;
use rss::Item;

use super::{rss_item_ext::IwtRssExtension, syndicated_post::SyndicatedPost};

#[async_trait(?Send)]
pub trait Target {