
use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, put},
    Extension, Router,
};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rusqlite::OptionalExtension;
use tokio_rusqlite::Connection;

#[derive(Clone)]
struct State {
    db_conn: Connection,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path =
        env::var("IWT_URL_SHORTENER_DB_PATH").expect("IWT_URL_SHORTENER_DB_PATH must be set.");
    let http_port = env::var("IWT_URL_SHORTENER_HTTP_PORT")
        .expect("IWT_URL_SHORTENER_HTTP_PORT must be set.")
        .parse()
        .expect("IWT_URL_SHORTENER_HTTP_PORT cannot be parsed as u16");
    let db_conn = Connection::open(db_path).await.unwrap();
    db_conn
        .call(|conn| {
            conn.execute(
                "
        CREATE TABLE IF NOT EXISTS permashortlink (
            url   TEXT PRIMARY KEY,
            short VARCHAR(5)
        )
        ",
                (),
            )
        })
        .await
        .unwrap();

    let state = State { db_conn };
