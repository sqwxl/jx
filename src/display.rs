use std::io::{self, BufWriter, Stdout, Write};

use anyhow::Result;
use crossterm::{cursor, execute, queue, terminal};

use crate::{renderer::Vec2, style::StyledStr};

/// Foreground & Background
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style(pub crossterm::style::Color, pub crossterm::style::Color);

pub struct Display {
    pub size: Vec2,
    out: BufWriter<Stdout>,
    buf: Vec<Option<(Style, char)>>,
}

impl Display {
    pub fn new() -> Result<Self> {
        // Build empty screen buffer
        let size: Vec2 = terminal::size().map(|s| (s.0 as usize, s.1 as usize))?;

        let buf = std::iter::repeat(None).take(size.0 * size.1).collect();

        Ok(Display {
            size,
            out: BufWriter::new(io::stdout()),
            buf,
        })
    }

    pub fn resize(&mut self, size: Vec2) -> bool {
        if self.size != size {
            self.size = size;
            // Resize and clear the buffer.
            self.buf.resize(size.0 * size.1, Some(blank()));
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self, (x, y): (usize, usize), (w, h): (usize, usize), style: Option<Style>) {
        let fill = match style {
            Some(style) => Some((style, ' ')),
            None => Some(blank()),
        };

        for i in x..x + w {
            if i >= self.size.0 {
                break;
            }
            for j in y..y + h {
                if j >= self.size.1 {
                    break;
                }
                self.buf[i + j * w] = fill;
            }
        }
    }

    /// Add content to the screen buffer.
    pub fn draw(&mut self, x: usize, y: usize, styled_str: &StyledStr) -> (usize, usize) {
        let StyledStr { style, text } = styled_str;

        let mut i = usize::clamp(x, 0, self.size.0 - 1);
        let mut j = usize::clamp(y, 0, self.size.1 - 1);

        for char in text.chars() {
            if char == '\n' {
                j += 1;
                i = 0;
            }

            let index = (i + j * self.size.0).clamp(0, self.buf.len() - 1);

            self.buf[index] = Some((style.to_owned(), char));

            i += 1;
        }

        (i, j)
    }

    /// Send the contents of the screen buffer to stdout.
    pub fn show(&mut self) -> Result<()> {
        let mut current_style = crate::style::STYLE_INACTIVE;

        queue!(
            self.out,
            cursor::MoveTo(0, 0),
            cursor::Hide,
            terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::style::SetForegroundColor(current_style.0),
            crossterm::style::SetBackgroundColor(current_style.1),
        )?;

        for (i, sc) in self.buf.iter().enumerate() {
            if let Some((style, char)) = sc {
                if *style != current_style {
                    current_style = style.to_owned();
                    queue!(
                        self.out,
                        crossterm::style::SetForegroundColor(current_style.0),
                        crossterm::style::SetBackgroundColor(current_style.1),
                    )?;
                }

                let x = i % self.size.0;
                let y = i / self.size.0;
                queue!(
                    self.out,
                    cursor::MoveTo(x as u16, y as u16),
                    crossterm::style::Print(char)
                )?;
            }
        }

        self.out.flush().map_err(anyhow::Error::from)
    }
}

impl Drop for Display {
    /// Restore the terminal to its original state when the Display is dropped.
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.out, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        self.out.flush().unwrap();
    }
}

fn blank() -> (Style, char) {
    (
        Style(
            crossterm::style::Color::White,
            crossterm::style::Color::Black,
        ),
        ' ',
    )
}
