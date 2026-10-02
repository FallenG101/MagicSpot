//! Playlist, album, and Liked Songs pages: a hero, actions, and a track table.

use std::sync::Arc;

use egui::{Align, Layout, Rect, Sense, Vec2, pos2, vec2};

use crate::api::models::{Album, PlayableItem, Playlist, pick_image};
use crate::app::App;
use crate::model::{
    Action, Dialog, DragTrack, Loadable, Page, PagedList, RowContext, SortColumn, TableItem,
    TableRowsCache, TableSort,
};
use crate::theme::{self, Icon, Palette};
use crate::util;

use super::widgets::{self, TrackRow};

pub struct Hero<'a> {
    pub image: Option<&'a str>,
    pub liked: bool,
    pub kind: &'a str,
    pub title: &'a str,
    pub description: Option<String>,
    pub byline: Vec<(String, Option<Page>)>,
    pub round: bool,
}

fn hero_cover(app: &App, ui: &mut egui::Ui, hero: &Hero<'_>, cover_size: f32) {
    let palette = app.palette;
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(cover_size), Sense::hover());
    let radius = if hero.round { cover_size / 2.0 } else { 12.0 };
    widgets::paint_shadow(ui, &palette, rect, radius);
    if hero.liked {
        super::sidebar::liked_cover(ui, rect, radius);
    } else {
        widgets::paint_cover(
            ui,
            &palette,
            hero.image,
            rect,
            radius,
            if hero.round { Icon::User } else { Icon::Music },
            Some(app.backend.art()),
        );
    }
}

fn hero_details(app: &mut App, ui: &mut egui::Ui, hero: &Hero<'_>, cover_size: f32) {
    let palette = app.palette;
    let width = ui.available_width();
    ui.set_width(width);
    ui.spacing_mut().item_spacing.y = 6.0;
    ui.add_space(cover_size * 0.10);
    theme::text(
        ui,
        hero.kind.to_uppercase(),
        theme::semibold(11.5),
        palette.accent,
    );
    let mut size = if cover_size > 200.0 { 48.0 } else { 40.0 };
    let display_title = crate::bidi::display_text(hero.title);
    loop {
        let galley =
            ui.painter()
                .layout_no_wrap(display_title.to_string(), theme::bold(size), palette.text);
        if galley.size().x <= width || size <= 22.0 {
            break;
        }
        size -= 6.0;
    }
    theme::text(ui, hero.title, theme::bold(size), palette.text);
    if let Some(description) = &hero.description
        && !description.is_empty()
    {
        theme::text(
            ui,
            description.as_str(),
            theme::regular(13.5),
            palette.secondary,
        );
    }
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (index, (text, page)) in hero.byline.iter().enumerate() {
            if index > 0 {
                theme::text(ui, "•", theme::regular(13.5), palette.secondary);
            }
            match page {
                Some(page) => {
                    if theme::link(ui, text, theme::semibold(13.5), palette.text).clicked() {
                        app.actions.push(Action::Open(page.clone()));
                    }
                }
                None => {
                    theme::text(ui, text, theme::regular(13.5), palette.secondary);
                }
            }
        }
    });
}

pub fn hero(app: &mut App, ui: &mut egui::Ui, hero: Hero<'_>) {
    ui.add_space(12.0);
    let palette = app.palette;
    let available = ui.available_width();
    let wide = available >= 700.0;
    let cover_size = if wide {
        (available * 0.24).clamp(210.0, 280.0)
    } else {
        available.min(190.0)
    };
    egui::Frame::new()
        .fill(palette.surface)
        .stroke(egui::Stroke::new(1.0, palette.outline))
        .corner_radius(egui::CornerRadius::same(18))
        .inner_margin(egui::Margin::same(if wide { 24 } else { 18 }))
        .show(ui, |ui| {
            let rect = ui.max_rect();
            ui.painter().circle_filled(
                pos2(rect.right() - 24.0, rect.top() + 12.0),
                cover_size * 0.85,
                palette
                    .accent
                    .gamma_multiply(if palette.dark { 0.09 } else { 0.05 }),
            );
            if wide {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 30.0;
                    let details_width = (ui.available_width() - cover_size - 30.0).max(240.0);
                    ui.allocate_ui_with_layout(
                        vec2(details_width, cover_size),
                        Layout::top_down(Align::Min),
                        |ui| hero_details(app, ui, &hero, cover_size),
                    );
                    hero_cover(app, ui, &hero, cover_size);
                });
            } else {
                ui.vertical_centered(|ui| hero_cover(app, ui, &hero, cover_size));
                ui.add_space(12.0);
                hero_details(app, ui, &hero, 150.0);
            }
        });
    ui.add_space(18.0);
}

pub struct Actions<'a> {
    pub play_uri: Option<String>,
    /// A sorted or filtered view: the exact list on screen, which the big
    /// button plays instead of the context's own order.
    pub view: Option<Arc<[String]>>,
    pub saved: Option<(String, bool)>,
    pub saved_icons: (Icon, Icon),
    pub saved_tooltips: (&'a str, &'a str),
    pub owned_playlist: Option<Playlist>,
    pub name: &'a str,
}

/// The big play button and its neighbours; returns the filter text if a
/// filter field was shown.
pub fn actions_row(
    app: &mut App,
    ui: &mut egui::Ui,
    actions: Actions<'_>,
    filter: Option<&mut String>,
) {
    let palette = app.palette;
    let filter_below = filter.is_some() && ui.available_width() < 620.0;
    let mut filter = filter;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 18.0;
        if let Some(uri) = &actions.play_uri {
            let now_playing_here = app.playing_context_uri().as_deref() == Some(uri.as_str())
                && app.believed_playing();
            let icon = if now_playing_here {
                Icon::PauseFilled
            } else {
                Icon::PlayFilled
            };
            if app.play_pending(uri) {
                theme::circle_spinner(ui, 56.0, palette.accent, palette.on_accent, "Starting…");
            } else if theme::circle_button(
                ui,
                icon,
                56.0,
                palette.accent,
                palette.accent_hover,
                palette.on_accent,
                if now_playing_here { "Pause" } else { "Play" },
            )
            .clicked()
            {
                if now_playing_here {
                    app.actions.push(Action::TogglePlay);
                } else if let Some(uris) = actions.view.clone() {
                    app.actions.push(Action::PlayFromRow {
                        context: RowContext::View {
                            uris: Arc::clone(&uris),
                            context_uri: uri.clone(),
                        },
                        uri: String::new(),
                        index: 0,
                    });
                } else {
                    app.actions.push(Action::PlayContext {
                        uri: uri.clone(),
                        offset_uri: None,
                        offset_index: None,
                    });
                }
            }
            let context_here = app.playing_context_uri().as_deref() == Some(uri.as_str());
            let shuffling_here = context_here && app.playing_context_shuffle();
            if theme::icon_button(
                ui,
                Icon::Shuffle,
                26.0,
                if shuffling_here {
                    palette.accent
                } else {
                    palette.secondary
                },
                palette.text,
                if shuffling_here {
                    "Shuffle off"
                } else if context_here {
                    "Shuffle"
                } else {
                    "Shuffle play"
                },
            )
            .clicked()
            {
                if context_here {
                    app.actions.push(Action::SetShuffle(!shuffling_here));
                } else {
                    app.actions.push(Action::ShufflePlay(uri.clone()));
                }
            }
        }
        if let Some((uri, saved)) = &actions.saved {
            let (icon, tooltip, color) = if *saved {
                (
                    actions.saved_icons.1,
                    actions.saved_tooltips.1,
                    palette.accent,
                )
            } else {
                (
                    actions.saved_icons.0,
                    actions.saved_tooltips.0,
                    palette.secondary,
                )
            };
            if theme::icon_button(ui, icon, 26.0, color, palette.text, tooltip).clicked() {
                app.actions.push(Action::ToggleSaved(uri.clone()));
            }
        }
        if let Some(uri) = &actions.play_uri {
            let more = theme::icon_button(
                ui,
                Icon::Ellipsis,
                26.0,
                palette.secondary,
                palette.text,
                "More",
            );
            egui::Popup::menu(&more)
                .frame(widgets::menu_frame(&palette))
                .show(|ui| {
                    widgets::context_menu_items(
                        ui,
                        app,
                        uri,
                        actions.name,
                        actions.owned_playlist.as_ref(),
                    )
                });
        }
        if !filter_below && let Some(filter) = filter.as_deref_mut() {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                widgets::search_field(
                    ui,
                    &palette,
                    egui::Id::new(("collection-filter", actions.name)),
                    filter,
                    "Filter",
                    220.0,
                );
            });
        }
    });
    if filter_below && let Some(filter) = filter {
        widgets::search_field(
            ui,
            &palette,
            egui::Id::new(("collection-filter", actions.name)),
            filter,
            "Filter",
            ui.available_width().min(320.0),
        );
        ui.add_space(8.0);
    }
    ui.add_space(14.0);
}

