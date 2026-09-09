//! The words of the playing track, in a side panel that follows the song.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Frame, Layout, Margin, Sense};

use crate::app::App;
use crate::model::{Action, Loadable};
use crate::theme::{self, Icon};

use super::widgets;

/// How long a line takes to light up or fade.
const LIGHT_UP_SECONDS: f32 = 0.22;

fn blend(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from(egui::Rgba::from(from) * (1.0 - t) + egui::Rgba::from(to) * t)
}

/// Layered album-colour pools give the panel depth without a blur pass,
/// extra textures, or a continuous animation loop.
fn paint_backdrop(ui: &egui::Ui, base: egui::Color32, tint: egui::Color32) {
    let rect = ui.max_rect();
    let mut mesh = egui::Mesh::default();
    const STEPS: u32 = 20;
    for row in 0..=STEPS {
        let y = row as f32 / STEPS as f32;
        for column in 0..=STEPS {
            let x = column as f32 / STEPS as f32;
            let upper = (-6.0 * ((x - 0.08).powi(2) + (y * 1.35).powi(2))).exp();
            let middle = (-7.0 * ((x - 0.92).powi(2) + ((y - 0.38) * 1.2).powi(2))).exp();
            let lower = (-6.5 * ((x - 0.35).powi(2) + ((y - 0.92) * 1.4).powi(2))).exp();
            let color = blend(
                base,
                tint,
                (upper * 0.62 + middle * 0.48 + lower * 0.34).min(0.68),
            );
            mesh.colored_vertex(
                egui::pos2(
                    rect.left() + x * rect.width(),
                    rect.top() + y * rect.height(),
                ),
                color,
            );
        }
    }
    for row in 0..STEPS {
        for column in 0..STEPS {
            let a = row * (STEPS + 1) + column;
            let b = a + STEPS + 1;
            mesh.add_triangle(a, a + 1, b);
            mesh.add_triangle(a + 1, b + 1, b);
        }
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}

fn track_hero(app: &App, ui: &mut egui::Ui, now: &crate::app::NowPlaying, expanded: bool) {
    let palette = app.palette;
    let tint = app.now_playing_tint().unwrap_or(palette.surface);
    let width = ui.available_width();
    let height = if expanded {
        (ui.available_height() * 0.46).clamp(300.0, 470.0)
    } else {
        (ui.available_height() * 0.50).clamp(260.0, 360.0)
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    widgets::paint_shadow(ui, &palette, rect.shrink(4.0), 24.0);
    let glass = blend(
        palette.panel,
        tint,
        if palette.dark {
            if expanded { 0.38 } else { 0.44 }
        } else {
            0.24
        },
    );
    ui.painter().rect_filled(rect, 24.0, glass);
    ui.painter().rect_stroke(
        rect,
        24.0,
        egui::Stroke::new(
            1.0,
            if palette.dark {
                egui::Color32::from_white_alpha(42)
            } else {
                egui::Color32::from_black_alpha(24)
            },
        ),
        egui::StrokeKind::Inside,
    );
    let glow = egui::Color32::from_rgba_unmultiplied(tint.r(), tint.g(), tint.b(), 42);
    ui.painter().circle_filled(
        egui::pos2(rect.right() - 30.0, rect.top() + 20.0),
        height * 0.42,
        glow,
    );
    ui.painter().line_segment(
        [
            egui::pos2(rect.left() + 26.0, rect.top() + 1.0),
            egui::pos2(rect.right() - 26.0, rect.top() + 1.0),
        ],
        egui::Stroke::new(1.0, egui::Color32::from_white_alpha(52)),
    );

    let content = rect.shrink2(if expanded {
        egui::vec2(30.0, 26.0)
    } else {
        egui::vec2(22.0, 20.0)
    });
    let cover_size = (content.height() - 8.0)
        .min(content.width() * if expanded { 0.36 } else { 0.48 })
        .min(300.0);
    let cover_rect = egui::Rect::from_center_size(
        egui::pos2(content.left() + cover_size / 2.0, content.center().y),
        egui::Vec2::splat(cover_size),
    );
    widgets::paint_shadow(ui, &palette, cover_rect, 18.0);
    widgets::paint_cover(
        ui,
        &palette,
        now.art_url.as_deref().or(now.art_small.as_deref()),
        cover_rect,
        18.0,
        Icon::Music,
        Some(app.backend.art()),
    );
    let text_gap = if expanded { 30.0 } else { 22.0 };
    let text_rect = egui::Rect::from_min_max(
        egui::pos2(cover_rect.right() + text_gap, content.top()),
        content.right_bottom(),
    );
    let mut text_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(text_rect)
            .layout(Layout::top_down(Align::Min)),
    );
    text_ui.add_space((text_rect.height() * if expanded { 0.27 } else { 0.18 }).max(14.0));
    let title_size = if expanded {
        (cover_size * 0.14).clamp(28.0, 42.0)
    } else {
        (cover_size * 0.13).clamp(21.0, 28.0)
    };
    text_ui.add(
        egui::Label::new(
            egui::RichText::new(&now.title)
                .font(theme::bold(title_size))
                .color(palette.text),
        )
        .wrap()
        .selectable(false),
    );
    text_ui.add_space(6.0);
    theme::text(
        &mut text_ui,
        &now.subtitle,
        theme::medium(if expanded { 17.0 } else { 14.0 }),
        palette.secondary,
    );
    if !now.album_name.is_empty() && now.album_name != now.title {
        text_ui.add_space(3.0);
        theme::text(
            &mut text_ui,
            &now.album_name,
            theme::regular(if expanded { 14.0 } else { 12.5 }),
            palette.dim,
        );
    }
}

fn paint_edge_fades(ui: &egui::Ui, rect: egui::Rect, color: egui::Color32) {
    let height = 32.0_f32.min(rect.height() * 0.12);
    for (top, bottom, top_color, bottom_color) in [
        (
            rect.top(),
            rect.top() + height,
            color,
            egui::Color32::TRANSPARENT,
        ),
        (
            rect.bottom() - height,
            rect.bottom(),
            egui::Color32::TRANSPARENT,
            color,
        ),
    ] {
        let fade = egui::Rect::from_min_max(
            egui::pos2(rect.left(), top),
            egui::pos2(rect.right(), bottom),
        );
        let mut mesh = egui::Mesh::default();
        mesh.colored_vertex(fade.left_top(), top_color);
        mesh.colored_vertex(fade.right_top(), top_color);
        mesh.colored_vertex(fade.right_bottom(), bottom_color);
        mesh.colored_vertex(fade.left_bottom(), bottom_color);
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        ui.painter().add(egui::Shape::mesh(mesh));
    }
}

/// Reveals complete words across the time available to a line. Spotify and
/// LRCLIB only supply line timestamps, so this is deliberately an estimate.
fn word_progress_offset(text: &str, position_ms: u32, start_ms: u32, end_ms: u32) -> usize {
    if position_ms < start_ms {
        return 0;
    }
    let words: Vec<(usize, usize)> = text
        .match_indices(|character: char| !character.is_whitespace())
        .fold(Vec::new(), |mut words, (at, character)| {
            let end = at + character.len();
            if let Some((_, held_end)) = words.last_mut()
                && *held_end == at
            {
                *held_end = end;
            } else {
                words.push((at, end));
            }
            words
        });
    if words.is_empty() || end_ms <= start_ms {
        return text.len();
    }
    let elapsed = position_ms.saturating_sub(start_ms).min(end_ms - start_ms);
    let shown = ((u64::from(elapsed) * words.len() as u64) / u64::from(end_ms - start_ms))
        .min(words.len() as u64) as usize;
    if shown == 0 { 0 } else { words[shown - 1].1 }
}

pub fn side_panel(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let tint = app.now_playing_tint().unwrap_or(palette.panel);
    let panel = egui::Panel::right("lyrics-panel")
        .resizable(true)
        .default_size(app.settings.lyrics_width)
        .size_range(theme::SIDE_PANEL_MIN_WIDTH..=920.0)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(palette.panel)
                .inner_margin(Margin::symmetric(20, 16)),
        );
    let response = panel.show(ui, |ui| {
        paint_backdrop(ui, palette.panel, tint);
        let window_controls = super::window_controls_reservation(
            ui.ctx(),
            app.show_queue_panel,
            app.show_lyrics_panel,
            ui.available_width(),
        );
        ui.add_space(window_controls.lyrics_top);
        ui.horizontal(|ui| {
            ui.add_space(4.0);
            theme::text(ui, "Lyrics", theme::semibold(18.0), palette.text);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if theme::icon_button(ui, Icon::X, 18.0, palette.secondary, palette.text, "Close")
                    .clicked()
                {
                    app.actions.push(Action::ToggleLyricsPanel);
                }
                let loaded = matches!(&app.lyrics, Loadable::Loaded(Some(_)));
                if loaded
                    && !app.lyrics_following
                    && theme::pill_button(ui, &palette, "Follow", false).clicked()
                {
                    app.lyrics_following = true;
                    app.lyrics_line_shown = None;
                }
            });
        });
        ui.add_space(12.0);
        if let Some(now) = app.now_playing() {
            let spacious = ui.available_width() >= 560.0;
            track_hero(app, ui, &now, spacious);
            ui.add_space(if spacious { 16.0 } else { 12.0 });
        }
        contents(app, ui);
    });
    let current_width = response.response.rect.width();
    if (app.settings.lyrics_width - current_width).abs() > 1.0 {
        app.settings.lyrics_width = current_width;
        app.lyrics_line_shown = None;
        app.actions.push(Action::SettingsChanged);
    }
}

