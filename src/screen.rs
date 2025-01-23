use anyhow::Result;
use crossterm::{cursor, execute, queue, terminal};
use std::io::{self, BufWriter, Stdout, Write};

use crate::style::StyledStr;

/// Foreground & Background
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Style(pub crossterm::style::Color, pub crossterm::style::Color);

pub struct Screen {
    out: BufWriter<Stdout>,
    buf: Vec<Option<(Style, char)>>,
    w: usize,
    h: usize,
}

impl Screen {
    pub fn new() -> Result<Self> {
        // Build empty screen buffer
        let (w, h) = terminal::size()?;
        let buf = std::iter::repeat(None)
            .take(w as usize * h as usize)
            .collect();

        Ok(Screen {
            out: BufWriter::new(io::stdout()),
            buf,
            w: w as usize,
            h: h as usize,
        })
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;

        // resize the buffer and fill with blanks
        self.buf.resize(w * h, Some(blank()));
    }

    pub fn clear(&mut self, x: usize, y: usize, w: usize, h: usize, style: Option<Style>) {
        let fill = match style {
            Some(style) => Some((style, ' ')),
            None => Some(blank()),
        };

        for i in x..x + w {
            for j in y..y + h {
                self.buf[i + j * self.w] = fill;
            }
        }
    }

    /// Add content to the screen buffer.
    pub fn draw(&mut self, x: usize, y: usize, styled_str: &StyledStr) -> (usize, usize) {
        let StyledStr { style, text } = styled_str;

        let mut i = x;
        let mut j = y;
        for char in text.chars() {
            if char == '\n' {
                j += 1;
                i = 0;
            }

            self.buf[i + j * self.w] = Some((style.to_owned(), char));

            i += 1;
        }

        (i, j)
    }

    /// Send the contents of the screen buffer to stdout.
    pub fn render(&mut self) -> Result<()> {
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

                let x = i % self.w;
                let y = i / self.w;
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

impl Drop for Screen {
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