/// A track table with virtualised rows and paging.
pub struct Table<'a> {
    pub total_rows: Option<u32>,
    pub items: &'a [TableItem],
    /// Spotify index represented by the first item.
    pub row_offset: u32,
    pub context: RowContext,
    pub show_album: bool,
    pub show_cover: bool,
    pub show_added: bool,
    pub show_added_by: bool,
    pub page: Page,
    pub loading: bool,
    pub error: Option<&'a str>,
    pub can_load_more: bool,
    pub filter: &'a str,
    pub items_revision: u64,
}

#[derive(Clone)]
pub struct TableCache {
    pub sort: Option<TableSort>,
    pub needle: String,
    pub items_revision: u64,
    pub user_names_revision: u64,
    pub visible: Arc<[usize]>,
    pub view_uris: Option<Arc<[String]>>,
}

pub fn table_items_hit(
    app: &App,
    page: &Page,
    generation: u64,
    items_revision: u64,
    user_names_revision: u64,
) -> Option<Arc<Vec<TableItem>>> {
    app.table_rows.get(page).and_then(|cached| {
        (cached.generation == generation
            && cached.items_revision == items_revision
            && cached.user_names_revision == user_names_revision)
            .then(|| Arc::clone(&cached.items))
    })
}

pub fn remember_table_items(
    app: &mut App,
    page: Page,
    generation: u64,
    items_revision: u64,
    user_names_revision: u64,
    items: Vec<TableItem>,
) -> Arc<Vec<TableItem>> {
    let items = Arc::new(items);
    app.table_rows.insert(
        page.clone(),
        TableRowsCache {
            generation,
            items_revision,
            user_names_revision,
            items: Arc::clone(&items),
            playlist_positions: None,
            playlist_raw_count: 0,
            playlist_duration_ms: 0,
            playlist_owner: None,
            playlist_append_revision: None,
        },
    );
    app.retain_table_rows(&page);
    items
}

/// Cached table rows for one page. Rebuilt only when the source list,
/// contributor names, or page generation change, not every frame.
///
/// The cache lives on `App`, not in egui temp data. It is dropped when the
/// page is evicted, when the account resets, and when more than two tables
/// would be retained. Recreated pages get a new generation, so an old
/// revision number cannot resurrect stale rows.
pub fn cached_table_items(
    app: &mut App,
    page: Page,
    generation: u64,
    items_revision: u64,
    user_names_revision: u64,
    build: impl FnOnce() -> Vec<TableItem>,
) -> Arc<Vec<TableItem>> {
    if let Some(items) =
        table_items_hit(app, &page, generation, items_revision, user_names_revision)
    {
        app.retain_table_rows(&page);
        return items;
    }
    remember_table_items(
        app,
        page,
        generation,
        items_revision,
        user_names_revision,
        build(),
    )
}

pub fn prepare_table_view(
    ui: &mut egui::Ui,
    app: &App,
    page: &Page,
    items: &[TableItem],
    needle: &str,
    sort: Option<TableSort>,
    items_revision: u64,
) -> Arc<TableCache> {
    let cache_id = egui::Id::new("table-view-cache").with(page);
    let cached = ui.data(|d| d.get_temp::<Arc<TableCache>>(cache_id));

    let is_valid = cached.as_ref().is_some_and(|c| {
        c.sort == sort
            && c.needle == needle
            && c.items_revision == items_revision
            && c.user_names_revision == app.user_names_revision
    });

    if let Some(entry) = cached.filter(|_| is_valid) {
        entry
    } else {
        let visible = view_indices(items, needle, sort);
        let view_uris = sort.map(|_| {
            visible
                .iter()
                .map(|&index| items[index].0.uri().to_string())
                .collect::<Arc<[String]>>()
        });
        let entry = Arc::new(TableCache {
            sort,
            needle: needle.to_string(),
            items_revision,
            user_names_revision: app.user_names_revision,
            visible: visible.into(),
            view_uris,
        });
        ui.data_mut(|d| d.insert_temp(cache_id, Arc::clone(&entry)));
        entry
    }
}

