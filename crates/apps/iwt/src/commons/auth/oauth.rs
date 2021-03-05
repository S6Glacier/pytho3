
use std::rc::Rc;

use async_mutex::Mutex;

use crate::social::Network;

use super::token_db::TokenDB;
use oauth2::{
    basic::BasicClient, http::HeaderValue, reqwest::async_http_client, AccessToken, RefreshToken,
    TokenResponse,
};
use reqwest::{header::AUTHORIZATION, Client, Request, Response, StatusCode};

struct TokenCredentials {
    access_token: AccessToken,
    refresh_token: RefreshToken,
}

pub struct AuthedClient<DB: TokenDB> {
    oauth_client: BasicClient,
    db: Rc<DB>,
    social_network: Network,
    http_client: Client,
    // TODO: do we need this async mutex here? Couldn't we use TokenDB / sled directly?
    tokens: Mutex<TokenCredentials>,
}

impl<DB: TokenDB> AuthedClient<DB> {
    pub fn new(social_network: Network, oauth_client: BasicClient, db: Rc<DB>) -> Self {
        let access_token = db
            .get_access_token(&social_network)
            .expect("Couldn't load access token");
        let refresh_token = db
            .get_refresh_token(&social_network)
            .expect("Couldn't load refresh token");
        Self {
            oauth_client,
            db,
            social_network,
            http_client: reqwest::Client::new(),
            tokens: Mutex::new(TokenCredentials {
                access_token,
                refresh_token,
            }),
        }
    }

    pub async fn authed_request(
        &self,
        mut request: Request,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        {
            let tokens = self.tokens.lock().await;
            authorize_request(&mut request, &tokens);
        }

        let mut cloned_request = request.try_clone().expect("Request cannot be cloned");

        log::debug!("headers: {:?}", request.headers());
        let response = self.http_client.execute(request).await?;
        log::debug!("response from execue: {:?}", response);

        if response.status() == StatusCode::UNAUTHORIZED {
            log::debug!("recieved unauthorized response, refresing token");
            let mut tokens = self.tokens.lock().await;
            log::debug!("token credentials lock acquired");
            *tokens = self.exchange_refresh_token(&tokens).await?;

            authorize_request(&mut cloned_request, &tokens);
            log::debug!(
                "headers after token refresh: {:?}",
                cloned_request.headers()
            );
            self.http_client
                .execute(cloned_request)
                .await
                .map(|res| {
                    log::debug!("response: {:?}", res);
                    res
                })