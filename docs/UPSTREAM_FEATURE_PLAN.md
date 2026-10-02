# Feature and Upstream Improvement Plan

This plan records the changes Grant approved for implementation in a later pass. All ten listed items (L2 and C1–C8 plus C10) are in scope. It compares useful upstream behavior with MagicSpot’s current design; it does not propose copying upstream wholesale. Keep MagicSpot’s branding, lyrics presentation, settings, and other intentional product choices.

## Implementation handoff

The next implementation session should treat every item in “Selected implementation scope” as approved work and complete the full set. Do not stop after producing another plan. Work in the existing MagicSpot checkout, inspect its current state and project instructions first, preserve existing uncommitted user changes, and do not push, publish, tag, or create a release unless Grant asks in that session.

Implement in small, reviewable stages. Before each change, trace the existing UI, action/state flow, API behavior, and tests that own it. Prefer focused changes that fit MagicSpot’s current architecture and visual language. Do not import broad upstream patches or reintroduce product directions MagicSpot intentionally removed (including Winamp-related behavior). Keep network and playback work off the UI thread, preserve backward compatibility for settings, and never log credentials.

Start with the C1 API feasibility check and C6/C7 data-flow checks described below. Resolve uncertainties from the repository, tests, and current official Spotify documentation. If an API path is unavailable, implement the best supported behavior and explain the limitation rather than blocking unrelated approved items. Keep each feature independently testable; after each stage, run the narrow relevant tests and fix regressions before continuing.

Finish with the project’s complete documented verification: formatting, focused and full tests, strict Clippy, and the documented release build check. Review the final diff for accidental changes, update user-facing documentation/help where shortcuts or behavior changed, and report what shipped, test results, and any external API limitations. Do not commit unless asked.

## What the selected items mean

### C1 — Radio pages

Spotify already offers Radio: a fresh collection of music based on an artist, album, or song, and a radio can be saved to Your Library. So this is not simply a request to make MagicSpot look like Spotify or to invent a capability Spotify lacks.

The upstream feature makes radio a browsable page inside the app: open a radio from a song, artist, album, or playlist; inspect and play its tracks; refresh the mix; and save it as a playlist. The value for MagicSpot would be making Spotify’s discovery flow easier to browse and manage without leaving the app. Before implementation, check which sources and endpoints work for MagicSpot’s Spotify app/account mode, since API availability and development-mode limits can change. Reuse the existing song-radio and recommendations paths where they are supported; design a clear fallback if they are not. Do not promise identical results to Spotify’s native Radio.

### C4 — Library cover grid

This is a second way to browse the existing Library sidebar: show albums, artists, or playlists as artwork-led cards instead of only text rows, with a way to switch back to the compact list. It is a visual scanning and navigation option, not a new library or a requirement for Spotify parity. Preserve folders, pinned items, sorting, drag/reorder behavior, and a usable compact mode.

### C5 — Narrow-window layout fixes

This means rearranging controls when space is limited so Settings controls can wrap instead of colliding or becoming unreadable, and the Library heading can give way to a compact icon when the sidebar is very narrow. First compare the proposed sizes with MagicSpot’s supported minimum window size and sidebar limits. Improve the widths MagicSpot supports; only lower a minimum if the resulting app remains usable.

## Selected implementation scope

| ID | Planned change | Main implementation notes |
|---|---|---|
| L2 | Smooth lyrics backdrop changes | In the full-screen lyrics view, retain a sensible backdrop while a new track’s artwork loads. Prefer available cached/small artwork as an immediate fallback; replace it after the new image is decoded and blend the change to avoid a black/blank flash. Keep MagicSpot’s existing lyrics layout, active-line follow, tint, and glow. |
| C1 | Radio pages | Validate API and account-mode support first. Build a page for supported seed types with track browsing, play, refresh, and save-as-playlist actions. Handle loading, empty results, offline/API errors, refresh races, and playlist creation errors. Keep the existing song-radio flow working. |
| C2 | Keyboard track editing | Add select-all, copy, cut, and paste for playlist tracks, integrated with existing row selection and bulk actions. Use Spotify track links/URIs as the clipboard representation. Keep normal text-field editing shortcuts intact. Cut removes tracks only from a context where the user can edit them; paste adds tracks to the active writable playlist and reports duplicates or failures clearly. |
| C3 | Enter in add-to-playlist picker | When the playlist picker is focused, Enter should choose the first matching playlist; arrow keys should allow choosing another result. Avoid treating Enter in an unrelated text field as an add action. Reuse the existing add/duplicate feedback. |
| C4 | Library cover grid | Add a persisted list/grid choice and responsive cover cards. Preserve existing playlist folders, pins, ordering, context menus, and keyboard accessibility. |
| C5 | Narrow-window layout fixes | Let Settings rows wrap cleanly and compact the Library heading/sidebar controls at narrow widths. Verify against the actual supported window and sidebar ranges before changing minimum dimensions. |
| C6 | Safer playlist detail updates | Send only playlist fields that the user changed. Handle Spotify’s description-clearing limitation with a specific explanation, while allowing other changed fields to save. Do not send unchanged values as accidental overwrites. |
| C7 | Correct removal from filtered/sorted playlists | Resolve selected rows to their actual playlist entries, not their displayed positions after sorting or filtering. Preserve correct behavior for duplicate tracks, and remove only the intended entries. |
| C8 | Explain playlist reorder behavior | When a drag first changes a playlist from following Spotify’s order to MagicSpot’s local custom order, explain that the change affects MagicSpot’s ordering. Make the explanation concise and avoid repeating it on every reorder; explain how to return to Spotify’s ordering. |
| C10 | Remove “Go to song” control on large playlists | Remove the separate position-entry control from large playlist views after confirming that virtualized scrolling and the scrollbar make navigation practical and accessible. Check whether the control remains useful for small playlists before removing it everywhere. |

