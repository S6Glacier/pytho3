use std::fmt::Display;

use async_trait::async_trait;
use reqwest;

use super::permashort_link::PermashortCitation;

#[derive(Debug)]
pub struct ClientError {
    pub message: Str