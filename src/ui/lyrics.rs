//! The words of the playing track, in a side panel that follows the song.

use egui::text::{LayoutJob, TextFormat};
use egui::{Align, Frame, Layout, Margin, Sense};

use crate::app::App;
use crate::model::{Action, Loadable};
use crate::settings::{LyricsAlignment, LyricsFont};
use crate::theme::{self, Icon};

use super::widgets;

/// How long a line takes to light up or fade.
const LIGHT_UP_SECONDS: f32 = 0.22;

fn blend(from: egui::Color32, to: egui::Color32, t: f32) -> egui::Color32 {
    let t = t.clamp(0.0, 1.0);
    egui::Color32::from(egui::Rgba::from(from) * (1.0 - t) + egui::Rgba::from(to) * t)
}

fn track_hero(app: &App, ui: &mut egui::Ui, now: &crate::app::NowPlaying, expanded: bool) {
    let palette = app.palette;
    let width = ui.available_width();
    let height = if expanded {
        (ui.available_height() * 0.30).clamp(220.0, 300.0)
    } else {
        (ui.available_height() * 0.25).clamp(176.0, 230.0)
    };
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), Sense::hover());
    let content = rect.shrink2(if expanded {
        egui::vec2(18.0, 12.0)
    } else {
        egui::vec2(10.0, 10.0)
    });
    let cover_size = (content.height() - 4.0)
        .min(content.width() * if expanded { 0.36 } else { 0.48 })
        .min(if expanded { 220.0 } else { 170.0 });
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
    text_ui.add_space((text_rect.height() * if expanded { 0.24 } else { 0.18 }).max(10.0));
    let title_size = if expanded {
        (cover_size * 0.14).clamp(26.0, 34.0)
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

fn lyrics_font(choice: LyricsFont, size: f32) -> egui::FontId {
    match choice {
        LyricsFont::Inter => theme::bold(size),
        LyricsFont::Manrope => theme::manrope_lyrics(size),
        LyricsFont::Lora => theme::lora_lyrics(size),
    }
}

fn lyric_galley(
    ui: &egui::Ui,
    text: &str,
    font: egui::FontId,
    color: egui::Color32,
    width: f32,
    _alignment: LyricsAlignment,
) -> std::sync::Arc<egui::Galley> {
    if crate::bidi::is_rtl(text) {
        return crate::bidi::layout(ui.painter(), text, font, color, width, usize::MAX, None);
    }
    let mut job = LayoutJob::simple(text.to_owned(), font, color, width);
    job.wrap.break_anywhere = false;
    // Place the completed paragraph as a block below. Asking epaint to center
    // inside its wrap width can give the galley negative glyph coordinates.
    job.halign = Align::LEFT;
    ui.painter().layout_job(job)
}

fn paint_lyric_galley(
    ui: &mut egui::Ui,
    galley: std::sync::Arc<egui::Galley>,
    glow: Option<std::sync::Arc<egui::Galley>>,
    alignment: LyricsAlignment,
    sense: Sense,
) -> egui::Response {
    let width = ui.available_width();
    let height = galley.size().y.max(1.0);
    let (rect, response) = ui.allocate_exact_size(egui::vec2(width, height), sense);
    let x = match alignment {
        LyricsAlignment::Left => rect.left(),
        LyricsAlignment::Center => rect.center().x - galley.size().x * 0.5,
    };
    let position = egui::pos2(x.max(rect.left()), rect.top());
    if let Some(glow) = glow {
        for offset in [
            egui::vec2(-0.7, 0.0),
            egui::vec2(0.7, 0.0),
            egui::vec2(0.0, -0.7),
            egui::vec2(0.0, 0.7),
        ] {
            ui.painter()
                .galley(position + offset, glow.clone(), egui::Color32::TRANSPARENT);
        }
    }
    ui.painter()
        .galley(position, galley, egui::Color32::TRANSPARENT);
    response
}

/// Reveals complete words across the time available to a line. Spotify and
/// LRCLIB only supply line timestamps, so this is deliberately an estimate.
fn word_progress_offset(text: &str, position_ms: u32, start_ms: u32, end_ms: u32) -> usize {
    if position_ms < start_ms {
        return 0;
    }
    let word_count = text.split_whitespace().count();
    if word_count == 0 || end_ms <= start_ms {
        return text.len();
    }
    let elapsed = position_ms.saturating_sub(start_ms).min(end_ms - start_ms);
    let shown = ((u64::from(elapsed) * word_count as u64) / u64::from(end_ms - start_ms))
        .min(word_count as u64) as usize;
    if shown == 0 {
        return 0;
    }

    // Find the end of the last revealed word without building a temporary
    // word table on every playback repaint.
    let mut completed = 0;
    let mut in_word = false;
    let mut revealed_end = 0;
    for (at, character) in text.char_indices() {
        if character.is_whitespace() {
            if in_word {
                completed += 1;
                in_word = false;
                if completed == shown {
                    return revealed_end;
                }
            }
        } else {
            in_word = true;
            revealed_end = at + character.len_utf8();
        }
    }
    revealed_end
}

pub fn side_panel(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let tint = app.now_playing_tint().unwrap_or(palette.panel);
    let strength = f32::from(app.settings.lyrics_tint_strength.min(100)) / 100.0;
    let panel_fill = blend(
        palette.panel,
        tint,
        strength * if palette.dark { 0.32 } else { 0.20 },
    );
    // New id resets egui's persisted splitter position for the redesigned
    // panel; the user's saved lyrics width remains the default.
    let panel = egui::Panel::right("lyrics-panel-v2")
        .resizable(true)
        .default_size(app.settings.lyrics_width)
        .size_range(theme::SIDE_PANEL_MIN_WIDTH..=920.0)
        .show_separator_line(false)
        .frame(
            Frame::new()
                .fill(panel_fill)
                .stroke(egui::Stroke::new(1.0, palette.outline))
                .corner_radius(egui::CornerRadius::same(14))
                .outer_margin(Margin {
                    left: 4,
                    right: 8,
                    top: 8,
                    bottom: 8,
                })
                .inner_margin(Margin::symmetric(20, 16)),
        );
    let response = panel.show(ui, |ui| {
        // The frame gives content its reading inset. Expand the artwork back
        // through that inset so the image still reaches every rounded edge.
        let backdrop = ui.max_rect().expand2(egui::vec2(20.0, 16.0));
        if app.settings.accent_from_art
            && app.settings.lyrics_tint_strength > 0
            && let Some(now) = app.now_playing()
        {
            let opacity = (110.0 + f32::from(app.settings.lyrics_tint_strength.min(100)) * 1.30)
                .round() as u8;
            widgets::paint_blurred_art(
                ui,
                now.art_url.as_deref().or(now.art_small.as_deref()),
                backdrop,
                14.0,
                opacity,
                app.backend.art(),
            );
            ui.painter().rect_filled(
                backdrop,
                14.0,
                egui::Color32::from_rgba_unmultiplied(
                    palette.panel.r(),
                    palette.panel.g(),
                    palette.panel.b(),
                    (160.0 - strength * 70.0).round() as u8,
                ),
            );
        }
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
                ui.add_space(window_controls.lyrics_width);
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
        ui.add_space(6.0);
        if let Some(now) = app.now_playing() {
            let spacious = ui.available_width() >= 560.0;
            track_hero(app, ui, &now, spacious);
            ui.add_space(4.0);
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
                "Lyrics are not available for this track yet.",
            );
            return;
        }
        Loadable::Loaded(Some(lyrics)) if lyrics.instrumental => {
            widgets::empty_state(
                ui,
                &palette,
                Icon::Music,
                "Instrumental",
                "Enjoy the music — this track has no vocals to follow.",
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
    let line_gap = line_size * (f32::from(app.settings.lyrics_line_spacing.clamp(35, 110)) / 100.0);
    let alignment = app.settings.lyrics_alignment;
    let focus_padding = (ui.available_height() * 0.5 - line_size).max(12.0);
    let mut scroll_area = egui::ScrollArea::vertical()
        .id_salt(("lyrics-scroll", &now.uri))
        .animated(true)
        .auto_shrink([false, false]);
    if lyrics.synced && active.is_none() {
        scroll_area = scroll_area.vertical_scroll_offset(0.0);
    }
    let scroll = scroll_area.show(ui, |ui| {
        // Before the first line there is nothing to highlight, so the
        // panel sits at the top rather than wherever it was left.
        if follow && lyrics.synced && active.is_none() {
            let top = ui.cursor().min;
            ui.scroll_to_rect(
                egui::Rect::from_min_size(top, egui::vec2(1.0, 1.0)),
                Some(Align::Min),
            );
        }
        // Keep the first line comfortably below the hero without putting a
        // scrollable half-panel of empty space above it. Early active lines
        // may sit below centre until enough preceding lyrics exist.
        ui.add_space(16.0);
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
                    palette.dim,
                    (distance as f32 * 0.055).min(0.20),
                )
            } else {
                palette.secondary
            };
            let color = blend(quiet, palette.text, lit);
            let font = lyrics_font(app.settings.lyrics_font, line_size);
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
            let width = ui.available_width();
            let galley = if is_active && lyrics.synced && app.settings.lyrics_word_progress_beta {
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
                job.halign = Align::LEFT;
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
                        font_id: font.clone(),
                        color: palette.dim,
                        ..Default::default()
                    },
                );
                ui.painter().layout_job(job)
            } else {
                lyric_galley(ui, text, font.clone(), color, width, alignment)
            };
            let glow = app.settings.lyrics_glow.then(|| {
                lyric_galley(
                    ui,
                    text,
                    font,
                    egui::Color32::from_rgba_unmultiplied(
                        palette.accent.r(),
                        palette.accent.g(),
                        palette.accent.b(),
                        34,
                    ),
                    width,
                    alignment,
                )
            });
            let response = paint_lyric_galley(ui, galley, glow, alignment, sense);
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
            ui.add_space(line_gap);
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
