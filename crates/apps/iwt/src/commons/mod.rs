use std::{error::Error, fmt::Display};

pub mod auth;
pub mod permashort_link;
pub mod text;
pub mod url_shortener;

#[derive(Debug)]
pub struct SqlConversionError {
    pub message: String,
}

impl Display for SqlConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("SqlConversionError: {}", self.message))
    }
}

impl Error for SqlConversionError {}
