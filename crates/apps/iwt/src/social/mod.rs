
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