
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