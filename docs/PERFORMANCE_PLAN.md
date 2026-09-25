# MagicSpot performance plan

Updated 2026-09-25. This plan covers load time, interaction latency, steady-state
CPU, memory, and network use across Windows, macOS, and Linux. It is an audit
and work plan; the timings below need a measured baseline before targets can be
set.

## Implementation status

This pass addresses bottlenecks visible in code without requiring a Spotify
account or a release-profile timing rig:

- Cold track setup and audio-key retrieval now overlap.
- Album and playlist detail pages use known metadata immediately; an album's
  embedded first track page is reused, and playlist metadata requests ask only
  for header fields. Playlist cache rows still wait for snapshot validation.
- Collaborator display-name lookups run with a concurrency limit of four,
  allowing names to appear as their individual requests complete.
- Playlist cache serialization and disk replacement run on a blocking worker,
  preserving write order without holding up backend commands.
- Verbose API logs separate cooldown, permit, token, transfer, and JSON decode
  time. Large JSON responses decode on a blocking worker.
- Windows media controls skip position-only state messages; Linux already
  throttles position publication. Explicit seeks still publish immediately.
  The frame path also reuses one now-playing snapshot for these controls and
  the local control channel.
- Verbose logs now record album and playlist navigation through metadata,
  first-page response, first useful header frame, and first usable rows frame.
  They also record process start through the first UI frame. The summary
  script computes median and p95 from collected runs.
- Artwork on disk has a 512 MiB target. Startup and post-write cleanup remove
  oldest unreferenced files and partial writes on a blocking worker. Active
  images and the cover handed to desktop media controls are protected.
  Concurrent direct and UI artwork requests for one URL share a fetch.

The remaining phases require comparative release-build measurements, an
authenticated account, and platform-specific runs. The code changes above are
functional improvements, not measured speedup claims. Record the baseline and
repeat it after each further tuning batch before setting numeric targets.

## Measuring album, playlist, and playback latency

Run an optimized build with `--verbose`. On Windows the current-run log is
`%LOCALAPPDATA%\falleng101\magicspot\data\MagicSpot.log`; the app replaces it at launch, so
copy it into a private measurement directory before each restart. Do not
publish raw logs, which may include account or library details from other
subsystems. The new `magicspot::page_timing` entries contain only page type,
cache flags, source, and elapsed time. `magicspot::startup_timing` records the
first UI frame. Existing `magicspot::playback_timing` summaries record time to
the first queued audio buffer, which is earlier than audible output.
If `RUST_LOG` is set, it overrides the default `--verbose` filter; include
`magicspot=debug` or the individual timing targets in that environment value.

Use at least five repetitions of each scenario in the same build and network
conditions:

1. Open an album and playlist with no in-memory page entry, then revisit each
   while its page entry remains. Note whether rows are already loaded; the
   log's `warm_header` and `warm_rows` flags classify this automatically.
2. Restart for process-start timing. Capture the first UI frame and, if the
   session restores an album or playlist, its page timing separately.
3. For playback, label trials manually as uncached, disk-cached, or preloaded.
   Avoid clearing the audio cache between repeated disk-cached trials. Use
   distinct tracks or a cleared audio cache only for intentionally uncached
   trials. The `kind` field in the playback trace is the player's own
   classification and should be retained when comparing results.

Summarize copied logs with PowerShell:

```powershell
.\scripts\performance-summary.ps1 -Path .\measurements\pages-*.log
.\scripts\performance-summary.ps1 -Path .\measurements\cold-*.log -PlaybackScenario uncached
```

The output reports sample count, median, and nearest-rank p95 in milliseconds
per scenario and milestone. Keep build revision, machine, network condition,
and trial preparation alongside the results. A p95 from five trials is simply
the slowest trial; collect more repetitions before drawing a tight conclusion.

The first Windows release-build validation on 2026-09-25 confirmed that the
markers and summary script work against a real restored playlist. Spotify was
returning repeated rate limits during the run, so the single recorded trial
is a functional check, not a cold/warm performance baseline. Repeat the
comparative runs once requests are no longer being throttled.

## Goals

