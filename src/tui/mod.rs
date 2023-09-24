use std::path::PathBuf;

use crate::events::Action::*;
use crate::events::Direction::*;
use crate::json::Json;
use crate::json::StyledStr;
use anyhow::Result;
use crossterm::terminal;
use serde_json::Value;

use self::screen::Screen;

pub mod screen;

const INDENT: usize = 2;

pub struct Tui {
    filepath: Option<PathBuf>,
    json: Json,
    screen: Screen,
    w: usize,
    h: usize,
}

impl Tui {
    pub fn with_value(value: &Value, filepath: &Option<PathBuf>) -> Result<Self, std::io::Error> {
        let (w, h) = terminal::size()?;
        Ok(Tui {
            filepath: filepath.clone(),
            json: Json::new(value),
            screen: Screen::new()?,
            w: w as usize,
            h: h as usize,
        })
    }
    pub fn run(&mut self) -> Result<(), std::io::Error> {
        // initial draw
        self.screen.clear(0, 0, self.w, self.h, None);
        self.draw_interface();
        self.screen.render()?;

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
        self.draw_title();
        self.draw_tree((0, 1), (self.w, self.h - 2));
        self.draw_pointer();
    }

    fn draw_title(&mut self) {
        let mut title = " ".repeat(self.w);
        match &self.filepath {
            Some(path) => {
                let path = format!("{}", path.display());
                // TODO try to shorten path if too long
                let path = if path.len() > self.w {
                    &path[path.len() - self.w..]
                } else {
                    &path
                };
                title.replace_range(0..path.len(), path);
            }
            _ => {
                let stdin = "stdin";
                title.replace_range(0..stdin.len(), stdin);
            }
        }
        self.screen.draw(
            0,
            0,
            &StyledStr {
                style: crate::json::STYLE_TITLE,
                text: title,
            },
        );
    }

    fn draw_pointer(&mut self) {
        let y = self.h - 1;
        let mut pointer = self.json.pointer.to_string();
        // TODO try to shorten pointer if too long
        let r_pad = if pointer.len() > self.w {
            0
        } else {
            self.w - pointer.len()
        };

        pointer.push_str(&" ".repeat(r_pad));
        self.screen.draw(
            0,
            y,
            &StyledStr {
                style: crate::json::STYLE_POINTER,
                text: pointer,
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
