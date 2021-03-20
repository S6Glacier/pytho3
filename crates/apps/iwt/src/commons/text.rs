
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
    let cleaned = fragment
        .select(&Selector::parse("html").unwrap())
        .next()
        .unwrap()
        .text()
        .collect::<Vec<_>>()
        .join("");

    log::debug!("cleaned desc:\n{}\n", cleaned);

    (cleaned, shortened)
}

#[cfg(test)]
mod test {
    use crate::commons::permashort_link::PermashortCitation;

    use super::{shorten, shorten_with_permashort_citation};

    #[test]
    fn test_short_returns_same_if_short() {
        let short_text = "This is some text.";
        assert_eq!(shorten(short_text, 100), short_text);
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_on_dot() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 18), "This is some");
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_after_dot() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 19), "This is some text.");
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_with_ellipsis() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 21), "This is some text.");
    }

    #[test]
    fn test_shorten_returns_shortened_sentence_limit_with_ellipsis_longer() {
        let text = "This is some text. Looooong word.";
        assert_eq!(shorten(text, 23), "This is some text.");
    }

    #[test]
    fn test_shorten_with_permashort_citation_should_add_hashtags() {
        let short_text = "This is some text.";
        let permashort_citation = PermashortCitation::new(
            "http".to_string(),
            "localhost".to_string(),
            "asdf".to_string(),
        );
        assert_eq!(
            shorten_with_permashort_citation(
                short_text,
                100,