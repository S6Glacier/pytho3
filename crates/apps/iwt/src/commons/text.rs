
use super::permashort_link::PermashortCitation;
use convert_case::{Case, Casing};
use regex::Regex;
use scraper::{Html, Selector};

#[must_use]
pub fn shorten(text: &str, limit: usize) -> &str {
    let words = words(text);
    let mut len = 0;
    let mut i = 0;

    while i < words.len() && (len + words[i].len() + 1) < limit {
        len += words[i].len() + usize::from(i != 0);