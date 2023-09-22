use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use json::Json;
use serde_json::Value;
use std::fs::File;
use std::io::{stdin, BufReader};
use tui::Tui;

mod args;
mod events;
mod json;
mod tui;

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
        let stdin = stdin();
        let reader = stdin.lock();

        serde_json::from_reader(reader).context("Could not parse JSON from stdin")?
    };

    let json = Json::new(json);

    let mut tui = Tui::with_json(json)?;

    tui.run()?;

    Ok(())
}
