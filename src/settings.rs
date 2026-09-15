//! User preferences, stored as one readable JSON file.

use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeChoice {
    #[default]
    Dark,
    Light,
    System,
    Oled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LyricsAlignment {
    #[default]
    Left,
    Center,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LyricsFont {
    #[default]
    Inter,
    Manrope,
    Lora,
}

impl LyricsFont {
    pub const ALL: [Self; 3] = [Self::Inter, Self::Manrope, Self::Lora];

    pub fn label(self) -> &'static str {
        match self {
            Self::Inter => "Inter",
            Self::Manrope => "Manrope",
            Self::Lora => "Lora",
        }
    }
}

impl LyricsAlignment {
    pub const ALL: [Self; 2] = [Self::Left, Self::Center];

    pub fn label(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Center => "Center",
        }
    }
}

/// Accent and surface character, independent of light/dark mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorTheme {
    #[default]
    Aqua,
    Violet,
    Rose,
    Amber,
    Neutral,
    /// Kept only so settings written by earlier MagicSpot builds can migrate.
    Oled,
}

impl ColorTheme {
    pub const ALL: [Self; 5] = [
        Self::Aqua,
        Self::Violet,
        Self::Rose,
        Self::Amber,
        Self::Neutral,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Aqua => "Aqua",
            Self::Violet => "Violet",
            Self::Rose => "Rose",
            Self::Amber => "Amber",
            Self::Neutral => "Neutral gray",
            Self::Oled => "Aqua",
        }
    }
}

/// A user-authored interface palette. Colors stay as plain RGB so exported
/// themes are portable; transparency is applied uniformly to layered panels.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomTheme {
    pub name: String,
    pub accent: [u8; 3],
    pub surface: [u8; 3],
    pub panel: [u8; 3],
    pub text: [u8; 3],
    /// Visual softness for menus and floating surfaces, 0..=100.
    pub blur: u8,
    /// Transparency of layered panels, 0..=60.
    pub transparency: u8,
}

impl Default for CustomTheme {
    fn default() -> Self {
        Self {
            name: "My theme".into(),
            accent: [0x55, 0xe6, 0xdf],
            surface: [0x17, 0x25, 0x2a],
            panel: [0x0f, 0x1a, 0x1e],
            text: [0xf2, 0xf4, 0xf6],
            blur: 50,
            transparency: 8,
        }
    }
}

impl CustomTheme {
    pub fn normalized(mut self) -> Self {
        self.name = self.name.trim().to_string();
        if self.name.is_empty() {
            self.name = "My theme".into();
        }
        self.name.truncate(48);
        self.blur = self.blur.min(100);
        self.transparency = self.transparency.min(60);
        self
    }
}

/// Mini-player visualizer mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VisMode {
    #[default]
    Bars,
    Scope,
    Off,
}

impl VisMode {
    /// Next mode in the display's click cycle.
    pub fn next(self) -> Self {
        match self {
            Self::Bars => Self::Scope,
            Self::Scope => Self::Off,
            Self::Off => Self::Bars,
        }
    }
}

