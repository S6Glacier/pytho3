use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use crate::social::Network::Twitter;
use axum::{
    extract::Query,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use rusqlite::Connection;
use serde_derive::Deserialize;
use tokio::sync::mpsc::Sender;

use super