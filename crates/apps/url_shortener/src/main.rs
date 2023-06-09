
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

    let sock_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), http_port);
    let app = Router::new()
        .route("/u/:url", put(add_url))
        .route("/u/:url", get(get_short_url))
        .route("/s/:short", get(redirect))
        // shate the state with the request handler
        .layer(Extension(Arc::new(state)));

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

async fn add_url(
    Path(url): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> impl IntoResponse {
    state
        .db_conn
        .call(move |conn| {
            if let Some(short) = find_short(&url, conn).unwrap() {
                (StatusCode::OK, short)
            } else {
                let short = gen_unique_short(conn);

                persist(&url, &short, conn).unwrap();

                (StatusCode::CREATED, short)
            }
        })
        .await
}

async fn get_short_url(
    Path(url): Path<String>,
    Extension(state): Extension<Arc<State>>,
) -> Result<String, StatusCode> {
    state
        .db_conn
        .call(move |conn| {
            if let Some(short) = find_url(&url, conn).unwrap() {
                Ok(short)
            } else {
                Err(StatusCode::NOT_FOUND)
            }
        })
        .await
}

async fn redirect(Path(short): Path<String>, Extension(state): Extension<Arc<State>>) -> Response {
    state
        .db_conn
        .call(move |conn| {
            if let Some(url) = find_url(&short, conn).unwrap() {
                Redirect::permanent(url.as_str()).into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        })
        .await
}

fn gen_unique_short(conn: &rusqlite::Connection) -> String {
    let mut short = gen_short();

    while find_url(&short, conn).unwrap().is_some() {
        short = gen_short();
    }

    short
}

fn gen_short() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(4)
        .map(char::from)
        .collect()
}

fn find_short(url: &str, conn: &rusqlite::Connection) -> rusqlite::Result<Option<String>> {
    let mut statement = conn.prepare("SELECT short FROM permashortlink WHERE url = :url")?;

    statement
        .query_row(&[(":url", url)], |row| row.get(0))
        .optional()
}

fn find_url(short: &str, conn: &rusqlite::Connection) -> rusqlite::Result<Option<String>> {
    let mut statement = conn.prepare("SELECT url FROM permashortlink WHERE short = :short")?;

    statement
        .query_row(&[(":short", short)], |row| row.get(0))
        .optional()
}

fn persist(url: &String, short: &String, conn: &rusqlite::Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO permashortlink (url, short) VALUES (?, ?)",
        [url, short],
    )
}