//! Custom palette editor with live preview and portable JSON import/export.

use egui::{Align, CornerRadius, Frame, Layout, Margin, Stroke, Vec2};

use crate::app::App;
use crate::model::{Action, Page};
use crate::settings::CustomTheme;
use crate::theme::{self, Icon, Palette};

#[derive(Clone)]
struct EditorState {
    draft: CustomTheme,
    original_name: Option<String>,
    import_text: String,
    import_error: Option<String>,
}

pub fn show(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let state_id = egui::Id::new("custom-theme-editor-state");
    let mut state = if app.theme_preview.is_none() {
        initial_state(app)
    } else {
        ui.data(|data| data.get_temp::<EditorState>(state_id))
            .unwrap_or_else(|| initial_state(app))
    };

    fn initial_state(app: &App) -> EditorState {
        let active = app.settings.active_custom_theme.as_ref().and_then(|name| {
            app.settings
                .custom_themes
                .iter()
                .find(|theme| &theme.name == name)
        });
        let draft = app
            .theme_preview
            .clone()
            .or_else(|| active.cloned())
            .unwrap_or_default();
        EditorState {
            original_name: active.map(|theme| theme.name.clone()),
            draft,
            import_text: String::new(),
            import_error: None,
        }
    }

    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            theme::text(ui, "Custom theme editor", theme::bold(28.0), palette.text);
            theme::text(
                ui,
                "Tune the palette while the whole interface previews every change.",
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

    editor_card(app, ui, &mut state);
    ui.add_space(14.0);
    saved_card(app, ui, &mut state);
    ui.add_space(14.0);
    portable_card(app, ui, &mut state);

    ui.data_mut(|data| data.insert_temp(state_id, state));
}

fn editor_card(app: &mut App, ui: &mut egui::Ui, state: &mut EditorState) {
    let palette = app.palette;
    card(ui, &palette, |ui| {
        ui.horizontal(|ui| {
            theme::text(ui, "Live palette", theme::bold(17.0), palette.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::soft_button(ui, &palette, Some(Icon::Refresh), "Reset", false).clicked() {
                    let name = state.draft.name.clone();
                    state.draft = CustomTheme {
                        name,
                        ..CustomTheme::default()
                    };
                    state.import_error = None;
                    app.actions
                        .push(Action::PreviewCustomTheme(state.draft.clone()));
                }
                let valid_name = !state.draft.name.trim().is_empty();
                if ui
                    .add_enabled(
                        valid_name,
                        egui::Button::new(
                            egui::RichText::new("Save theme").color(palette.on_accent),
                        )
                        .fill(palette.accent)
                        .corner_radius(20.0),
                    )
                    .clicked()
                {
                    save_theme(app, state);
                }
            });
        });
        ui.add_space(10.0);

        let mut changed = false;
        setting(
            ui,
            &palette,
            "Theme name",
            "Edit the name, then save to rename this theme.",
            |ui| {
                changed |= ui
                    .add(
                        egui::TextEdit::singleline(&mut state.draft.name)
                            .font(theme::regular(13.5))
                            .desired_width(220.0),
                    )
                    .changed();
            },
        );
        color_setting(
            ui,
            &palette,
            "Accent",
            &mut state.draft.accent,
            &mut changed,
        );
        color_setting(
            ui,
            &palette,
            "Surface",
            &mut state.draft.surface,
            &mut changed,
        );
        color_setting(ui, &palette, "Panel", &mut state.draft.panel, &mut changed);
        color_setting(ui, &palette, "Text", &mut state.draft.text, &mut changed);
        setting(
            ui,
            &palette,
            "Blur",
            "Softens menu and floating-surface shadows.",
            |ui| {
                changed |= ui
                    .add(egui::Slider::new(&mut state.draft.blur, 0..=100).suffix("%"))
                    .changed();
            },
        );
        setting(
            ui,
            &palette,
            "Transparency",
            "Lets the window colour show through layered panels and surfaces.",
            |ui| {
                changed |= ui
                    .add(egui::Slider::new(&mut state.draft.transparency, 0..=60).suffix("%"))
                    .changed();
            },
        );

        if changed {
            app.actions
                .push(Action::PreviewCustomTheme(state.draft.clone()));
        }

        preview(ui, &state.draft);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            theme::subtle(
                ui,
                &palette,
                "Preview changes are discarded when you leave unless saved.",
            );
        });
    });
}

fn save_theme(app: &mut App, state: &mut EditorState) {
    let saved_name = state.draft.name.trim().to_string();
    app.actions.push(Action::SaveCustomTheme {
        theme: state.draft.clone(),
        original_name: state.original_name.clone(),
    });
    state.draft.name = saved_name.clone();
    state.original_name = Some(saved_name);
}

