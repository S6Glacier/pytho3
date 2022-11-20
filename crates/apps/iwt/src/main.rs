use std::fmt::Display;

use clap::Parser;
use clap::Subcommand;

use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

mod app_auth;
pub mod commons;
pub mod config;
m