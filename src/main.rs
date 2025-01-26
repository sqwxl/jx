#![allow(dead_code)]
mod display;
mod events;
mod json;
mod prompt;
mod renderer;
mod style;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{cursor, execute, terminal};
use json::Json;
use std::fs::File;
use std::io::{self, stdin, BufReader};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_parser = validate_path)]
    pub path: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    execute!(io::stdout(), terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    // let result = std::panic::catch_unwind(|| prompt::run(&args.path, parse_input(&args)?));
    prompt::run(&args.path, parse_input(&args)?)?;

    terminal::disable_raw_mode()?;

    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;

    // if let Err(err) = result {
    //     let msg = downcast_panic(err);
    //
    //     eprintln!("{:?}", msg);
    //
    //     std::process::exit(1);
    // }

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

/// https://github.com/rust-lang/rust/blob/1.84.0/library/std/src/panicking.rs#L747-L755
fn downcast_panic(err: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = err.downcast_ref::<&'static str>() {
        return s.to_string();
    };

    if let Some(s) = err.downcast_ref::<String>() {
        return s.clone();
    };

    "Unknown panic".to_string()
}
