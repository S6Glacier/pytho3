use std::rc::Rc;

use rss::Item;
use rusqlite::Connection;

use crate::social::Network;

#[derive(Debug, PartialEq, Clone)] // TODO: Clone is only needed for the tests
pub struct SyndicatedPost {
    pub social_network: Network,
    pub id: String,
    pub original_guid: String,
    pub original_uri: String,
}

impl SyndicatedPost {
    pub fn new(social_network: Network, id: &str, item: &Item) -> Self {
        Self {
            social_network,
            id: String::from(id),
            original_guid: String::from(item.guid().unwrap().v