impl ThemeChoice {
    pub const ALL: [ThemeChoice; 4] = [Self::Dark, Self::Light, Self::System, Self::Oled];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dark => "Dark",
            Self::Light => "Light",
            Self::System => "Follow system",
            Self::Oled => "OLED",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// The Spotify Connect name other devices see.
    pub device_name: String,
    /// 96, 160, or 320 kbps.
    pub bitrate: u16,
    pub normalisation: bool,
    pub autoplay: bool,
    pub gapless: bool,
    /// librespot backend name; `None` picks the platform default.
    pub audio_backend: Option<String>,
    pub audio_device: Option<String>,
    /// Windows output buffer in milliseconds. Smaller values may click under
    /// load; larger values delay playback controls.
    /// See [`crate::sink::DEFAULT_BUFFER_MS`].
    #[serde(default = "default_buffer_ms")]
    pub audio_buffer_ms: u32,
    pub audio_cache: bool,
    pub audio_cache_mb: u64,
    pub theme: ThemeChoice,
    pub color_theme: ColorTheme,
    /// User-authored palettes, stored locally with the other appearance prefs.
    pub custom_themes: Vec<CustomTheme>,
    /// Name of the selected custom palette, or `None` for a built-in palette.
    pub active_custom_theme: Option<String>,
    /// Tint the interface with the colour of the playing album's art.
    pub accent_from_art: bool,
    /// Last local volume, 0..=65535.
    pub volume: u16,
    /// Whether the library sidebar is visible.
    pub sidebar_visible: bool,
    /// The playing album's art docked large at the sidebar's bottom.
    pub art_expanded: bool,
    /// Use compact single-line rows without cover art in the sidebar.
    pub sidebar_compact: bool,
    pub sidebar_width: f32,
    pub lyrics_width: f32,
    /// Lyrics text size in logical points, independent of interface zoom.
    pub lyrics_font_size: u8,
    /// Space after each lyric line, as a percentage of the font size.
    pub lyrics_line_spacing: u8,
    pub lyrics_alignment: LyricsAlignment,
    /// Weight and character of the lyrics face.
    pub lyrics_font: LyricsFont,
    /// Draw a restrained halo behind lyric text.
    pub lyrics_glow: bool,
    /// Strength of album-art colour in the lyrics surfaces, 0..=100.
    pub lyrics_tint_strength: u8,
    /// Estimate word progress between line timestamps for a karaoke effect.
    pub lyrics_word_progress_beta: bool,
    pub queue_width: f32,
    /// Use compact single-line rows without cover art in track lists.
    pub tracklist_compact: bool,
    pub search_history: Vec<String>,
    pub show_shortcut_hints: bool,
    /// An optional personal Spotify Web API application id. The shared
    /// application remains active for coverage when this is present.
    pub web_client_id: Option<String>,
    /// When the slow-Spotify personal-app reminder was last shown.
    pub personal_app_nudge_at: Option<String>,
    /// Local playback has been authorized at least once on this machine, so
    /// the app can resume it silently instead of prompting.
    pub playback_authorized: bool,
    /// Closing the window hides to the tray and keeps the music playing.
    pub keep_playing_in_background: bool,
    /// Ask GitHub once a day whether a newer release exists.
    pub check_for_updates: bool,
    /// Context URIs pinned to the top of the sidebar, in pin order.
    pub pinned_contexts: Vec<String>,
    /// The sidebar's own playlist order, set by dragging rows. Empty means
    /// the automatic order: the pinned block first, then recently played.
    pub sidebar_order: Vec<String>,
    /// Interface zoom, egui's zoom factor; Ctrl+plus/minus changes it.
    pub zoom: f32,
    /// The Winamp window is open.
    pub winamp_window: bool,
    /// Skin file or folder name. `None` selects the built-in skin.
    pub skin: Option<String>,
    /// Screen pixels per skin pixel; `None` picks double size for the
    /// display.
    pub skin_scale: Option<u8>,
    /// The Winamp window stays above other windows.
    pub winamp_on_top: bool,
    /// The mini player's visualiser: bars, scope, or off.
    pub vis: VisMode,
    /// The playlist window is open under the mini player.
    pub playlist_open: bool,
    /// How tall the playlist window is, in skin pixels.
    pub playlist_height: u32,
    /// The equalizer window is open under the mini player.
    pub eq_open: bool,
    /// The equalizer shapes local playback.
    pub eq_on: bool,
    /// The preamp, in decibels, never above zero.
    pub eq_preamp_db: f32,
    /// The ten bands, in decibels, 60 Hz to 16 kHz.
    pub eq_bands_db: [f32; 10],
    /// The balance, -1 all left to 1 all right.
    pub balance: f32,
    /// Play both channels the same.
    pub mono: bool,
    /// The playlist window is rolled up to its title bar.
    pub playlist_shaded: bool,
    /// The equalizer window is rolled up to its title bar.
    pub eq_shaded: bool,
    /// The main window is rolled up to its title bar.
    pub winamp_shaded: bool,
    /// The MilkDrop window is open (its own window, not part of the skin).
    pub milkdrop_open: bool,
    /// How long each preset plays before the next, in seconds.
    pub milkdrop_seconds: u32,
    /// How many frames a second the MilkDrop window draws; 0 is uncapped.
    pub milkdrop_fps: u32,
    /// Last reported MilkDrop screen refresh rate. The first value sets the
    /// default frame rate; this field is not directly configurable.
    pub milkdrop_screen_hz: u32,
    /// The picture's inner resolution: 1 full, 2 half, 4 quarter.
    pub milkdrop_scale: u32,
    /// The MilkDrop window fills the screen.
    pub milkdrop_fullscreen: bool,
    /// The MilkDrop window's size in logical points, when not full-screen.
    pub milkdrop_size: [f32; 2],
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            device_name: "MagicSpot".to_string(),
            bitrate: 320,
            normalisation: false,
            autoplay: true,
            gapless: true,
            audio_backend: None,
            audio_device: None,
            audio_buffer_ms: default_buffer_ms(),
            audio_cache: true,
            audio_cache_mb: 1024,
            theme: ThemeChoice::Dark,
            color_theme: ColorTheme::Aqua,
            custom_themes: Vec::new(),
            active_custom_theme: None,
            accent_from_art: true,
            volume: (u16::MAX as u32 * 70 / 100) as u16,
            sidebar_visible: true,
            art_expanded: false,
            sidebar_compact: false,
            sidebar_width: 250.0,
            lyrics_width: 360.0,
            lyrics_font_size: 26,
            lyrics_line_spacing: 65,
            lyrics_alignment: LyricsAlignment::Left,
            lyrics_font: LyricsFont::Inter,
            lyrics_glow: false,
            lyrics_tint_strength: 22,
            lyrics_word_progress_beta: false,
            queue_width: 360.0,
            tracklist_compact: false,
            search_history: Vec::new(),
            show_shortcut_hints: true,
            web_client_id: None,
            personal_app_nudge_at: None,
            playback_authorized: false,
            keep_playing_in_background: true,
            check_for_updates: false,
            pinned_contexts: Vec::new(),
            sidebar_order: Vec::new(),
            zoom: 1.0,
            winamp_window: false,
            skin: None,
            skin_scale: None,
            winamp_on_top: false,
            vis: VisMode::default(),
            playlist_open: false,
            playlist_height: 174,
            eq_open: false,
            eq_on: false,
            eq_preamp_db: 0.0,
            eq_bands_db: [0.0; 10],
            balance: 0.0,
            mono: false,
            playlist_shaded: false,
            eq_shaded: false,
            winamp_shaded: false,
            milkdrop_open: false,
            milkdrop_seconds: crate::milkdrop::DEFAULT_SECONDS,
            milkdrop_fps: crate::milkdrop::DEFAULT_FPS,
            milkdrop_screen_hz: 0,
            milkdrop_scale: 1,
            milkdrop_fullscreen: false,
            milkdrop_size: crate::milkdrop::DEFAULT_SIZE,
        }
    }
}

