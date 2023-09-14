use clap::Parser;
use serde_json::Value;
use std::{
    error::Error,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_parser = validate_path)]
    path: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn Error>> {
    // parse args
    let args = Args::parse();
    let json: Value;

    if let Some(path) = args.path {
        // read from file
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        json = serde_json::from_reader(reader)?;
    } else {
        // read from stdin
        let stdin = std::io::stdin();
        let reader = stdin.lock();

        json = serde_json::from_reader(reader)?;
    }

    println!("{}", json);

    Ok(())
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
