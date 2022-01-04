
use rss::{extension::Extension, Item};

use crate::social;

/// Rust representation of the Indieweb Tools RSS extension
#[derive(Debug, PartialEq)]
pub struct IwtRssExtension {
    /// The target networks where Item should be syndicated to
    pub target_networks: Vec<IwtRssTargetNetwork>,
    /// Content Warning, this is only used by Mastodon
    pub content_warning: Option<String>,
    /// Tags of the item
    pub tags: Vec<String>,
}

#[derive(Debug, PartialEq)]
pub struct IwtRssTargetNetwork {
    pub network: social::Network,
}

pub trait RssItemExt {
    fn get_iwt_extension(&self) -> Option<IwtRssExtension>;
}

fn get_children<'a>(ext: &'a Extension, key: &str) -> Vec<&'a Extension> {
    ext.children()
        .get(key)
        .iter()
        .flat_map(|children| children.iter())
        .collect::<Vec<_>>()
}

fn get_key<'a>(ext: &'a Extension, key: &str) -> Option<&'a Extension> {
    ext.children()
        .get(key)
        .and_then(|children| children.first())
}

fn get_value<'a>(ext: &'a Extension, key: &str) -> Option<&'a str> {
    get_key(ext, key).and_then(rss::extension::Extension::value)
}

impl RssItemExt for Item {
    fn get_iwt_extension(&self) -> Option<IwtRssExtension> {
        // todo!()
        self.extensions()
            .get(&"iwt".to_string())
            .and_then(|iwt_root| iwt_root.get("extension").map(|extensions| &extensions[0]))
            .map(|iwt_extension| {
                // println!("iwt: {:?}", iwt_extension);
                let target_networks = get_key(iwt_extension, "targetNetworks")
                    .iter()
                    .flat_map(|target_networks| get_children(target_networks, "targetNetwork"))
                    .map(|target_network| {
                        let target_network_name = target_network.value().unwrap();
                        match target_network_name {
                            "twitter" => IwtRssTargetNetwork {
                                network: social::Network::Twitter,
                            },
                            "mastodon" => IwtRssTargetNetwork {
                                network: social::Network::Mastodon,
                            },
                            _ => panic!("Unknown netowrk: {target_network_name}"),
                        }
                    })
                    .collect::<Vec<_>>();

                let tags = get_children(iwt_extension, "tags")
                    .iter()
                    .flat_map(|tags| get_children(tags, "tag"))
                    .map(|tag| tag.value().unwrap().to_string())
                    .collect();

                let content_warning =
                    get_value(iwt_extension, "contentWarning").map(std::borrow::ToOwned::to_owned);

                IwtRssExtension {
                    target_networks,
                    content_warning,
                    tags,
                }
            })
    }