use std::path::PathBuf;

use crossterm::{
    cursor, queue,
    style::{ContentStyle, Print, PrintStyledContent, ResetColor},
};

use crate::{
    help::render_help,
    json::{bracket_fold, curly_fold, Json, PointerData, PointerValue},
    screen::Screen,
    search::SearchResults,
    style::{
        styled, StyledLine, STYLE_HEADER, STYLE_LINE_NUMBER, STYLE_SEARCH_MATCH,
        STYLE_SEARCH_MATCH_CURRENT, STYLE_SEARCH_PROMPT, STYLE_SEARCH_STATUS, STYLE_SELECTION_BAR,
    },
};

static SELECTION_SYM: &str = "┃";
static SELECTION_COL_WIDTH: usize = 1;

/// Builds the UI and sends it off to be rendered.
pub struct UI {
    screen: Screen,
    header_height: usize,
    pub footer_height: usize,
    scroll_offset: usize,
    scroll_x: usize,
    line_wrap: bool,
    no_numbers: bool,
}

impl UI {
    pub fn new(no_numbers: bool) -> anyhow::Result<Self> {
        Ok(Self {
            screen: Screen::new()?,
            header_height: 1,
            footer_height: 0,
            scroll_offset: 0,
            scroll_x: 0,
            line_wrap: false,
            no_numbers,
        })
    }

    pub fn toggle_line_numbers(&mut self) {
        self.no_numbers = !self.no_numbers;
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
        let max_offset = max_lines.saturating_sub(self.screen.size.1 - 1);
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
        if bounds.0 <= self.scroll_offset {
            self.scroll_offset = bounds.0;
        } else if bounds.1 >= self.scroll_offset + self.body_height() {
            self.scroll_offset = bounds.1.saturating_sub(self.body_height() - 1);
        }
    }

    /// Scrolls horizontally to show a match, scrolling as far left as possible while keeping match visible
    pub fn ensure_x_visible(&mut self, col: usize, width: usize) {
        if self.line_wrap {
            return;
        }
        // Account for selection bar taking 1 column
        let usable_width = self.screen.size.0.saturating_sub(1);
        let match_end = col + width;
        let visible_end = self.scroll_x + usable_width;

        if match_end > visible_end {
            // Match is off-screen to the right - scroll right minimally to show it
            if width >= usable_width {
                self.scroll_x = col;
            } else {
                self.scroll_x = match_end.saturating_sub(usable_width);
            }
        } else if col < self.scroll_x {
            // Match is off-screen to the left - scroll left to show it
            // Go as far left as possible while keeping match visible
            if width >= usable_width {
                self.scroll_x = col;
            } else {
                self.scroll_x = match_end.saturating_sub(usable_width);
            }
        }
        // Otherwise match is already visible, don't scroll
    }

    pub fn resize(&mut self, size: (usize, usize)) -> bool {
        self.screen.resize(size)
    }

    pub fn render(
        &mut self,
        filepath: &Option<PathBuf>,
        json: &Json,
        search_input: Option<&str>,
        search_results: Option<&SearchResults>,
        help_visible: bool,
    ) -> anyhow::Result<()> {
        self.screen.clear()?;

        self.render_header(filepath)?;
        let body_height = self.screen.size.1 - self.header_height - self.footer_height;
        self.render_body(
            json,
            (0, self.header_height),
            (self.screen.size.0, body_height),
            search_results,
        )?;
        self.render_footer(search_input, search_results)?;

        if help_visible {
            render_help(&mut self.screen.out, self.screen.size)?;
        }

        self.screen.print()
    }

    fn render_header(&mut self, filepath: &Option<PathBuf>) -> anyhow::Result<()> {
        let fp = if let Some(path) = filepath {
            let path = format!("{}", path.display());
            if path.len() > self.screen.size.0 {
                path[..self.screen.size.0].to_owned()
            } else {
                path.clone()
            }
        } else {
            "stdin".to_string()
        };

        let header = format!("{fp:<width$}", width = self.screen.size.0);

        queue!(self.screen.out, cursor::MoveToColumn(0), ResetColor)?;

        queue!(
            self.screen.out,
            PrintStyledContent(styled(STYLE_HEADER, &header))
        )?;

        Ok(())
    }

