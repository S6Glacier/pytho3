
use std::fs;

use oauth2::{AccessToken, ClientId};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    pub rss: Rss,
    pub db: DB,
    pub twitter: Twitter,
    pub mastodon: Mastodon,
    pub url_shortener: UrlShortener,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Rss {
    pub urls: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct DB {
    pub path: String,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Twitter {
    pub client_id: ClientId,
}

#[derive(Debug, Deserialize)]
pub struct Mastodon {
    pub base_uri: String,
    pub access_token: AccessToken,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct UrlShortener {
    pub protocol: String,
    pub domain: String,
    pub put_base_uri: Option<String>,
}

impl PartialEq for Mastodon {
    fn eq(&self, other: &Self) -> bool {
        self.base_uri == other.base_uri && self.access_token.secret() == other.access_token.secret()
    }
}

impl Config {
    pub fn from_file(file_name: &str) -> Result<Config, toml::de::Error> {
        let config_str = fs::read_to_string(file_name)
            .unwrap_or_else(|_| panic!("Cannot found file: {file_name}"));

        toml::from_str(&config_str)
    }
}

#[cfg(test)]
mod test {
    use oauth2::AccessToken;
    use oauth2::ClientId;

    use super::Config;
    use super::Mastodon;
    use super::Rss;
    use super::Twitter;
    use super::UrlShortener;
    use super::DB;

    #[test]
    fn config_model_should_be_deserializable() {
        let config = r#"
        [rss]
        urls = [
          "http://exmample.com/rss.xml",
          "http://exmample.com/some-site/rss.xml"
        ]
        [db]
        path = "some/path"
        [twitter]
        client_id = "some_client_id"
        [mastodon]
        base_uri = "https://mastodon.social"
        access_token = "some-access-token"
        [url_shortener]
        protocol = "http"
        domain = "localhost:9000"
        "#;

        assert_eq!(
            toml::from_str::<Config>(config),
            Ok(Config {
                rss: Rss {
                    urls: vec![
                        "http://exmample.com/rss.xml".to_string(),
                        "http://exmample.com/some-site/rss.xml".to_string()
                    ]
                },
                db: DB {
                    path: String::from("some/path")
                },
                twitter: Twitter {
                    client_id: ClientId::new(String::from("some_client_id"))
                },
                mastodon: Mastodon {
                    base_uri: String::from("https://mastodon.social"),
                    access_token: AccessToken::new(String::from("some-access-token"))
                },
                url_shortener: UrlShortener {
                    protocol: String::from("http"),
                    domain: String::from("localhost:9000"),
                    put_base_uri: None,
                }
            })
        );
    }
}