- Show useful content quickly after launch and navigation.
- Start audio quickly for both cached and cold tracks.
- Keep scrolling, search, controls, lyrics, and theme changes responsive on
  large libraries and modest hardware.
- Bound CPU, memory, background requests, and disk caches without degrading
  correctness or account synchronization.
- Keep the native Rust, egui, and librespot architecture and all three desktop
  platforms working.

## Current foundations

- Backend work runs outside egui. API requests are dispatched asynchronously,
  the HTTP client allows up to six in-flight requests per session, and
  background requests have a separate concurrency limit.
- Album and playlist collections paginate. Playlist rows have a snapshot-aware
  disk cache; track metadata and detailed pages have bounded in-memory caches.
- Track rows and card grids are virtualized. Two table-row caches are retained.
- Artwork downloads run asynchronously, share a loader, retain decoded art up
  to a memory budget, and use a disk cache. Blurred lyric art is derived off
  the UI thread.
- Playback logs milestones through `magicspot::playback_timing`; the vendored
  player incrementally streams audio and preloads upcoming tracks.
- The active worktree already contains a change that overlaps cold audio-file
  setup with its audio-key request, shorter playlist metadata fields, and
  metadata seeding for album and playlist pages. Include these changes in the
  baseline; do not count their gains twice.

## Phase 0: measure before tuning

Build a repeatable Windows baseline first, then sample macOS and Linux before
making platform-sensitive changes. Use release builds for user-facing timings;
debug builds distort CPU and audio results.

Record at least five runs per scenario and report median and p95:

1. Process start to first interactive frame; start to signed-in Home; start to
   first useful content after session restore.
2. Navigation to album or playlist: first visible header, first usable rows,
   complete first page, and a warm revisit.
3. Search keystroke to results, then result click to visible detail content.
4. Play press to first audio queued and audible for cold uncached, warm disk
   cache, and preloaded next-track paths.
5. Scroll and type in small, medium, and very large collections, including
   lyrics open, artwork-heavy pages, and each color theme.
6. Idle and playing resource use: CPU, process memory, artwork memory, audio
   cache size, artwork disk size, request count, and bytes transferred.

Use the existing verbose API request durations, playlist-cache logs, and
`magicspot::playback_timing` traces. Add opt-in timing only where a gap remains:
startup milestones, page-data phases, first-paint milestones, and frame-time
sampling. Keep credentials, track contents, and personal data out of logs.
Capture a baseline report and machine/build details before choosing numeric
acceptance targets.

## Phase 1: fastest visible content and page loading

### Album and playlist details

- Keep the current metadata seeding for pages opened from library, search, and
  discovery results. Measure how often it avoids a blank/loading header and
  how long until fresh rows arrive.
- Keep playlist metadata responses limited to fields the header uses; compare
  response bytes and decode time for large playlists.
- Inspect the album endpoint's embedded track page and use it as the first
  page when present. Request another page only when the embedded page says one
  exists. Avoid duplicate first-page requests.
- Add a bounded, account-independent catalogue cache for album metadata and
  first-page tracks if measurements show cold direct links remain slow. Give
  cached content a clear freshness policy and always reconcile with Spotify.
- For playlists, render a validated cached prefix as soon as its snapshot can
  be confirmed. Measure the current wait for live metadata before cache
  adoption; consider a small cached snapshot record only if it can be checked
  safely without allowing stale rows to replace current playlist contents.
- Keep refresh failures from blanking usable seeded or cached data, while
  exposing refresh errors and retry paths.

### API request scheduling and payloads

- Trace each page's request dependency graph. Start independent calls together;
  keep cursor/offset pagination ordered where later requests depend on earlier
  responses.
- Prioritize requests that unblock visible content over membership checks,
  collaborator names, recommendations, and other background work.
- Audit Home's playlist pagination and folder-order fetch. Fetch the visible
  portion first, and only batch independent later offsets if ordering,
  duplicate handling, rate limits, and error recovery remain correct.
- Keep artist names and collaborator lookups from serially delaying the rows;
  use bounded parallelism and deduplicate repeated IDs if profiling finds a
  visible wait.
- Audit catalog response sizes and JSON parsing cost. Use supported `fields`
  selection where available and avoid requesting data the page immediately
  fetches elsewhere.
