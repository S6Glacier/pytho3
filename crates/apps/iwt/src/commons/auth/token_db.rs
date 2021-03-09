
use std::rc::Rc;

use oauth2::{AccessToken, RefreshToken};
use rusqlite::Connection;

use crate::social::Network;

pub trait TokenDB {
    fn get_access_token(
        &self,
        social_network: &Network,
    ) -> Result<AccessToken, Box<dyn std::error::Error>>;
    fn get_refresh_token(
        &self,
        social_network: &Network,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>>;
    fn store(
        &self,
        social_network: &Network,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

pub struct SqliteTokenDB {
    conn: Rc<Connection>,
}

impl SqliteTokenDB {
    pub fn new(conn: Rc<Connection>) -> Self {
        Self { conn }
    }
}

impl TokenDB for SqliteTokenDB {
    fn get_access_token(
        &self,
        social_network: &Network,
    ) -> Result<AccessToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT access_token FROM auth_token WHERE social_network = :social_network",
                &[(":social_network", social_network.to_string().as_str())],
                |row| row.get("access_token").map(AccessToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn get_refresh_token(
        &self,
        social_network: &Network,
    ) -> Result<RefreshToken, Box<dyn std::error::Error>> {
        self.conn
            .query_row(
                "SELECT refresh_token FROM auth_token WHERE social_network = :social_network",
                &[(":social_network", social_network.to_string().as_str())],
                |row| row.get("refresh_token").map(RefreshToken::new),
            )
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
    }

    fn store(
        &self,
        social_network: &Network,
        access_token: &AccessToken,
        refresh_token: &RefreshToken,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.conn.execute(
            "INSERT INTO auth_token (social_network, access_token, refresh_token)
             VALUES (?1, ?2, ?3)
             ON CONFLICT (social_network) 