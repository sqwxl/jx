use std::path::PathBuf;

use crossterm::{
    cursor, queue,
    style::{Print, PrintStyledContent, ResetColor, SetAttributes, SetUnderlineColor},
};

use crate::{
    json::{bracket_fold, curly_fold, Json, NodeType, PointerData},
    screen::Screen,
    style::{StyledLine, STYLE_POINTER, STYLE_SELECTION, STYLE_TITLE},
};

/// Builds the UI and sends it off to be rendered.
pub struct UI {
    screen: Screen,
    header_height: usize,
    footer_height: usize,
}

impl UI {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            header_height: 1,
            footer_height: 0,
        })
    }

    pub fn resize(&mut self, size: (usize, usize)) -> bool {
        self.screen.resize(size)
    }

    pub fn render(&mut self, filepath: &Option<PathBuf>, json: &Json) -> anyhow::Result<()> {
        self.screen.clear()?;

        self.render_header(filepath, &json.path())?;
        self.render_body(
            json,
            (0, self.header_height),
            (self.screen.size.0, self.screen.size.1 - self.header_height),
        )?;

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

    fn render_body(
        &mut self,
        json: &Json,
        offset: (usize, usize),
        size: (usize, usize),
    ) -> anyhow::Result<()> {
        let selection_bounds = match json.pointer_map.get(&json.tokens()) {
            Some(&PointerData { bounds, .. }) => bounds,
            None => (0, json.formatted.len()),
        };

        // TODO: Skip to visible line
        let mut line_idx = 0;

        let mut cursor_y = offset.1;

        while let Some(StyledLine {
            line_number,
            indent,
            pointer,
            elements,
        }) = json.formatted.get(line_idx)
        {
            if cursor_y >= size.1 {
                break;
            }

            queue!(
                self.screen.out,
                cursor::MoveTo((*indent + offset.0) as u16, (cursor_y) as u16,),
                ResetColor
            )?;

            cursor_y += 1;

            if json.folds.contains(pointer) {
                let &PointerData {
                    node_type,
                    bounds,
                    children,
                } = json.pointer_map.get(pointer).unwrap();
                let fold_string = match node_type {
                    NodeType::Object => curly_fold(children),
                    NodeType::Array => bracket_fold(children),
                    NodeType::Value => vec![],
                };

                fold_string
                    .iter()
                    .for_each(|el| queue!(self.screen.out, Print(el)).expect("should print"));
                line_idx = bounds.1 + 1;
                continue;
            }

            for el in elements.iter() {
                if selection_bounds.0 <= *line_number && *line_number <= selection_bounds.1 {
                    queue!(
                        self.screen.out,
                        SetAttributes(STYLE_SELECTION.attributes),
                        SetUnderlineColor(STYLE_SELECTION.underline_color.unwrap())
                    )?;
                }
                queue!(self.screen.out, Print(el))?;
            }

            line_idx += 1;
        }
        Ok(())
    }

    fn render_footer(&mut self) {
        // self.screen.draw_line(&self.footer);
        // TODO: Show keyboard shortcuts
    }
}
