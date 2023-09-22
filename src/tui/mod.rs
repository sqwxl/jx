use crate::events::Action::*;
use crate::events::Direction::*;
use crate::json::Json;
use crate::json::StyledStr;
use crossterm::terminal;

use self::screen::Screen;

pub mod screen;

const INDENT: usize = 2;

pub struct Tui {
    json: Json,
    screen: Screen,
    w: usize,
    h: usize,
}

impl Tui {
    pub fn with_json(json: Json) -> Result<Self, std::io::Error> {
        let (w, h) = terminal::size()?;
        Ok(Tui {
            json,
            screen: Screen::new()?,
            w: w as usize,
            h: h as usize,
        })
    }
    pub fn run(&mut self) -> Result<(), std::io::Error> {
        // initial draw
        self.screen.clear(None);
        self.draw_tree();
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
                self.draw_tree();
                self.draw_pointer();
                self.screen.render()?;
            }
        }

        Ok(())
    }

    fn draw_pointer(&mut self) {
        let y = self.h - 1;
        self.screen.draw(
            0,
            y,
            &StyledStr {
                style: crate::json::STYLE_POINTER,
                text: self
                    .json
                    .pointer
                    .iter()
                    .map(|s| format!("/\"{}\"", s))
                    .collect(),
            },
        );
    }

    fn draw_tree(&mut self) {
        let styled = self.json.style_json();
        self.screen.clear(None);

        let mut y = 0;
        for (depth, styled_str) in styled {
            if y >= self.h {
                break;
            }
            let x = depth * INDENT;
            self.screen.draw(x, y, &styled_str);
            if styled_str.text.ends_with('\n') {
                y += 1;
            }
        }
    }
}
