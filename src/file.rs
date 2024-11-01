use std::path::PathBuf;

use walkdir::WalkDir;

use thiserror::Error;

use std;

pub mod content;
pub mod name;

/// Walk the directories and get just the files
pub fn get_files(dirs: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for path in dirs {
        let walk = WalkDir::new(path);
        for entry in walk.into_iter().filter_map(Result::ok) {
            if entry.file_type().is_file() {
                out.push(entry.into_path());
            }
        }
    }
    out
}

/// A bunch of bad things can happen while you're reading files,
/// This covers most of them.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error reading the file.")]
    IoError(#[from] std::io::Error),
    #[error("Error parsing the yaml based on expected template.")]
    SerdeError(#[from] serde_yaml::Error),
    #[error("Found duplicate property {0} in file contents")]
    DuplicateProperty(String),
}
