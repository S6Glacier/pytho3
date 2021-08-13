
use std::rc::Rc;

use crate::commons::auth::token_db::SqliteTokenDB;
use crate::commons::url_shortener::ReqwestClient;
use crate::config::Config;
use mastodon::Mastodon;
use rusqlite::Connection;
use syndicated_post::SqliteSyndycatedPostStorage;
use target::Target;
use twitter::Twitter;

mod mastodon;
mod rss;
mod rss_item_ext;
mod syndicate;
mod syndicated_post;
mod target;
mod twitter;

pub async fn execute(config: &Config, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Rc::new(Connection::open(&config.db.path).expect("Couldn't open DB"));

    let token_db = Rc::new(SqliteTokenDB::new(Rc::clone(&conn)));

    let url_shortener_client = Rc::new(ReqwestClient::new(
        &config.url_shortener.protocol,
        &config.url_shortener.domain,
        config.url_shortener.put_base_uri.as_ref(),
    ));

    let targets: Vec<Box<dyn Target>> = vec![
        Box::new(Twitter::new(
            config.twitter.client_id.clone(),
            token_db,
            Rc::clone(&url_shortener_client),
        )),