# Using MagicSpot

## Lyrics

Open lyrics with the microphone button or `L`. Drag the panel's left edge to
move continuously between compact and wide layouts. The album card and type
scale respond to the available width; there is no separate expanded mode.

Lyrics follow the playing line by default. Select **Follow** after manually
scrolling, or click a timed line to seek. Change lyric size under
**Settings → Appearance**. **Word-by-word lyrics (Beta)** estimates progress
between line timestamps and is disabled by default.

## Appearance

Choose Light, Dark, Follow System, or OLED at the top of Appearance settings.
Then choose Aqua, Violet, Rose, Amber, or Neutral Gray independently. Neutral
Gray uses darker charcoal surfaces, and both it and OLED accept album-art tint.
Disable **Colour from album art** for a stable palette.

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

## Files and reset behavior

MagicSpot separates preferences, credentials and session state, and disposable
caches using each platform's standard directories. Typical Linux locations are
`~/.config/magicspot`, `~/.local/state/magicspot`, and `~/.cache/magicspot`.
Windows uses the `falleng101\magicspot` directories under AppData, and macOS
uses `com.falleng101.magicspot` under Application Support and Caches.

Deleting a cache is safe and does not sign you out. Deleting state removes
credentials and session history. Builds made before the namespace cleanup are
moved from the earlier MagicSpot location automatically when possible.
