# MagicSpot agent guide

MagicSpot is a lightweight native Spotify desktop client derived from
Fastpotify. Keep Windows, macOS, and Linux builds working and preserve the
small native egui/librespot architecture.

- UI code belongs in `src/ui/`; apply emitted actions in `src/app.rs`.
- Keep network and playback work off the UI thread.
- Keep settings backward compatible and never log credentials.
- Prefer Spotify-backed, account-synced features. Avoid local-only folders,
  playback history, or library state that would make MagicSpot inconsistent
  with the user's mobile Spotify experience; keep local state to clearly
  installation-specific preferences, caches, and session restoration.
- Do not add a browser engine, telemetry, hosted backend, alternate Spotify
  audio source, DRM bypass, or unsupported lossless claims.
- Add focused tests for state and migration changes. Use the `demo` feature
  for deterministic UI work.
- Run formatting, tests, and strict Clippy before committing.
- Version releases by user-visible scope: fixes and visual corrections bump
  the patch number, a feature bumps the minor number, and a multi-feature or
  overhaul release bumps the major number. A mixed release takes its highest
  applicable bump.
- Read `docs/HANDOFF.md` before changing behavior or publishing a release.
- Hosted GitHub workflows are manual. Do not start them for routine iteration;
  work locally and run packaging workflows for requested releases.

Work on `main`. Keep the upstream Fastpotify remote so useful fixes can be
reviewed and integrated without losing project ancestry.