fn default_buffer_ms() -> u32 {
    crate::sink::DEFAULT_BUFFER_MS
}

impl Settings {
    pub fn load(path: &Path) -> Self {
        let mut settings = match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text).unwrap_or_else(|error| {
                log::warn!("settings at {} are unreadable: {error}", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        };
        if settings.color_theme == ColorTheme::Oled {
            settings.theme = ThemeChoice::Oled;
            settings.color_theme = ColorTheme::Aqua;
        }
        settings.custom_themes = settings
            .custom_themes
            .drain(..)
            .map(CustomTheme::normalized)
            .collect();
        if settings.active_custom_theme.as_ref().is_some_and(|active| {
            !settings
                .custom_themes
                .iter()
                .any(|theme| &theme.name == active)
        }) {
            settings.active_custom_theme = None;
        }
        settings
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let text = match serde_json::to_string_pretty(self) {
            Ok(text) => text,
            Err(error) => {
                log::warn!("unable to encode settings: {error}");
                return;
            }
        };
        let temporary = path.with_extension("json.tmp");
        let written = std::fs::write(&temporary, text)
            .and_then(|()| crate::util::replace_file(&temporary, path));
        if let Err(error) = written {
            log::warn!("unable to save settings to {}: {error}", path.display());
        }
    }

    pub fn platform_backend(&self) -> Option<String> {
        self.audio_backend.clone().or_else(|| {
            if cfg!(target_os = "linux") {
                Some("pulseaudio".to_string())
            } else {
                None
            }
        })
    }

    pub fn remember_search(&mut self, query: &str) {
        let query = query.trim();
        if query.is_empty() {
            return;
        }
        self.search_history.retain(|entry| entry != query);
        self.search_history.insert(0, query.to_string());
        self.search_history.truncate(12);
    }
}

#[cfg(test)]
mod tests {
    use super::{ColorTheme, CustomTheme, LyricsAlignment, LyricsFont, Settings, ThemeChoice};

    #[test]
    fn lyrics_size_is_backward_compatible_and_persists() {
        let older: super::Settings = serde_json::from_str(r#"{"lyrics_width":420.0}"#).unwrap();
        assert_eq!(older.lyrics_font_size, 26);
        assert_eq!(older.lyrics_width, 420.0);
        assert_eq!(older.lyrics_line_spacing, 65);
        assert_eq!(older.lyrics_alignment, LyricsAlignment::Left);
        assert_eq!(older.lyrics_font, LyricsFont::Inter);
        assert!(!older.lyrics_glow);
        assert_eq!(older.lyrics_tint_strength, 22);
        assert_eq!(older.color_theme, ColorTheme::Aqua);
        assert!(!older.lyrics_word_progress_beta);
        let settings = super::Settings {
            lyrics_font_size: 36,
            ..older
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: super::Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, settings);
    }

    #[test]
    fn older_settings_keep_the_sidebar_visible() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(settings.sidebar_visible);
    }

