use std::rc::Rc;

use super::rss_item_ext::IwtRssExtension;
use super::syndicated_post::SyndicatedPost;
use super::target::Target;
use crate::commons::{text, url_shortener};
use crate::social::Network;
use async_trait::async_trait;
use futures::TryFutureEx