pub fn table(app: &mut App, ui: &mut egui::Ui, table: Table<'_>) {
    let palette = app.palette;
    let needle = table.filter.trim().to_lowercase();
    let sort = app.table_sorts.get(&table.page).copied();
    let entry = prepare_table_view(
        ui,
        app,
        &table.page,
        table.items,
        &needle,
        sort,
        table.items_revision,
    );
    let thin = app.settings.tracklist_compact;
    let show_cover = !thin && table.show_cover;
    let row_height = if thin {
        theme::THIN_ROW_HEIGHT
    } else {
        theme::ROW_HEIGHT
    };

    if !table.items.is_empty()
        && let Some(column) = widgets::table_header(
            ui,
            &palette,
            table.show_album,
            table.show_added,
            table.show_added_by,
            show_cover,
            sort,
        )
    {
        // Ascending, descending, back to the list's own order.
        let next = match sort {
            Some(sort) if sort.column == column && sort.ascending => Some(TableSort {
                column,
                ascending: false,
            }),
            Some(sort) if sort.column == column => None,
            // The # stands for the list's own order: from any other sort
            // it returns there rather than layering a sort of its own.
            Some(_) if column == SortColumn::Index => None,
            // Ascending by # is the list's own order, a click that would
            // change nothing; the first click on # reverses instead.
            _ => Some(TableSort {
                column,
                ascending: column != SortColumn::Index,
            }),
        };
        match next {
            Some(sort) => {
                app.table_sorts.insert(table.page.clone(), sort);
                app.note_session_change();
                // A sort covers the whole list, so the rest must load.
                app.actions.push(Action::LoadMore(table.page.clone()));
            }
            None => {
                app.table_sorts.remove(&table.page);
                app.note_session_change();
            }
        }
    }
    // What is displayed is what plays: a sorted view plays in its own
    // order, as a plain list of tracks, and its rows cannot edit server
    // positions that no longer match the screen.
    let context = if let Some(uris) = &entry.view_uris {
        match &table.context {
            RowContext::Context { uri, .. } => RowContext::View {
                uris: Arc::clone(uris),
                context_uri: uri.clone(),
            },
            _ => RowContext::Uris(Arc::clone(uris)),
        }
    } else {
        table.context.clone()
    };
    let sorted = sort.is_some();
    let whole_playlist = table
        .total_rows
        .filter(|_| sort.is_none() && needle.is_empty());
    let virtual_count = whole_playlist.map_or(entry.visible.len(), |total| total as usize);
    let positions = app
        .table_rows
        .get(&table.page)
        .and_then(|cache| cache.playlist_positions.clone());
    // Allow playlist reordering only when displayed rows match server order.
    let move_playlist = (sort.is_none() && needle.is_empty())
        .then(|| match &table.context {
            RowContext::Context {
                editable_playlist: Some((id, _)),
                ..
            } => Some(id.clone()),
            _ => None,
        })
        .flatten();
    // Calculate the nearest drop slot from fixed row height because virtualized
    // rows are not all available during drawing.
    let list_top = ui.cursor().top();
    let move_slot = move_playlist.as_ref().and_then(|playlist_id| {
        let track = egui::DragAndDrop::payload::<DragTrack>(ui.ctx())?;
        let (origin, _) = track.from.as_ref()?;
        if origin != playlist_id {
            return None;
        }
        let pos = ui
            .ctx()
            .pointer_latest_pos()
            .filter(|pos| ui.clip_rect().contains(*pos))?;
        let row = (pos.y - list_top) / row_height;
        (row >= 0.0 && row <= virtual_count as f32)
            .then(|| (row.round() as usize).min(virtual_count))
    });
    // Selection uses display indices. Clear it when sorting, filtering, or row
    // count changes.
    let view = format!(
        "{sort:?}|{needle}|{}|{}|{}",
        entry.visible.len(),
        table.items_revision,
        app.table_rows
            .get(&table.page)
            .map_or(0, |cache| cache.generation)
    );
    app.keep_picked_rows_for(&table.page, &view);
    let mut picked: std::collections::BTreeSet<usize> =
        app.picked_rows(&table.page).cloned().unwrap_or_default();
    // Keep complete rows for immediate optimistic playlist additions.
    let mut picked_songs: Vec<PlayableItem> = picked
        .iter()
        .filter_map(|row| entry.visible.get(*row))
        .filter_map(|index| table.items.get(*index))
        .map(|(item, _, _)| item.clone())
        .collect();
    let rows = entry.visible.len();
    let row_focus = ui.memory(|m| m.focused()).is_some_and(|id| {
        ui.data(|d| d.get_temp::<bool>(id.with("track-focus")))
            .unwrap_or(false)
    });
    let keyboard_allowed = app.dialog.is_none()
        && app.page() == &table.page
        && (row_focus || ui.memory(|m| m.focused().is_none()));
    if keyboard_allowed {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::A)) {
            // Select-all replaces the old selection, including a partial selection.
            picked = (0..rows).collect();
            picked_songs = entry
                .visible
                .iter()
                .map(|index| table.items[*index].0.clone())
                .collect();
            app.selection = Some((
                table.page.clone(),
                view.clone(),
                crate::model::RowSelection {
                    rows: picked.clone(),
                    anchor: Some(0),
                },
            ));
        }
        let events = ui.input(|i| i.events.clone());
        for event in events {
            match event {
                egui::Event::Copy | egui::Event::Cut if !picked_songs.is_empty() => {
                    ui.ctx().copy_text(
                        picked_songs
                            .iter()
                            .map(|item| item.uri())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                    if matches!(event, egui::Event::Cut) {
                        if let RowContext::Context {
                            editable_playlist: Some((id, _)),
                            ..
                        } = &table.context
                        {
                            let entries = picked
                                .iter()
                                .filter_map(|row| entry.visible.get(*row))
                                .map(|index| {
                                    let slot = app
                                        .table_rows
                                        .get(&table.page)
                                        .and_then(|c| c.playlist_positions.as_ref())
                                        .and_then(|p| p.get(*index))
                                        .copied()
                                        .unwrap_or(*index);
                                    (
                                        table.items[*index].0.uri().to_string(),
                                        table.row_offset.saturating_add(slot as u32),
                                    )
                                })
                                .collect();
                            app.actions.push(Action::RemoveFromPlaylist {
                                playlist_id: id.clone(),
                                entries,
                            });
                        } else {
                            app.toast("Copied songs. This source cannot be edited.");
                        }
                    }
                }
                egui::Event::Paste(text) => {
                    if let RowContext::Context {
                        editable_playlist: Some((id, _)),
                        ..
                    } = &table.context
                    {
                        app.actions.push(Action::PasteTracks {
                            playlist_id: id.clone(),
                            text,
                        });
                    } else {
                        app.toast("Paste songs into a writable playlist.");
                    }
                }
                _ => {}
            }
        }
    }
    let mut pick = None;
    let mut missing_position = None;
    widgets::virtual_rows(ui, virtual_count, row_height, |ui, virtual_row| {
        let row = if whole_playlist.is_some() {
            let local = (virtual_row as u32)
                .checked_sub(table.row_offset)
                .map(|slot| slot as usize);
            let loaded =
                local.and_then(|slot| positions.as_ref().and_then(|p| p.binary_search(&slot).ok()));
            let Some(row) = loaded else {
                let in_loaded_range = local.is_some_and(|slot| {
                    app.table_rows
                        .get(&table.page)
                        .is_some_and(|cache| slot < cache.playlist_raw_count)
                });
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), row_height), Sense::hover());
                if ui.is_rect_visible(rect) {
                    ui.painter().text(
                        rect.left_center() + vec2(12.0, 0.0),
                        egui::Align2::LEFT_CENTER,
                        if in_loaded_range {
                            "Unavailable song"
                        } else {
                            "Loading songs…"
                        },
                        theme::regular(13.0),
                        palette.secondary,
                    );
                    if !in_loaded_range && missing_position.is_none() {
                        missing_position = Some(virtual_row as u32 + 1);
                    }
                }
                return;
            };
            row
        } else {
            virtual_row
        };
        let index = entry.visible[row];
        let actual_index = app
            .table_rows
            .get(&table.page)
            .and_then(|cache| cache.playlist_positions.as_ref())
            .and_then(|positions| positions.get(index))
            .copied()
            .map_or_else(
                || absolute_row_index(table.row_offset, index),
                |slot| absolute_row_index(table.row_offset, slot),
            );
        let (item, added_at, added_by) = &table.items[index];
        // Shift neighboring rows around the current drop slot.
        let shift = ui.ctx().animate_value_with_time(
            ui.id().with(("table-move-shift", row)),
            match move_slot {
                Some(slot) if row < slot => -4.0,
                Some(_) => 4.0,
                None => 0.0,
            },
            0.12,
        );
        if let Some(asked) = widgets::track_row(
            ui,
            app,
            TrackRow {
                playlist_entry: match &table.context {
                    RowContext::Context {
                        editable_playlist: Some((id, _)),
                        ..
                    } => Some((id.clone(), actual_index as u32)),
                    _ => None,
                },
                index: if sorted { row } else { actual_index },
                number: Some(if sorted { row + 1 } else { actual_index + 1 }),
                item,
                context: &context,
                show_cover,
                show_album: table.show_album,
                added_at: added_at.as_deref(),
                added_by: added_by.as_deref(),
                show_added_by: table.show_added_by,
                compact: false,
                thin,
                shift,
                picked: picked.contains(&row),
                picked_songs: &picked_songs,
            },
        ) {
            pick = Some((row, asked));
        }
    });
    if !table.loading
        && table.error.is_none()
        && let Some(position) = missing_position
        && let Page::Playlist(id) = &table.page
    {
        let end = table.row_offset.saturating_add(
            app.table_rows
                .get(&table.page)
                .map_or(table.items.len(), |cache| cache.playlist_raw_count) as u32,
        );
        if position > end
            && position <= end.saturating_add(crate::backend::PLAYLIST_PAGE_SIZE)
            && table.can_load_more
        {
            app.actions.push(Action::LoadMore(table.page.clone()));
        } else {
            app.actions.push(Action::JumpToPlaylistPosition {
                id: id.clone(),
                position,
            });
        }
    }
    if let Some((row, asked)) = pick {
        app.pick_row(&table.page, &view, row, asked, rows);
    }
    // Escape clears the current selection.
    if !picked.is_empty() && ui.input(|input| input.key_pressed(egui::Key::Escape)) {
        app.clear_picked_rows();
    }
    if let Some(slot) = move_slot {
        // Draw the destination line between shifted rows.
        let y = list_top + slot as f32 * row_height;
        ui.painter().hline(
            ui.max_rect().x_range().shrink(8.0),
            y,
            egui::Stroke::new(2.0, palette.accent),
        );
        // Accept only a drag payload from this playlist.
        if ui.input(|input| input.pointer.any_released())
            && let Some(track) = egui::DragAndDrop::take_payload::<DragTrack>(ui.ctx())
            && let Some((playlist_id, from)) = track.from.clone()
        {
            let to = if whole_playlist.is_some() {
                slot as u32
            } else {
                table.row_offset.saturating_add(
                    positions
                        .as_ref()
                        .and_then(|p| p.get(slot))
                        .copied()
                        .unwrap_or(slot) as u32,
                )
            };
            // The slot is Spotify's insert_before, exactly what the
            // action's handler sends; a row dropped back on its own
            // edges moves nothing.
            if to != from && to != from + 1 {
                app.actions.push(Action::MoveInPlaylist {
                    playlist_id,
                    from,
                    to,
                });
            }
        }
    }
    if table.loading {
        ui.add_space(8.0);
        widgets::loading_row(ui, &palette);
    }
    if let Some(error) = table.error {
        ui.add_space(8.0);
        widgets::error_row(ui, app, error, Some(table.page.clone()));
    }
    if table.items.is_empty() && !table.loading && table.error.is_none() {
        widgets::empty_state(
            ui,
            &palette,
            Icon::Music,
            "Nothing here yet",
            "Added songs appear here.",
        );
    } else if entry.visible.is_empty()
        && !needle.is_empty()
        && table.can_load_more
        && !table.loading
    {
        // Filtering a partially loaded list: keep fetching so matches appear.
        app.actions.push(Action::LoadMore(table.page));
    } else {
        widgets::load_more_when_near_end(
            ui,
            app,
            table.page,
            whole_playlist.is_none() && table.can_load_more && !table.loading,
        );
    }
}

fn absolute_row_index(row_offset: u32, local_index: usize) -> usize {
    (row_offset as usize).saturating_add(local_index)
}

fn sort_by_text_key(visible: &mut [usize], ascending: bool, key: impl Fn(usize) -> String) {
    if ascending {
        visible.sort_by_cached_key(|&index| key(index));
    } else {
        visible.sort_by_cached_key(|&index| std::cmp::Reverse(key(index)));
    }
}