    #[test]
    fn the_old_oled_accent_migrates_to_oled_appearance() {
        let root = std::env::temp_dir().join(format!(
            "magicspot-oled-settings-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = root.join("settings.json");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(&path, r#"{"theme":"dark","color_theme":"oled"}"#).unwrap();

        let settings = Settings::load(&path);

        assert_eq!(settings.theme, ThemeChoice::Oled);
        assert_eq!(settings.color_theme, ColorTheme::Aqua);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn older_settings_keep_the_winamp_window_closed_and_the_built_in_skin() {
        let settings: Settings = serde_json::from_str(r#"{"zoom": 1.2}"#).unwrap();
        assert!(!settings.winamp_window);
        assert_eq!(settings.skin, None);
        assert_eq!(settings.skin_scale, None);
        assert!(!settings.winamp_on_top);
        assert_eq!(settings.vis, super::VisMode::Bars);
        assert!(!settings.playlist_open);
        assert_eq!(settings.playlist_height, 174);
        assert!(!settings.eq_on);
        assert_eq!(settings.eq_bands_db, [0.0; 10]);
        assert_eq!(settings.balance, 0.0);
        assert!(!settings.mono);
        assert!(!settings.playlist_shaded);
        assert!(!settings.eq_shaded);
        assert!(!settings.winamp_shaded);
    }

    #[test]
    fn the_visualiser_cycles_bars_scope_off() {
        use super::VisMode;
        assert_eq!(VisMode::Bars.next(), VisMode::Scope);
        assert_eq!(VisMode::Scope.next(), VisMode::Off);
        assert_eq!(VisMode::Off.next(), VisMode::Bars);
        let settings: Settings = serde_json::from_str(r#"{"vis": "scope"}"#).unwrap();
        assert_eq!(settings.vis, VisMode::Scope);
    }

    #[test]
    fn a_chosen_skin_round_trips() {
        let settings = Settings {
            winamp_window: true,
            skin: Some("Zaxon.wsz".into()),
            skin_scale: Some(3),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, settings);
    }

    #[test]
    fn hidden_sidebar_round_trips() {
        let settings = Settings {
            sidebar_visible: false,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(!restored.sidebar_visible);
    }

    #[test]
    fn older_settings_default_to_standard_sidebar() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.sidebar_compact);
    }

    #[test]
    fn compact_sidebar_round_trips() {
        let settings = Settings {
            sidebar_compact: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.sidebar_compact);
    }

    #[test]
    fn older_settings_default_to_standard_tracklist() {
        let settings: Settings = serde_json::from_str("{}").unwrap();
        assert!(!settings.tracklist_compact);
    }

    #[test]
    fn compact_tracklist_round_trips() {
        let settings = Settings {
            tracklist_compact: true,
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert!(restored.tracklist_compact);
    }

    #[test]
    fn personal_app_nudge_time_is_backward_compatible_and_round_trips() {
        let older: Settings = serde_json::from_str("{}").unwrap();
        assert_eq!(older.personal_app_nudge_at, None);

        let settings = Settings {
            personal_app_nudge_at: Some("2026-09-03T15:00:00Z".into()),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(
            restored.personal_app_nudge_at,
            settings.personal_app_nudge_at
        );
    }

    #[test]
    fn custom_themes_are_backward_compatible_and_round_trip() {
        let older: Settings = serde_json::from_str("{}").unwrap();
        assert!(older.custom_themes.is_empty());
        assert_eq!(older.active_custom_theme, None);

        let custom = CustomTheme {
            name: "Midnight Drive".into(),
            accent: [12, 34, 56],
            transparency: 24,
            blur: 72,
            ..CustomTheme::default()
        };
        let settings = Settings {
            custom_themes: vec![custom.clone()],
            active_custom_theme: Some(custom.name.clone()),
            ..Settings::default()
        };
        let json = serde_json::to_string(&settings).unwrap();
        let restored: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.custom_themes, vec![custom]);
        assert_eq!(
            restored.active_custom_theme.as_deref(),
            Some("Midnight Drive")
        );
    }

    #[test]
    fn imported_theme_values_are_safely_normalized() {
        let theme: CustomTheme =
            serde_json::from_str(r#"{"name":"  ","blur":255,"transparency":255,"accent":[1,2,3]}"#)
                .unwrap();
        let theme = theme.normalized();
        assert_eq!(theme.name, "My theme");
        assert_eq!(theme.blur, 100);
        assert_eq!(theme.transparency, 60);
        assert_eq!(theme.accent, [1, 2, 3]);
    }
}

/// The last playlist tree received for one account.
///
/// Only folder order is cached. Edit grants must always come from the live
/// session because they can be revoked.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CachedRootlist {
    pub account_id: String,
    pub entries: Vec<crate::player::RootlistEntry>,
}

/// Restorable UI session: what was open when the app last closed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SessionState {
    pub last_page: Option<String>,
    /// Context URIs most recently played, newest first.
    pub recent_contexts: Vec<String>,
    /// What was playing when the app closed, to resume from a cold start.
    pub last_context: Option<String>,
    pub last_track: Option<String>,
    pub last_position_ms: u32,
    /// Manually queued songs to restore with the remembered track.
    ///
    /// Context rows are excluded to prevent duplicates. This replaced the old
    /// `last_queue` field, so sessions using that field restore no added rows.
    pub last_added_queue: Vec<String>,
    /// Queue rows displayed on the next start. Playback restores manual rows
    /// from `last_added_queue`; it does not enqueue this list.
    pub last_queue_rows: Vec<crate::api::models::PlayableItem>,
    /// Sidebar folders rolled up, by their rootlist ids.
    pub collapsed_folders: Vec<String>,
    /// Last good playlist tree, scoped to the account that supplied it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootlist: Option<CachedRootlist>,
    /// Shuffle mode saved across contexts and restarts.
    pub shuffle_on: bool,
    /// Each table's chosen sort, by encoded page, restored at start.
    pub sorts: Vec<(String, crate::model::TableSort)>,
    /// Last window inner size, to restore on next launch.
    pub window_size: Option<[f32; 2]>,
    /// Last window outer position, to restore on next launch.
    pub window_pos: Option<[f32; 2]>,
    /// Whether the queue panel was open.
    pub queue_open: Option<bool>,
    /// Which tab the queue panel showed: `queue` or `recents`.
    pub queue_tab: Option<String>,
    /// Last outer position of the Winamp window.
    pub winamp_pos: Option<[f32; 2]>,
    /// Last outer position of the MilkDrop window.
    pub milkdrop_pos: Option<[f32; 2]>,
}

impl SessionState {
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let text = match serde_json::to_string(self) {
            Ok(text) => text,
            Err(error) => {
                log::warn!("unable to encode session: {error}");
                return;
            }
        };
        let temporary = path.with_extension("json.tmp");
        let written = std::fs::write(&temporary, text)
            .and_then(|()| crate::util::replace_file(&temporary, path));
        if let Err(error) = written {
            log::warn!("unable to save session to {}: {error}", path.display());
        }
    }
}

#[cfg(test)]
mod session_tests {
    use super::{CachedRootlist, SessionState};
    use crate::player::RootlistEntry;

    #[test]
    fn old_sessions_without_a_playlist_tree_remain_readable() {
        let state: SessionState = serde_json::from_str(r#"{"last_page":"home"}"#).unwrap();
        assert_eq!(state.last_page.as_deref(), Some("home"));
        assert_eq!(state.rootlist, None);
    }

    #[test]
    fn the_playlist_tree_round_trips_with_its_account() {
        let state = SessionState {
            rootlist: Some(CachedRootlist {
                account_id: "listener".into(),
                entries: vec![
                    RootlistEntry::FolderStart {
                        id: "folder".into(),
                        name: "Favorites".into(),
                    },
                    RootlistEntry::Playlist("spotify:playlist:one".into()),
                    RootlistEntry::FolderEnd,
                ],
            }),
            ..SessionState::default()
        };

        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(serde_json::from_str::<SessionState>(&json).unwrap(), state);
    }

    #[test]
    fn a_new_session_atomically_replaces_the_previous_one() {
        let root = std::env::temp_dir().join(format!(
            "magicspot-session-test-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = root.join("session.json");
        let state = |page: &str| SessionState {
            last_page: Some(page.into()),
            ..SessionState::default()
        };

        state("home").save(&path);
        state("liked").save(&path);

        assert_eq!(
            SessionState::load(&path).last_page.as_deref(),
            Some("liked")
        );
        assert!(!path.with_extension("json.tmp").exists());
        let _ = std::fs::remove_dir_all(root);
    }
}
