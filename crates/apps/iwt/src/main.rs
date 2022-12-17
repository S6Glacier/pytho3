use std::fmt::Display;

use clap::Parser;
use clap::Subcommand;

use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

mod app_auth;
pub mod commons;
pub mod config;
mod cross_publisher;
pub mod social;

use config::Config;

#[derive(Debug)]
pub struct IwtError {
    message: String,
}

impl IwtError {
    #[must_use]
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl Display for IwtError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("IwtError: {}", self.message))
    }
}

impl std::error::Error for IwtError {}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
    /// Print debug information, including secrets!
    #[clap(short, long, action)]
    debug: bool,
    /// Path to the config file
    #[clap(long, value_parser, default_value_t = String::from("config.toml"))]
    config: String,
}

#[derive(Subcommand)]
enum Command {
    /// App Authentication helper
    AppAuth {
        #[clap(subcommand)]
        sub_command: app_a