
use super::rss;
use ::rss::Channel;
use futures::{Future, FutureExt, StreamExt, TryFutureExt};

use super::rss_item_ext::RssItemExt;
use super::syndicated_post;
use super::target::Target;
use crate::{Config, IwtError};

/// Orchestrates syndication
pub async fn syndicate<R, S>(
    config: &Config,
    rss_client: &R,
    targets: &[Box<dyn Target>],
    storage: &S,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>>