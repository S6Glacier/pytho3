use std::rc::Rc;

use super::rss_item_ext::IwtRssExtension;
use super::syndicated_post::SyndicatedPost;
use super::target::Target;
use crate::commons::{text, url_shortener};
use crate::social::Network;
use async_trait::async_trait;
use futures::TryFutureExt;
use oauth2::AccessToken;
use reqwest::Client;
use rss::Item;

pub struct Mastodon<USClient: url_shortener::Client> {
    base_uri: String,
    access_token: AccessToken,
    http_client: Client,
    url_shortener_client: Rc<USClient>,
}

impl<USClient: url_shortener::Client> Mastodon<USClient> {
    pub fn new(
        base_uri: String,
        access_token: AccessToken,
        url_shortener_client: Rc<USClient>,
    ) -> Self {
        Self {
            base_uri,
            access_token,
            http_client: Client::new(),
            url_shortener_client,
        }
    }
}

#[derive(serde::Serialize)]
struct UpdateStatusRequest {
    status: String,
    spoiler_text: Option<String>,
}

#[derive(serde::Deserialize)]
struct MastodonResponse {
    id: String,
}

#[async_tr