fn saved_card(app: &mut App, ui: &mut egui::Ui, state: &mut EditorState) {
    let palette = app.palette;
    card(ui, &palette, |ui| {
        theme::text(ui, "Saved themes", theme::bold(17.0), palette.text);
        ui.add_space(4.0);
        if app.settings.custom_themes.is_empty() {
            theme::subtle(ui, &palette, "No custom themes saved yet.");
            return;
        }
        let themes = app.settings.custom_themes.clone();
        for saved in themes {
            ui.horizontal(|ui| {
                let selected = state.original_name.as_deref() == Some(saved.name.as_str());
                if theme::soft_button(ui, &palette, None, &saved.name, selected).clicked() {
                    state.draft = saved.clone();
                    state.original_name = Some(saved.name.clone());
                    state.import_error = None;
                    app.actions.push(Action::PreviewCustomTheme(saved.clone()));
                }
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if theme::soft_button(ui, &palette, None, "Delete", false).clicked() {
                        app.actions
                            .push(Action::DeleteCustomTheme(saved.name.clone()));
                        if selected {
                            state.draft = CustomTheme::default();
                            state.original_name = None;
                            app.actions
                                .push(Action::PreviewCustomTheme(state.draft.clone()));
                        }
                    }
                });
            });
        }
    });
}

fn portable_card(app: &mut App, ui: &mut egui::Ui, state: &mut EditorState) {
    let palette = app.palette;
    card(ui, &palette, |ui| {
        theme::text(ui, "Import and export", theme::bold(17.0), palette.text);
        theme::subtle(
            ui,
            &palette,
            "Themes use a small JSON format that can be copied between MagicSpot installations.",
        );
        ui.add_space(8.0);
        if theme::soft_button(
            ui,
            &palette,
            Some(Icon::Copy),
            "Export current theme",
            false,
        )
        .clicked()
            && let Ok(text) = serde_json::to_string_pretty(&state.draft.clone().normalized())
        {
            app.actions.push(Action::CopyText {
                text,
                confirmation: "Theme JSON copied".into(),
            });
        }
        ui.add_space(8.0);
        ui.add(
            egui::TextEdit::multiline(&mut state.import_text)
                .hint_text("Paste theme JSON here")
                .font(egui::TextStyle::Monospace)
                .desired_width(f32::INFINITY)
                .desired_rows(5),
        );
        ui.horizontal(|ui| {
            if theme::pill_button(ui, &palette, "Import JSON", false).clicked() {
                match serde_json::from_str::<CustomTheme>(&state.import_text) {
                    Ok(theme) => {
                        state.draft = theme.normalized();
                        state.original_name = None;
                        state.import_error = None;
                        app.actions
                            .push(Action::PreviewCustomTheme(state.draft.clone()));
                    }
                    Err(error) => state.import_error = Some(format!("Invalid theme JSON: {error}")),
                }
            }
            if let Some(error) = &state.import_error {
                theme::text(ui, error, theme::regular(12.0), palette.danger);
            }
        });
    });
}

fn color_setting(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    color: &mut [u8; 3],
    changed: &mut bool,
) {
    setting(ui, palette, label, "", |ui| {
        *changed |= ui.color_edit_button_srgb(color).changed();
        theme::text(
            ui,
            format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]),
            theme::regular(12.5),
            palette.secondary,
        );
    });
}

fn setting(
    ui: &mut egui::Ui,
    palette: &Palette,
    label: &str,
    detail: &str,
    control: impl FnOnce(&mut egui::Ui),
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.set_width((ui.available_width() - 280.0).max(0.0));
            theme::text(ui, label, theme::medium(14.0), palette.text);
            if !detail.is_empty() {
                theme::subtle(ui, palette, detail);
            }
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.horizontal(control);
        });
    });
    ui.add_space(10.0);
}

fn preview(ui: &mut egui::Ui, custom: &CustomTheme) {
    let palette = Palette::from_custom(true, custom);
    ui.add_space(4.0);
    let width = ui.available_width().min(520.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(width, 112.0), egui::Sense::hover());
    ui.painter().rect_filled(rect, 12.0, palette.window);
    let panel = rect.shrink(12.0);
    ui.painter().rect_filled(panel, 9.0, palette.panel);
    ui.painter().text(
        panel.left_top() + Vec2::new(14.0, 16.0),
        egui::Align2::LEFT_TOP,
        "A theme that feels like yours",
        theme::semibold(15.0),
        palette.text,
    );
    let button = egui::Rect::from_min_size(
        panel.left_bottom() + Vec2::new(14.0, -42.0),
        Vec2::new(118.0, 28.0),
    );
    ui.painter().rect_filled(button, 14.0, palette.accent);
    ui.painter().text(
        button.center(),
        egui::Align2::CENTER_CENTER,
        "Preview",
        theme::medium(12.5),
        palette.on_accent,
    );
    let surface = egui::Rect::from_min_size(
        panel.right_bottom() - Vec2::new(160.0, 42.0),
        Vec2::new(146.0, 28.0),
    );
    ui.painter().rect_filled(surface, 7.0, palette.surface);
}

fn card(ui: &mut egui::Ui, palette: &Palette, contents: impl FnOnce(&mut egui::Ui)) {
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
            contents(ui);
        });
}
