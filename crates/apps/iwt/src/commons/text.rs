
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
        let shortened = shorten(
            &cleaned,
            limit - 23 - 4 - hash_tags.len() - 2, /* Link + space + ellipsis + quuotes + hastags + space around hash_tags*/
        );

        format!(
            "\"{}…\"\n{} {}",
            shortened,
            hash_tags,
            permashort_citation.to_uri()
        )
    }
}

fn words(input: &str) -> Vec<&str> {
    input.split(' ').collect()
}

#[must_use]
pub fn clean_description(description: &str) -> (String, bool) {
    let re = Regex::new(r"<h[1-6] ").unwrap();

    let summary: String = re.split(description).next().unwrap().to_string();

    let shortened = summary != description;

    let str = summary
        .replace("<li>", "<li>- ")
        .replace("<code>", "`")
        .replace("</code>", "`")
        .replace('\n', " ")
        .replace("</p> ", "\n\n");

    log::debug!("original desc:\n{}\n", str);

    let fragment = Html::parse_document(&format!("<html>{}</html>", &str));