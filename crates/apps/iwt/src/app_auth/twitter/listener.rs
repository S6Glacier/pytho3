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
    // Request handler after receiving the auth code
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(10);

    // Initialise the shared state
    let state = Arc::new(State {
        challenge: challenge.to_string(),
        oauth_state: csrf_state.to_string(),
        client_id: config.twitter.client_id.to_string(),
        shutdown_signal: tx,
        db_path: config.db.path.clone(),
    });

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6009);
    let app = Router::new()
        .route("/", get(receive_token))
        // shate the state with the request handler
        .layer(Extension(state));

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        // gracefuly shut down the server when we receive a message on the
        // previously created channel
        .with_graceful_shutdown(async { rx.recv().await.unwrap() })
        .await
        .map_err(|_| Error::ListenerError())
}

#[derive(Deserialize)]
struct TokenResponse {
    token_type: String,
    access_token: String,
    refresh_token: String,
}

async fn receive_token(
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    let state_param = params.get("state").expect("state param not found");

    assert!(
        state_param != &state.oauth_state,
        "Invalid state param,
expected: {}
got     : {}",
        state.oauth_state,
        state_param
    );

    let auth_code = params.get("code").