use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    file::{
        content::wikilink::Alias,
        name::{Filename, FilenameLowercase},
    },
    rules::ErrorCode,
    sed::{ReplacePair, ReplacePairCompilationError},
};

use super::{NewConfigError, Partial};

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

    /// Convert an alias to a filename
    /// Kinda like a sed command
    #[serde(default)]
    pub alias_to_filename: (String, String),

    /// Convert a filename to an alias
    /// Kinda like a sed command
    #[serde(default)]
    pub filename_to_alias: (String, String),
}

impl Config {
    pub fn new(path: &Path) -> Result<Self, NewConfigError> {
        let contents =
            std::fs::read_to_string(path).map_err(NewConfigError::FileDoesNotReadError)?;
        toml::from_str(&contents).map_err(NewConfigError::FileDoesNotParseError)
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

    fn filename_spacing_pattern(&self) -> Option<String> {
        self.filename_spacing_pattern.clone()
    }

    fn filename_match_threshold(&self) -> Option<i64> {
        self.filename_match_threshold
    }

    fn exclude(&self) -> Option<Vec<ErrorCode>> {
        let out = self.exclude.clone();
        if out.is_empty() {
            None
        } else {
            Some(out.into_iter().map(ErrorCode::new).collect())
        }
    }

    fn alias_to_filename(
        &self,
    ) -> Option<Result<ReplacePair<Alias, FilenameLowercase>, ReplacePairCompilationError>> {
        let (to, from) = self.alias_to_filename.clone();
        match (to.is_empty(), from.is_empty()) {
            (true, true) => None,
            (false, false) => Some(ReplacePair::new(&to, &from)),
            (true, false) => Some(Err(ReplacePairCompilationError::ToError(
                regex::Error::Syntax("To is empty".to_string()),
            ))),
            (false, true) => Some(Err(ReplacePairCompilationError::FromError(
                regex::Error::Syntax("From is empty".to_string()),
            ))),
        }
    }

    fn filename_to_alias(
        &self,
    ) -> Option<Result<ReplacePair<Filename, Alias>, ReplacePairCompilationError>> {
        let (to, from) = self.alias_to_filename.clone();
        match (to.is_empty(), from.is_empty()) {
            (true, true) => None,
            (false, false) => Some(ReplacePair::new(&to, &from)),
            (true, false) => Some(Err(ReplacePairCompilationError::ToError(
                regex::Error::Syntax("To is empty".to_string()),
            ))),
            (false, true) => Some(Err(ReplacePairCompilationError::FromError(
                regex::Error::Syntax("From is empty".to_string()),
            ))),
        }
    }
}
