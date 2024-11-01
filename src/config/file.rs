use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sed::{ReplacePair, ReplacePairError};

use super::{Error, Partial};

#[derive(Serialize, Deserialize, Debug, Default)]
pub(super) struct Config {
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

    /// Regex pattern for wikilinks
    #[serde(default)]
    pub wikilink_pattern: Option<String>,

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

impl Partial for Config {
    fn directories(&self) -> Option<Vec<PathBuf>> {
        let out = self.directories.clone();
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn ngram_size(&self) -> Option<usize> {
        self.ngram_size
    }

    fn boundary_pattern(&self) -> Option<String> {
        self.boundary_pattern.clone()
    }

    fn wikilink_pattern(&self) -> Option<String> {
        self.wikilink_pattern.clone()
    }

    fn filename_spacing_pattern(&self) -> Option<String> {
        self.filename_spacing_pattern.clone()
    }

    fn filename_match_threshold(&self) -> Option<i64> {
        self.filename_match_threshold
    }

    fn exclude(&self) -> Option<Vec<String>> {
        let out = self.exclude.clone();
        if out.is_empty() {
            None
        } else {
            Some(out)
        }
    }

    fn filepath_to_title(&self) -> Option<Result<Vec<Vec<ReplacePair>>, ReplacePairError>> {
        let out = self.filepath_to_title.clone();
        if out.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            for inner in out {
                let mut inner_res = Vec::new();
                for (find, replace) in inner {
                    match ReplacePair::new(&find, &replace) {
                        Ok(pair) => inner_res.push(pair),
                        Err(e) => return Some(Err(e)),
                    }
                }
                res.push(inner_res);
            }
            Some(Ok(res))
        }
    }
    fn title_to_filepath(&self) -> Option<Result<Vec<Vec<ReplacePair>>, ReplacePairError>> {
        let out = self.title_to_filepath.clone();
        if out.is_empty() {
            None
        } else {
            let mut res = Vec::new();
            for inner in out {
                let mut inner_res = Vec::new();
                for (find, replace) in inner {
                    match ReplacePair::new(&find, &replace) {
                        Ok(pair) => inner_res.push(pair),
                        Err(e) => return Some(Err(e)),
                    }
                }
                res.push(inner_res);
            }
            Some(Ok(res))
        }
    }
}
