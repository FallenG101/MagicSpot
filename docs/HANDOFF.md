# MagicSpot maintainer handoff

Updated 2026-09-11. This is the starting point for a new maintainer, coding
agent, or chat that does not have the project's conversation history.

## Current baseline

- The working branch is `main`; `origin` is
  `https://github.com/FallenG101/MagicSpot.git`.
- Fastpotify remains configured as the `upstream` remote at
  `https://github.com/crmne/fastpotify.git`.
- The latest user release is **v0.8.0**. It contains the collection, lyrics,
  queue, and responsive navigation improvements documented in the changelog,
  with Windows and universal macOS downloads.
- Main may contain documentation or development commits newer than the latest
  release tag. Do not bump or tag a new version for routine changes.
- The repository is public. Standard GitHub-hosted runners are therefore free,
  but all workflows remain intentionally manual so cross-platform checks and
  packaging happen at deliberate milestones. Continue development with local
  checks and batch changes into milestone releases.

Start every session with:

```sh
git status --short
git log -5 --oneline --decorate
git remote -v
```

Do not discard an existing working tree. Inspect and preserve work already in
progress before editing.

## Product direction

MagicSpot is a desktop-only Spotify client for Windows, macOS, and Linux. Its
main goals are low resource use, fast native interaction, a polished layout,
and a better lyrics experience. The current shell takes layout inspiration
from Juxtopposed's Spotify redesign concepts while retaining its own visuals.

Keep the Rust, egui, and librespot architecture. Do not add an embedded browser
engine, telemetry, a hosted MagicSpot backend, alternate audio sources, DRM
bypasses, or unsupported lossless claims. Lossless playback is deferred until
the legitimate Spotify/librespot stack supports it. VSCodium is the preferred
editor, although no editor-specific project files are required.

The repository is public, while the product remains an early personal,
feature-rich fork. Keep broadly useful fixes separable for possible upstream
contribution.

## Current interface decisions

- The desktop shell uses a full-height library rail, top navigation, rounded
  inset content, and a contained bottom player bar.
- Playlist and album pages use a rounded collection stage with metadata on the
  left and large artwork on the right. It stacks vertically below 700 points;
  playlist filtering moves below the action row below 620 points.
- Lyrics use a resizable right panel. Drag its left edge to change size; there
  is no expand button or separate full-screen mode.
- The lyrics panel uses one edge-to-edge blurred-art canvas inside its rounded
  frame. Its compact, unframed album header and scrolling text deliberately
  share that surface; do not restore the separate opaque album card.
- Synced lines follow and center the active lyric. Manual scrolling disables
  following until **Follow** is selected or a line is clicked.
- The list starts with a fixed 16-point inset. Do not restore a viewport-sized
  spacer above the first line; it creates the large blank area fixed in v0.7.4.
- Lyric size, line spacing, left/center alignment, Inter/Manrope/Lora faces,
  optional glow, and blurred-art visibility live in the collapsed Lyrics
  appearance menu in Appearance settings.
- Lyrics scroll state is keyed to the playing track. Preserve this when
  changing the follow logic so one song cannot inherit another song's offset.
- Estimated word-by-word progress is labelled **Beta** and defaults to off. It
  interpolates between line timestamps; it is not true per-word timing.
- Appearance choices are Dark, Light, Follow system, and OLED. Accent choices
  use circular Aqua, Violet, Rose, Amber, and Neutral gray swatches. Album-art
  color is optional; OLED and Neutral remain tintable.
- Blurred lyric backdrops use a 128-pixel derived PNG made off the UI thread and
  cached through `ArtLoader`; do not blur full-size artwork every frame.
- The bottom player bar uses the selected theme surface rather than its own
  album tint.
- Queue sections distinguish the playing row, manually queued rows, and the
  current context. Manual rows can be moved or removed only when the local
  MagicSpot player is active; this is implemented by clearing and rebuilding
  librespot's manual queue while preserving context rows.
- The top bar switches to compact navigation below 780 points and an icon-only
  search below 560 points. Account-menu entries keep hidden utilities reachable.
- Back/Forward hints name their destination. Alt+arrow and extra mouse-button
  navigation remain supported, and Escape closes the outermost open panel.

The main UI files are `src/ui/mod.rs`, `src/ui/topbar.rs`,
`src/ui/sidebar.rs`, `src/ui/player_bar.rs`, `src/ui/collection.rs`, and
`src/ui/lyrics.rs`. Shared palettes, type, dimensions, and icons live in
`src/theme.rs` and `src/ui/widgets.rs`.

## Architecture

- `src/main.rs` creates the native window, parses links and control commands,
  and supports deterministic demo screenshots behind the `demo` feature.
- `src/app.rs` owns UI-visible state. Views emit `Action` values, and the app
  applies them centrally.
- `src/backend.rs` owns the Tokio runtime. Spotify API, authentication,
  playback, lyrics, and image work must stay off the egui thread.
- `src/api/` handles Spotify Web API models, clients, and shared/personal app
  routing.
- `src/player.rs`, `src/sink.rs`, `src/resample.rs`, `src/eq.rs`, and
  `src/limiter.rs` implement local playback and audio processing.