/// The indices of `items` as a view presents them: filtered by `needle`
/// (already lowercased), then ordered by `sort`.
fn view_indices(items: &[TableItem], needle: &str, sort: Option<TableSort>) -> Vec<usize> {
    let mut visible: Vec<usize> = items
        .iter()
        .enumerate()
        .filter(|(_, (item, _, _))| {
            if needle.is_empty() {
                return true;
            }
            let haystack = match item {
                PlayableItem::Track(track) => format!(
                    "{} {} {}",
                    track.name,
                    track.artist_names(),
                    track
                        .album
                        .as_ref()
                        .map(|album| album.name.as_str())
                        .unwrap_or("")
                ),
                PlayableItem::Episode(episode) => episode.name.clone(),
            };
            haystack.to_lowercase().contains(needle)
        })
        .map(|(index, _)| index)
        .collect();
    if let Some(sort) = sort {
        match sort.column {
            SortColumn::Title => sort_by_text_key(&mut visible, sort.ascending, |index| {
                items[index].0.name().to_lowercase()
            }),
            SortColumn::Album => {
                sort_by_text_key(&mut visible, sort.ascending, |index| {
                    match &items[index].0 {
                        PlayableItem::Track(track) => track
                            .album
                            .as_ref()
                            .map(|album| album.name.to_lowercase())
                            .unwrap_or_default(),
                        PlayableItem::Episode(_) => String::new(),
                    }
                })
            }
            SortColumn::AddedBy => sort_by_text_key(&mut visible, sort.ascending, |index| {
                items[index].2.as_deref().unwrap_or_default().to_lowercase()
            }),
            SortColumn::Added | SortColumn::Index | SortColumn::Duration => {
                visible.sort_by(|a, b| {
                    let ordering = match sort.column {
                        SortColumn::Added => items[*a].1.cmp(&items[*b].1),
                        SortColumn::Index => a.cmp(b),
                        SortColumn::Duration => {
                            items[*a].0.duration_ms().cmp(&items[*b].0.duration_ms())
                        }
                        _ => unreachable!("text columns are handled above"),
                    };
                    if sort.ascending {
                        ordering
                    } else {
                        ordering.reverse()
                    }
                });
            }
        }
    }
    visible
}

fn total_duration(items: &[TableItem]) -> u64 {
    items
        .iter()
        .map(|(item, _, _)| item.duration_ms() as u64)
        .sum()
}

fn playlist_rows(
    list: &[crate::api::models::PlaylistItem],
    start: usize,
    owner_id: Option<&str>,
    owner_name: &str,
    names: &std::collections::HashMap<String, Option<String>>,
) -> (Vec<TableItem>, Vec<usize>, u64) {
    let mut rows = Vec::new();
    let mut positions = Vec::new();
    let mut duration_ms = 0;
    for (index, item) in list.iter().enumerate() {
        if let Some(mut playable) = item.playable().cloned() {
            if let PlayableItem::Track(track) = &mut playable {
                track.is_local |= item.is_local;
            }
            duration_ms += playable.duration_ms() as u64;
            let adder = item
                .added_by
                .as_ref()
                .and_then(|user| user.id.as_deref())
                .map(|id| {
                    if Some(id) == owner_id {
                        owner_name.to_string()
                    } else {
                        names
                            .get(id)
                            .and_then(|name| name.clone())
                            .unwrap_or_else(|| id.to_string())
                    }
                });
            positions.push(start + index);
            rows.push((playable, item.added_at.clone(), adder));
        }
    }
    (rows, positions, duration_ms)
}

pub(crate) fn playlist_cached_table_items(
    app: &mut App,
    id: &str,
    generation: u64,
    list: &PagedList<crate::api::models::PlaylistItem>,
    owner_id: Option<&str>,
    owner_name: &str,
) -> (Arc<Vec<TableItem>>, Arc<Vec<usize>>, u64) {
    let key = Page::Playlist(id.to_string());
    let revision = list.revision;
    let names_revision = app.user_names_revision;
    let same_owner = |cache: &TableRowsCache| {
        cache
            .playlist_owner
            .as_ref()
            .is_some_and(|(id, name)| id.as_deref() == owner_id && name == owner_name)
    };
    if let Some(cache) = app.table_rows.get(&key)
        && cache.generation == generation
        && cache.items_revision == revision
        && cache.user_names_revision == names_revision
        && same_owner(cache)
        && let Some(positions) = &cache.playlist_positions
    {
        let result = (
            Arc::clone(&cache.items),
            Arc::clone(positions),
            cache.playlist_duration_ms,
        );
        app.retain_table_rows(&key);
        return result;
    }

    let append_from = app.table_rows.get(&key).and_then(|cache| {
        (cache.generation == generation
            && cache.user_names_revision == names_revision
            && same_owner(cache)
            && cache.playlist_append_revision == Some(revision)
            && cache.playlist_raw_count <= list.items.len())
        .then_some(cache.playlist_raw_count)
    });
    if let Some(start) = append_from {
        let (new_rows, new_positions, new_duration) = playlist_rows(
            &list.items[start..],
            start,
            owner_id,
            owner_name,
            &app.user_names,
        );
        if let Some(cache) = app.table_rows.get_mut(&key)
            && let Some(positions) = cache.playlist_positions.as_mut()
            && let Some(rows) = Arc::get_mut(&mut cache.items)
            && let Some(old_positions) = Arc::get_mut(positions)
        {
            rows.extend(new_rows);
            old_positions.extend(new_positions);
            cache.items_revision = revision;
            cache.playlist_raw_count = list.items.len();
            cache.playlist_duration_ms += new_duration;
            cache.playlist_append_revision = None;
            let result = (
                Arc::clone(&cache.items),
                Arc::clone(positions),
                cache.playlist_duration_ms,
            );
            app.retain_table_rows(&key);
            return result;
        }
    }

    let (rows, positions, duration_ms) =
        playlist_rows(&list.items, 0, owner_id, owner_name, &app.user_names);
    let items = remember_table_items(app, key.clone(), generation, revision, names_revision, rows);
    let positions = Arc::new(positions);
    if let Some(cache) = app.table_rows.get_mut(&key) {
        cache.playlist_positions = Some(Arc::clone(&positions));
        cache.playlist_raw_count = list.items.len();
        cache.playlist_duration_ms = duration_ms;
        cache.playlist_owner = Some((owner_id.map(str::to_string), owner_name.to_string()));
    }
    (items, positions, duration_ms)
}

/// A complete, ranked view of the listener's current top tracks.
pub fn top_songs(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    ui.add_space(12.0);
    theme::text(ui, "Your top songs", theme::bold(30.0), palette.text);
    ui.add_space(4.0);
    theme::text(
        ui,
        "Your most-played tracks from the last four weeks.",
        theme::regular(13.5),
        palette.secondary,
    );
    ui.add_space(18.0);

    let tracks = match &app.home.top_songs {
        Loadable::Loaded(tracks) => tracks,
        Loadable::Loading | Loadable::NotLoaded => {
            widgets::loading_row(ui, &palette);
            return;
        }
        Loadable::Failed(error) => {
            let error = error.clone();
            widgets::error_row(ui, app, &error, Some(Page::TopSongs));
            return;
        }
    };
    let generation = app.home.top_songs_generation;
    let names = app.user_names_revision;
    let items =
        if let Some(items) = table_items_hit(app, &Page::TopSongs, generation, generation, names) {
            items
        } else {
            let rows = tracks
                .iter()
                .cloned()
                .map(|track| (PlayableItem::Track(track), None, None))
                .collect();
            remember_table_items(app, Page::TopSongs, generation, generation, names, rows)
        };
    let uris: Arc<[String]> = items
        .iter()
        .map(|(item, _, _)| item.uri().to_string())
        .collect::<Vec<_>>()
        .into();
    table(
        app,
        ui,
        Table {
            total_rows: None,
            items: &items,
            row_offset: 0,
            context: RowContext::Uris(Arc::clone(&uris)),
            show_album: true,
            show_cover: true,
            show_added: false,
            show_added_by: false,
            page: Page::TopSongs,
            loading: app.home.top_songs_loading,
            error: None,
            can_load_more: false,
            filter: "",
            items_revision: app.home.top_songs_generation,
        },
    );
}

