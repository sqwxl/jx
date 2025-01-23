mod events;
mod json;
mod prompt;
mod screen;
mod style;

use crate::prompt::Prompt;
use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{cursor, execute, terminal};
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

    let json = parse_input(&args)?;

    execute!(io::stdout(), terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let result = std::panic::catch_unwind(|| Prompt::new(&json, &args.path)?.run());

    terminal::disable_raw_mode()?;

    execute!(io::stdout(), terminal::LeaveAlternateScreen, cursor::Show)?;

    if let Err(err) = result {
        // https://github.com/rust-lang/rust/blob/1.84.0/library/std/src/panicking.rs#L747-L755
        let msg = match err.downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match err.downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Unknown error",
            },
        };

        eprintln!("{:?}", msg);

        std::process::exit(1);
    }

    Ok(())
}

fn parse_input(args: &Args) -> Result<serde_json::Value> {
    let json: serde_json::Value = if let Some(ref path) = args.path {
        let file = File::open(path.clone())?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .context(format!("Error parsing JSON from file {}", path.display()))?
    } else {
        let stdin = stdin();
        let reader = stdin.lock();

        serde_json::from_reader(reader).context("Could not parse JSON from stdin")?
    };

    Ok(json)
}

fn validate_path(file: &str) -> Result<PathBuf, String> {
    let path = Path::new(file);

    if !path.exists() {
        return Err(format!("Path {} does not exist", file));
    }

    if !path.is_file() {
        return Err(format!("Path {} is not a file", file));
    }

    Ok(path.to_owned())
}
