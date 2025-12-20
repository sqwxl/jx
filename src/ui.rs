use std::path::PathBuf;

use crossterm::{
    cursor, queue,
    style::{Print, PrintStyledContent, ResetColor, SetAttributes, SetUnderlineColor},
};

use crate::{
    json::{bracket_fold, curly_fold, Json, PointerData, PointerValue},
    screen::Screen,
    style::{StyledLine, STYLE_POINTER, STYLE_SELECTION, STYLE_TITLE},
};

/// Builds the UI and sends it off to be rendered.
pub struct UI {
    screen: Screen,
    header_height: usize,
    footer_height: usize,
    scroll_offset: usize,
    line_wrap: bool,
}

impl UI {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            header_height: 1,
            footer_height: 0,
            scroll_offset: 0,
            line_wrap: false,
        })
    }

    pub fn toggle_line_wrap(&mut self) {
        self.line_wrap = !self.line_wrap;
    }

    pub fn body_height(&self) -> usize {
        self.screen
            .size
            .1
            .saturating_sub(self.header_height + self.footer_height)
    }

    pub fn scroll_by(&mut self, delta: isize, max_lines: usize) -> bool {
        let old = self.scroll_offset;
        let max_offset = max_lines.saturating_sub(1);
        self.scroll_offset = if delta < 0 {
            self.scroll_offset.saturating_sub(delta.unsigned_abs())
        } else {
            (self.scroll_offset + delta as usize).min(max_offset)
        };
        self.scroll_offset != old
    }

    pub fn scroll_to_top(&mut self) -> bool {
        let old = self.scroll_offset;
        self.scroll_offset = 0;
        self.scroll_offset != old
    }

    pub fn scroll_to_bottom(&mut self, max_lines: usize) -> bool {
        let old = self.scroll_offset;
        self.scroll_offset = max_lines.saturating_sub(self.body_height());
        self.scroll_offset != old
    }

    pub fn ensure_visible(&mut self, bounds: (usize, usize)) {
        let body_height = self.body_height();
        if bounds.0 < self.scroll_offset {
            self.scroll_offset = bounds.0;
        } else if bounds.1 >= self.scroll_offset + body_height {
            self.scroll_offset = bounds.1.saturating_sub(body_height - 1);
        }
    }

    pub fn resize(&mut self, size: (usize, usize)) -> bool {
        self.screen.resize(size)
    }

    pub fn render(&mut self, filepath: &Option<PathBuf>, json: &Json) -> anyhow::Result<()> {
        self.screen.clear()?;

        self.render_header(filepath, &json.period_path())?;
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
        let selection_bounds = json.bounds();
        let mut line_idx = 0;
        let mut visible_line = 0;
        let mut cursor_y = offset.1;
        let max_col = size.0;

        while let Some(StyledLine {
            line_number,
            indent,
            pointer,
            elements,
        }) = json.formatted.get(line_idx)
        {
            let is_folded = json.folds.contains(pointer);
            let fold_data = is_folded.then(|| json.pointer_map.get(pointer).unwrap());

            // Skip lines before scroll_offset
            if visible_line < self.scroll_offset {
                visible_line += 1;
                line_idx = if let Some(data) = fold_data {
                    data.bounds.1 + 1
                } else {
                    line_idx + 1
                };
                continue;
            }

            // Stop if we've filled the screen
            if cursor_y >= offset.1 + size.1 {
                break;
            }

            queue!(
                self.screen.out,
                cursor::MoveTo((*indent + offset.0) as u16, cursor_y as u16),
                ResetColor
            )?;

            let mut col = *indent;

            if let Some(PointerData {
                value,
                bounds,
                children,
            }) = fold_data
            {
                let key = pointer.last().and_then(|t| t.as_key());
                let fold_string = match value {
                    PointerValue::Object => curly_fold(key.as_deref(), *children),
                    PointerValue::Array => bracket_fold(*children),
                    PointerValue::Primitive => panic!("should not fold primitives"),
                };
                for el in &fold_string {
                    if !self.line_wrap && col >= max_col {
                        break;
                    }
                    let text = &el.0;
                    if !self.line_wrap && col + text.len() > max_col {
                        let remaining = max_col.saturating_sub(col);
                        if remaining > 0 {
                            let truncated = &text[..remaining];
                            queue!(self.screen.out, Print(el.1.apply(truncated)))?;
                        }
                        break;
                    }
                    queue!(self.screen.out, Print(el))?;
                    col += text.len();
                }
                line_idx = bounds.1 + 1;
            } else {
                for el in elements.iter() {
                    if !self.line_wrap && col >= max_col {
                        break;
                    }
                    if selection_bounds.0 <= *line_number && *line_number <= selection_bounds.1 {
                        queue!(
                            self.screen.out,
                            SetAttributes(STYLE_SELECTION.attributes),
                            SetUnderlineColor(STYLE_SELECTION.underline_color.unwrap())
                        )?;
                    }
                    let text = &el.0;
                    if !self.line_wrap && col + text.len() > max_col {
                        let remaining = max_col.saturating_sub(col);
                        if remaining > 0 {
                            let truncated = &text[..remaining];
                            queue!(self.screen.out, Print(el.1.apply(truncated)))?;
                        }
                        break;
                    }
                    queue!(self.screen.out, Print(el))?;
                    col += text.len();
                }
                line_idx += 1;
            }

            visible_line += 1;
            cursor_y += 1;
        }
        Ok(())
    }

    fn render_footer(&mut self) {
        // self.screen.draw_line(&self.footer);
        // TODO: Show keyboard shortcuts
    }
}
