use crossterm::{cursor, execute, queue, style, terminal};
use std::io::{stdout, BufWriter, Stdout, Write};

/// Foreground & Background
#[derive(Clone, Copy, PartialEq)]
pub struct Style(pub crossterm::style::Color, pub crossterm::style::Color);

pub struct Screen {
    out: BufWriter<Stdout>,
    buf: Vec<Option<(Style, String)>>,
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
        let buf = std::iter::repeat(Some(blank()))
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
            Some(style) => Some((style, " ".to_owned())),
            None => Some(blank()),
        };
        self.buf = std::iter::repeat(fill).take(self.w * self.h).collect();
    }

    /// Add content to the screen buffer.
    pub fn draw(&mut self, pos: (usize, usize), styled_text: &(Style, String)) {
        let (x, y) = pos;
        let i = x + y * self.w;
        self.buf[i] = Some(styled_text.to_owned());
    }

    /// Send the contents of the screen buffer to stdout.
    pub fn render(&mut self) -> Result<(), std::io::Error> {
        let mut curr_style = &crate::json::STYLE_INACTIVE;

        queue!(
            self.out,
            cursor::MoveTo(0, 0),
            terminal::Clear(crossterm::terminal::ClearType::All),
            style::SetForegroundColor(curr_style.0),
            style::SetBackgroundColor(curr_style.1),
        )?;

        for y in 0..self.h {
            let x = 0;
            if let Some((style, text)) = &self.buf[x + y * self.w] {
                if style != curr_style {
                    curr_style = style;
                    queue!(
                        self.out,
                        style::SetForegroundColor(curr_style.0),
                        style::SetBackgroundColor(curr_style.1),
                    )?;
                }
                queue!(self.out, style::Print(text), cursor::MoveToNextLine(1))?;
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

fn blank() -> (Style, String) {
    (
        Style(
            crossterm::style::Color::White,
            crossterm::style::Color::Black,
        ),
        " ".to_owned(),
    )
}
