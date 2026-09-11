# Changelog

MagicSpot follows semantic version tags for downloadable milestones. Changes
after the newest tag remain development work until the next release.

## Unreleased

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
