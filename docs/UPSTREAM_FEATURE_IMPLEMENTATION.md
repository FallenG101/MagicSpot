# Approved feature implementation

Implemented scope: L2, C1–C8, and C10 from [the approved plan](UPSTREAM_FEATURE_PLAN.md). Existing README, maintainer handoff, and plan changes were preserved and included with the implementation.

## Using the features

| Item | Behavior |
| --- | --- |
| L2 | Full-window lyrics keeps the last decoded backdrop while the current artwork loads, uses available small artwork, and blends into a decoded replacement. Missing or failed artwork retains a sensible fallback. The lyric layout, follow behavior, tint, and glow remain intact. |
| C1 | Song, artist, album, and playlist context menus open a browsable Radio page. Play the mix, Refresh it, or Save as playlist. Albums and playlists supply up to five track seeds from their first 50 items. Results use Spotify recommendations and can differ from native Spotify Radio. The page explains unavailable API access and offers Open in Spotify; song seeds also retain the local song-radio fallback. |
| C2 | With a track list active, Ctrl+A (Cmd+A on macOS) selects its loaded, displayed rows. Ctrl/Cmd+C copies their Spotify URIs in display order. Ctrl/Cmd+X removes only selected occurrences in writable playlists; other sources copy with feedback. Ctrl/Cmd+V adds supported Spotify track links or URIs to the active writable playlist. Metadata resolves off the UI thread before unknown tracks are added. Existing duplicate feedback applies, repeated clipboard occurrences are reported, and text fields retain normal editing shortcuts. |
| C3 | In Add to playlist, type to filter, use Up/Down to choose a match, and press Enter to add. A new query starts at its first match. Empty results and Enter in unrelated fields do not add anything. |
| C4 | The Library's List/Grid buttons persist the chosen view. Cover cards adapt to sidebar width, retaining folders, pins, menus, ordering, track drops, and keyboard activation. |
| C5 | Settings rows wrap at narrow widths, and the Library heading compacts when space is limited. The existing 760 × 520 minimum window and 210–440 sidebar range remain supported. |
| C6 | Playlist details send only changed fields. Clearing a nonempty description shows Spotify's API limitation; other changes can still save. An unchanged dialog sends no update. |
| C7 | Removal uses original playlist positions and snapshots, including sorted/filtered views, repeated tracks, unavailable rows, and relinked tracks. Only selected occurrences are removed. Failed writes reload the playlist; partial batch failures explain that earlier batches may have succeeded. |
| C8 | The first drag that creates a custom Library playlist order explains that it applies in MagicSpot. Use the playlist menu's Reset library order to return to Spotify's ordering. The notice is persisted and does not repeat for every drag. |
| C10 | Large playlists expose their full range through virtual scrolling and the scrollbar, loading the requested range on demand. The separate Go to song input is removed; small playlists already had no such input. |

## API constraints

Radio uses the existing account-mode routing for recommendations. Spotify's [recommendations endpoint](https://developer.spotify.com/documentation/web-api/reference/get-recommendations) is deprecated and access depends on the app/account mode; it accepts at most five combined track/artist/genre seeds. This implementation uses track and artist seeds and displays an explicit failure when access is unavailable. See the [February 2026 migration guide](https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide) for development-mode restrictions.

Playlist edits retain the existing authenticated write routing and scopes. [Playlist detail updates](https://developer.spotify.com/documentation/web-api/reference/change-playlist-details) use optional fields. [Item removal](https://developer.spotify.com/documentation/web-api/reference/remove-items-playlist) uses the playlist snapshot and occurrence positions. Adds and removals are batched in groups of at most 100; removals proceed from higher positions to lower positions so earlier positions stay valid.

Live Spotify requests were not exercised in this session. Account-mode availability, occurrence-specific removal on the current service, playback, and playlist creation remain pending real-account validation. Automated tests validate request construction and state transitions, not the remote service.

## Verification

Completed locally on Windows:

- `cargo fmt --all --check`
- `cargo test --locked --features demo --all-targets -- --quiet`: 324 library tests and 5 executable tests passed.
- `cargo clippy --locked --features demo --all-targets -- -D warnings`
- `powershell -ExecutionPolicy Bypass -File .\build.ps1 -Release`: normal optimized build, without demo features.
- `git diff --check`

Regression coverage includes dirty-field request bodies, duplicate occurrence identities, filtered/sorted removal, gaps and relinking, clipboard focus and asynchronous metadata failures, picker navigation and same-frame search/Enter, local ordering and reset, full-range playlist scrolling, stale radio results and API failures, lyrics backdrop loading/fallback, and settings migration.

Headless UI checks cover 1280 × 800 and 760 × 520 windows with sidebars at 210, 250, 400, and 440 pixels, list/grid views, folders, pins, and Settings. Demo screenshots were visually inspected for the wide grid, minimum-size grid and Settings, full-window lyrics, and Radio. Screenshot files are local ignored artifacts under `target/scope-*.png`.

The initial implementation pass used local Windows verification. The subsequently requested [v3.0.0 release](https://github.com/FallenG101/MagicSpot/releases/tag/v3.0.0) is published as the latest stable milestone with Windows installer/portable ZIP, universal macOS DMG, checksum files, and GitHub source archives. Live Spotify behavior remains unverified.

Release verification subsequently passed:

- [Cross-platform CI](https://github.com/FallenG101/MagicSpot/actions/runs/36973798634): formatting, strict lint, and tests on Windows, Linux, and macOS.
- [Windows packaging](https://github.com/FallenG101/MagicSpot/actions/runs/36973800630): installer and portable ZIP.
- [Universal macOS packaging](https://github.com/FallenG101/MagicSpot/actions/runs/36973952752): Apple Silicon and Intel binaries combined into the DMG.
- All five uploaded assets are present and nonempty. The Windows and macOS checksum manifests match GitHub's SHA-256 asset digests. The release is public, neither draft nor prerelease, with user-facing notes and both source archive links.
