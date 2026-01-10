use std::path::PathBuf;

use arboard::Clipboard;
use serde_json::{to_string_pretty, Value};

use crate::events::{read_event, Action::*, Direction::*};
use crate::json::Json;
use crate::search::{perform_search, SearchResults};
use crate::ui::{FlashMode, UI};

/// Starts the main loop responsible for listening to user events and triggering UI updates.
pub fn event_loop(
    filepath: &Option<PathBuf>,
    mut json: Json,
    no_numbers: bool,
) -> anyhow::Result<Option<String>> {
    let mut clipboard = Clipboard::new()?;

    let mut ui = UI::new(no_numbers)?;

    let mut output: Option<String> = None;

    // Search state
    let mut search_input: Option<String> = None;
    let mut search_results: Option<SearchResults> = None;
    let mut last_search: Option<SearchResults> = None;

    // Help state
    let mut help_visible = false;

    ui.render(
        filepath,
        &json,
        search_input.as_deref(),
        search_results.as_ref(),
        help_visible,
    )?;

    loop {
        let mut needs_redraw = false;
        let search_mode = search_input.is_some();

        if ui.clear_flash_if_expired() {
            needs_redraw = true;
        }

        let timeout = ui.flash_remaining();

        let action = match read_event(search_mode, help_visible, timeout)? {
            Some(action) => action,
            None => {
                if ui.clear_flash_if_expired() {
                    ui.render(
                        filepath,
                        &json,
                        search_input.as_deref(),
                        search_results.as_ref(),
                        help_visible,
                    )?;
                }
                continue;
            }
        };

        match action {
            Resize(w, h) => {
                needs_redraw = ui.resize((w, h));
            }

            Quit => {
                if search_results.is_some() {
                    last_search = search_results.take();
                    needs_redraw = true;
                } else {
                    break;
                }
            }

            Move(direction) => {
                needs_redraw = match direction {
                    Up => json.go_prev(),
                    Down => json.go_next(),
                    Left => json.go_out(),
                    Right => json.go_in(),
                };
                if needs_redraw {
                    ui.ensure_visible(json.visible_bounds());
                }
            }

            ScrollLine(dir) => {
                let delta = if matches!(dir, Up) { -1 } else { 1 };
                needs_redraw = ui.scroll_y_by(delta, json.visible_line_count());
            }
            ScrollHalf(dir) => {
                let half = (ui.body_height() / 2).max(1) as isize;
                let delta = if matches!(dir, Up) { -half } else { half };
                needs_redraw = ui.scroll_y_by(delta, json.visible_line_count());
            }
            ScrollFull(dir) => {
                let full = ui.body_height().max(1) as isize;
                let delta = if matches!(dir, Up) { -full } else { full };
                needs_redraw = ui.scroll_y_by(delta, json.visible_line_count());
            }
            ScrollTop => {
                needs_redraw = ui.scroll_y_min();
            }
            ScrollBottom => {
                needs_redraw = ui.scroll_y_max(json.visible_line_count());
            }
            ScrollLeft => {
                needs_redraw = ui.scroll_x_by(-4, json.width);
            }
            ScrollRight => {
                needs_redraw = ui.scroll_x_by(4, json.width);
            }
            ScrollLeftMax => {
                needs_redraw = ui.scroll_x_min();
            }
            ScrollRightMax => {
                needs_redraw = ui.scroll_x_max(json.width);
            }

            ToggleFold => {
                needs_redraw = json.toggle_fold();
            }
            ToggleFoldAll => {
                needs_redraw = json.toggle_fold_all();
            }

            Sort => {}
            SortReverse => {}

            Search => {
                search_input = Some(String::new());
                ui.footer_height = 1;
                needs_redraw = true;
            }

            SearchInput(c) => {
                if let Some(ref mut input) = search_input {
                    input.push(c);
                    search_results = Some(perform_search(&json.formatted, input));
                    // Auto-scroll to first match
                    if let Some(ref results) = search_results {
                        let match_len = results.query.chars().count();
                        if let Some(m) = results.matches.first() {
                            ensure_match_visible(
                                &mut ui,
                                &mut json,
                                m.line_number,
                                m.element_index,
                                m.char_offset,
                                match_len,
                            );
                        }
                    }
                    needs_redraw = true;
                }
            }
            SearchBackspace => {
                if let Some(ref mut input) = search_input {
                    input.pop();
                    if input.is_empty() {
                        search_results = None;
                    } else {
                        search_results = Some(perform_search(&json.formatted, input));
                        if let Some(ref results) = search_results {
                            let match_len = results.query.chars().count();
                            if let Some(m) = results.matches.first() {
                                ensure_match_visible(
                                    &mut ui,
                                    &mut json,
                                    m.line_number,
                                    m.element_index,
                                    m.char_offset,
                                    match_len,
                                );
                            }
                        }
                    }
                    needs_redraw = true;
                }
            }
            SearchConfirm => {
                if let Some(ref mut results) = search_results {
                    if !results.matches.is_empty() {
                        results.current_index = Some(0);
                        let match_len = results.query.chars().count();
                        if let Some(m) = results.current() {
                            ensure_match_visible(
                                &mut ui,
                                &mut json,
                                m.line_number,
                                m.element_index,
                                m.char_offset,
                                match_len,
                            );
                        }
                    }
                    last_search = Some(results.clone());
                }
                search_input = None;
                // Keep footer visible if there are results
                ui.footer_height = if search_results.is_some() { 1 } else { 0 };
                needs_redraw = true;
            }
            SearchCancel => {
                // Restore previous search if any
                search_results = last_search.clone();
                search_input = None;
                // Keep footer visible if there are results
                ui.footer_height = if search_results.is_some() { 1 } else { 0 };
                needs_redraw = true;
            }
            RepeatSearch => {
                // Revive search if cleared
                if search_results.is_none() {
                    search_results = last_search.clone();
                    if search_results.is_some() {
                        ui.footer_height = 1;
                    }
                }
                if let Some(ref mut results) = search_results {
                    let match_len = results.query.chars().count();
                    if let Some(m) = results.next() {
                        ensure_match_visible(
                            &mut ui,
                            &mut json,
                            m.line_number,
                            m.element_index,
                            m.char_offset,
                            match_len,
                        );
                        needs_redraw = true;
                    }
                }
            }
            RepeatSearchBackward => {
                if search_results.is_none() {
                    search_results = last_search.clone();
                    if search_results.is_some() {
                        ui.footer_height = 1;
                    }
                }
                if let Some(ref mut results) = search_results {
                    let match_len = results.query.chars().count();
                    if let Some(m) = results.prev() {
                        ensure_match_visible(
                            &mut ui,
                            &mut json,
                            m.line_number,
                            m.element_index,
                            m.char_offset,
                            match_len,
                        );
                        needs_redraw = true;
                    }
                }
            }
            Filter => {}
            ClearSearch => {
                if search_results.is_some() {
                    last_search = search_results.take();
                    ui.footer_height = 0;
                    needs_redraw = true;
                }
            }

            OutputSelectionPretty => {
                if let Some((key, value)) = json.token_value_pair() {
                    output = Some(selection_pretty(key, value)?);
                    break;
                }
            }
            OutputValuePretty => {
                if let Some(value) = json.value() {
                    output = Some(value.to_string());
                    break;
                }
            }
            OutputSelectionRaw => {
                if let Some((key, value)) = json.token_value_pair() {
                    output = Some(selection(key, value)?);
                    break;
                }
            }
            OutputValueRaw => {
                if let Some(value) = json.value() {
                    output = Some(value.to_string());
                    break;
                }
            }

            CopySelectionPretty => {
                if let Some((key, value)) = json.token_value_pair() {
                    clipboard.set_text(selection_pretty(key, value)?)?;
                    ui.start_flash(FlashMode::Selection);
                    needs_redraw = true;
                }
            }
            CopyValuePretty => {
                if let Some(s) = json.value().map(to_string_pretty) {
                    clipboard.set_text(s?)?;
                    ui.start_flash(FlashMode::Value);
                    needs_redraw = true;
                }
            }
            CopySelectionRaw => {
                if let Some((key, value)) = json.token_value_pair() {
                    clipboard.set_text(selection(key, value)?)?;
                    ui.start_flash(FlashMode::Selection);
                    needs_redraw = true;
                }
            }
            CopyValueRaw => {
                if let Some(s) = json.value().map(|v| v.to_string()) {
                    clipboard.set_text(s)?;
                    ui.start_flash(FlashMode::Value);
                    needs_redraw = true;
                }
            }

            ToggleLineNumbers => {
                ui.toggle_line_numbers();
                needs_redraw = true;
            }
            ToggleLineWrapping => {
                ui.toggle_line_wrap();
                needs_redraw = true;
            }

            ShowHelp => {
                help_visible = true;
                needs_redraw = true;
            }
            DismissHelp => {
                help_visible = false;
                needs_redraw = true;
            }

            Ignore => {}
        }

        if needs_redraw {
            ui.render(
                filepath,
                &json,
                search_input.as_deref(),
                search_results.as_ref(),
                help_visible,
            )?;
        }
    }

    Ok(output)
}