fn contents(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let Some(now) = app.now_playing() else {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Mic,
            "Nothing playing",
            "Play a song to see its lyrics.",
        );
        return;
    };
    let lyrics = match &app.lyrics {
        Loadable::NotLoaded | Loadable::Loading => {
            widgets::loading_row(ui, &palette);
            return;
        }
        Loadable::Failed(error) => {
            let message = format!("Couldn't fetch the lyrics: {error}");
            ui.add_space(8.0);
            theme::text(ui, message, theme::regular(13.0), palette.secondary);
            ui.add_space(8.0);
            if theme::pill_button(ui, &palette, "Try again", false).clicked() {
                app.request_lyrics();
            }
            return;
        }
        Loadable::Loaded(None) => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Mic,
                "No lyrics",
                "No lyrics found for this track.",
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) if lyrics.instrumental => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Music,
                "Instrumental",
                "No timed lyrics for this track.",
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) => lyrics.clone(),
    };

    let active = lyrics.active_line(now.position_ms);
    let follow = app.lyrics_following && app.lyrics_line_shown != Some(active);
    // Keep the font weight fixed so highlighting never changes line wrapping
    // or shifts the scroll target. Only colour animates, for 220 ms.
    let line_size = f32::from(app.settings.lyrics_font_size.clamp(20, 44));
    let focus_padding = (ui.available_height() * 0.5 - line_size).max(12.0);
    let scroll = egui::ScrollArea::vertical()
        .id_salt("lyrics-scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            // Before the first line there is nothing to highlight, so the
            // panel sits at the top rather than wherever it was left.
            if follow && lyrics.synced && active.is_none() {
                let top = ui.cursor().min;
                ui.scroll_to_rect(
                    egui::Rect::from_min_size(top, egui::vec2(1.0, 1.0)),
                    Some(Align::Min),
                );
            }
            ui.add_space(if lyrics.synced { focus_padding } else { 20.0 });
            for (index, line) in lyrics.lines.iter().enumerate() {
                let is_active = active == Some(index);
                let lit = ui.ctx().animate_bool_with_time(
                    egui::Id::new("lyric-line").with(&now.uri).with(index),
                    is_active,
                    LIGHT_UP_SECONDS,
                );
                let distance = active.map_or(0, |current| current.abs_diff(index));
                let quiet = if lyrics.synced && active.is_some() {
                    blend(
                        palette.secondary,
                        palette.panel,
                        (distance as f32 * 0.08).min(0.3),
                    )
                } else {
                    palette.secondary
                };
                let color = blend(quiet, palette.text, lit);
                let font = theme::bold(line_size);
                // A timed line with no words is the band playing on.
                let text = if line.text.is_empty() && lyrics.synced {
                    "\u{266a}"
                } else {
                    line.text.as_str()
                };
                let sense = if lyrics.synced {
                    Sense::click()
                } else {
                    Sense::hover()
                };
                let response = if crate::bidi::is_rtl(text) {
                    let galley = crate::bidi::layout(
                        ui.painter(),
                        text,
                        font,
                        color,
                        ui.available_width(),
                        usize::MAX,
                        None,
                    );
                    ui.add(egui::Label::new(galley).sense(sense))
                } else if is_active && lyrics.synced && app.settings.lyrics_word_progress_beta {
                    let start = line.at_ms.unwrap_or(now.position_ms);
                    let end = lyrics
                        .lines
                        .get(index + 1)
                        .and_then(|next| next.at_ms)
                        .unwrap_or(now.duration_ms)
                        .max(start + 1);
                    let offset = word_progress_offset(text, now.position_ms, start, end);
                    let mut job = LayoutJob::default();
                    job.wrap.max_width = ui.available_width();
                    job.append(
                        &text[..offset],
                        0.0,
                        TextFormat {
                            font_id: font.clone(),
                            color: palette.text,
                            ..Default::default()
                        },
                    );
                    job.append(
                        &text[offset..],
                        0.0,
                        TextFormat {
                            font_id: font,
                            color: palette.dim,
                            ..Default::default()
                        },
                    );
                    ui.add(egui::Label::new(job).sense(sense))
                } else {
                    ui.add(
                        egui::Label::new(egui::RichText::new(text).font(font).color(color))
                            .sense(sense),
                    )
                };
                let rect = response.rect;
                if lyrics.synced {
                    let response = response.on_hover_cursor(egui::CursorIcon::PointingHand);
                    if response.clicked()
                        && let Some(at_ms) = line.at_ms
                    {
                        app.actions.push(Action::Seek(at_ms));
                        app.lyrics_following = true;
                    }
                }
                if is_active && follow {
                    ui.scroll_to_rect(rect, Some(Align::Center));
                }
                ui.add_space(line_size * 0.65);
            }
            // Words without timing can only be followed by the clock: sit
            // at the part of the text the song is probably at.
            if app.lyrics_following && !lyrics.synced && now.duration_ms > 0 {
                let fraction =
                    (f64::from(now.position_ms) / f64::from(now.duration_ms)).clamp(0.0, 1.0);
                let content = ui.min_rect();
                let y = content.top() + content.height() * fraction as f32;
                ui.scroll_to_rect(
                    egui::Rect::from_min_max(
                        egui::pos2(content.left(), y),
                        egui::pos2(content.right(), y + 1.0),
                    ),
                    Some(Align::Center),
                );
            }
            ui.add_space(if lyrics.synced { focus_padding } else { 60.0 });
        });
    // Scrolling by hand means the reader wants to look elsewhere; the
    // Follow button in the header picks the song back up.
    if ui.rect_contains_pointer(scroll.inner_rect)
        && ui.input(|input| input.smooth_scroll_delta.y != 0.0)
    {
        app.lyrics_following = false;
    }
    let fade = blend(
        palette.panel,
        app.now_playing_tint().unwrap_or(palette.panel),
        0.16,
    );
    paint_edge_fades(ui, scroll.inner_rect, fade);
    app.lyrics_line_shown = Some(active);
}

#[cfg(test)]
mod tests {
    use super::word_progress_offset;

    #[test]
    fn estimated_word_progress_reveals_complete_words() {
        let text = "one two three";
        assert_eq!(word_progress_offset(text, 999, 1_000, 4_000), 0);
        assert_eq!(
            &text[..word_progress_offset(text, 2_000, 1_000, 4_000)],
            "one"
        );
        assert_eq!(word_progress_offset(text, 4_000, 1_000, 4_000), text.len());
    }

    #[test]
    fn estimated_word_progress_keeps_utf8_boundaries() {
        let text = "déjà vu";
        let offset = word_progress_offset(text, 1_500, 1_000, 2_000);
        assert_eq!(&text[..offset], "déjà");
    }
}
