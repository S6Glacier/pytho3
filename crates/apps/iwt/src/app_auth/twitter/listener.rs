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

use super::Error;
use crate::config::Config;

struct State {
    challenge: String,
    oauth_state: String,
    client_id: String,
    shutdown_signal: Sender<()>,
    db_path: String,
}

pub async fn start(config: &Config, challenge: &str, csrf_state: &str) -> Result<(), Error> {
    // Create a channel to be able to shut down the webserver from the
    // Request handler after receiv