    fn render_body(
        &mut self,
        json: &Json,
        offset: (usize, usize),
        size: (usize, usize),
        search_results: Option<&SearchResults>,
    ) -> anyhow::Result<()> {
        let selection_bounds = json.bounds();
        let mut line_idx = 0;
        let mut visible_line = 0;
        let mut cursor_y = offset.1;

        let col_numbers_w = if self.no_numbers {
            0
        } else {
            json.formatted.len().to_string().len()
        };

        let gutter_width = SELECTION_COL_WIDTH + col_numbers_w;

        let col_numbers = offset.0;
        let col_selection = col_numbers + col_numbers_w;
        let col_json = gutter_width;
        let col_max = size.0.saturating_sub(col_json);

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

            if !self.no_numbers {
                let line_num_str = format!("{:>width$} ", line_number + 1, width = col_numbers_w);
                queue!(
                    self.screen.out,
                    cursor::MoveTo(col_numbers as u16, cursor_y as u16),
                    PrintStyledContent(styled(STYLE_LINE_NUMBER, line_num_str))
                )?;
            }

            let is_selected =
                selection_bounds.0 <= *line_number && *line_number <= selection_bounds.1;

            if is_selected {
                queue!(
                    self.screen.out,
                    cursor::MoveTo(col_selection as u16, cursor_y as u16),
                    PrintStyledContent(styled(STYLE_SELECTION_BAR, SELECTION_SYM))
                )?;
            }

            // Position cursor at JSON column + indent (accounting for horizontal scroll)
            queue!(
                self.screen.out,
                cursor::MoveTo(
                    (col_json + indent.saturating_sub(self.scroll_x)) as u16,
                    cursor_y as u16
                ),
                ResetColor
            )?;

            let mut col = *indent; // Absolute column position

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
                        if col >= col_max {
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

                for (elem_idx, el) in elements.iter().enumerate() {
                    let text = &el.0;

                    // Get search match info (style and which chars to highlight)
                    let (search_style, match_positions) = search_results
                        .map(|sr| {
                            if let Some(m) = sr.get_current(*line_number, elem_idx) {
                                (Some(STYLE_SEARCH_MATCH_CURRENT), Some(&m.char_positions))
                            } else if let Some(m) = sr.get_match(*line_number, elem_idx) {
                                (Some(STYLE_SEARCH_MATCH), Some(&m.char_positions))
                            } else {
                                (None, None)
                            }
                        })
                        .unwrap_or((None, None));

                    if self.line_wrap {
                        // Print with manual wrapping (no horizontal scroll in wrap mode)
                        for (char_idx, ch) in text.chars().enumerate() {
                            if col >= col_max {
                                cursor_y += 1;
                                col = continuation_col;
                                queue!(
                                    self.screen.out,
                                    cursor::MoveTo((col + gutter_width) as u16, cursor_y as u16)
                                )?;
                            }
                            let should_highlight = match_positions
                                .map(|pos| pos.contains(&char_idx))
                                .unwrap_or(false);
                            let styled = if should_highlight {
                                apply_with_bg(el.1.apply(ch), search_style.unwrap())
                            } else {
                                el.1.apply(ch)
                            };
                            queue!(self.screen.out, Print(styled))?;
                            col += 1;
                        }
                    } else {
                        // Print char by char to handle UTF-8 correctly
                        for (char_idx, ch) in text.chars().enumerate() {
                            if col >= col_max {
                                break;
                            }
                            if col >= self.scroll_x {
                                let should_highlight = match_positions
                                    .map(|pos| pos.contains(&char_idx))
                                    .unwrap_or(false);
                                let styled = if should_highlight {
                                    apply_with_bg(el.1.apply(ch), search_style.unwrap())
                                } else {
                                    el.1.apply(ch)
                                };
                                queue!(self.screen.out, Print(styled))?;
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

    fn render_footer(
        &mut self,
        search_input: Option<&str>,
        search_results: Option<&SearchResults>,
    ) -> anyhow::Result<()> {
        if self.footer_height == 0 {
            return Ok(());
        }

        let footer_y = self.screen.size.1 - self.footer_height;
        queue!(
            self.screen.out,
            cursor::MoveTo(0, footer_y as u16),
            ResetColor
        )?;

        // Render search prompt or status
        if let Some(input) = search_input {
            // Active search mode: show /input
            queue!(
                self.screen.out,
                PrintStyledContent(styled(STYLE_SEARCH_PROMPT, "/")),
                Print(input)
            )?;
        }

        // Render match count on the right
        if let Some(results) = search_results {
            let status = results.status_text();
            let status_col = self.screen.size.0.saturating_sub(status.len() + 1);
            queue!(
                self.screen.out,
                cursor::MoveTo(status_col as u16, footer_y as u16),
                PrintStyledContent(styled(STYLE_SEARCH_STATUS, &status))
            )?;
        }

        Ok(())
    }
}

/// Helper to apply a background color while preserving the foreground
fn apply_with_bg<D: std::fmt::Display + Clone>(
    styled: crossterm::style::StyledContent<D>,
    bg_style: ContentStyle,
) -> crossterm::style::StyledContent<D> {
    let mut new_style = *styled.style();
    if let Some(bg) = bg_style.background_color {
        new_style.background_color = Some(bg);
    }
    if let Some(fg) = bg_style.foreground_color {
        new_style.foreground_color = Some(fg);
    }
    crossterm::style::StyledContent::new(new_style, styled.content().clone())
}
