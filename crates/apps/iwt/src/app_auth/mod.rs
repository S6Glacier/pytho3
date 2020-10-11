
use crate::config::Config;

use clap::Subcommand;

mod twitter;

#[derive(Subcommand)]
pub enum AuthSubcommand {
    /// Twitter Oauth flow
    Twitter,
    /// Mastodon Oauth flow
    Mastodon,
}

pub async fn execute(
    command: AuthSubcommand,
    config: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AuthSubcommand::Twitter => twitter::start_flow(config)
            .await
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error>),
        AuthSubcommand::Mastodon => todo!(),
    }
}