- Measure token refresh, semaphore wait, cooldown, HTTP transfer, and JSON
  decoding separately. Respect Spotify `Retry-After`; do not trade faster
  screens for higher request volume or quota exhaustion.
- Review the 280 ms search debounce against typing responsiveness, request
  volume, and cancellation of obsolete searches.

### Pagination and prefetch

- Measure first-page sizes for playlists, albums, saved library, search, and
  artist discographies. Tune page size only against response time and decode
  cost, while observing Spotify's endpoint limits.
- Keep the current near-end loading threshold responsive on slow links. Test
  whether fetching the next page slightly before it is visible prevents blank
  gaps without producing wasteful requests.
- Consider prefetching the next page on deliberate idle time for the open
  collection, with cancellation when the user navigates away.
- Preserve direct playlist position jumps and avoid loading every preceding
  page just to reach a distant row.

## Phase 2: launch and navigation

- Measure time spent loading settings/session state, installing fonts, scanning
  system fonts, creating the window, restoring the player, authorizing the API,
  and loading the restored page.
- System fallback-font discovery walks system font directories on Windows and
  Linux. It is already cached once per process; measure its first-window cost
  and defer or reduce work if it affects time to first frame.
- Draw the shell and restored page immediately, then restore playback and
  refresh account-backed data in the background where ordering allows.
- Preserve the last useful page data in bounded session caches so Back,
  Forward, and recently opened pages can paint before refresh completes.
- Separate account-independent artwork/catalog cache entries from
  account-scoped playlist caches. Bound cache size and age, and make cleanup
  incremental rather than blocking startup.
- Check settings/session reads and writes for large serialized structures or
  synchronous work on the UI thread. Persist only when values change and keep
  compatibility with older settings.

## Phase 3: cold and warm playback

- Extend the existing playback trace to distinguish audio-item metadata,
  audio-key service, CDN URL resolution, first CDN bytes, decoder readiness,
  output opening, and first packet queued. Compare cached and uncached tracks.
- Measure the current parallel audio-key and file-open change on cold tracks;
  retain it only if it improves p50/p95 without worsening error handling.
- Profile the initial CDN range size, first-byte waits, range scheduling,
  throughput adaptation, and decoder read-ahead. Tune by observed network
  latency and underruns rather than increasing buffers blindly.
- Measure output-device discovery/open time separately from Spotify loading.
  Keep device opening off the UI thread and avoid reopening it on ordinary
  track transitions.
- Tune next-track preload trigger and cushion for slow and fast connections.
  Bound work when the user skips rapidly or changes context; cancel obsolete
  preload work promptly.
- Profile audio decoding, normalization, resampling, equalizer, limiter, and
  sink queue CPU under release settings. Optimize only the measured hot stage
  and keep audio quality and underrun behavior stable.
- Keep the audio cache bounded and verify disk hits, eviction, concurrent file
  access, and startup cost. Do not claim crossfade or unsupported audio quality
  based on buffering changes.

## Phase 4: UI and interaction cost

- Capture egui frame-time distributions for Home, Search, playlist, album,
  artist, queue, lyrics, settings, and theme editor. Include large data sets and
  narrow windows.
- Virtualized track rows and card grids are already in place. Profile row
  construction, sorting, filtering, accessibility metadata, text shaping,
  artwork texture upload, and drag/drop before changing list architecture.
- `collection.rs` caches table rows and visible indices. Measure cold sort and
  filter work on large lists; move expensive transformations off-frame or
  incrementally cache them if typing/scrolling shows stalls.
- Audit per-frame clones and formatting around `now_playing`, control
  snapshots, media controls, queue state, and lyric layout. Cache stable values
  by revision and update them when state changes.
- Lyrics already cache lyric documents and derive blurred art off-thread.
  Profile galley layout, active-line restyling, glow drawing, and 250 ms
  playback repaints. Avoid relayout when text, width, font, and style are
  unchanged.
- Profile themed gradients, shadows, and album-color effects on integrated
  graphics. Keep visual changes theme-aware and compare GPU/CPU frame time.
