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

Prefer features that use Spotify account data and remain consistent with the
user's mobile and other Spotify clients. Avoid local-only folders, listening
history, or library structures that could be mistaken for synced Spotify data.
Installation-specific preferences, caches, and session restoration remain
appropriate when the interface makes their local scope clear.

Playback depends on Spotify Premium and librespot. Lossless playback remains
out of scope until it is lawfully supported by the upstream playback stack.

## Versioning

MagicSpot versions describe the user-visible scope of a release:

- Bug fixes and visual corrections increment the patch number (`x.y.Z`).
- A feature release increments the minor number (`x.Y.0`).
- A release containing multiple features or a broad overhaul increments the
  major number (`X.0.0`).

When a release mixes categories, use the highest applicable increment.
