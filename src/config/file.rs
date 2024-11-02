use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::sed::{ReplacePair, ReplacePairError};

use super::{Error, Partial};

#[derive(Serialize, Deserialize, Debug, Default)]
pub(super) struct Config {
    /// See [`super::cli::Config::directories`]
    #[serde(default)]
    pub directories: Vec<PathBuf>,

    /// See [`super::cli::Config::ngram_size`]
    #[serde(default)]
    pub ngram_size: Option<usize>,

    /// See [`super::cli::Config::boundary_pattern`]
    #[serde(default)]
    pub boundary_pattern: Option<String>,

    /// See [`super::cli::Config::wikilink_pattern`]
    #[serde(default)]
    pub wikilink_pattern: Option<String>,

    /// See [`super::cli::Config::filename_spacing_pattern`]
    #[serde(default)]
    pub filename_spacing_pattern: Option<String>,

    /// See [`super::cli::Config::filename_match_threshold`]
    #[serde(default)]
    pub filename_match_threshold: Option<i64>,

    /// See [`super::cli::Config::exclude`]
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
