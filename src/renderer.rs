use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;

use crate::{
    display::Display,
    json::Json,
    style::{StyledJson, StyledStr, Styler},
};

const INDENT: usize = 4;

pub type Vec2 = (usize, usize);

/// Renders the JSON tree onto the display buffer.
pub struct Renderer {
    display: Display,
    style_map: HashMap<String, StyledJson>,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            display: Display::new()?,
            style_map: HashMap::new(),
        })
    }

    pub fn resize(&mut self, size: Vec2) -> bool {
        self.display.resize(size)
    }

    /// Take a JSON object and applies styling to it.
    fn render_json(&mut self, json: &Json) -> StyledJson {
        self.style_map
            .entry(json.pointer.active_path().join(""))
            .or_insert(Styler::style_json(json))
            .to_owned()
    }

    pub fn draw(&mut self, filepath: &Option<PathBuf>, json: &Json) -> Result<()> {
        self.draw_title((0, 0), filepath, json);
        self.draw_tree((0, 1), json);

        self.display.show()
    }

    fn draw_title(&mut self, (x, y): Vec2, filepath: &Option<PathBuf>, json: &Json) {
        let mut title = String::new();

        match &filepath {
            Some(path) => {
                let path = format!("{}", path.display());

                // TODO try to shorten path if too long
                let w = self.display.size.0;
                let path = if path.len() > w { &path[..w] } else { &path };

                title.push_str(path);
            }
            _ => {
                let stdin = "stdin";
                title.push_str(stdin);
            }
        }

        let (x, y) = self.display.draw(
            x,
            y,
            &StyledStr {
                style: crate::style::STYLE_TITLE,
                text: title,
            },
        );

        self.draw_path((x + 1, y), json);
    }

    fn draw_path(&mut self, (x, y): Vec2, json: &Json) {
        self.display.clear((x, y), (self.display.size.0, 1), None);

        let path = format!("{}", json.pointer);

        self.display.draw(
            x,
            y,
            &StyledStr {
                style: crate::style::STYLE_POINTER,
                text: path,
            },
        );
    }

    fn draw_tree(&mut self, (x, y): Vec2, json: &Json) {
        let styled = self.render_json(json);

        let (selection_top, selection_bottom) = styled.selection;
        self.display.clear((x, y), self.display.size, None);

        let (_, h) = self.display.size;

        let top = if selection_top < h / 2 {
            0
        } else if selection_bottom > styled.lines.len() - h / 2 {
            styled.lines.len() - h
        } else {
            selection_top - h / 2
        };

        let mut y = y;
        for (depth, styled_str) in styled.lines.iter().skip(top).take(h) {
            let x = x + depth * INDENT;
            self.display.draw(x, y, styled_str);
            y += 1;
        }
    }
}
