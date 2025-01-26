use crate::events::Action::*;
use crate::events::Direction::*;
use crate::json::Json;
use crate::renderer::Renderer;
use anyhow::Result;
use arboard::Clipboard;
use serde_json::to_string_pretty;
use std::path::PathBuf;

/// Starts the main loop responsible for listening to user events and triggering UI updates.
pub fn run(filepath: &Option<PathBuf>, mut json: Json) -> Result<()> {
    let filepath = filepath.clone();

    let mut renderer = Renderer::new()?;

    renderer.draw(&filepath, &mut json)?;

    let mut clipboard = Clipboard::new()?;

    loop {
        let mut needs_redraw = false;

        match crate::events::user_event()? {
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
            Fold => {
                todo!();
            }
            Scroll(direction) => {
                needs_redraw = match direction {
                    Up => todo!(),
                    Down => todo!(),
                    _ => false,
                }
            }
            Resize(w, h) => {
                needs_redraw = renderer.resize((w, h));
            }
            // TODO: Visual feedback
            CopySelection => {
                if let Some(s) = json.get_value(None).map(to_string_pretty) {
                    clipboard.set_text(s?)?;
                }
            }
            CopyRawValue => {
                if let Some(s) = json.get_value(None).map(|v| v.to_string()) {
                    clipboard.set_text(s)?;
                }
            }
            _ => {}
        }

        if needs_redraw {
            renderer.draw(&filepath, &mut json)?;
        }
    }

    Ok(())
}
