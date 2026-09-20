# MagicSpot maintainer handoff

Updated 2026-09-20. This is the starting point for a new maintainer, coding
agent, or chat that does not have the project's conversation history.

## Current baseline

- The working branch is `main`; `origin` is
  `https://github.com/FallenG101/MagicSpot.git`.
- Fastpotify remains configured as the `upstream` remote at
  `https://github.com/crmne/fastpotify.git`.
- The latest user release is **v1.0.0**. It hardens playback authorization,
  updates the pinned librespot fork's connection setup, adds playback timing
  diagnostics, and fixes confirmed-seek sink handling while preserving
  gapless track boundaries.
- The v1.0.0 release was validated with formatting, the complete demo-feature
  test suite (294 library tests and 5 binary tests), strict Clippy, and a
  normal release build. Windows and macOS packaging are manual GitHub Actions
  jobs and publish assets to the existing tag.
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

Prefer features backed by Spotify account data and supported Spotify APIs so
the experience remains consistent with Spotify on mobile and other devices.
Generally avoid MagicSpot-only library structures, playback history, folders,
or other local state that appears to be part of the user's Spotify account but
cannot follow them between clients. Local state is appropriate for desktop UI
preferences, caches, session restoration, and features that are clearly
presented as specific to this installation.

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
  share that surface; do not restore the separate opaque album card or dark
  scroll-edge masks.
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
- Appearance choices are Dark, Light, Follow system, and OLED, in that order.
  Accent choices are Blue, Red, Green, Yellow, White, Purple, and Cyan.
  Album-art color is optional. Saved custom palettes can be selected directly
  in Settings or changed in the editor, which supports layered transparency,
  shadow blur, reset, and portable JSON import/export. OLED and custom themes
  are mutually exclusive; the most recent selection takes effect.
- Blurred lyric backdrops use a 128-pixel derived PNG made off the UI thread and
  cached through `ArtLoader`; do not blur full-size artwork every frame.
- Loaded lyric documents are held behind `Arc` because the playback repaint
  path reads them four times per second. Beta word progress must remain
  allocation-free in that path.
- The bottom player bar uses the selected theme surface rather than its own
  album tint. Its outer spacing is painted with `palette.window`, so the root
  egui fill cannot show through as a differently coloured strip.
- Below 720 points the player bar switches to a compact layout with cover and
  clipped metadata, Previous/Play/Next, and a full-width bottom seek line.
  Keep the optional controls out of this layout so regions cannot overlap.
- Queue sections distinguish the playing row, manually queued rows, and the
  current context. Manual rows can be moved or removed only when the local
  MagicSpot player is active; this is implemented by clearing and rebuilding
  librespot's manual queue while preserving context rows.
- Spotify playlist folders are read and rendered as a collapsible tree.
  MagicSpot intentionally does not create or reorganize folders because those
  operations are absent from Spotify's supported Web API.
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
- Detailed page caches use per-type LRU caps, and track metadata is capped at
  800 entries. The open page, playing context, and playlists with pending or
  unconfirmed edits are protected from eviction. Preserve those guarantees
  when changing navigation or playlist writes.
- `src/backend.rs` owns the Tokio runtime. Spotify API, authentication,
  playback, lyrics, and image work must stay off the egui thread.
- `src/api/` handles Spotify Web API models, clients, and shared/personal app
  routing.
- `src/player.rs`, `src/sink.rs`, `src/resample.rs`, `src/eq.rs`, and
  `src/limiter.rs` implement local playback and audio processing.
- `src/playback_timing.rs` records low-overhead local playback milestones for
  verbose diagnostics; it must remain off the UI thread and must not record
  credentials.
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

```powershell
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Release
```

The result on Windows is `target/release/magicspot.exe`. The equivalent direct
command on every platform is:

```sh
cargo build --locked --release
```

Run these before committing code:

```sh
cargo fmt --all --check
cargo test --locked --features demo --all-targets
cargo clippy --locked --features demo --all-targets -- -D warnings
```

On this Windows workstation Cargo may need the explicit path
`C:\Users\Grant\.cargo\bin\cargo.exe`. A normal build has no projectM native
toolchain and does not need CMake, libclang, or vcpkg.

For a deterministic UI image:

```powershell
cargo run --locked --features demo -- `
  --demo-shot target\magicspot-demo.png `
  --demo-shot-delay 1200 `
  --demo-size 1400x900 `
  --demo-show lyrics
```

Useful `--demo-show` values include `lyrics`, `lyrics-expanded`, `lyrics-beta`,
`lyrics-center`, `lyrics-glow`, `lyrics-lora`, `queue`, `playing-next`,
`local-queue`, `recents`, `devices`, `shortcuts`, `premium`, `create`,
`duplicate`, `light`, `focus`, `resume`, `resume-next`, `eq`, `art`, `folders`,
`compact`, `pins`, `sorted`, `neutral`, `oled`, and `scripts`. Allow
follow-scroll animations to settle before judging a screenshot.

## Authentication and updates

Spotify Premium is required for local librespot playback. Initial setup has two
parts: Web API sign-in and separate local playback authorization. Shared API
quota, Spotify device discovery, and the first transfer can make the first
connection slower. A personal Development Mode Client ID in Settings routes
supported Web API calls through the user's developer account but does not
replace playback authorization. Its registered redirect must be exactly
`http://127.0.0.1:8989/login`; MagicSpot never needs the Client Secret. Spotify
currently requires the development-app owner to have Premium, restricts its
authorized users, and counts development quota per developer account.

MagicSpot has a release checker, not an installer updater. Checks default to
off; when enabled they query GitHub at most once per day and open the release
page for a newer version.

The current release can still show a roughly 2–3 second local playback startup
or skip delay on some sessions. Use `magicspot --verbose` and the
`magicspot::playback_timing` log entries to distinguish track loading, output
opening, decoder startup, and audio-queue starvation when investigating it.

## Release runbook

Classify versions by user-visible scope. Bug fixes and visual corrections bump
the patch number (`x.y.Z`), a feature bumps the minor number (`x.Y.0`), and a
release with multiple features or a broad overhaul bumps the major number
(`X.0.0`). A mixed release takes its highest applicable bump. Do not change the
version or create a release unless the user requests one.

When publishing a requested release:

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

- Exercise the v1.0.0 connection, playback authorization, custom theme, and
  local playback flows across varied real Spotify accounts and devices.
- Continue profiling the remaining local playback startup and skip delay in
  the shared librespot/Fastpotify playback path, especially decoder buffering,
  preloading, and reconnect behavior.
- Continue theme-aware window and title-bar polish while preserving performance.
- Improve lyric timing only when reliable metadata is available.
- Favor account-synced Spotify features over local-only library organization or
  listening-history features.
- Add Developer ID signing, notarization, and possibly a Homebrew Cask as the
  downloads mature for broader distribution.
- Consider a true updater later; the current release checker is intentionally
  described accurately in the UI and documentation.
- Revisit lossless only after upstream playback support exists.

The requested v1.0.0 implementation and local validation are complete.
Continue stabilization from user feedback and real-account testing.
