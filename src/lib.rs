//! MagicSpot's internals, exposed so diagnostics and tests can reach them.

/// User-facing app name. Local builds may override this without changing the
/// packaged product identity used by GitHub releases.
pub const DISPLAY_NAME: &str = match option_env!("MAGICSPOT_DISPLAY_NAME") {
    Some(name) => name,
    None => "MagicSpot",
};

pub mod api;
pub mod app;
pub mod auth;
pub mod backend;
pub mod bidi;
#[cfg(any(test, feature = "demo"))]
pub mod demo;
pub mod eq;
pub mod history;
pub mod images;
pub mod limiter;
pub mod link;
pub mod lyrics;
#[cfg(target_os = "macos")]
pub mod mac_fonts;
#[cfg(target_os = "macos")]
pub mod mac_links;
#[cfg(target_os = "macos")]
pub mod mac_menu;
pub mod media;
#[cfg(target_os = "linux")]
#[path = "mpris.rs"]
pub mod media_controls;
#[cfg(not(target_os = "linux"))]
#[path = "media_native.rs"]
pub mod media_controls;
pub mod model;
pub mod opener;
pub mod paths;
pub mod player;
pub mod resample;
pub mod settings;
pub mod single_instance;
pub mod sink;
pub mod system_fonts;
pub mod theme;
#[cfg(target_os = "linux")]
pub mod tray;
#[cfg(not(target_os = "linux"))]
#[path = "tray_native.rs"]
pub mod tray;
pub mod ui;
pub mod updates;
pub mod util;
pub mod vis;
pub mod window;
pub mod zeroconf;
