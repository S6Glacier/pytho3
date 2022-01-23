
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
where
    R: rss::Client,
    S: syndicated_post::Storage,
{
    log::debug!("Received config: {:?}", config);
    run_and_collect(config.rss.urls.iter(), |url| {
        rss_client
            .get_channel(url)
            .and_then(|channel| syndycate_channel(channel, targets, storage, dry_run))
    })
    .await
}

/// Syndicates a single channel
async fn syndycate_channel<S: syndicated_post::Storage>(
    channel: Channel,
    targets: &[Box<dyn Target>],
    storage: &S,
    dry_run: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    run_and_collect(targets.iter(), |target| {
        run_and_collect(channel.items.iter(), |post| {
            log::info!(
                "{} |> Syndicating post to {}",
                post.link().unwrap(),
                target.network().to_string()
            );
            let stored = storage.find(&post.guid.as_ref().unwrap().value, &target.network());

            // println!("Post: {:?}", post);

            async {
                match stored {
                    Ok(None) => {
                        log::info!(
                            "{} |> Post not found in DB, syndycating to {}",
                            post.link().unwrap(),
                            target.network().to_string()
                        );

                        if let Some(extension) = post.get_iwt_extension() {
                            if extension
                                .target_networks
                                .iter()
                                .any(|tn| tn.network == target.network())
                            {
                                if dry_run {
                                    log::info!(
                                        "{} |> Publishing to {} is skipped due to --dry-run",
                                        post.link().unwrap(),
                                        target.network().to_string()
                                    );
                                    Ok(())
                                } else {
                                    log::info!(
                                        "{} |> Publishing to {}",
                                        post.link().unwrap(),
                                        target.network().to_string()
                                    );
                                    let result = target
                                        .publish(post, &extension)
                                        .map(|result| {
                                            result.and_then(|syndicated| {
                                                storage.store(syndicated).map_err(|err| {
                                                    Box::new(err) as Box<dyn std::error::Error>
                                                })
                                            })
                                        })
                                        .await;
                                    log::info!(
                                        "{} |> Published to {}",
                                        post.link().unwrap(),
                                        target.network().to_string()
                                    );
