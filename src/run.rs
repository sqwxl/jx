use std::path::PathBuf;

use anyhow::Result;
use arboard::Clipboard;
use serde_json::{to_string_pretty, Value};

use crate::events::{read_event, Action::*, Direction::*};
use crate::json::Json;
use crate::renderer::Renderer;

/// Starts the main loop responsible for listening to user events and triggering UI updates.
pub fn run(filepath: &Option<PathBuf>, mut json: Json) -> Result<Option<String>> {
    let mut clipboard = Clipboard::new()?;

    let mut output: Option<String> = None;

    let mut renderer = Renderer::new()?;

    renderer.draw(filepath, &mut json)?;

    loop {
        let mut needs_redraw = false;

        match read_event()? {
            Quit => {
                break;
            }

            Move(direction) => {
                needs_redraw = match direction {
                    Up => json.go_prev(),
                    Down => json.go_next(),
                    Left => json.go_out(),
                    Right => json.go_in(),
                }
            }

            Scroll(direction) => {
                needs_redraw = match direction {
                    Up => todo!(),
                    Down => todo!(),
                    _ => false,
                }
            }

            ScrollPage(direction) => {}

            Resize(w, h) => {
                needs_redraw = renderer.resize((w, h));
            }

            Fold => {
                todo!();
            }

            OutputSelectionPretty => {
                if let Some((key, value)) = json.get_selection() {
                    output = Some(selection_pretty(key, value)?);
                    break;
                }
            }
            OutputValuePretty => {
                if let Some(value) = json.get_value(None) {
                    output = Some(value.to_string());
                    break;
                }
            }
            OutputSelection => {
                if let Some((key, value)) = json.get_selection() {
                    output = Some(selection(key, value)?);
                    break;
                }
            }
            OutputValue => {
                if let Some(value) = json.get_value(None) {
                    output = Some(value.to_string());
                    break;
                }
            }

            // TODO: Visual feedback
            CopySelectionPretty => {
                if let Some((key, value)) = json.get_selection() {
                    clipboard.set_text(selection_pretty(key, value)?)?;
                }
            }
            CopyValuePretty => {
                if let Some(s) = json.get_value(None).map(to_string_pretty) {
                    clipboard.set_text(s?)?;
                }
            }
            CopySelection => {
                if let Some((key, value)) = json.get_selection() {
                    clipboard.set_text(selection(key, value)?)?;
                }
            }
            CopyValue => {
                if let Some(s) = json.get_value(None).map(|v| v.to_string()) {
                    clipboard.set_text(s)?;
                }
            }

            Ignore => {}
        }

        if needs_redraw {
            renderer.draw(filepath, &mut json)?;
        }
    }

    Ok(output)
}

fn selection(key: Option<String>, value: &Value) -> Result<String> {
    Ok(if let Some(key) = key {
        format!("\"{}\": {}", key, value)
    } else {
        value.to_string()
    })
}

fn selection_pretty(key: Option<String>, value: &Value) -> Result<String> {
    Ok(if let Some(key) = key {
        format!("\"{}\": {}", key, to_string_pretty(value)?)
    } else {
        to_string_pretty(value)?
    })
}
