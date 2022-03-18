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
            original_guid: String::from(item.guid().unwrap().value()),
            original_uri: String::from(item.link().unwrap()),
        }
    }
}

#[derive(Debug)]
pub enum StorageError {
    PersistenceError(String),
    SqlError(rusqlite::Error),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StorageError")
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(e: rusqlite::Error) -> Self {
        StorageError::SqlError(e)
    }
}

impl std::error::Error for StorageError {}

pub trait Storage {
    fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), StorageError>;
    fn find(
        &self,
        original_guid: &str,
        social_network: &Network,
    ) -> Result<Option<SyndicatedPost>, StorageError>;
}

pub struct SqliteSyndycatedPostStorage {
    conn: Rc<Connection>,
}

impl SqliteSyndycatedPostStorage {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }

    pub fn init_table(&self) -> Result<(), StorageError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS post (
              id VARCHAR(64) NOT NULL,
              social_network VARCHAR(20) NOT NULL,
              original_guid TEXT NOT NULL,
              original_uri TEXT NOT NULL,
            
              PRIMARY KEY (id, social_network)
            )",
                (),
            )
            .map(|_| ())
            .map_err(|err| StorageError::PersistenceError(format!("{err:?}")))
    }
}

impl Storage for SqliteSyndycatedPostStorage {
    fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), StorageError> {
        self.conn
  