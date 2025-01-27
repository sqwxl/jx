#![allow(dead_code)]
use std::fs::File;
use std::io::{self, stdin, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    cursor,
    event::{
        DisableFocusChange, EnableFocusChange, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
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
#[command(author, version, about, long_about = None)]
// TODO: Add --help
pub struct Args {
    #[arg(value_parser = validate_path)]
    pub path: Option<PathBuf>,
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
        EnableFocusChange,
        EnterAlternateScreen,
    )?;

    let output = run::run(&args.path, parse_input(&args)?)?;

    execute!(
        stdout,
        cursor::Show,
        DisableFocusChange,
        LeaveAlternateScreen,
    )?;

    if supports_keyboard_enhancement {
        queue!(stdout, PopKeyboardEnhancementFlags)?;
    }

    disable_raw_mode()?;

    if let Some(str) = output {
        println!("{}", str);
    };

    Ok(())
}

fn parse_input(args: &Args) -> Result<Json> {
    let value: serde_json::Value = if let Some(ref path) = args.path {
        let file = File::open(path.clone())?;
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
