
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