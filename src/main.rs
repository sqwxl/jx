mod events;
mod json;
mod prompt;
mod screen;
mod style;

use anyhow::{Context, Result};
use clap::Parser;
use prompt::Prompt;
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufReader};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(value_parser = validate_path)]
    pub path: Option<PathBuf>,
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

fn main() -> Result<()> {
    // parse args
    let args = Args::parse();

    let json = parse_input(&args)?;

    let mut prompt = Prompt::new(&json, &args.path)?;

    prompt.run()?;

    Ok(())
}

fn parse_input(args: &Args) -> Result<Value, anyhow::Error> {
    let json: Value = if let Some(ref path) = args.path {
        // read from file
        let file = File::open(path.clone())?;
        let reader = BufReader::new(file);

        serde_json::from_reader(reader)
            .context(format!("Error parsing JSON from file {}", path.display()))?
    } else {
        // read from stdin
        let stdin = stdin();
        let reader = stdin.lock();

        serde_json::from_reader(reader).context("Could not parse JSON from stdin")?
    };

    Ok(json)
}
