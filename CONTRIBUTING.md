# Contributing to MagicSpot

MagicSpot is a feature-focused fork of
[Fastpotify](https://github.com/crmne/fastpotify). Contributions should keep
the app lightweight, native, and usable on Windows, macOS, and Linux.

Before opening a pull request, run:

```sh
cargo fmt --all --check
cargo test --locked --no-default-features --features demo --all-targets
cargo clippy --locked --no-default-features --features demo --all-targets -- -D warnings
```

Keep changes focused and explain their user-visible effect. Do not include
credentials, generated build output, Spotify audio from alternate sources,
DRM bypasses, telemetry, or an embedded browser engine.

Playback depends on Spotify Premium and librespot. Lossless playback remains
out of scope until it is lawfully supported by the upstream playback stack.
