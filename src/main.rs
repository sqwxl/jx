#![allow(dead_code)]
use std::fs::File;
use std::io::{self, stdin, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::{
    cursor,
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute, queue,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::json::Json;

mod display;
mod events;
mod json;
mod renderer;
mod run;
mod style;

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

fn main() -> Result<()> {
    let args = Args::parse();

    enable_raw_mode()?;

    let mut stdout = io::stdout();

    let supports_keyboard_enhancement = matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    );

    if supports_keyboard_enhancement {
        queue!(
            stdout,
            PushKeyboardEnhancementFlags(
                KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES
                    | KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS
                    | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
            )
        )?;
    }

    execute!(
        stdout,
        cursor::Hide,
        EnableMouseCapture,
        EnterAlternateScreen
    )?;

    let output = run::run(&args.path, parse_input(&args)?)?;

    if supports_keyboard_enhancement {
        queue!(stdout, PopKeyboardEnhancementFlags)?;
    }

    execute!(
        stdout,
        cursor::Show,
        DisableMouseCapture,
        LeaveAlternateScreen
    )?;

    disable_raw_mode()?;

    if let Some(str) = output {
        println!("{}", str);
    };

    Ok(())
}

fn parse_input(args: &Args) -> Result<Json> {
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

    Ok(Json::new(&value))
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
