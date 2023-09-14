use std::path::{Path, PathBuf};

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(value_parser = validate_path)]
    path: Option<PathBuf>,
}

fn main() {
    // parse args
    let args = Args::parse();
    println!("Args: {:?}", args);

    if let Some(path) = args.path {
        println!("Path: {}", path.display());
    }
}

fn validate_path(path_str: &str) -> Result<PathBuf, String> {
    let path = Path::new(path_str);

    if !path.exists() {
        return Err(format!("Path {} does not exist", path_str));
    }

    if !path.is_file() {
        return Err(format!("Path {} is not a file", path_str));
    }

    Ok(path.to_owned())
}
