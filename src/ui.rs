use std::path::PathBuf;

use crossterm::{
    cursor, queue,
    style::{Print, PrintStyledContent, ResetColor},
};

use crate::{
    json::{bracket_fold, curly_fold, Json, PointerData, PointerValue},
    screen::Screen,
    style::{StyledLine, STYLE_POINTER, STYLE_SELECTION_BAR, STYLE_TITLE},
};

/// Builds the UI and sends it off to be rendered.
pub struct UI {
    screen: Screen,
    header_height: usize,
    footer_height: usize,
    scroll_offset: usize,
    scroll_x: usize,
    line_wrap: bool,
}

impl UI {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            header_height: 1,
            footer_height: 0,
            scroll_offset: 0,
            scroll_x: 0,
            line_wrap: false,
        })
    }

    pub fn toggle_line_wrap(&mut self) {
        self.line_wrap = !self.line_wrap;
        self.scroll_x = 0; // Reset horizontal scroll when toggling wrap
    }

    pub fn scroll_x_by(&mut self, delta: isize) -> bool {
        let old = self.scroll_x;
        self.scroll_x = if delta < 0 {
            self.scroll_x.saturating_sub(delta.unsigned_abs())
        } else {
            self.scroll_x + delta as usize
        };
        self.scroll_x != old
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

            let is_selected =
                selection_bounds.0 <= *line_number && *line_number <= selection_bounds.1;

            // Draw selection indicator bar
            if is_selected {
                queue!(
                    self.screen.out,
                    cursor::MoveTo(offset.0 as u16, cursor_y as u16),
                    PrintStyledContent(STYLE_SELECTION_BAR.apply("▌"))
                )?;
            }

            // Calculate visible start position accounting for horizontal scroll
            let visible_start = indent.saturating_sub(self.scroll_x);
            queue!(
                self.screen.out,
                cursor::MoveTo((visible_start + offset.0) as u16, cursor_y as u16),
                ResetColor
            )?;

            let mut col = *indent; // Absolute column position
            let scroll_end = self.scroll_x + max_col; // Right edge of visible area

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
                    let text = &el.0;
                    for ch in text.chars() {
                        if col >= scroll_end {
                            break;
                        }
                        if col >= self.scroll_x {
                            queue!(self.screen.out, Print(el.1.apply(ch)))?;
                        }
                        col += 1;
                    }
                }
                line_idx = bounds.1 + 1;
            } else {
                // Find continuation column (after ": " for object entries)
                let mut continuation_col = *indent;
                let mut found_colon = false;
                for el in elements.iter() {
                    if found_colon {
                        break;
                    }
                    continuation_col += el.0.chars().count();
                    if el.0 == ": " {
                        found_colon = true;
                    }
                }
                if !found_colon {
                    continuation_col = *indent;
                }

                for el in elements.iter() {
                    let text = &el.0;

                    if self.line_wrap {
                        // Print with manual wrapping (no horizontal scroll in wrap mode)
                        for ch in text.chars() {
                            if col >= max_col {
                                cursor_y += 1;
                                col = continuation_col;
                                queue!(
                                    self.screen.out,
                                    cursor::MoveTo(col as u16, cursor_y as u16)
                                )?;
                            }
                            queue!(self.screen.out, Print(el.1.apply(ch)))?;
                            col += 1;
                        }
                    } else {
                        // Print char by char to handle UTF-8 correctly
                        for ch in text.chars() {
                            if col >= scroll_end {
                                break;
                            }
                            if col >= self.scroll_x {
                                queue!(self.screen.out, Print(el.1.apply(ch)))?;
                            }
                            col += 1;
                        }
                    }
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
