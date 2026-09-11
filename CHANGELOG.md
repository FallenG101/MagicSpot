# Changelog

MagicSpot follows semantic version tags for downloadable milestones. Changes
after the newest tag remain development work until the next release.

## Unreleased

- Fixed left lyric alignment by routing every line through one explicit
  full-width layout path.
- Replaced named colour-theme chips with compact circular colour swatches.
- Replaced flat lyric tint decoration with cached blurred cover-art backdrops
  and a clearer artwork-visibility control.
- Added bundled Inter, Manrope, and Lora lyric faces plus an optional subtle
  glow.
- Unified the lyric header and scrolling text on one edge-to-edge blurred-art
  canvas, reduced the album header height, and improved inactive-line contrast.
- Removed the extra square top stroke that interfered with the rounded
  now-playing bar corners.

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
