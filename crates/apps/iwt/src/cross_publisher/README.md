# Cross Publisher

Syndicates microblog posts to Twitter and Mastodon


## Development

Incremental builds using `cargo`:

```bash
$ nix develop -c cargo build -p iwt_cross_publisher
```

Running the application using cargo (incremental build)

```bash
$ nix develop -c cargo run -p iwt -- --config indieweb.toml cross-publish --help
```

Running the application using nix:

```bash
$ nix run .#iwt --config indieweb.toml cross-publish --help
```

