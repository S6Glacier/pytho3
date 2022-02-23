use std::rc::Rc;

use rss::Item;
use rusqlite::Connection;

use crate::social::Network;

#[derive(Debug, PartialEq, Clone)] // TODO: Clone is only needed for the tests
pub struct SyndicatedPost {
    pub social_network: Network,
    pu