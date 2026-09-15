# Upstream maintenance

MagicSpot is based on Fastpotify and retains its Git history. The configured
remotes are:

```text
origin    https://github.com/FallenG101/MagicSpot.git
upstream  https://github.com/crmne/fastpotify.git
```

Review upstream changes regularly rather than waiting through several major
releases:

```sh
git fetch upstream
git log --oneline main..upstream/main
git diff --stat main...upstream/main
```

Integrate focused fixes in small, reviewable batches by cherry-picking or
reimplementing them, then run the full checks from `docs/BUILDING.md`. Avoid a
wholesale merge unless the divergence has first been audited. Files most likely to conflict are
`src/app.rs`, `src/settings.rs`, `src/theme.rs`, and `src/ui/` because MagicSpot
intentionally changes the shell, appearance, and lyrics behavior.

At the 2026-09-13 audit, Fastpotify was 76 commits ahead of the shared fork
point while MagicSpot was 23 commits ahead, with broad overlap across core UI
and application files. Full merges are therefore high-conflict; selective
playback, API, platform, accessibility, and performance integrations remain
manageable and worthwhile. Review upstream weekly or before each substantial
MagicSpot feature batch.

Keep the feature-rich interface and branding in MagicSpot. Generic fixes for
playback, Spotify API behavior, accessibility, performance, or platform support
are good candidates for small upstream pull requests. This keeps useful work
shared without asking Fastpotify to adopt MagicSpot's complete product design.

Do not merge upstream packaging or automation blindly. Confirm that repository
URLs, application IDs, data paths, executable names, artwork, and release jobs
still identify MagicSpot before pushing.
