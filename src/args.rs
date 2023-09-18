use clap::Parser;
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
