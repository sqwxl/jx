use crossterm::{cursor, execute, queue, terminal};
use std::io::{stdout, BufWriter, Stdout, Write};

use crate::json::StyledStr;

/// Foreground & Background
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style(pub crossterm::style::Color, pub crossterm::style::Color);

pub struct Screen {
    out: BufWriter<Stdout>,
    buf: Vec<Option<(Style, char)>>,
    w: usize,
    h: usize,
}

impl Screen {
    pub fn new() -> Result<Self, std::io::Error> {
        let mut out = BufWriter::new(stdout());

        execute!(out, terminal::EnterAlternateScreen)?;
        out.flush()?;
        terminal::enable_raw_mode()?;

        // Build empty screen buffer
        let (w, h) = terminal::size()?;
        let buf = std::iter::repeat(None)
            .take(w as usize * h as usize)
            .collect();

        Ok(Screen {
            out,
            buf,
            w: w as usize,
            h: h as usize,
        })
    }

    pub fn resize(&mut self, w: usize, h: usize) {
        self.w = w;
        self.h = h;
        self.buf.resize(w * h, Some(blank()));
    }

    pub fn clear(&mut self, style: Option<Style>) {
        let fill = match style {
            Some(style) => Some((style, ' ')),
            None => Some(blank()),
        };
        self.buf = std::iter::repeat(fill).take(self.w * self.h).collect();
    }

    /// Add content to the screen buffer.
    pub fn draw(&mut self, x: usize, y: usize, styled_str: &StyledStr) {
        let StyledStr { style, text } = styled_str;

        let mut x = x;
        for char in text.chars() {
            if char == '\n' {
                continue;
            }
            self.buf[x + y * self.w] = Some((style.to_owned(), char));
            x += 1;
            if x > self.w {
                break;
            }
        }
    }

    /// Send the contents of the screen buffer to stdout.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let mut curr_style = crate::json::STYLE_INACTIVE;

        queue!(
            self.out,
            cursor::MoveTo(0, 0),
            cursor::Hide,
            terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::style::SetForegroundColor(curr_style.0),
            crossterm::style::SetBackgroundColor(curr_style.1),
        )?;

        for (i, sc) in self.buf.iter().enumerate() {
            if let Some((style, char)) = sc {
                if *style != curr_style {
                    curr_style = style.to_owned();
                    queue!(
                        self.out,
                        crossterm::style::SetForegroundColor(curr_style.0),
                        crossterm::style::SetBackgroundColor(curr_style.1),
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

        self.out.flush()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(
            self.out,
            terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::style::ResetColor,
            terminal::LeaveAlternateScreen,
            crossterm::cursor::Show,
        )
        .unwrap();
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
