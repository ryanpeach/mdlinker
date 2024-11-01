use std::path::PathBuf;

use walkdir::WalkDir;

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