/// Unfolds ancestors, sets selection, and scrolls to make a match visible
fn ensure_match_visible(
    ui: &mut UI,
    json: &mut Json,
    line_number: usize,
    element_index: usize,
    char_offset: usize,
    match_len: usize,
) {
    // Extract data from line before mutable operations
    let (pointer, col) = match json.formatted.get(line_number) {
        Some(line) => {
            let mut col = line.indent;
            for (idx, elem) in line.elements.iter().enumerate() {
                if idx == element_index {
                    break;
                }
                col += elem.0.chars().count();
            }
            // Add char_offset to get to the actual match position within the element
            col += char_offset;
            (line.pointer.clone(), col)
        }
        None => return,
    };

    // Unfold any ancestors of this line
    for i in 0..pointer.len() {
        let ancestor = pointer[..=i].to_vec();
        json.folds.remove(&ancestor);
    }

    // Set the selection to the matched node's path
    json.set_selection(pointer);

    // Scroll horizontally to make the match visible
    ui.ensure_x_visible(col, match_len);

    // Scroll vertically to make visible
    if let Some(visible_line) = json.line_to_visible(line_number) {
        ui.ensure_visible((visible_line, visible_line));
    }
}

fn selection(key: Option<String>, value: &Value) -> anyhow::Result<String> {
    Ok(if let Some(key) = key {
        format!("\"{}\": {}", key, value)
    } else {
        value.to_string()
    })
}

fn selection_pretty(key: Option<String>, value: &Value) -> anyhow::Result<String> {
    Ok(if let Some(key) = key {
        format!("\"{}\": {}", key, to_string_pretty(value)?)
    } else {
        to_string_pretty(value)?
    })
}
