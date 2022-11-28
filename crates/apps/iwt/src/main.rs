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
