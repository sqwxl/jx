use crate::events::Action::*;
use crate::events::Direction::*;
use crate::json::Json;
use crate::screen::Screen;
use crate::style::StyledStr;
use anyhow::Result;
use arboard::Clipboard;
use crossterm::terminal;
use serde_json::Value;
use std::path::PathBuf;

const INDENT: usize = 4;

pub struct Prompt {
    filepath: Option<PathBuf>,
    json: Json,
    screen: Screen,
    w: usize,
    h: usize,
}

impl Prompt {
    pub fn new(value: &Value, filepath: &Option<PathBuf>) -> Result<Self> {
        let (w, h) = terminal::size()?;
        Ok(Prompt {
            filepath: filepath.clone(),
            json: Json::new(value),
            screen: Screen::new()?,
            w: w as usize,
            h: h as usize,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        self.screen.clear(0, 0, self.w, self.h, None);
        self.draw_interface();
        self.screen.render()?;

        let mut clipboard = Clipboard::new()?;

        loop {
            let mut needs_redraw = false;

            match crate::events::user_event()? {
                Quit => {
                    break;
                }
                Move(direction) => {
                    needs_redraw = match direction {
                        Up => self.json.go_prev(),
                        Down => self.json.go_next(),
                        Left => self.json.go_out(),
                        Right => self.json.go_in(),
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
                    self.w = w;
                    self.h = h;
                    self.screen.resize(w, h);
                    needs_redraw = true;
                }
                // TODO: copy feedback
                CopySelection => {
                    if let Some(selection) = self.json.selection_string() {
                        clipboard.set_text(selection)?;
                    }
                }
                CopyRawValue => {
                    if let Some(value) = self.json.value_string() {
                        clipboard.set_text(value)?;
                    }
                }
                _ => {}
            }

            if needs_redraw {
                self.draw_interface();
                self.screen.render()?;
            }
        }

        Ok(())
    }

    fn draw_interface(&mut self) {
        let mut cursor = (0, 0);
        cursor = self.draw_title(cursor.0, cursor.1);
        self.draw_path(cursor.0 + 2, cursor.1);
        self.draw_tree((0, 1), (self.w, self.h - 2));
    }

    fn draw_title(&mut self, x: usize, y: usize) -> (usize, usize) {
        let mut title = String::new();

        match &self.filepath {
            Some(path) => {
                let path = format!("{}", path.display());

                // TODO try to shorten path if too long
                let path = if path.len() > self.w {
                    &path[..self.w]
                } else {
                    &path
                };

                title.push_str(path);
            }
            _ => {
                let stdin = "stdin";
                title.push_str(stdin);
            }
        }

        self.screen.draw(
            x,
            y,
            &StyledStr {
                style: crate::style::STYLE_TITLE,
                text: title,
            },
        )
    }

    fn draw_path(&mut self, x: usize, y: usize) {
        self.screen.clear(x, y, self.w, 1, None);

        let path = format!("{}", self.json.pointer);

        self.screen.draw(
            x,
            y,
            &StyledStr {
                style: crate::style::STYLE_POINTER,
                text: path,
            },
        );
    }

    fn draw_tree(&mut self, (x, y): (usize, usize), (w, h): (usize, usize)) {
        let styled = self.json.style_json();

        let (selection_top, selection_bottom) = styled.selection;
        self.screen.clear(x, y, w, h, None);

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
            self.screen.draw(x, y, styled_str);
            y += 1;
        }
    }
}
