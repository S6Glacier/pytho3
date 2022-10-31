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
            .execute(
                "INSERT INTO post (id, social_network, original_guid, original_uri) 
                 VALUES (:id, :social_network, :original_guid, :original_url)",
                &[
                    (":id", &syndicated_post.id),
                    (
                        ":social_network",
                        &syndicated_post.social_network.to_string(),
                    ),
                    (":original_guid", &syndicated_post.original_guid),
                    (":original_url", &syndicated_post.original_uri),
                ],
            )
            .map(|_| ())
            .map_err(|err| StorageError::PersistenceError(format!("{err:?}")))
    }

    fn find(
        &self,
        original_guid: &str,
        social_network: &Network,
    ) -> Result<Option<SyndicatedPost>, StorageError> {
        let mut statement = self.conn.prepare(
            "SELECT id, social_network, original_guid, original_uri FROM post
            WHERE original_guid = :original_guid AND social_network = :social_network",
        )?;

        statement
            .query_map(
                &[
                    (":original_guid", original_guid),
                    (":social_network", social_network.to_string().as_str()),
                ],
                |row| {
                    Ok(SyndicatedPost {
                        id: row.get(0).unwrap(),
                        social_network: row.get(1).unwrap(),
                        original_guid: row.get(2).unwrap(),
                        original_uri: row.get(3).unwrap(),
                    })
                },
            )
            .map(|iter| {
                // TODO: this needs some clean up
                iter.map(Result::unwrap)
                    .collect::<Vec<_>>()
                    .first()
                    .map(|r| (*r).clone())
            })
            .map_err(|_| StorageError::PersistenceError(String::from("foo"))) // TODO: this needs some clean up
    }
}

#[cfg(test)]
pub mod stubs {
    use std::sync::Mutex;

    use crate::social::Network;

    use super::{Storage, SyndicatedPost};

    #[derive(Default)]
    pub struct SyndicatedPostStorageStub {
        pub posts: Mutex<Vec<SyndicatedPost>>,
    }

    impl Storage for SyndicatedPostStorageStub {
        fn store(&self, syndicated_post: SyndicatedPost) -> Result<(), super::StorageError> {
            let mut posts = self.posts.lock().unwrap();
            posts.push(syndicated_post);

            Ok(())
        }

        fn find(
            &self,
            original_guid: &str,
            social_network: &Network,
        ) -> Result<Option<SyndicatedPost>, super::StorageError> {
            let posts = self.posts.lock().unwrap();

            Ok(posts
                .iter()
                .find(|p| p.original_guid == *original_guid && p.social_network == *social_network)
                .map(|p| (*p).clone()))
        }
    }
}
