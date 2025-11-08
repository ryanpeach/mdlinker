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

use super::{Config as MasterConfig, NewConfigError, Partial};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Config {
    /// See [`super::cli::Config::files`]
    #[serde(default)]
    pub files: Option<Vec<String>>,

    /// See [`super::cli::Config::new_files_directory`]
    #[serde(default)]
    pub new_files_directory: Option<PathBuf>,

    /// See [`super::cli::Config::ngram_size`]
    #[serde(default)]
    pub ngram_size: Option<usize>,

    /// See [`super::cli::Config::boundary_pattern`]
    #[serde(default)]
    pub boundary_pattern: Option<String>,

    /// See [`super::cli::Config::filename_spacing_pattern`]
    #[serde(default)]
    pub filename_spacing_pattern: Option<String>,

    /// See [`super::cli::Config::filename_match_threshold`]
    #[serde(default)]
    pub filename_match_threshold: Option<i64>,

    /// See [`super::cli::Config::exclude`]
    #[serde(default)]
    pub exclude: Vec<String>,

    /// In the [`crate::rules::similar_filename::SimilarFilename`] rule, ignore certain word pairs
    /// Prevents some annoying and frequent false positives
    #[serde(default)]
    pub ignore_word_pairs: Vec<(String, String)>,

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

    #[must_use]
    pub fn original_file_globs(&self) -> Option<Vec<String>> {
        self.files.clone()
    }
}

impl From<MasterConfig> for Config {
    fn from(value: MasterConfig) -> Self {
        Self {
            files: value.original_file_globs,
            new_files_directory: Some(value.new_files_directory),
            ngram_size: Some(value.ngram_size),
            boundary_pattern: Some(value.boundary_pattern),
            filename_spacing_pattern: Some(value.filename_spacing_pattern),
            filename_match_threshold: Some(value.filename_match_threshold),
            exclude: value.exclude.into_iter().map(|x| x.0).collect(),
            ignore_word_pairs: value.ignore_word_pairs,
            alias_to_filename: value.alias_to_filename.into(),
            filename_to_alias: value.filename_to_alias.into(),
        }
    }
}

impl Partial for Config {
    fn files(&self) -> Option<Vec<PathBuf>> {
        let mut out = Vec::new();
        match &self.files {
            None => return None,
            Some(files) if files.is_empty() => return None,
            Some(files) => {
                for file in files {
                    if file.contains('*') {
                        let globs = glob::glob(file);
                        match globs {
                            Ok(globs) => {
                                for glob in globs {
                                    match glob {
                                        Ok(path) => {
                                            out.push(path);
                                        }
                                        Err(e) => {
                                            eprintln!("Error processing glob '{file}': {e}",);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error parsing glob pattern '{file}': {e}");
                                return None;
                            }
                        }
                    } else {
                        let path = PathBuf::from(file);
                        if path.exists() {
                            out.push(path);
                        } else {
                            eprintln!("File does not exist: {file}");
                        }
                    }
                }
            }
        }
        if out.is_empty() {
            eprintln!("No valid files found in the configuration.");
            None
        } else {
            Some(out)
        }
    }

    fn new_files_directory(&self) -> Option<PathBuf> {
        self.new_files_directory.clone()
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
    fn fix(&self) -> Option<bool> {
        None
    }
    fn allow_dirty(&self) -> Result<Option<bool>, NewConfigError> {
        Ok(None)
    }
    fn allow_staged(&self) -> Result<Option<bool>, NewConfigError> {
        Ok(None)
    }
    fn ignore_word_pairs(&self) -> Option<Vec<(String, String)>> {
        if self.ignore_word_pairs.is_empty() {
            None
        } else {
            Some(self.ignore_word_pairs.clone())
        }
    }

    fn ignore_remaining(&self) -> Option<bool> {
        None
    }
}
