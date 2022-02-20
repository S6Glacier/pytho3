
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
        let client_calls = Arc::clone(&client.urls);
        let stub_target = StubTarget::new(Network::Mastodon);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await
        .expect("Should be Ok()");

        let calls = (*client_calls).lock().await;

        assert_eq!(*calls, vec![feed]);
    }

    #[tokio::test]
    async fn test_syndycate_fetches_multiple_feeds() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::new(&gen_items(&[feed1, feed2]));
        let client_calls = Arc::clone(&client.urls);
        let stub_target = StubTarget::new(Network::Mastodon);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await
        .expect("Should be Ok()");

        let calls = (*client_calls).lock().await;

        assert_eq!(*calls, vec![feed1, feed2]);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_posts_to_targets() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::new(&gen_items(&[feed]));
        let stub_target = StubTarget::new(Network::Mastodon);
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await
        .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, *gen_items(&[feed]).get(feed).unwrap());
    }

    #[tokio::test]
    async fn test_syndycate_should_skip_published_posts() {
        let feed = "http://example.com/rss.xml";
        let config = config(vec![feed.to_string()]);

        let client = StubRssClient::new(&gen_items(&[feed]));
        let stub_target = StubTarget::new(Network::Mastodon);
        let target_calls = Arc::clone(&stub_target.calls);
        let targets = vec![stub_target.into()];

        let items = gen_items(&[feed]);
        let storage = SyndicatedPostStorageStub::default();

        for item in items.get(feed).unwrap() {
            storage
                .store(SyndicatedPost::new(
                    Network::Mastodon,
                    &String::from("id"),
                    item,
                ))
                .unwrap();
        }

        syndicate(&config, &client, &targets, &storage, false)
            .await
            .expect("Should be Ok()");

        let calls = (*target_calls).lock().await;

        assert_eq!(*calls, []);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_from_multiple_feeds_to_multiple_targets() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let items = &gen_items(&[feed1, feed2]);
        let client = StubRssClient::new(items);
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await
        .expect("Should be Ok()");

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        let expected: Vec<Item> = merged_items(items, &[feed1, feed2]);

        assert_eq!(*calls1, expected);
        assert_eq!(*calls2, expected);
    }

    #[rustfmt::skip]
    fn gen_target_combinations(feed1: &str, feed2: &str) -> HashMap<String, Vec<Item>> {
        let mut items: HashMap<String, Vec<Item>> = gen_items_with_extension(&[feed1], 2, 0, &create_iwt_extension_map(&[social::Network::Mastodon], None, &Vec::new()));
        items.get_mut(feed1)
            .unwrap()
            .extend(
                gen_items_with_extension(&[feed1], 1, 2, &create_iwt_extension_map(&[social::Network::Twitter], None, &Vec::new()))
                    .get(feed1).unwrap().iter().cloned()
            );
        items.get_mut(feed1)
            .unwrap()
            .extend(
                gen_items_with_extension(&[feed1], 1, 3, &create_iwt_extension_map(&[social::Network::Twitter, social::Network::Mastodon], None, &Vec::new()))
                    .get(feed1).unwrap().iter().cloned()
            );
        items.extend(
                gen_items_with_extension(&[feed2], 1, 0, &create_iwt_extension_map(&[social::Network::Mastodon], None, &Vec::new()))
            );
        items.get_mut(feed2)
            .unwrap()
            .extend(
                gen_items_with_extension(&[feed2], 2, 1, &create_iwt_extension_map(&[social::Network::Twitter], None, &Vec::new()))
                    .get(feed2).unwrap().iter().cloned()
            );
        items.get_mut(feed2)
            .unwrap()
            .extend(
                gen_items_with_extension(&[feed2], 2, 3, &create_iwt_extension_map(&[social::Network::Twitter, social::Network::Mastodon], None, &Vec::new()))
                    .get(feed2).unwrap().iter().cloned()
            );
        // items.extend(gen_items_with_extension(&[feed1], 1, 4, create_iwt_extension_map(&[social::Network::Twitter, social::Network::Mastodon])));
        // items.extend(gen_items_with_extension(&[feed2], 1, 0, create_iwt_extension_map(&[social::Network::Mastodon])));
        // items.extend(gen_items_with_extension(&[feed2], 2, 2, create_iwt_extension_map(&[social::Network::Twitter])));
        // items.extend(gen_items_with_extension(&[feed2], 2, 4, create_iwt_extension_map(&[social::Network::Twitter, social::Network::Mastodon])));
        items
    }

    fn merged_items(items_hash: &HashMap<String, Vec<Item>>, keys: &[&str]) -> Vec<Item> {
        let mut items = Vec::new();
        for key in keys {
            items.extend(items_hash.get(&(*key).to_string()).unwrap().clone());
        }
        items
    }

    #[tokio::test]
    async fn test_syndycate_publishes_from_multiple_feeds_only_to_selected_targets() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let items = gen_target_combinations(feed1, feed2);
        let client = StubRssClient::new(&items);
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await
        .expect("Should be Ok()");

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        let expected_target1 = merged_items(&items, &[feed1, feed2])
            .into_iter()
            .filter(|item| {
                item.get_iwt_extension()
                    .unwrap()
                    .target_networks
                    .iter()
                    .any(|tn| tn.network == social::Network::Mastodon)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            *calls1, expected_target1,
            "\niTarget1 Expected:\n{:#?}\n,Got:\n{:#?}\n",
            expected_target1, calls1
        );

        let expected_target2 = merged_items(&items, &[feed1, feed2])
            .into_iter()
            .filter(|item| {
                item.get_iwt_extension()
                    .unwrap()
                    .target_networks
                    .iter()
                    .any(|tn| tn.network == social::Network::Twitter)
            })
            .collect::<Vec<_>>();
        assert_eq!(
            *calls2, expected_target2,
            "\nTarget2 Expected:\n{:#?}\n,Got:\n{:#?}\n",
            expected_target2, calls2
        );
    }

    #[tokio::test]
    async fn test_syndycate_does_not_publish_when_dry_run_is_true() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let client = StubRssClient::new(&gen_items(&[feed1, feed2]));
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            true,
        )
        .await
        .expect("Should be Ok()");

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        assert!(calls1.is_empty());
        assert!(calls2.is_empty());
    }

    #[tokio::test]
    async fn test_syndycate_publishes_when_single_feed_fails() {
        let feed1 = "http://example.com/rss.xml?failure=1";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let feed2_items = gen_items(&[feed2]).get(feed2).unwrap().clone();
        let mut items = gen_items(&[feed1]);
        items.insert(feed2.to_string(), feed2_items.clone());

        let client = StubRssClient::new(&items);
        let stub_target1 = StubTarget::new(Network::Mastodon);
        let target_calls1 = Arc::clone(&stub_target1.calls);
        let stub_target2 = StubTarget::new(Network::Twitter);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(
            &config,
            &client,
            &targets,
            &SyndicatedPostStorageStub::default(),
            false,
        )
        .await;

        assert!(result.is_err());

        let calls1 = (*target_calls1).lock().await;
        let calls2 = (*target_calls2).lock().await;

        assert_eq!(*calls1, feed2_items);
        assert_eq!(*calls2, feed2_items);
    }

    #[tokio::test]
    async fn test_syndycate_publishes_when_single_target_fails() {
        let feed1 = "http://example.com/rss.xml";
        let feed2 = "https://blog.example.com/rss.xml";
        let config = config(vec![feed1.to_string(), feed2.to_string()]);

        let items = gen_items(&[feed1, feed2]);
        let client = StubRssClient::new(&items);
        let stub_target1 = FailingStubTarget::default();
        let stub_target2 = StubTarget::new(Network::Mastodon);
        let target_calls2 = Arc::clone(&stub_target2.calls);

        let targets = vec![stub_target1.into(), stub_target2.into()];

        let result = syndicate(
            &config,
            &client,
            &targets,