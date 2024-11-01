use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::ngrams::up_to_n;

use super::get_files;

/// Get the filename from a path
/// Does not include the file extension
#[must_use]
pub fn get_filename(path: &Path) -> String {
    let fname = path
        .file_name()
        .expect("We were given a guaranteed file path, not a directory")
        .to_string_lossy();
    return fname
        .split('.')
        .next()
        .expect("File paths will either have a file extension or not, it makes no difference")
        .to_string();
}

/// Get the segments of a filename based on [`boundary_regex`]
#[must_use]
pub fn filename_segments(path: &Path, boundary_regex: &Regex) -> Vec<String> {
    let filename = get_filename(path);
    boundary_regex
        .split(&filename)
        .map(std::string::ToString::to_string)
        .collect()
}

/// Generate n-grams from the filenames found in the directories
#[must_use]
pub fn ngrams(
    dirs: Vec<PathBuf>,
    ngram_size: usize,
    boundary_regex: &Regex,
    filename_spacing_regex: &Regex,
) -> HashMap<String, PathBuf> {
    let files = get_files(dirs);
    let mut file_name_ngrams = HashMap::new();
    for filepath in files {
        let filename = get_filename(&filepath);
        let ngrams = up_to_n(
            &filename,
            ngram_size,
            boundary_regex,
            filename_spacing_regex,
        );
        log::debug!("Filename: {}, ngrams: {:?}", filename, ngrams.len());
        file_name_ngrams.insert(filename, filepath);
    }
    file_name_ngrams
}
