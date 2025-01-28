#![allow(dead_code)]
use std::fs::File;
use std::io::{self, stdin, BufReader};
use std::panic;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::queue;
use crossterm::{
    cursor, execute,
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

    setup_terminal()?;

    // Exit the alternate screen and disable raw mode before panicking
    // https://github.com/helix-editor/helix/blob/0c8f0c0334d449dd71928a697cfba0207be74a63/helix-term/src/application.rs#L1226
    let hook = std::panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        hook(info);
    }));

    let output = run::event_loop(&args.path, parse_input(&args)?)?;

    restore_terminal()?;

    if let Some(str) = output {
        println!("{}", str);
    };

    Ok(())
}

fn supports_keyboard_enhancement() -> bool {
    matches!(
        crossterm::terminal::supports_keyboard_enhancement(),
        Ok(true)
    )
}

fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = io::stdout();

    execute!(stdout, cursor::Hide, EnterAlternateScreen)?;

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

fn restore_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();

    if supports_keyboard_enhancement() {
        queue!(stdout, PopKeyboardEnhancementFlags)?;
    }

    execute!(stdout, cursor::Show, LeaveAlternateScreen)?;

    disable_raw_mode()
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