pub fn playlist(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let Some(mut page) = app.playlist_pages.remove(id) else {
        app.ensure_loaded(Page::Playlist(id.to_string()));
        return;
    };
    let palette = app.palette;
    let user_id = app.user_id().unwrap_or("").to_string();
    match &page.playlist {
        Loadable::Loaded(playlist) => {
            let (items, _positions, duration_ms) = playlist_cached_table_items(
                app,
                id,
                page.generation,
                &page.items,
                playlist.owner.id.as_deref(),
                playlist.owner_name(),
            );
            let count = page
                .items
                .total
                .unwrap_or_else(|| playlist.track_total())
                .max(items.len() as u32);
            // Spotify's collaborative flag covers secret collaborations; a
            // playlist made together today is recognised by who added songs.
            let owner_id = playlist.owner.id.as_deref();
            // Spotify's own playlists carry adder ids of their machinery;
            // nothing about them is a collaboration.
            let editorial = owner_id == Some("spotify");
            let others = if editorial {
                0
            } else {
                page.contributors
                    .iter()
                    .filter(|id| !id.is_empty() && Some(id.as_str()) != owner_id)
                    .count()
            };
            let made_together = playlist.collaborative || others > 0;
            let mut byline = vec![(playlist.owner_name().to_string(), None)];
            if others > 0 {
                let named: Vec<String> = page
                    .contributors
                    .iter()
                    .filter(|id| Some(id.as_str()) != owner_id)
                    .filter_map(|id| app.user_names.get(id)?.clone())
                    .collect();
                byline.push((
                    if named.len() == others && others <= 2 {
                        format!("with {}", named.join(" and "))
                    } else if others == 1 {
                        "and 1 other".to_string()
                    } else {
                        format!("and {others} others")
                    },
                    None,
                ));
            }
            let count_text = if page.items.is_complete() {
                format!(
                    "{} songs, {}",
                    util::format_count(count as u64),
                    util::format_total_ms(duration_ms)
                )
            } else {
                format!("{} songs", util::format_count(count as u64))
            };
            byline.push((count_text, None));
            hero(
                app,
                ui,
                Hero {
                    image: pick_image(&playlist.images, 300),
                    liked: false,
                    kind: if made_together {
                        "Collaborative Playlist"
                    } else if playlist.public == Some(true) {
                        "Public Playlist"
                    } else {
                        "Playlist"
                    },
                    title: &playlist.name,
                    description: playlist.description.as_deref().map(util::strip_html),
                    byline,
                    round: false,
                },
            );
            let owned = playlist.owned_by(&user_id);
            let saved = app.is_saved(&playlist.uri).unwrap_or(false);
            let needle = page.filter.trim().to_lowercase();
            let sort = app
                .table_sorts
                .get(&Page::Playlist(id.to_string()))
                .copied();
            let table_view = prepare_table_view(
                ui,
                app,
                &Page::Playlist(id.to_string()),
                &items,
                &needle,
                sort,
                page.items.revision,
            );
            let view_play = table_view.view_uris.as_ref().map(Arc::clone);
            let playlist_clone = playlist.clone();
            actions_row(
                app,
                ui,
                Actions {
                    play_uri: Some(playlist.uri.clone()),
                    view: view_play,
                    saved: (!owned).then(|| (playlist.uri.clone(), saved)),
                    saved_icons: (Icon::CirclePlus, Icon::CircleCheck),
                    saved_tooltips: ("Add to Your Library", "Remove from Your Library"),
                    owned_playlist: owned.then_some(playlist_clone),
                    name: &playlist.name,
                },
                Some(&mut page.filter),
            );
            if page.items.base_offset > 0 && !page.filter.trim().is_empty() {
                app.actions
                    .push(Action::LoadMore(Page::Playlist(id.to_string())));
            }
            let editable = app
                .can_edit_playlist(playlist)
                .then(|| (playlist.id.clone(), playlist.snapshot_id.clone()));
            table(
                app,
                ui,
                Table {
                    total_rows: Some(count),
                    items: &items,
                    row_offset: page.items.base_offset,
                    context: RowContext::Context {
                        uri: playlist.uri.clone(),
                        editable_playlist: editable,
                    },
                    show_album: true,
                    show_cover: true,
                    show_added: true,
                    show_added_by: made_together,
                    page: Page::Playlist(id.to_string()),
                    loading: page.items.loading,
                    error: page.items.error.as_deref(),
                    can_load_more: page.items.can_load_more(),
                    filter: &page.filter,
                    items_revision: page.items.revision,
                },
            );
        }
        Loadable::Loading | Loadable::NotLoaded => {
            ui.add_space(40.0);
            widgets::loading_row(ui, &palette);
        }
        Loadable::Failed(error) => {
            let error = error.clone();
            ui.add_space(40.0);
            widgets::error_row(ui, app, &error, Some(Page::Playlist(id.to_string())));
        }
    }
    app.playlist_pages.insert(id.to_string(), page);
}

pub fn album(app: &mut App, ui: &mut egui::Ui, id: &str) {
    let Some(page) = app.album_pages.remove(id) else {
        app.ensure_loaded(Page::Album(id.to_string()));
        return;
    };
    let palette = app.palette;
    match &page.album {
        Loadable::Loaded(album) => {
            album_hero(app, ui, album, &page.tracks);
            let generation = page.generation;
            let revision = page.tracks.revision;
            let names = app.user_names_revision;
            let key = Page::Album(id.to_string());
            let items = if let Some(items) = table_items_hit(app, &key, generation, revision, names)
            {
                items
            } else {
                let rows = page
                    .tracks
                    .items
                    .iter()
                    .cloned()
                    .map(|mut track| {
                        if track.album.is_none() {
                            track.album = Some(Album {
                                id: album.id.clone(),
                                name: album.name.clone(),
                                uri: album.uri.clone(),
                                images: album.images.clone(),
                                ..Album::default()
                            });
                        }
                        (PlayableItem::Track(track), None, None)
                    })
                    .collect();
                remember_table_items(app, key, generation, revision, names, rows)
            };
            let saved = app.is_saved(&album.uri).unwrap_or(false);
            let sort = app.table_sorts.get(&Page::Album(id.to_string())).copied();
            let table_view = prepare_table_view(
                ui,
                app,
                &Page::Album(id.to_string()),
                &items,
                "",
                sort,
                page.tracks.revision,
            );
            let album_view = table_view.view_uris.as_ref().map(Arc::clone);
            actions_row(
                app,
                ui,
                Actions {
                    play_uri: Some(album.uri.clone()),
                    view: album_view,
                    saved: Some((album.uri.clone(), saved)),
                    saved_icons: (Icon::CirclePlus, Icon::CircleCheck),
                    saved_tooltips: ("Save to Your Library", "Remove from Your Library"),
                    owned_playlist: None,
                    name: &album.name,
                },
                None,
            );
            table(
                app,
                ui,
                Table {
                    total_rows: None,
                    items: &items,
                    row_offset: 0,
                    context: RowContext::Context {
                        uri: album.uri.clone(),
                        editable_playlist: None,
                    },
                    show_album: false,
                    show_cover: false,
                    show_added: false,
                    show_added_by: false,
                    page: Page::Album(id.to_string()),
                    loading: page.tracks.loading,
                    error: page.tracks.error.as_deref(),
                    can_load_more: page.tracks.can_load_more(),
                    filter: "",
                    items_revision: page.tracks.revision,
                },
            );
            ui.add_space(24.0);
            if let Some(date) = &album.release_date {
                theme::text(
                    ui,
                    util::format_date(date),
                    theme::regular(12.5),
                    palette.secondary,
                );
            }
            // Labels file the same line under both kinds of copyright;
            // one line wearing both marks reads better than the line twice.
            let mut credits: Vec<(String, Vec<&str>)> = Vec::new();
            for copyright in &album.copyrights {
                let core = copyright
                    .text
                    .trim_start_matches(['©', '℗'])
                    .trim_start_matches("(C)")
                    .trim_start_matches("(P)")
                    .trim()
                    .to_string();
                let mark = if copyright.kind == "P" { "℗" } else { "©" };
                match credits.iter_mut().find(|(held, _)| *held == core) {
                    Some((_, marks)) => {
                        if !marks.contains(&mark) {
                            marks.push(mark);
                        }
                    }
                    None => credits.push((core, vec![mark])),
                }
            }
            for (core, marks) in credits {
                theme::text(
                    ui,
                    format!("{} {core}", marks.join(" ")),
                    theme::regular(11.5),
                    palette.dim,
                );
            }
        }
        Loadable::Loading | Loadable::NotLoaded => {
            ui.add_space(40.0);
            widgets::loading_row(ui, &palette);
        }
        Loadable::Failed(error) => {
            let error = error.clone();
            ui.add_space(40.0);
            widgets::error_row(ui, app, &error, Some(Page::Album(id.to_string())));
        }
    }
    app.album_pages.insert(id.to_string(), page);
}

fn album_hero(
    app: &mut App,
    ui: &mut egui::Ui,
    album: &Album,
    tracks: &PagedList<crate::api::models::Track>,
) {
    let mut byline: Vec<(String, Option<Page>)> = album
        .artists
        .iter()
        .map(|artist| (artist.name.clone(), artist.id.clone().map(Page::Artist)))
        .collect();
    if let Some(year) = album.year() {
        byline.push((year.to_string(), None));
    }
    let count = album.total_tracks.unwrap_or(tracks.items.len() as u32);
    let duration: u64 = tracks
        .items
        .iter()
        .map(|track| track.duration_ms as u64)
        .sum();
    let count_text = if tracks.is_complete() {
        format!("{count} songs, {}", util::format_total_ms(duration))
    } else {
        format!("{count} songs")
    };
    byline.push((count_text, None));
    hero(
        app,
        ui,
        Hero {
            image: pick_image(&album.images, 300),
            liked: false,
            kind: album.kind_label(),
            title: &album.name,
            description: None,
            byline,
            round: false,
        },
    );
}

