
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
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error>)
        } else {
            Ok(response)
        }
    }

    async fn exchange_refresh_token(
        &self,
        tokens: &TokenCredentials,
    ) -> Result<TokenCredentials, Box<dyn std::error::Error>> {
        log::debug!("exchanging refresh_token...");

        let response = self
            .oauth_client
            .exchange_refresh_token(&tokens.refresh_token)
            .request_async(async_http_client)
            .await;

        match response {
            Err(error) => {
                log::error!("token refresh failed: {:?}", error);
                Err(Box::new(error) as Box<dyn std::error::Error>)
            }
            Ok(token_response) => {
                let tokens = TokenCredentials {
                    access_token: token_response.access_token().clone(),
                    refresh_token: token_response
                        .refresh_token()
                        .unwrap_or(&tokens.refresh_token)
                        .clone(),
                };

                log::debug!(
                    "access token refresh successful, new token: {}",
                    tokens.access_token.secret()
                );

                self.db
                    .store(
                        &self.social_network,
                        &tokens.access_token,
                        &tokens.refresh_token,
                    )
                    .map(|_| tokens)
            }
        }
    }
}

fn authorize_request(request: &mut Request, tokens: &TokenCredentials) {
    request.headers_mut().remove(AUTHORIZATION);
    request.headers_mut().append(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", tokens.access_token.secret())).unwrap(),
    );
}

#[cfg(test)]
mod test {
    use std::rc::Rc;

    use crate::commons::auth::token_db::TokenDB;
    use crate::social::Network;
    use oauth2::{basic::BasicClient, AuthUrl, ClientId, TokenUrl};
    use reqwest::{Method, Request, StatusCode, Url};
    use wiremock::{
        http::HeaderName,
        matchers::{headers, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::stubs::auth::token_db::stubs::StubTokenDB;

    use crate::commons::auth::oauth::AuthedClient;

    fn basic_client(base_url: &str) -> BasicClient {
        BasicClient::new(
            ClientId::new(String::from("some-client-id")),
            None,
            AuthUrl::new(format!("{base_url}/oauth/2")).unwrap(),
            Some(TokenUrl::new(format!("{base_url}/oauth/token")).unwrap()),
        )
    }

    fn create_authed_client(base_url: &str) -> (Rc<impl TokenDB>, AuthedClient<impl TokenDB>) {
        let db = StubTokenDB::new();
        let shared_db = Rc::new(db);
        (
            Rc::clone(&shared_db),
            AuthedClient::new(Network::Twitter, basic_client(base_url), shared_db),
        )
    }

    #[tokio::test]
    async fn test_token_is_not_refreshed_if_response_is_not_401() {
        let mock_server = MockServer::start().await;

        let (_, authed_client) = create_authed_client(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let request = Request::new(
            Method::GET,
            Url::parse(format!("{}/restricted", mock_server.uri()).as_str()).unwrap(),
        );

        let result = authed_client.authed_request(request).await;
        // There is a response
        assert!(result.is_ok(), "{result:?}");

        // Response is expected
        assert_eq!(result.unwrap().status(), StatusCode::OK);

        let requests = mock_server
            .received_requests()
            .await
            .expect("Requests expected");

        // There was exactly 1 requests
        assert_eq!(requests.len(), 1);

        // The first request was to GET the /restricted url
        assert_eq!(requests[0].url.path(), "/restricted");
        assert_eq!(requests[0].method, wiremock::http::Method::Get);
    }

    #[tokio::test]
    async fn test_token_is_refreshed_if_response_is_401() {
        let mock_server = MockServer::start().await;

        let (db, authed_client) = create_authed_client(&mock_server.uri());

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers(
                "Authorization",
                vec!["Bearer initial-access-token"],
            ))
            .respond_with(ResponseTemplate::new(401))
            .mount(&mock_server)
            .await;

        Mock::given(method("GET"))
            .and(path("/restricted"))
            .and(headers("Authorization", vec!["Bearer new-access-token"]))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/oauth/token"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(
                    r#"
            {
                "token_type":"bearer",
                "expires_in":7200,
                "access_token":"new-access-token",
                "scope":"some-scope",
                "refresh_token":"new-refresh-token"
            }
            "#
                    .as_bytes()
                    .to_owned(),
                    "application/json",
                ),
            )
            .mount(&mock_server)
            .await;

        let request = Request::new(
            Method::GET,
            Url::parse(format!("{}/restricted", mock_server.uri()).as_str()).unwrap(),
        );

        let result = authed_client.authed_request(request).await;
        // There is a response
        assert!(result.is_ok(), "{result:?}");

        // Response is expected