- True crossfade is not implemented. The current librespot fork exposes one
  decoder stream and gives its sink no track-boundary callback during gapless
  playback. Crossfade requires a coordinated player change that overlaps two
  decoded tracks; do not label the existing 10 ms skip envelope as crossfade.
- `src/lyrics.rs` fetches and parses lyrics; `src/ui/lyrics.rs` presents them.
- `src/settings.rs` and `src/paths.rs` define persistent formats and platform
  storage. Preserve backward compatibility when adding settings.
- `src/demo.rs` supplies offline data and headless UI coverage. Use it for UI
  changes instead of requiring a Spotify account.
- `packaging/` contains independent MagicSpot desktop identities, icons,
  Windows installer metadata, and the macOS bundle script.

## Build and validation

The ordinary distributable deliberately excludes optional MilkDrop support:

```powershell
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Release
```

The result on Windows is `target/release/magicspot.exe`. The equivalent direct
command on every platform is:

```sh
cargo build --locked --release --no-default-features
```

Run these before committing code:

```sh
cargo fmt --all --check
cargo test --locked --no-default-features --features demo --all-targets
cargo clippy --locked --no-default-features --features demo --all-targets -- -D warnings
```

On this Windows workstation Cargo may need the explicit path
`C:\Users\Grant\.cargo\bin\cargo.exe`. Default features include projectM and
need CMake, libclang, Visual Studio 2022, and vcpkg on Windows. Do not interpret
a missing `VCPKG_INSTALLATION_ROOT` as a failure of the standard lightweight
build.

For a deterministic UI image:

```powershell
cargo run --locked --no-default-features --features demo -- `
  --demo-shot target\magicspot-demo.png `
  --demo-shot-delay 1200 `
  --demo-size 1400x900 `
  --demo-show lyrics
```

Useful `--demo-show` values include `lyrics`, `lyrics-expanded`, `lyrics-beta`,
`lyrics-center`, `lyrics-glow`, `lyrics-lora`, `queue`, `devices`, `light`,
`neutral`, and `oled`. Allow follow-scroll animations to settle before judging
a screenshot.

## Authentication and updates

Spotify Premium is required for local librespot playback. Initial setup has two
parts: Web API sign-in and separate local playback authorization. Shared API
quota, Spotify device discovery, and the first transfer can make the first
connection slower. A personal Development Mode client ID in Settings gives Web
API requests a separate quota but does not replace playback authorization.

MagicSpot has a release checker, not an installer updater. Checks default to
off; when enabled they query GitHub at most once per day and open the release
page for a newer version.

## Release runbook

Avoid releases for isolated cosmetic fixes. Accumulate and test a useful group
of changes, then:

1. Update the package version in both `Cargo.toml` and MagicSpot's package entry
   in `Cargo.lock`.
2. Run formatting, the complete lightweight test suite, strict Clippy, a local
   release build, and relevant demo screenshots.
3. Commit and push `main`.
4. Create and push an annotated tag such as `v0.8.0`.
5. From GitHub's Actions page, manually run **CI** if cross-platform validation
   is warranted.
6. Manually run **Release** with the exact tag to create or update the Windows
   installer, portable ZIP, raw executable, and checksums.
7. Manually run **macOS release** with the same tag to attach the universal DMG
   and checksum. Run Windows first because the macOS job expects a GitHub
   release to exist.
8. Verify that the release is neither a draft nor prerelease and that every
   expected asset is present before reporting completion.

The workflows are idempotent for an existing release: Windows uploads with
`--clobber`, and macOS already does the same. Standard hosted runners are free
for this public repository, but do not trigger a full matrix merely to validate
a documentation or routine UI commit.

## Distribution limitations

- Windows packages are unsigned and may show a publisher warning.
- macOS packages are universal and ad hoc signed, but not Apple-notarized.
- Homebrew is deferred until public releases are stable.
- Linux currently has source/Nix builds rather than a maintained store package.
- True word timing, lossless playback, and automatic in-place updates are not
  implemented.

## Upstream work

Follow `docs/UPSTREAM.md`. Review upstream frequently and integrate in small
batches. Expect conflicts in `src/app.rs`, `src/settings.rs`, `src/theme.rs`,
and `src/ui/`. Preserve MagicSpot branding, IDs, paths, packaging, themes, and
lyrics behavior. Playback, API, platform, accessibility, and performance fixes
are the strongest candidates to contribute independently to Fastpotify.

## Near-term backlog

- The local v0.8.0 workstream now contains the playlist/album page overhaul and
  lyric presentation controls. Finish user review before versioning or making
  a hosted release; no GitHub workflow has been triggered for this work.
- Add a custom theme editor after the current collection and lyric work settles.
- Improve first-connection feedback. The main queue interaction pass is now in
  the local v0.8.0 workstream.
- Continue theme-aware window and title-bar polish while preserving performance.
- Improve lyric timing only when reliable metadata is available.
- Add Developer ID signing, notarization, and possibly a Homebrew Cask as the
  downloads mature for broader distribution.
- Consider a true updater later; the current release checker is intentionally
  described accurately in the UI and documentation.
- Revisit lossless only after upstream playback support exists.

There is no known unfinished code task recorded at this handoff. Check open
issues, the working tree, and the newest user request before choosing work.
