use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use core::Core;
use crossterm::{execute, queue, terminal};
use events::{Action::*, Direction::*};
use serde_json::Value;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Write};

mod args;
mod core;
mod events;

fn main() -> Result<()> {
    // parse args
    let args = Args::parse();

    let json: Value = if let Some(path) = args.path {
        // read from file
        let file = File::open(path.clone())?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .context(format!("Error parsing JSON from file {}", path.display()))?
    } else {
        // read from stdin
        let stdin = io::stdin();
        let reader = stdin.lock();

        serde_json::from_reader(reader).context("Could not parse JSON from stdin")?
    };

    run(json)?;

    Ok(())
}

fn run(json: Value) -> Result<(), io::Error> {
    let mut stdout = io::stdout();

    let size = terminal::size()?;

    let mut obj = Core::new(json.clone(), size);

    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    loop {
        let mut changed = false;
        match events::user_event()? {
            Quit => break,
            Move(direction) => {
                changed = match direction {
                    Up => obj.go_prev(),
                    Down => obj.go_next(),
                    Left => obj.go_out(),
                    Right => obj.go_in(),
                }
            }
            Fold => {
                todo!();
            }
            Scroll(direction) => {
                changed = match direction {
                    Up => todo!(),
                    Down => todo!(),
                    _ => false,
                }
            }
            Resize(w, h) => {
                obj.view.resize((w, h));
                changed = true;
            }
            _ => {}
        }

        if changed {
            redraw(&mut stdout, &obj)?;
            stdout.flush()?;
        }
    }

    execute!(stdout, terminal::LeaveAlternateScreen)?;

    terminal::disable_raw_mode()
}

fn redraw(stdout: &mut io::Stdout, obj: &Core) -> Result<(), io::Error> {
    todo!()
}
