use hashbrown::HashMap;
use std::{
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
};

use regex::Regex;

use crate::ngrams::{up_to_n, Ngram};

use super::get_files;

/// A filename is a representation of the file name in its original casing
/// And with its original seperators
/// but without its extension and without its path
///
/// # Example
/// `asdf/Foo___Bar.md` -> `Foo___Bar`
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct Filename(String);

impl Filename {
    #[must_use]
    pub fn new(filename: &str) -> Self {
        Self(filename.to_owned())
    }
    #[must_use]
    pub fn lowercase(&self) -> FilenameLowercase {
        FilenameLowercase::new(&self.0)
    }
}

impl Display for Filename {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Filename {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

/// Sometimes you are given a lowercase [`Filename`] and you have to make due
#[derive(Clone, Debug, Eq, PartialEq, Hash, PartialOrd, Ord)]
pub struct FilenameLowercase(String);

impl FilenameLowercase {
    #[must_use]
    pub fn new(filename: &str) -> Self {
        Self(filename.to_owned().to_lowercase())
    }
}

impl Display for FilenameLowercase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for FilenameLowercase {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

/// Get the filename from a path
/// Does not include the file extension
#[must_use]
pub fn get_filename(path: &Path) -> Filename {
    let fname = path
        .file_name()
        .expect("We were given a guaranteed file path, not a directory")
        .to_string_lossy();
    return Filename::new(
        fname
            .split('.')
            .next()
            .expect("File paths will either have a file extension or not, it makes no difference"),
    );
}

/// Generate n-grams from the filenames found in the directories
#[must_use]
pub fn ngrams(
    files: &Vec<PathBuf>,
    ngram_size: usize,
    boundary_regex: &Regex,
    filename_spacing_regex: &Regex,
) -> HashMap<Ngram, PathBuf> {
    let mut file_name_ngrams = HashMap::new();
    for filepath in files {
        let filename = get_filename(filepath);
        let ngrams = up_to_n(
            &filename.to_string(),
            ngram_size,
            boundary_regex,
            filename_spacing_regex,
        );
        log::debug!("Filename: {:?}, ngrams: {:?}", filename, ngrams.len());
        for ngram in ngrams {
            file_name_ngrams.insert(ngram, filepath.clone());
        }
    }
    file_name_ngrams
}
