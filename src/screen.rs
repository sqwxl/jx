use std::io::{self, BufWriter, Stdout, Write};

use crossterm::{cursor, execute, style::ResetColor, terminal};

pub struct Screen {
    pub size: (usize, usize),
    pub out: BufWriter<Stdout>,
}

impl Screen {
    pub fn new() -> anyhow::Result<Self> {
        let size = terminal::size().map(|s| (s.0 as usize, s.1 as usize))?;

        Ok(Screen {
            size,
            out: BufWriter::new(io::stdout()),
        })
    }

    pub fn resize(&mut self, size: (usize, usize)) -> bool {
        if self.size != size {
            self.size = size;

            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) -> anyhow::Result<()> {
        execute!(
            self.out,
            cursor::MoveTo(0, 0),
            terminal::Clear(crossterm::terminal::ClearType::All),
            ResetColor,
        )?;

        Ok(())
    }

    pub fn print(&mut self) -> anyhow::Result<()> {
        self.out.flush().map_err(anyhow::Error::from)
    }
}

impl Drop for Screen {
    /// Restore the terminal to its original state when the Display is dropped.
    fn drop(&mut self) {
        terminal::disable_raw_mode().unwrap();
        execute!(self.out, terminal::LeaveAlternateScreen, cursor::Show).unwrap();
        self.out.flush().unwrap();
    }
}
