# Changelog

MagicSpot follows semantic version tags for downloadable milestones. Changes
after the newest tag remain development work until the next release.

## v0.9.99 — 2026-09-19

- Removed the Winamp mini player, skin subsystem, and MilkDrop/projectM
  visualizer, including their native build dependencies and background audio
  capture path.
- Kept older settings and session files readable while dropping obsolete
  visualizer fields from newly saved files; existing local skin and preset
  folders are left untouched.
- Reduced local-playback hot-path work by removing visualization sample
  buffering, locking, normalization-factor tracking, and shared-memory copies.
- Simplified appearance, shortcut, top-bar, and build documentation while
  retaining the standard player, queue, lyrics, and equalizer.
- Removed the redundant local playback activation request and shortened
  reconnect resume scheduling to 250 ms.
- Known issue: local playback can still take roughly 2–3 seconds to begin on
  some sessions; this release does not claim that underlying streaming delay
  is resolved.

## v0.9.91 — 2026-09-15

- Simplified built-in accent colors and allowed saved custom themes to be
  applied directly from Appearance settings.
- Fixed switching between OLED and custom themes so the most recent selection
  takes effect.

## v0.9.9 — 2026-09-15

- Added explicit sign-in, playback authorization, device discovery, and
  playback-transfer progress with actionable retries and diagnostics.
- Added guided personal Spotify Client ID setup with a copyable redirect URI.
- Added a live custom theme editor with saved themes and JSON import/export.

## v0.9.2 — 2026-09-13

- Added a compact now-playing layout below 720 points so track information,
  Previous/Play/Next, and the seek control no longer overlap in small windows.
- Made fixed-size demo screenshots ignore restored interactive-window geometry.
- Refreshed the user, build, authentication, and maintainer documentation for
  the current application and release process.
- Documented the preference for Spotify-backed, account-synced features over
  local-only library organization and listening history.

## v0.9.1 — 2026-09-12

- Prevented malformed or non-ASCII album release dates from crashing year
  rendering.
- Bounded detailed page, track metadata, and artwork-tint caches during long
  sessions while protecting playlist edits that are still being confirmed.
- Shared the current lyric document across repaints and removed the temporary
  word table previously allocated on every beta word-progress frame.

## v0.9.0 — 2026-09-11

- Fixed left lyric alignment by routing every line through one explicit
  full-width layout path.
- Replaced named colour-theme chips with compact circular colour swatches.
- Replaced flat lyric tint decoration with cached blurred cover-art backdrops
  and a clearer artwork-visibility control.
- Added bundled Inter, Manrope, and Lora lyric faces plus an optional subtle
  glow.
- Unified the lyric header and scrolling text on one edge-to-edge blurred-art
  canvas, reduced the album header height, improved inactive-line contrast,
  and removed the dark rectangular scroll-edge masks.
- Removed the extra square top stroke that interfered with the rounded
  now-playing bar corners and made its surrounding strip match the selected
  window theme.

## v0.8.0 — 2026-09-10

- Split the queue into clearer Now Playing, manually queued, and current-context
  sections with counts, refresh/save controls, and a stronger playing card.
- Added remove and move-earlier/later controls for manual queue rows while
  MagicSpot controls local playback.
- Made the top bar adapt at narrow widths, kept utility features in the account
  menu, added destination-aware Back/Forward hints, and let Escape close panels.
- Reworked playlist and album headers into rounded, responsive stages with
  larger right-side artwork on wide windows and a stacked narrow layout.
- Improved narrow playlist actions and made the current track easier to find
  in collection tables.
- Added lyric line-spacing, left/center alignment, and background-tint controls
  under a compact Lyrics appearance menu.
- Isolated lyric scroll state per track and refined unavailable and
  instrumental lyric messages.

## v0.7.4 — 2026-09-10

- Removed the large scrollable spacer above the first lyric.
- Kept a small fixed inset below the song card while preserving active-line
  following and centering.

## v0.7.3 — 2026-09-10

- Corrected square tint layers and malformed corners in rounded panels.
- Reduced lyrics and album-card tint strength for better readability.
- Reset stale synced-lyrics scroll state before the first timed line.
- Made the bottom player bar use the selected theme surface.
- Made page-header gradients follow rounded top corners.

## v0.7.2 — 2026-09-09

- Refined lyrics spacing and rounded the lyrics panel.
- Integrated window controls with the responsive lyrics header.
- Added a universal Apple Silicon and Intel macOS DMG release.

## v0.7.1 — 2026-09-09

- Established MagicSpot's independent name, executable, IDs, data paths,
  artwork, documentation, command protocol, packaging, and update endpoint.
- Introduced the redesigned desktop shell, resizable album-card lyrics panel,
  configurable lyric size, and optional beta word progress.
- Added Dark, Light, Follow system, OLED, and tintable accent themes.
- Added the Windows installer, portable ZIP, raw executable, and checksums.
