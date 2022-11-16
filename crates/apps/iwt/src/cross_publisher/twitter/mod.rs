
use std::rc::Rc;

use crate::commons::permashort_link::PermashortCitation;
use crate::commons::text;
use crate::IwtError;
use async_trait::async_trait;

use futures::TryFutureExt;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
use reqwest::Client;
use rss::Item;

use super::rss_item_ext::IwtRssExtension;
use super::syndicated_post::SyndicatedPost;
use super::target::Target;
use crate::commons::auth::oauth::AuthedClient;
use crate::commons::auth::token_db::TokenDB;
use crate::commons::url_shortener;
use crate::social::Network;

pub struct Twitter<DB: TokenDB, USClient: url_shortener::Client> {
    authed_client: AuthedClient<DB>,
    http_client: Client,
    url_shortener_client: Rc<USClient>,
}

impl<DB: TokenDB, USClient: url_shortener::Client> Twitter<DB, USClient> {
    pub fn new(client_id: ClientId, db: Rc<DB>, url_shortener_client: Rc<USClient>) -> Self {
        Self {
            authed_client: AuthedClient::new(
                Network::Twitter,
                BasicClient::new(
                    client_id,
                    None,
                    AuthUrl::new("https://twitter.com/i/oauth2/authorize".to_string())
                        .expect("Twitter auth url is invalid."),
                    Some(
                        TokenUrl::new("https://api.twitter.com/2/oauth2/token".to_string())
                            .expect("Twitter token url is invalid"),
                    ),
                ),
                db,
            ),
            http_client: Client::new(),
            url_shortener_client,
        }
    }
}

#[derive(serde::Serialize)]
struct TweetsRequest {
    text: String,
}

#[derive(serde::Deserialize)]
struct TweetResponse {
    data: TweetResponseData,
}

#[derive(serde::Deserialize)]
struct TweetResponseData {
    id: String,
}