## Suggested implementation order

### 1. Validate dependencies and data identity

1. For C1, verify the current Spotify endpoints, app mode, scopes, quotas, and supported seed types. Map the existing `PlayTrackRadio` and recommendations implementation before designing the page. This is a short feasibility check that prevents designing around an unavailable API behavior.
2. For C7, trace how the table maps sorted/filtered rows to playlist entries, and how Spotify identifies duplicate occurrences. Decide on a stable entry identity before changing removal behavior.
3. For C6, inspect playlist editing and the API’s optional update fields; define dirty-field and blank-description behavior before changing the request.

### 2. Correctness and focused interaction improvements

Implement C6 and C7 first, since they prevent unintended edits and removals. Then implement C3 and C8, followed by C10 after validating large-list navigation. These changes are independently reviewable and do not require adopting upstream visual design.

### 3. Playback, lyrics, and keyboard workflows

Implement L2 and verify image transitions across track changes, missing artwork, slow artwork responses, and cached artwork. Then implement C2, building on the existing multi-select and bulk-action model. Ensure selection is scoped to the intended playlist and that keyboard shortcuts continue to edit text normally when a text input has focus.

### 4. Discovery and library presentation

Once the C1 feasibility check succeeds, implement the radio page in small steps: supported entry points, result page, refresh, then save-as-playlist. If a seed type is unsupported, omit that entry point or give a clear explanation rather than pretending it works. Implement C4 and C5 as contained sidebar/layout changes, preserving the current Library structure.

## Verification plan

- Run `cargo fmt --all --check`, the focused Rust tests, the project test suite, strict Clippy, and the documented release build check.
- Add or update tests for changed-field-only playlist requests and the description-clearing explanation (C6); filtered, sorted, and duplicate-entry removal (C7); picker focus and Enter selection (C3); first reorder explanation and reset-to-Spotify-order behavior (C8); and large versus small playlist navigation (C10).
- Exercise C2 with row focus and text-field focus, including copy/cut/paste, non-editable sources, duplicates, and failed additions.
- Exercise L2 with cached and uncached art, slow/failing image loads, missing artwork, and rapid track changes so an older response cannot replace the current track’s backdrop.
- Exercise C1 with mocked supported and unsupported seeds, empty results, refresh races, offline/rate-limit/API failures, and playlist creation. Confirm the real API assumptions separately before release.
- Review C4/C5 at the normal window size, the supported minimum, narrow sidebar widths, and with Library folders, pins, list mode, and grid mode.

## Platform-specific work

Leave platform-specific improvements out of this pass unless testing shows one is required for a selected feature. In particular, do not bring back product directions MagicSpot has intentionally left behind, such as Winamp-related behavior.

Some platform items can be worthwhile later, but they are conditional:

- **Window repaint and hidden-window CPU use:** worth addressing if profiling shows excess CPU, battery use, or delayed redraws on a supported desktop. This can affect responsiveness, but it is not a prerequisite for the selected feature work.
- **Linux compositor/window-manager behavior and trackpad gestures:** important when a reproducible issue affects the target desktop environment; otherwise defer rather than add unverified platform branches.
- **Signing and checksums:** improve trust and verification for downloaded releases. This is a release/distribution improvement, not a loading or UI prerequisite, and can be planned separately.
- **Font/rendering, titlebar, dock, and desktop-specific integration tweaks:** valuable when they fix a demonstrated usability issue on that platform. Keep them separate from this pass unless a selected layout change exposes a concrete problem.

## References

- [Spotify Support: Spotify Radio](https://support.spotify.com/uk/article/spotify-radio/)
- [Spotify Web API documentation](https://developer.spotify.com/documentation/web-api)
- [Spotify Web API February 2026 migration guide](https://developer.spotify.com/documentation/web-api/tutorials/february-2026-migration-guide)