- Recheck repaint scheduling while paused, playing, animating, and idle. Keep
  playback progress smooth while avoiding unnecessary idle wakeups.
- Check platform media-control updates, tray updates, accessibility tree work,
  and window title updates for redundant sends; cache last-published state.

## Phase 5: artwork and memory

- Measure artwork cache hit rates, file sizes, network bytes, decode time,
  texture upload time, retained bytes, and memory churn on long sessions.
- Artwork memory is capped, but the disk cache currently has no stated size
  budget. Add size-based disk eviction with atomic writes and protect artwork
  currently used by media controls.
- Avoid repeated disk reads when `ArtLoader::fetch` runs concurrently for one
  URL. Preserve the in-flight request coalescing already used by the egui
  loader across direct fetch paths too.
- Use the smallest supported Spotify image appropriate to each surface. Keep
  the original art unchanged; do not crop or paint over the source artwork.
- Measure full-size decoding for small covers. If upload or decode dominates,
  create safe size-appropriate cached derivatives off the UI thread.
- Bound negative/failure cache entries and clean them without evicting visible
  or pending art.
- Track total memory, not only artwork bytes: page caches, table rows, track
  metadata, lyrics, textures, fonts, queue, and large API response bodies.

## Phase 6: background work and platform behavior

- Inspect Tokio worker utilization and blocking tasks during font scans,
  artwork decoding, JSON parsing, local filesystem work, receiver discovery,
  and playback startup. Keep blocking work away from the UI and avoid starving
  audio/network tasks.
- Add request cancellation or generation checks where obsolete search/page
  results still consume bandwidth or CPU after navigation.
- Ensure one slow cache write or font scan cannot delay playback controls or
  visible API responses.
- Compare Windows, macOS, and Linux audio output open, resampling, tray/media
  integration, font startup, and repaint behavior. Retain platform-specific
  code paths where one shared setting hurts a platform.
- Measure release binary startup and resident memory after dependency or build
  profile changes. Current release builds already use thin LTO and one codegen
  unit; build tuning is lower priority than runtime user-visible delays.

## Priority order

1. Establish baselines and stage timings.
2. Finish and measure immediate album/playlist content and payload reduction
   already in the worktree.
3. Profile collection, Home, and search API scheduling; fix requests that block
   first visible content.
4. Improve validated warm page/cache behavior and add a bounded album cache if
   cold direct navigation remains a top delay.
5. Instrument and tune cold audio startup, CDN read-ahead, and output opening.
6. Profile UI frame time, artwork, and memory, then optimize measured hot paths.
7. Recheck cross-platform release behavior and confirm regressions are absent.

## Acceptance and guardrails

- Compare the same machine, account, library, network, build profile, cache
  state, and scenario before and after each batch.
- Report p50 and p95, not just a best run. Separate cold, warm, and preloaded
  paths, and separate local rendering from network wait.
- Set concrete improvement targets after Phase 0. Every change must improve a
  measured user-facing delay or resource cost enough to justify its complexity.
- Keep visible content correct during refresh; validate playlist snapshots
  before adopting cached rows; cancel stale requests; keep cache bounds and
  account isolation intact.
- Track API calls and transferred bytes alongside speed so optimizations do
  not cause quota or privacy regressions.
- Re-run the project's formatting, demo tests, strict Clippy, and relevant
  release build after implementation batches. Use deterministic demo screens
  for UI review and real Spotify accounts only for network/playback timing.

## Main code areas

- Startup and frame loop: `src/main.rs`, `src/app.rs`, `src/theme.rs`,
  `src/system_fonts.rs`
- API, request routing, and cache: `src/api/client.rs`, `src/api/gateway.rs`,
  `src/backend.rs`, `src/model.rs`
- Album and playlist UI: `src/ui/collection.rs`, `src/ui/widgets.rs`
- Artwork and lyrics: `src/images.rs`, `src/lyrics.rs`, `src/ui/lyrics.rs`
- Local playback and audio: `src/player.rs`, `src/playback_timing.rs`,
  `src/sink.rs`, `src/resample.rs`, `src/eq.rs`, `src/limiter.rs`,
  `vendor/librespot-audio/`, `vendor/librespot-playback/`
