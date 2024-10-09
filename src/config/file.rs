use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::Error;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    /// The directories to search in
    /// May provide more than one directory
    #[serde(default)]
    pub directories: Vec<PathBuf>,

    /// Size of the n-grams to generate from filenames
    /// Will generate n-grams UP TO and INCLUDING this size
    #[serde(default)]
    pub ngram_size: Option<usize>,

    /// Regex pattern to stop n-gram generation on, like , or .")
    #[serde(default)]
    pub boundary_pattern: Option<String>,

    /// Regex pattern to split filenames on, like _ or -")
    #[serde(default)]
    pub filename_spacing_pattern: Option<String>,

    /// The minimum score to consider a match for filename ngrams
    #[serde(default)]
    pub filename_match_threshold: Option<i64>,

    /// Exclude certain error codes
    /// If an error code **starts with** this string, it will be excluded
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Link conversion to file path using sed regex
    /// Each outer vec contains an inner vec of a sequence of find/replace pairs
    /// Meaning you can have several different and independent sequences of find/replace pairs
    /// This makes it easier to manage multiple different types of links and conversions
    #[serde(default)]
    pub title_to_filepath: Vec<Vec<(String, String)>>,

    /// Convert a filepath to the "title" lr name in a wikilink
    /// Each outer vec contains an inner vec of a sequence of find/replace pairs
    /// Meaning you can have several different and independent sequences of find/replace pairs
    /// This makes it easier to manage multiple different types of links and conversions
    #[serde(default)]
    pub filepath_to_title: Vec<Vec<(String, String)>>,
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, Error> {
        let contents = std::fs::read_to_string(path).map_err(Error::FileDoesNotReadError)?;
        toml::from_str(&contents).map_err(Error::FileDoesNotParseError)
    }
}
