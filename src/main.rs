#![allow(dead_code)]
use std::fs::File;
use std::io::{self, stdin, BufReader};
use std::panic;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Context;
use clap::Parser;
use crossterm::{
    cursor,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::json::Json;

mod events;
mod json;
mod run;
mod screen;
mod style;
mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(value_parser = validate_path)]
    path: Option<PathBuf>,

    #[arg(long, help = "Show line numbers")]
    numbered: bool,

    #[arg(long, help = "Disable syntax highlighting")]
    no_syntax: bool,

    #[arg(long, help = "Disable color")]
    no_color: bool,
}

fn main() -> anyhow::Result<()> {
    prepare_terminal()?;

    setup_panic_hook();

    let result = (|| -> anyhow::Result<Option<String>> {
        let args = Args::try_parse()?;

        let json = parse_input(&args)?;

        run::event_loop(&args.path, json)
    })()
    .transpose();

    restore_terminal()?;

    if let Some(o) = result {
        match o {
            Ok(output) => println!("{}", output),
            Err(e) => return Err(e),
        }
    };

    Ok(())
}

/// Parses input from a file or stdin and returns runtime structs.
fn parse_input(args: &Args) -> anyhow::Result<Json> {
    let value: serde_json::Value = if let Some(ref path) = args.path {
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .context(format!("Error parsing JSON from file {}.", path.display()))?
    } else {
        let stdin = stdin();
        let reader = stdin.lock();

        serde_json::from_reader(reader).context("Error parsing JSON from stdin.")?
    };

    let json = Json::from(Rc::new(value));

    Ok(json)
}

/// Sets up a hook that will restore the terminal on panic.
/// See: https://github.com/helix-editor/helix/blob/0c8f0c0334d449dd71928a697cfba0207be74a63/helix-term/src/application.rs#L1226
fn setup_panic_hook() {
    let hook = std::panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        hook(info);
    }));
}

fn supports_keyboard_enhancement() -> bool {
    matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    )
}

/// Puts the terminal into raw mode, (this lets us read raw key presses),
/// hides the cursor, enables the alternate screen, and
/// optionally enables keyboard enhancement.
fn prepare_terminal() -> anyhow::Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide)?;

    execute!(stdout, EnterAlternateScreen)?;

    if supports_keyboard_enhancement() {
        queue!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
            )
        )?;
    }

    Ok(())
}

/// Restores the terminal to its default state.
fn restore_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();

    if supports_keyboard_enhancement() {
        queue!(stdout, PopKeyboardEnhancementFlags)?;
    }

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;

    disable_raw_mode()
}

fn validate_path(file: &str) -> Result<PathBuf, String> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(format!("Path {} does not exist.", file));
    }

    if !path.is_file() {
        return Err(format!("Path {} is not a file.", file));
    }

    Ok(path.to_owned())
}
