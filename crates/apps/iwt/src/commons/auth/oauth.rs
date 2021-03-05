
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