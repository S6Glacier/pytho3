
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
        i += 1;
    }
    &text[0..len]
}

#[must_use]
pub fn shorten_with_permashort_citation(
    text: &str,
    limit: usize,
    permashort_citation: &PermashortCitation,
    tags: &[String],
) -> String {
    let hash_tags = tags
        .iter()
        .map(|tag| String::from("#") + &tag.to_case(Case::Pascal))
        .collect::<Vec<_>>()
        .join(" ");

    let (cleaned, short) = clean_description(text);

    let suffix = if short {
        format!("\n{hash_tags} {}", permashort_citation.to_uri())
    } else {
        format!("\n{hash_tags} ({})", permashort_citation.to_string())
    };

    let shortened = shorten(&cleaned, limit - suffix.len());

    if shortened == cleaned {
        let mut appended = cleaned;
        appended.push_str(&suffix);
        appended
    } else {