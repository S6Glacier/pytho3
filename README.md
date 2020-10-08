
# indieweb-tools

My collection of indieweb tools

## Components

- [indieweb tools](crates/apps/iwt), `iwt` commands:
  - [app-auth](crates/libraries/app_auth): Oauth2 app authentication helper
  - [cross-publish](crates/libraries/cross_publisher): Microblog syndication to Twitter and Mastodon
  
- [url shortener](crates/apps/url_shortener)

## Basic usage

1) Create a config file, i.e. `indieweb.toml`:

```toml
[rss]
urls = [ "http://example.com/rss.xml" ]

[db]
path = "indieweb.db"

[twitter]
# only the client id is required here, access and resfresh tokens should be stored in the db so that
# they can be updated
client_id = "your_client_id..."

[mastodon]
base_uri = "http://your-mastodon-instance.example.com"
access_token = "your_access_token..."

[url_shortener]
protocol = "https"
domain = "short.domain"
```

2) Get Twitter auth tokens:

```bash
$ nix run .#iwt -- --config indieweb.toml app-auth twitter
```

3) Syndicate posts to Twitter and Mastodon

```bash
$ nix run .#iwt -- --config indieweb.toml cross-publish
```