# Privacy and network access

MagicSpot has no telemetry, analytics service, advertising system, or hosted
backend.

- Spotify authentication occurs on Spotify's website. MagicSpot stores refresh
  tokens and the local-playback credential in the platform state directory.
- An optional personal Spotify app stores its public Client ID in preferences
  and its authorization token with the other credentials. MagicSpot never asks
  for the app's Client Secret.
- Spotify Web API requests load library and catalogue data and control playback.
- Librespot connects to Spotify for local playback and Spotify Connect.
- When Spotify has no lyrics, MagicSpot can request them from LRCLIB using the
  track title, artist, album, and duration. Lyrics are cached locally.
- Album art and audio may be cached locally within configured limits.
- Opening MilkDrop for the first time may download projectM preset packs from
  GitHub when the preset folder is empty.
- When update checks are enabled, MagicSpot asks GitHub for the latest
  MagicSpot release at most once per day.

Clearing cache files does not remove sign-in credentials. Removing the state
directory signs the application out and removes locally retained session data.
