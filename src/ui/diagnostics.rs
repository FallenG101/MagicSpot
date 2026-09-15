//! Connection state, recovery actions, and a secret-free support report.

use egui::{Align, CornerRadius, Frame, Layout, Margin, Stroke};

use crate::app::App;
use crate::backend::{AuthStatus, LocalPlayback};
use crate::model::{Action, Page};
use crate::theme::{self, Icon, Palette};

use super::widgets;

#[derive(Clone, Copy)]
enum State {
    Ready,
    Working,
    Attention,
    Pending,
}

impl State {
    fn icon(self) -> Icon {
        match self {
            Self::Ready => Icon::CircleCheck,
            Self::Working => Icon::Loader,
            Self::Attention => Icon::CircleAlert,
            Self::Pending => Icon::Info,
        }
    }

    fn color(self, palette: &Palette) -> egui::Color32 {
        match self {
            Self::Ready => palette.accent,
            Self::Working => palette.accent,
            Self::Attention => palette.danger,
            Self::Pending => palette.dim,
        }
    }
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            theme::text(
                ui,
                "Connection diagnostics",
                theme::bold(28.0),
                palette.text,
            );
            theme::text(
                ui,
                "See every setup stage and fix failed steps without restarting MagicSpot.",
                theme::regular(13.5),
                palette.secondary,
            );
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if theme::soft_button(ui, &palette, Some(Icon::Settings), "Settings", false).clicked() {
                app.actions.push(Action::Open(Page::Settings));
            }
        });
    });
    ui.add_space(14.0);

    Frame::new()
        .fill(
            palette
                .surface
                .gamma_multiply(if palette.dark { 0.7 } else { 1.0 }),
        )
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(theme::RADIUS + 2))
        .inner_margin(Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().min(760.0));

            let (state, title, detail, retry) = match &app.auth {
                AuthStatus::Starting | AuthStatus::Connecting => (
                    State::Working,
                    "Signing in",
                    "Connecting the Spotify Web API…".to_string(),
                    None,
                ),
                AuthStatus::WaitingForBrowser { .. } => (
                    State::Working,
                    "Signing in",
                    "Finish the account authorization in your browser.".to_string(),
                    None,
                ),
                AuthStatus::Connected { username } => (
                    State::Ready,
                    "Spotify account",
                    format!("Signed in as {username}."),
                    None,
                ),
                AuthStatus::Failed(error) => (
                    State::Attention,
                    "Spotify sign-in failed",
                    error.clone(),
                    Some(Action::SignIn),
                ),
                AuthStatus::SignedOut => (
                    State::Pending,
                    "Spotify account",
                    "Sign in to use Spotify.".to_string(),
                    Some(Action::SignIn),
                ),
            };
            status_row(app, ui, state, title, &detail, retry, "Try again");

            let (state, title, detail, retry) = match &app.local_playback {
                LocalPlayback::Authorizing => (
                    State::Working,
                    "Authorizing playback",
                    "Finish the one-time playback authorization in your browser.".to_string(),
                    None,
                ),
                LocalPlayback::Connecting => (
                    State::Working,
                    "Connecting local playback",
                    "Starting this computer as a Spotify Connect device…".to_string(),
                    None,
                ),
                LocalPlayback::Ready { .. } => (
                    State::Ready,
                    "Local playback",
                    format!("{} is ready.", app.settings.device_name),
                    None,
                ),
                LocalPlayback::Failed(error) => (
                    State::Attention,
                    "Local playback failed",
                    error.clone(),
                    Some(Action::EnablePlayback),
                ),
                LocalPlayback::Unavailable => (
                    State::Pending,
                    "Local playback",
                    "Optional. Spotify Premium is required to play on this computer.".to_string(),
                    Some(Action::EnablePlayback),
                ),
            };
            status_row(app, ui, state, title, &detail, retry, "Set up again");

            let discovery_working = app.devices_loading || app.receivers_loading;
            let discovery_error = app
                .devices_error
                .as_ref()
                .or(app.receivers_error.as_ref())
                .cloned();
            let (state, title, detail) = if discovery_working {
                (
                    State::Working,
                    "Discovering devices",
                    "Checking Spotify Connect and your local network…".to_string(),
                )
            } else if let Some(error) = discovery_error {
                (State::Attention, "Device discovery failed", error)
            } else {
                let count = app.devices.len() + app.receivers.len();
                if count == 0 {
                    (
                        State::Pending,
                        "Device discovery",
                        "No devices found. Open Spotify on another device, then scan again."
                            .to_string(),
                    )
                } else {
                    (
                        State::Ready,
                        "Device discovery",
                        format!(
                            "Found {count} available device{}.",
                            if count == 1 { "" } else { "s" }
                        ),
                    )
                }
            };
            status_row(
                app,
                ui,
                state,
                title,
                &detail,
                (!discovery_working).then_some(Action::RefreshDevices),
                "Scan again",
            );

            let transfer_error = app.transfer_error.clone();
            let (state, title, detail, retry) = if let Some(id) = &app.transfer_pending {
                (
                    State::Working,
                    "Transferring playback",
                    format!("Moving playback to {}…", device_name(app, id)),
                    None,
                )
            } else if let Some((id, error)) = transfer_error {
                (
                    State::Attention,
                    "Playback transfer failed",
                    error,
                    Some(Action::Transfer(id)),
                )
            } else if let Some(device) = active_device_name(app) {
                (
                    State::Ready,
                    "Playback device",
                    format!("Listening on {device}."),
                    None,
                )
            } else {
                (
                    State::Pending,
                    "Playback device",
                    "Nothing is active yet. Choose a device from the player bar.".to_string(),
                    None,
                )
            };
            status_row(app, ui, state, title, &detail, retry, "Retry transfer");
        });

    ui.add_space(14.0);
    Frame::new()
        .fill(palette.panel)
        .stroke(Stroke::new(1.0, palette.outline))
        .corner_radius(CornerRadius::same(theme::RADIUS + 2))
        .inner_margin(Margin::symmetric(20, 16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width().min(760.0));
            theme::text(ui, "Support details", theme::bold(17.0), palette.text);
            theme::subtle(
                ui,
                &palette,
                "The copied report contains connection states and counts, never tokens or credentials.",
            );
            ui.add_space(8.0);
            widgets::setting_row(
                ui,
                &palette,
                "Web API access",
                if app.settings.web_client_id.is_some() {
                    "Personal Spotify app configured"
                } else {
                    "Shared MagicSpot app"
                },
                |ui| {
                    if theme::soft_button(ui, &palette, Some(Icon::Copy), "Copy report", false)
                        .clicked()
                    {
                        app.actions.push(Action::CopyText {
                            text: report(app),
                            confirmation: "Connection report copied".into(),
                        });
                    }
                },
            );
            widgets::setting_row(
                ui,
                &palette,
                "Personal app redirect URI",
                crate::auth::WEB_REDIRECT_URI,
                |ui| {
                    if theme::soft_button(ui, &palette, Some(Icon::Copy), "Copy", false)
                        .clicked()
                    {
                        app.actions.push(Action::CopyText {
                            text: crate::auth::WEB_REDIRECT_URI.into(),
                            confirmation: "Redirect URI copied".into(),
                        });
                    }
                },
            );
        });
}

