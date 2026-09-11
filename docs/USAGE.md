# Using MagicSpot

## Lyrics

Open lyrics with the microphone button or `L`. Drag the panel's left edge to
move continuously between compact and wide layouts. The album card and type
scale respond to the available width; there is no separate expanded mode.

Lyrics follow the playing line by default. Select **Follow** after manually
scrolling, or click a timed line to seek. Open **Lyrics appearance** under
**Settings → Appearance** to adjust text size, line spacing, left or centered
alignment, choose Inter, Manrope, or Lora type, enable a subtle glow, and set
how much blurred album artwork shows through the background. **Word-by-word
lyrics (Beta)** estimates progress between line timestamps and is disabled by
default.

The lyrics list has a small fixed gap below the song card. Following can center
the active line, but scrolling to the first line does not expose a large blank
area above it.

## Appearance

Choose Light, Dark, Follow System, or OLED at the top of Appearance settings.
Then choose the circular Aqua, Violet, Rose, Amber, or Neutral Gray swatch;
hover a swatch to see its name. Neutral Gray uses darker charcoal surfaces,
and both it and OLED accept album-art color. Disable **Colour from album art**
for a stable palette.

## Playlists and albums

Collection pages place their artwork on the right of a contained header when
the window is wide. Narrow windows stack the artwork over the title and move a
playlist's filter below its actions. The active track has an accent surface as
well as an accent title, so it remains visible while scanning a long list.

## Queue and navigation

The queue separates **Now playing**, **Playing next**, and tracks coming from
the current context. When this computer is the active MagicSpot player, the
buttons beside a manually queued song move it earlier or later or remove it.
Spotify does not expose those edits for another active device, so remote queues
remain readable and playable without edit controls. The queue can also be
refreshed, cleared, or saved as a private playlist.

Back and Forward support `Alt+Left` / `Alt+Right` and extra mouse buttons, and
their tooltips name the destination. At narrow widths the search field becomes
a search button and secondary utilities move into the account menu. `Escape`
closes the open popup, lyrics panel, or queue panel.

## Common shortcuts

| Shortcut | Action |
| --- | --- |
| `Space` | Play or pause |
| `Ctrl+←` / `Ctrl+→` | Previous or next |
| `Shift+←` / `Shift+→` | Seek ten seconds |
| `Ctrl+↑` / `Ctrl+↓` | Change volume |
| `M` | Mute |
| `B` | Like or unlike the playing song |
| `Q` | Toggle the queue |
| `L` | Toggle lyrics |
| `Ctrl+F` or `/` | Search |
| `Ctrl+B` | Toggle the library sidebar |
| `Ctrl+M` | Toggle the Winamp mini player |
| `Ctrl+,` | Settings |
| `Ctrl+/` or `?` | All shortcuts |
| `Ctrl+Q` | Quit |

macOS uses `Cmd` where the table says `Ctrl`.

## External controls

Linux exposes an MPRIS player named `magicspot`. On Windows and macOS, the
binary can control an already-running instance:

```text
magicspot play-pause
magicspot next
magicspot previous
magicspot volume 40
magicspot seek 15
magicspot like
magicspot now-playing
magicspot devices
```

Run `magicspot --help` for the complete list. Passing a Spotify URI or URL opens
it in the existing instance.

## First launch and updates

MagicSpot signs into the Spotify Web API first, then authorizes its local
librespot playback device separately. Spotify discovery, shared API quota, and
the first device transfer can make the initial connection slower than later
launches. A personal Spotify Development Mode client ID under **Settings →
Account** gives Web API requests their own quota; it does not replace the local
playback authorization.

Update checks are disabled by default. When enabled, MagicSpot checks GitHub at
most once per day and offers the release page when a newer version exists. The
app does not download or install updates in place; use the new installer, ZIP,
or DMG from GitHub Releases.

## Files and reset behavior

MagicSpot separates preferences, credentials and session state, and disposable
caches using each platform's standard directories. Typical Linux locations are
`~/.config/magicspot`, `~/.local/state/magicspot`, and `~/.cache/magicspot`.
Windows uses the `falleng101\magicspot` directories under AppData, and macOS
uses `com.falleng101.magicspot` under Application Support and Caches.

Deleting a cache is safe and does not sign you out. Deleting state removes
credentials and session history. Builds made before the namespace cleanup are
moved from the earlier MagicSpot location automatically when possible.
