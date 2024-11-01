use std::{fs, path::PathBuf};

use cached::proc_macro::cached;
use front_matter::FrontMatter;

use super::Error;

pub mod body;
pub mod front_matter;
pub mod lists;

/// Get all information from a file needed for the rest of the program
/// Importantly, this is cached, so you don't have to pass around the results
/// Just run it at the very beginning of the program
#[cached(result = true)]
pub fn from_file(path: PathBuf) -> Result<FrontMatter, Error> {
    let contents = fs::read_to_string(path).map_err(Error::IoError)?;
    FrontMatter::new(&contents)
}