fn status_row(
    app: &mut App,
    ui: &mut egui::Ui,
    state: State,
    title: &str,
    detail: &str,
    retry: Option<Action>,
    retry_label: &str,
) {
    let palette = app.palette;
    ui.horizontal(|ui| {
        let color = state.color(&palette);
        if matches!(state, State::Working) {
            theme::spinner(ui, 18.0, color);
        } else {
            ui.add(state.icon().image(color, 19.0));
        }
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 150.0).max(0.0));
            theme::text(ui, title, theme::semibold(14.0), palette.text);
            ui.add(
                egui::Label::new(
                    egui::RichText::new(detail)
                        .font(theme::regular(12.5))
                        .color(if matches!(state, State::Attention) {
                            palette.danger
                        } else {
                            palette.secondary
                        }),
                )
                .wrap(),
            );
        });
        if let Some(action) = retry {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::pill_button(ui, &palette, retry_label, false).clicked() {
                    app.actions.push(action);
                }
            });
        }
    });
    ui.add_space(13.0);
}

fn device_name(app: &App, id: &str) -> String {
    app.devices
        .iter()
        .find(|device| device.id.as_deref() == Some(id))
        .map(|device| device.name.clone())
        .unwrap_or_else(|| "the selected device".into())
}

fn active_device_name(app: &App) -> Option<String> {
    if app.local.is_active() {
        return Some(format!("{} (this computer)", app.settings.device_name));
    }
    app.devices
        .iter()
        .find(|device| device.is_active)
        .map(|device| device.name.clone())
}

fn report(app: &App) -> String {
    let auth = match &app.auth {
        AuthStatus::Starting => "starting",
        AuthStatus::SignedOut => "signed out",
        AuthStatus::WaitingForBrowser { .. } => "waiting for browser",
        AuthStatus::Connecting => "connecting",
        AuthStatus::Connected { .. } => "connected",
        AuthStatus::Failed(_) => "failed",
    };
    let playback = match &app.local_playback {
        LocalPlayback::Unavailable => "unavailable",
        LocalPlayback::Authorizing => "authorizing",
        LocalPlayback::Connecting => "connecting",
        LocalPlayback::Ready { .. } => "ready",
        LocalPlayback::Failed(_) => "failed",
    };
    format!(
        "MagicSpot {}\nOS: {} {}\nWeb API: {} ({})\nLocal playback: {}\nSpotify devices: {}\nNetwork receivers: {}\nDevice discovery: {}\nPlayback transfer: {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH,
        auth,
        if app.settings.web_client_id.is_some() {
            "personal app"
        } else {
            "shared app"
        },
        playback,
        app.devices.len(),
        app.receivers.len(),
        if app.devices_loading || app.receivers_loading {
            "in progress"
        } else if app.devices_error.is_some() || app.receivers_error.is_some() {
            "failed"
        } else {
            "complete"
        },
        if app.transfer_pending.is_some() {
            "in progress"
        } else if app.transfer_error.is_some() {
            "failed"
        } else {
            "idle"
        },
    )
}