pub fn liked(app: &mut App, ui: &mut egui::Ui) {
    let palette = app.palette;
    let revision = app.library.liked.revision;
    let names = app.user_names_revision;
    let items =
        if let Some(items) = table_items_hit(app, &Page::LikedSongs, revision, revision, names) {
            items
        } else {
            let rows = app
                .library
                .liked
                .items
                .iter()
                .map(|saved| {
                    (
                        PlayableItem::Track(saved.track.clone()),
                        saved.added_at.clone(),
                        None,
                    )
                })
                .collect();
            remember_table_items(app, Page::LikedSongs, revision, revision, names, rows)
        };
    let total = app.library.liked.total.unwrap_or(items.len() as u32);
    let user = app
        .user
        .as_ref()
        .map(|user| user.name().to_string())
        .unwrap_or_default();
    let count_text = if app.library.liked.is_complete() {
        format!(
            "{} songs, {}",
            util::format_count(total as u64),
            util::format_total_ms(total_duration(&items))
        )
    } else {
        format!("{} songs", util::format_count(total as u64))
    };
    hero(
        app,
        ui,
        Hero {
            image: None,
            liked: true,
            kind: "Playlist",
            title: "Liked Songs",
            description: None,
            byline: vec![(user, None), (count_text, None)],
            round: false,
        },
    );
    let collection_uri = app
        .user
        .as_ref()
        .map(|user| format!("spotify:user:{}:collection", user.id));
    let filter_id = egui::Id::new("liked-filter");
    let mut filter = ui
        .data(|data| data.get_temp::<String>(filter_id))
        .unwrap_or_default();
    let needle = filter.trim().to_lowercase();
    let sort = app.table_sorts.get(&Page::LikedSongs).copied();
    let table_view = prepare_table_view(
        ui,
        app,
        &Page::LikedSongs,
        &items,
        &needle,
        sort,
        app.library.liked.revision,
    );
    let liked_view = table_view.view_uris.as_ref().map(Arc::clone);
    actions_row(
        app,
        ui,
        Actions {
            play_uri: collection_uri.clone(),
            view: liked_view,
            saved: None,
            saved_icons: (Icon::Heart, Icon::HeartFilled),
            saved_tooltips: ("", ""),
            owned_playlist: None,
            name: "Liked Songs",
        },
        Some(&mut filter),
    );
    ui.data_mut(|data| data.insert_temp(filter_id, filter.clone()));
    let uris: Arc<[String]> = items
        .iter()
        .map(|(item, _, _)| item.uri().to_string())
        .collect::<Vec<_>>()
        .into();
    let context = match collection_uri {
        Some(uri) if app.library.liked.is_complete() => RowContext::Context {
            uri,
            editable_playlist: None,
        },
        _ => RowContext::Uris(uris),
    };
    let loading = app.library.liked.loading;
    let error = app.library.liked.error.clone();
    let can_load_more = app.library.liked.can_load_more();
    let _ = &palette;
    table(
        app,
        ui,
        Table {
            total_rows: None,
            items: &items,
            row_offset: 0,
            context,
            show_album: true,
            show_cover: true,
            show_added: true,
            show_added_by: false,
            page: Page::LikedSongs,
            loading,
            error: error.as_deref(),
            can_load_more,
            filter: &filter,
            items_revision: app.library.liked.revision,
        },
    );
}

#[allow(dead_code)]
fn playlist_dialog(app: &mut App, playlist: &Playlist) {
    app.actions.push(Action::ShowDialog(Dialog::EditPlaylist {
        original: (
            playlist.name.clone(),
            playlist.description.clone().unwrap_or_default(),
            playlist.public.unwrap_or(false),
        ),
        id: playlist.id.clone(),
        name: playlist.name.clone(),
        description: playlist.description.clone().unwrap_or_default(),
        public: playlist.public.unwrap_or(false),
    }));
}

#[allow(dead_code)]
fn rect_after(ui: &egui::Ui, height: f32) -> Rect {
    let cursor = ui.cursor();
    Rect::from_min_size(
        pos2(cursor.left(), cursor.top()),
        vec2(ui.available_width(), height),
    )
}

#[allow(dead_code)]
fn palette_of(app: &App) -> Palette {
    app.palette
}

