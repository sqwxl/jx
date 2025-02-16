use std::path::PathBuf;

use crossterm::{
    cursor, queue,
    style::{
        Attribute, Attributes, Color, ContentStyle, PrintStyledContent, ResetColor, SetAttributes,
        SetUnderlineColor,
    },
};

use crate::{
    formatter::{self, PointerMap, StyledLine},
    json::Json,
    screen::Screen,
};

pub const STYLE_SELECTION: ContentStyle = ContentStyle {
    foreground_color: None,
    background_color: None,
    attributes: Attributes::with(Attributes::none(), Attribute::Underlined),
    underline_color: Some(Color::White),
};

const STYLE_TITLE: ContentStyle = ContentStyle {
    foreground_color: Some(Color::White),
    background_color: None,
    attributes: Attributes::none(),
    underline_color: None,
};

const STYLE_POINTER: ContentStyle = STYLE_TITLE;

pub type Vec2 = (usize, usize);

/// Builds the UI and sends it off to be rendered.
#[derive(Default)]
pub struct UI {
    json_lines: Vec<StyledLine>,
    pointer_map: PointerMap,
    screen: Screen,
    header_height: usize,
    footer_height: usize,
}

impl From<&serde_json::Value> for UI {
    fn from(value: &serde_json::Value) -> Self {
        let (json_lines, pointer_map) = formatter::format(value);

        Self {
            json_lines,
            pointer_map,
            header_height: 1,
            footer_height: 0,
            ..Default::default()
        }
    }
}

impl UI {
    pub fn resize(&mut self, size: Vec2) -> bool {
        self.screen.resize(size)
    }

    pub fn render(&mut self, filepath: &Option<PathBuf>, json: &Json) -> anyhow::Result<()> {
        self.screen.clear()?;

        self.render_header(filepath, &json.path())?;
        self.render_body(json, (0, self.header_height))?;

        self.screen.print()
    }

    fn render_header(&mut self, filepath: &Option<PathBuf>, pointer: &str) -> anyhow::Result<()> {
        let mut title_text = String::new();

        match &filepath {
            Some(path) => {
                let path = format!("{}", path.display());

                // TODO try to shorten path if too long
                let w = self.screen.size.0;
                let path = if path.len() > w { &path[..w] } else { &path };

                title_text.push_str(path);
            }
            _ => {
                let stdin = "stdin";
                title_text.push_str(stdin);
            }
        }

        let pointer_text = " ".to_owned() + pointer;

        queue!(self.screen.out, cursor::MoveToColumn(0), ResetColor)?;

        queue!(
            self.screen.out,
            PrintStyledContent(STYLE_TITLE.apply(&title_text))
        )?;

        queue!(
            self.screen.out,
            PrintStyledContent(STYLE_POINTER.apply(&pointer_text))
        )?;

        Ok(())
    }

    fn render_body(&mut self, json: &Json, offset: (usize, usize)) -> anyhow::Result<()> {
        let selection_bounds = match self.pointer_map.get(&json.tokens()) {
            Some(&bounds) => bounds,
            None => (0, self.json_lines.len()),
        };

        let json_lines = &self.json_lines;

        for (
            j,
            StyledLine {
                elements,
                indent,
                line_number,
            },
        ) in json_lines.iter().enumerate()
        {
            queue!(
                self.screen.out,
                cursor::MoveTo((*indent + offset.0) as u16, (j + offset.1) as u16,),
                ResetColor
            )?;

            for (text, style) in elements.iter() {
                if selection_bounds.0 <= *line_number && *line_number <= selection_bounds.1 {
                    queue!(
                        self.screen.out,
                        SetAttributes(STYLE_SELECTION.attributes),
                        SetUnderlineColor(STYLE_SELECTION.underline_color.unwrap())
                    )?;
                }
                queue!(self.screen.out, PrintStyledContent(style.apply(text)))?;
            }
        }

        Ok(())
    }

    fn render_footer(&mut self) {
        // self.screen.draw_line(&self.footer);
        // TODO: Show keyboard shortcuts
    }
}
