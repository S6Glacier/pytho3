
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

                                    result
                                }
                            } else {
                                log::info!(
                                    "{} |> Not configured to be syndicated to {}",
                                    post.link().unwrap(),
                                    target.network().to_string()
                                );
                                Ok(())
                            }
                        } else {
                            Err(
                                Box::new(IwtError::new("Rss Item doesn't have an IWT extension"))
                                    as Box<dyn std::error::Error>,
                            )
                        }
                    }
                    Ok(Some(_)) => {
                        log::info!(
                            "{} |> Has been already syndicated to {}",
                            post.link().unwrap(),
                            target.network().to_string()
                        );
                        Ok(())
                    }
                    Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
                }
            }
        })
    })
    .await
}

async fn run_and_collect<C, I, F, Fu>(items: C, f: F) -> Result<(), Box<dyn std::error::Error>>
where
    C: Iterator<Item = I>,
    // TODO: understand why this didn't work: Fn(I) -> dyn Future<Output = Result<(), Box<dyn std::error::Error>>>
    //       or: Fn(I) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>>>>
    F: Fn(I) -> Fu,
    Fu: Future<Output = Result<(), Box<dyn std::error::Error>>>,
{
    futures::stream::iter(items)
        .map(f)
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::sync::Arc;

    use oauth2::{AccessToken, ClientId};
    use rss::Item;

    use super::syndicated_post::{Storage, SyndicatedPost};
    use crate::config::{Config, Mastodon, Rss, Twitter, UrlShortener, DB};
    use crate::cross_publisher::rss::stubs::gen_items_with_extension;
    use crate::cross_publisher::rss_item_ext::stubs::create_iwt_extension_map;
    use crate::cross_publisher::rss_item_ext::RssItemExt;
    use crate::cross_publisher::stubs::rss::{gen_items, StubRssClient};
    use crate::cross_publisher::stubs::syndycated_post::SyndicatedPostStorageStub;
    use crate::cross_publisher::stubs::target::FailingStubTarget;
    use crate::cross_publisher::stubs::target::StubTarget;
    use crate::social::{self, Network};

    use super::syndicate;

    fn config(urls: Vec<String>) -> Config {
        Config {
            rss: Rss { urls },
            db: DB {
                path: String::from("some/path"),
            },
            twitter: Twitter {
                client_id: ClientId::new(String::from("some_client_id")),
            },
            mastodon: Mastodon {
                base_uri: String::from("https://example.com/mastodon"),
                access_token: AccessToken::new(String::from("some-access-token")),
            },
            url_shortener: UrlShortener {
                protocol: String::from("http"),
                domain: String::from("shortly"),
                put_base_uri: Some(String::from("http://localhost:9000")),
            },
        }
    }

    #[tokio::test]
    async fn test_syndycate_fetches_a_feed() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::new(&gen_items(&[feed]));