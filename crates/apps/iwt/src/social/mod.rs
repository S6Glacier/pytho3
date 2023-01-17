
use std::fmt::Display;

use rusqlite::types::{FromSql, FromSqlError};

use crate::commons::SqlConversionError;

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Network {
    Twitter,
    Mastodon,
}

impl Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Network::Twitter => write!(f, "twitter"),
            Network::Mastodon => write!(f, "mastodon"),
        }
    }
}

impl FromSql for Network {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().and_then(|n| match n {
            "twitter" => Ok(Network::Twitter),
            "mastodon" => Ok(Network::Mastodon),
            n => Err(FromSqlError::Other(Box::new(SqlConversionError {
                message: format!("Unknown social network: {n}"),
            }))),
        })
    }
}