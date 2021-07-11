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

#[async_trait(?Send)]
impl<WHClient: url_shortener::Client> Target for Mastodon<WHClient> {
    async fn publish<'a>(
        &self,
        post: &Item,
        extension: &IwtRssExtension,
    ) -> Result<SyndicatedPost, Box<dyn std::error::Error + 'a>> {
        log::debug!("processing post: {:?},\nextension: {:?}", post, extension);

        let permashort_citation = self
            .url_shortener_client
            .put_uri(post.link.as_ref().unwrap())
            .await?;

        let status = text::shorten_with_permashort_citation(
            post.description().unwrap(),
            500,
            &permashort_citation,
            &extension.tags,
        );

        self.http_client
            // TODO: make mastodon instan