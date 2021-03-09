
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