pub fn radio(app: &mut App, ui: &mut egui::Ui, uri: &str) {
    let palette = app.palette;
    let title = app
        .track_cache
        .get(uri)
        .map(|track| format!("{} Radio", track.name))
        .or_else(|| match util::uri_kind(uri) {
            Some("album") => app
                .album_pages
                .get(util::uri_id(uri).unwrap_or_default())
                .and_then(|page| page.album.get())
                .map(|album| format!("{} Radio", album.name)),
            Some("artist") => app
                .artist_pages
                .get(util::uri_id(uri).unwrap_or_default())
                .and_then(|page| page.artist.get())
                .map(|artist| format!("{} Radio", artist.name)),
            Some("playlist") => app
                .playlist_pages
                .get(util::uri_id(uri).unwrap_or_default())
                .and_then(|page| page.playlist.get())
                .map(|playlist| format!("{} Radio", playlist.name)),
            _ => None,
        })
        .unwrap_or_else(|| "Radio".into());
    theme::text(ui, &title, theme::bold(28.0), palette.text);
    ui.label("A recommendation mix based on this source. Results may differ from Spotify Radio.");
    let state = app
        .radio
        .as_ref()
        .filter(|(seed, _, _)| seed == uri)
        .map(|(_, _, state)| state.clone())
        .unwrap_or(Loadable::Loading);
    let tracks = state.get().cloned().unwrap_or_default();
    let uris: Vec<_> = tracks.iter().map(|track| track.uri.clone()).collect();
    ui.horizontal_wrapped(|ui| {
        if !uris.is_empty() && theme::pill_button(ui, &palette, "Play", true).clicked() {
            app.actions.push(Action::PlayUris {
                uris: uris.clone(),
                index: 0,
            });
        }
        if theme::pill_button(ui, &palette, "Refresh", false).clicked() {
            app.actions.push(Action::RefreshRadio(uri.into()));
        }
        if !uris.is_empty() && theme::pill_button(ui, &palette, "Save as playlist", false).clicked()
        {
            app.actions.push(Action::ShowDialog(Dialog::CreatePlaylist {
                name: title.clone(),
                public: false,
                add_uris: uris.clone(),
            }));
        }
        if util::uri_kind(uri) == Some("track")
            && theme::pill_button(ui, &palette, "Play song radio locally", false).clicked()
        {
            app.actions.push(Action::PlayTrackRadio(uri.into()));
        }
        if theme::pill_button(ui, &palette, "Open in Spotify", false).clicked() {
            app.actions.push(Action::OpenInSpotify(uri.into()));
        }
    });
    match state {
        Loadable::Loaded(_) => {
            let items: Vec<TableItem> = tracks
                .into_iter()
                .map(|track| (PlayableItem::Track(track), None, None))
                .collect();
            table(
                app,
                ui,
                Table {
                    total_rows: None,
                    items: &items,
                    row_offset: 0,
                    context: RowContext::Uris(uris.into()),
                    show_album: true,
                    show_cover: true,
                    show_added: false,
                    show_added_by: false,
                    page: Page::Radio(uri.into()),
                    loading: false,
                    error: None,
                    can_load_more: false,
                    filter: "",
                    items_revision: app
                        .radio
                        .as_ref()
                        .map_or(0, |(_, generation, _)| *generation),
                },
            );
        }
        Loadable::Failed(error) => {
            ui.add(egui::Label::new(error).wrap());
        }
        _ => widgets::loading_row(ui, &palette),
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn range_boundaries_append_the_next_page_instead_of_discarding_previous_rows() {
        let mut app = test_app();
        app.backend.set_offline(true);
        let ctx = egui::Context::default();
        app.attach(&ctx);
        let page = Page::Playlist("boundary".into());
        let items: Vec<_> = (0..50)
            .map(|index| {
                (
                    PlayableItem::Track(crate::api::models::Track {
                        uri: format!("spotify:track:{index}"),
                        name: format!("Song {index}"),
                        ..Default::default()
                    }),
                    None,
                    None,
                )
            })
            .collect();
        remember_table_items(&mut app, page.clone(), 1, 1, 0, items.clone());
        let cache = app.table_rows.get_mut(&page).unwrap();
        cache.playlist_positions = Some(Arc::new((0..50).collect()));
        cache.playlist_raw_count = 50;
        for _ in 0..2 {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(900.0, 700.0))),
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical()
                        .vertical_scroll_offset(45.0 * theme::ROW_HEIGHT)
                        .show(ui, |ui| {
                            table(
                                &mut app,
                                ui,
                                Table {
                                    total_rows: Some(100),
                                    items: &items,
                                    row_offset: 0,
                                    context: RowContext::Context {
                                        uri: "spotify:playlist:boundary".into(),
                                        editable_playlist: None,
                                    },
                                    show_album: false,
                                    show_cover: false,
                                    show_added: false,
                                    show_added_by: false,
                                    page: page.clone(),
                                    loading: false,
                                    error: None,
                                    can_load_more: true,
                                    filter: "",
                                    items_revision: 1,
                                },
                            );
                        });
                },
            );
            output.textures_delta.clear();
        }
        assert!(app.actions.iter().any(
            |action| matches!(action, Action::LoadMore(Page::Playlist(id)) if id == "boundary")
        ));
        assert!(
            !app.actions
                .iter()
                .any(|action| matches!(action, Action::JumpToPlaylistPosition { .. }))
        );
        app.backend.shutdown();
    }

    fn keyboard_event(key: egui::Key, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers,
        }
    }
    #[test]
    fn sorted_filtered_cut_resolves_display_rows_to_server_slots_and_copy_order() {
        let mut app = test_app();
        app.backend.set_offline(true);
        let page = Page::Playlist("p".into());
        app.history.push(page.clone());
        app.history_index = app.history.len() - 1;
        app.table_sorts.insert(
            page.clone(),
            TableSort {
                column: SortColumn::Title,
                ascending: false,
            },
        );
        let items = make_test_tracks();
        let cached = remember_table_items(&mut app, page.clone(), 1, 1, 0, items.clone());
        app.table_rows.get_mut(&page).unwrap().playlist_positions =
            Some(Arc::new(vec![0, 2, 3, 5]));
        let ctx = egui::Context::default();
        app.attach(&ctx);
        let context = RowContext::Context {
            uri: "spotify:playlist:p".into(),
            editable_playlist: Some(("p".into(), Some("snapshot".into()))),
        };
        let render = |app: &mut App, events, filter: &str, text_focus: bool| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(900.0, 700.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    if text_focus {
                        let mut query = String::new();
                        ui.add(
                            egui::TextEdit::singleline(&mut query)
                                .id(egui::Id::new("ordinary-editor")),
                        );
                    }
                    table(
                        app,
                        ui,
                        Table {
                            total_rows: None,
                            items: &cached,
                            row_offset: 50,
                            context: context.clone(),
                            show_album: true,
                            show_cover: false,
                            show_added: false,
                            show_added_by: false,
                            page: page.clone(),
                            loading: false,
                            error: None,
                            can_load_more: false,
                            filter,
                            items_revision: 1,
                        },
                    );
                },
            );
            output.textures_delta.clear();
            output
        };
        render(&mut app, vec![], "", false);
        let output = render(
            &mut app,
            vec![
                keyboard_event(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Copy,
            ],
            "",
            false,
        );
        let visible = view_indices(
            &items,
            "",
            Some(TableSort {
                column: SortColumn::Title,
                ascending: false,
            }),
        );
        let expected = visible
            .iter()
            .map(|index| items[*index].0.uri())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(output.platform_output.commands.iter().any(
            |command| matches!(command, egui::OutputCommand::CopyText(text) if *text == expected)
        ));
        render(&mut app, vec![egui::Event::Cut], "", false);
        let entries = app
            .actions
            .iter()
            .find_map(|action| match action {
                Action::RemoveFromPlaylist { entries, .. } => Some(entries),
                _ => None,
            })
            .unwrap();
        assert_eq!(
            entries.iter().map(|(_, slot)| *slot).collect::<Vec<_>>(),
            visible
                .iter()
                .map(|index| [50, 52, 53, 55][*index])
                .collect::<Vec<_>>()
        );
        app.actions.clear();
        render(
            &mut app,
            vec![
                keyboard_event(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Cut,
            ],
            "queen",
            false,
        );
        assert!(app.actions.iter().any(|action| matches!(action, Action::RemoveFromPlaylist { entries, .. } if entries.len() == 1 && entries[0].1 == 50)));
        app.actions.clear();
        ctx.memory_mut(|m| m.request_focus(egui::Id::new("ordinary-editor")));
        render(
            &mut app,
            vec![
                keyboard_event(egui::Key::A, egui::Modifiers::COMMAND),
                egui::Event::Cut,
                egui::Event::Paste("spotify:track:abc".into()),
            ],
            "",
            true,
        );
        assert!(
            app.actions.is_empty(),
            "text fields must own editing shortcuts"
        );
        app.backend.shutdown();
    }
    #[test]
    fn scrollbar_fetches_distant_ranges_and_small_playlists_need_no_jump() {
        let mut app = test_app();
        app.backend.set_offline(true);
        let page = Page::Playlist("p".into());
        let ctx = egui::Context::default();
        app.attach(&ctx);
        let items = make_test_tracks();
        remember_table_items(&mut app, page.clone(), 1, 1, 0, items.clone());
        let cache = app.table_rows.get_mut(&page).unwrap();
        cache.playlist_positions = Some(Arc::new(vec![0, 1, 2, 3]));
        cache.playlist_raw_count = 4;
        let render = |app: &mut App, total, scroll| {
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(900.0, 700.0))),
                    ..Default::default()
                },
                |ui| {
                    egui::ScrollArea::vertical()
                        .id_salt(total)
                        .vertical_scroll_offset(scroll)
                        .show(ui, |ui| {
                            table(
                                app,
                                ui,
                                Table {
                                    total_rows: Some(total),
                                    items: &items,
                                    row_offset: 0,
                                    context: RowContext::Context {
                                        uri: "spotify:playlist:p".into(),
                                        editable_playlist: None,
                                    },
                                    show_album: false,
                                    show_cover: false,
                                    show_added: false,
                                    show_added_by: false,
                                    page: page.clone(),
                                    loading: false,
                                    error: None,
                                    can_load_more: total > 4,
                                    filter: "",
                                    items_revision: 1,
                                },
                            );
                        });
                },
            );
            output.textures_delta.clear();
        };
        render(&mut app, 4, 0.0);
        assert!(
            !app.actions
                .iter()
                .any(|a| matches!(a, Action::JumpToPlaylistPosition { .. }))
        );
        app.actions.clear();
        render(&mut app, 10_000, 400_000.0);
        render(&mut app, 10_000, 400_000.0);
        assert!(app.actions.iter().any(
            |a| matches!(a, Action::JumpToPlaylistPosition { position, .. } if *position > 5_000)
        ));
        app.backend.shutdown();
    }

    use super::*;
    use crate::api::models::{Album, ArtistRef, Image, Track};
    use crate::model::PlaylistPage;

    fn make_large_tracks(count: usize) -> Vec<TableItem> {
        (0..count)
            .map(|i| {
                let track = Track {
                    id: Some(format!("t_{i}")),
                    name: format!("Nested metadata song {i} with a longer title"),
                    uri: format!("spotify:track:large-{i}"),
                    duration_ms: 180_000,
                    artists: vec![ArtistRef {
                        id: Some(format!("artist-{i}")),
                        name: format!("Nested Artist Name {i}"),
                        uri: Some(format!("spotify:artist:artist-{i}")),
                    }],
                    album: Some(Album {
                        id: format!("alb-{i}"),
                        name: format!("Nested Album Title {i}"),
                        uri: format!("spotify:album:alb-{i}"),
                        images: vec![
                            Image {
                                url: format!("https://i.scdn.co/image/large-{i}-640"),
                                width: Some(640),
                                height: Some(640),
                            },
                            Image {
                                url: format!("https://i.scdn.co/image/large-{i}-300"),
                                width: Some(300),
                                height: Some(300),
                            },
                        ],
                        ..Album::default()
                    }),
                    ..Track::default()
                };
                (PlayableItem::Track(track), None, None)
            })
            .collect()
    }

    fn names_only_bytes(items: &[TableItem]) -> usize {
        items
            .iter()
            .map(|(item, ..)| item.uri().len() + item.name().len())
            .sum()
    }

    fn make_test_tracks() -> Vec<TableItem> {
        let titles = [
            "Bohemian Rhapsody",
            "Cancion Animal",
            "Despacito",
            "Ubermensch",
        ];
        let artists = ["Queen", "Soda Stereo", "Luis Fonsi", "Rammstein"];
        let albums = [
            "A Night at the Opera",
            "Cancion Animal Remastered",
            "Vida",
            "Mutter",
        ];

        (0..4)
            .map(|i| {
                let track = Track {
                    id: Some(format!("t_{i}")),
                    name: titles[i].to_string(),
                    uri: format!("spotify:track:t_{i}"),
                    duration_ms: (i as u32 + 1) * 60_000,
                    track_number: Some(i as u32 + 1),
                    disc_number: Some(1),
                    explicit: false,
                    is_local: false,
                    is_playable: Some(true),
                    artists: vec![
                        ArtistRef {
                            id: Some(format!("a_{i}")),
                            name: artists[i].to_string(),
                            uri: Some(format!("spotify:artist:a_{i}")),
                        },
                        ArtistRef {
                            id: Some(format!("feat_{i}")),
                            name: format!("Feat Artist {i}"),
                            uri: Some(format!("spotify:artist:feat_{i}")),
                        },
                    ],
                    album: Some(Album {
                        id: format!("alb_{i}"),
                        name: albums[i].to_string(),
                        uri: format!("spotify:album:alb_{i}"),
                        images: vec![],
                        release_date: Some("2020-01-01".to_string()),
                        album_type: Some("album".to_string()),
                        artists: vec![],
                        album_group: None,
                        total_tracks: Some(10),
                        label: None,
                        genres: vec![],
                        popularity: None,
                        tracks: None,
                        copyrights: vec![],
                        external_urls: Default::default(),
                    }),
                    popularity: None,
                    external_ids: Default::default(),
                    linked_from: None,
                    external_urls: Default::default(),
                };
                (
                    PlayableItem::Track(track),
                    Some(format!("2024-01-0{i}")),
                    Some(format!("User {i}")),
                )
            })
            .collect()
    }

    #[test]
    fn test_view_indices_filtering_and_sorting() {
        let items = make_test_tracks();

        // 1. Unfiltered and unsorted: natural order
        let visible = view_indices(&items, "", None);
        assert_eq!(visible, vec![0, 1, 2, 3]);

        // 2. Filter by track name
        let visible = view_indices(&items, "bohemian", None);
        assert_eq!(visible, vec![0]);

        // 3. Filter by artist name
        let visible = view_indices(&items, "soda", None);
        assert_eq!(visible, vec![1]);

        // 4. Filter by album name
        let visible = view_indices(&items, "mutter", None);
        assert_eq!(visible, vec![3]);

        // 5. Sort descending by title
        let sort = Some(TableSort {
            column: SortColumn::Title,
            ascending: false,
        });
        let visible = view_indices(&items, "", sort);
        assert_eq!(visible, vec![3, 2, 1, 0]);
    }

    #[test]
    fn text_sorts_preserve_case_insensitive_ties_in_both_directions() {
        let mut items = make_test_tracks();
        for (item, label) in items.iter_mut().zip(["Beta", "alpha", "ALPHA", "zeta"]) {
            let PlayableItem::Track(track) = &mut item.0 else {
                panic!("test rows are tracks");
            };
            track.name = label.into();
            track.album.as_mut().unwrap().name = label.into();
            item.2 = Some(label.into());
        }

        for column in [SortColumn::Title, SortColumn::Album, SortColumn::AddedBy] {
            assert_eq!(
                view_indices(
                    &items,
                    "",
                    Some(TableSort {
                        column,
                        ascending: true,
                    }),
                ),
                vec![1, 2, 0, 3],
                "{column:?} ascending"
            );
            assert_eq!(
                view_indices(
                    &items,
                    "",
                    Some(TableSort {
                        column,
                        ascending: false,
                    }),
                ),
                vec![3, 0, 1, 2],
                "{column:?} descending must keep tied rows in playlist order"
            );
        }
    }

    #[test]
    fn text_sort_normalizes_each_visible_row_once() {
        let labels = ["Beta", "alpha", "ALPHA", "zeta"];
        let mut visible = [0, 1, 2, 3];
        let calls = std::cell::Cell::new(0);

        sort_by_text_key(&mut visible, false, |index| {
            calls.set(calls.get() + 1);
            labels[index].to_lowercase()
        });

        assert_eq!(calls.get(), visible.len());
        assert_eq!(visible, [3, 0, 1, 2]);
    }

    #[test]
    fn test_table_cache_validation() {
        let sort = Some(TableSort {
            column: SortColumn::Title,
            ascending: true,
        });
        let cache = TableCache {
            sort,
            needle: "desp".to_string(),
            items_revision: 5,
            user_names_revision: 2,
            visible: Arc::new([2]),
            view_uris: Some(Arc::new(["spotify:track:t_2".to_string()])),
        };

        // Cache hit
        assert!(
            cache.sort == sort
                && cache.needle == "desp"
                && cache.items_revision == 5
                && cache.user_names_revision == 2
        );

        // Cache miss on sort change
        let diff_sort = Some(TableSort {
            column: SortColumn::Title,
            ascending: false,
        });
        assert_ne!(cache.sort, diff_sort);

        // Cache miss on filter change
        assert_ne!(cache.needle, "bohemian");

        // Cache miss on items_revision change
        assert_ne!(cache.items_revision, 6);

        // Cache miss on user_names_revision change
        assert_ne!(cache.user_names_revision, 3);
    }

    #[test]
    fn a_direct_playlist_page_keeps_spotify_row_numbers() {
        assert_eq!(absolute_row_index(6_900, 0) + 1, 6_901);
        assert_eq!(absolute_row_index(6_900, 6) + 1, 6_907);
    }

    fn test_app() -> App {
        let root = std::env::temp_dir().join(format!(
            "magicspot-table-cache-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        App::new(
            &crate::backend::Waker::default(),
            crate::paths::AppDirs {
                config: root.join("config"),
                state: root.join("state"),
                cache: root.join("cache"),
            },
            crate::settings::Settings::default(),
            crate::app::AppOptions {
                media_controls: false,
                tray: false,
            },
        )
    }

    #[test]
    fn table_row_cache_hits_until_revision_or_generation_changes() {
        let mut app = test_app();
        let mut builds = 0;
        let page = Page::LikedSongs;
        cached_table_items(&mut app, page.clone(), 1, 0, 0, || {
            builds += 1;
            make_test_tracks()
        });
        cached_table_items(&mut app, page.clone(), 1, 0, 0, || {
            builds += 1;
            panic!("cache hit rebuilt the table");
        });
        assert_eq!(builds, 1);
        cached_table_items(&mut app, page.clone(), 1, 1, 0, || {
            builds += 1;
            make_test_tracks()
        });
        assert_eq!(builds, 2, "revision change must rebuild");
        cached_table_items(&mut app, page, 2, 1, 0, || {
            builds += 1;
            make_test_tracks()
        });
        assert_eq!(builds, 3, "generation change must rebuild");
    }

    #[test]
    fn table_row_cache_memory_counts_nested_metadata_on_a_large_collection() {
        let mut app = test_app();
        let items = make_large_tracks(500);
        let names_only = names_only_bytes(&items);
        assert_eq!(app.table_rows_retained_bytes(), 0);
        cached_table_items(&mut app, Page::LikedSongs, 0, 0, 0, || items);
        let after = app.table_rows_retained_bytes();
        assert!(
            after > names_only,
            "retained bytes must include nested album, artist, and image strings, not just titles: names_only={names_only} after={after}"
        );
        assert!(
            after > 80_000,
            "500 tracks with nested metadata should retain a substantial copy: {after}"
        );
    }

    #[test]
    fn table_row_cache_drops_when_the_backing_page_is_evicted() {
        let mut app = test_app();
        let page = Page::Playlist("pl-gone".into());
        app.playlist_pages
            .insert("pl-gone".into(), PlaylistPage::default());
        cached_table_items(&mut app, page.clone(), 1, 0, 0, make_test_tracks);
        assert!(app.table_rows.contains_key(&page));
        app.playlist_pages.remove("pl-gone");
        app.open(Page::LikedSongs);
        assert!(
            !app.table_rows.contains_key(&page),
            "evicting the page map must drop the table-row copy"
        );
    }

    #[test]
    fn table_row_cache_keeps_at_most_two_pages() {
        let mut app = test_app();
        for i in 0..5 {
            let page = Page::Playlist(format!("pl{i}"));
            app.playlist_pages
                .insert(format!("pl{i}"), PlaylistPage::default());
            app.history.push(page.clone());
            app.history_index = app.history.len() - 1;
            cached_table_items(&mut app, page, 1, 0, 0, make_test_tracks);
        }
        assert_eq!(app.table_rows.len(), 2);
        assert!(
            app.table_rows.contains_key(&Page::Playlist("pl4".into())),
            "the open page stays"
        );
    }
}
