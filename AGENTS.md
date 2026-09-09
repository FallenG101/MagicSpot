# MagicSpot agent guide

MagicSpot is a lightweight native Spotify desktop client derived from
Fastpotify. Keep Windows, macOS, and Linux builds working and preserve the
small native egui/librespot architecture.

- UI code belongs in `src/ui/`; apply emitted actions in `src/app.rs`.
- Keep network and playback work off the UI thread.
- Keep settings backward compatible and never log credentials.
- Do not add a browser engine, telemetry, hosted backend, alternate Spotify
  audio source, DRM bypass, or unsupported lossless claims.
- Add focused tests for state and migration changes. Use the `demo` feature
  for deterministic UI work.
- Run formatting, tests, and strict Clippy before committing.

Work on `main`. Keep the upstream Fastpotify remote so useful fixes can be
reviewed and integrated without losing project ancestry.
