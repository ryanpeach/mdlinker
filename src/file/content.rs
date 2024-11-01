#![allow(clippy::needless_pass_by_value)]
use std::{fs, path::PathBuf};

use cached::proc_macro::cached;
use front_matter::FrontMatter;
use wikilink::Wikilink;

use super::Error;

pub mod front_matter;
pub mod wikilink;

#[derive(Clone)]
pub struct FromFile {
    pub path: PathBuf,
    pub front_matter: FrontMatter,
    pub wikilinks: Vec<Wikilink>,
}

/// Get all information from a file needed for the rest of the program
/// Importantly, this is cached, so you don't have to pass around the results
/// Just run it at the very beginning of the program
#[cached(result = true)]
pub fn from_file(path: PathBuf, wikilink_pattern: String) -> Result<FromFile, Error> {
    let contents = fs::read_to_string(path.clone()).map_err(Error::IoError)?;
    Ok(FromFile {
        path,
        front_matter: FrontMatter::new(&contents)?,
        wikilinks: Wikilink::get_wikilinks(&contents, &wikilink_pattern)
            .map_err(Error::RegexError)?,
    })
}
