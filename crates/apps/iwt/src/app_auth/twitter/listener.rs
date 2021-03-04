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

    let auth_code = params.get("code").expect("auth code param not found");
    log::debug!("Got auth code, exchanging for access token");
    log::debug!("auth_code is {}", auth_code);

    let challenge = state.challenge.to_string();
    let params = [
        ("code", auth_code.as_str()),
        ("grant_type", "authorization_code"),
        ("client_id", state.client_id.as_str()),
        ("code_verifier", challenge.as_str()),
        ("redirect_uri", "http://127.0.0.1:6009"),
    ];

    // Exchange the auth code to an access_token and a refresh_token
    let client = reqwest::Client::new();
    let result = client
        .post("https://api.twitter.com/2/oauth2/token")
        .form(&params)
        .send()
        .await
        .expect("Oauth request failed");

    let json = result.text().await.expect("Couldn't get response body");
    log::debug!("json: {}", json);
    let tokens =
        serde_json::from_str::<TokenResponse>(&json).expect("Coulnd't decode json response");

    println!(
        "
token_type: {}
access_token: {}
refresh_token: {}
",
        tokens.token_type, tokens.access_token, tokens.refresh_token
    );

    // TODO: add argument to be able to disable updating the db
    // if let Some(db_path) = state.db_path.clone() {
    persist_tokens(&tokens, &state.db_path).expect("couldn't persist tokens");
    // }

    // Send the shut down signal
    state.shutdown_signal.send(()).await.unwrap();

    Html("<h1>Hello from twitter-auth</h1><p>Your tokens are displayed on the standard output.</p>")
}

fn persist_tokens(tokens: &TokenResponse, db_path: &String) -> rusqlite::Result<()> {
    // Initialize db to store tokens
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS auth_token (
            social_network VARCHAR(20) PRIMARY KEY,
            access_token   TEXT,
            refresh_token  TEXT
        )
        ",
        (),
    )?;

    conn.execute(
        "INSERT INTO auth_token (social_network, access_token, refresh_token)
         VALUES (?1, ?2, ?3)
         ON CONFLICT (social_network) 
            DO UPDATE SET access_token = excluded.access_token, refresh_token = excluded.refresh_token",
        (Twitter.to_string().as_str(), tokens.access_token.clone(), tokens.refresh_token.clone())
    )?;

    Ok(())
}
