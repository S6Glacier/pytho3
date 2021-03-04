
use std::fmt::Display;

use crate::config::Config;

use rand::{rngs::OsRng, RngCore};

mod listener;

#[derive(Debug)]
pub enum Error {
    ListenerError(),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ListenerError")
    }
}

impl std::error::Error for Error {}

pub async fn start_flow(config: &Config) -> Result<(), Error> {
    // Create CSRF state and secret challenge
    let mut challenge = [0u8; 64];
    let mut csrf_state = [0u8; 64];

    OsRng.fill_bytes(&mut challenge);
    OsRng.fill_bytes(&mut csrf_state);

    let challenge = base64::encode(challenge);
    let csrf_state = base64::encode(csrf_state);

    let oauth_uri = construct_uri(&config.twitter.client_id, &csrf_state, &challenge);
    println!(
        "Open the following link in your browser:

{}
",
        oauth_uri
    );

    listener::start(config, &challenge, &csrf_state).await
}

fn construct_uri(client_id: &str, csrf_state: &str, challenge: &str) -> String {
    let query = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("response_type", "code")
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", "http://127.0.0.1:6009")
        .append_pair("scope", "tweet.read tweet.write users.read offline.access")
        .append_pair("state", csrf_state)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "plain")
        .finish();

    // Construct URI that starts the Oauth flow
    format!("https://twitter.com/i/oauth2/authorize?{query}")
}