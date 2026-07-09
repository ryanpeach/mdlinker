use std::path::Path;
use std::path::PathBuf;

use merge::Merge;
use serde::{Deserialize, Serialize};

use super::NewConfigError;

/// The "partial" config.
///
/// Every field is an [`Option`], so a missing value is distinguishable from a set
/// one. This single type serves two roles:
///
/// 1. It is the on-disk TOML representation (via [`serde`]), read by [`Config::new`].
/// 2. It is the merge target: [`super::cli::Config::to_partial`] maps the CLI into the
///    same shape, and [`merge::Merge`] combines the two. Because [`merge::Merge`] for
///    [`Option`] keeps the left-hand `Some` and only fills `None` holes from the
///    right-hand side, `cli.merge(file)` gives the CLI priority for free.
///
/// Raw, string-y types are used for anything that needs transformation or validation
/// (globs, sed pairs, error codes); that work happens once in [`super::resolve`].
#[derive(Serialize, Deserialize, Debug, Default, Clone, Merge)]
pub struct Config {
    /// See [`super::Config::files`]
    #[serde(default)]
    pub files: Option<Vec<String>>,

    /// See [`super::Config::new_files_directory`]
    #[serde(default)]
    pub new_files_directory: Option<PathBuf>,

    /// See [`super::Config::ngram_size`]
    #[serde(default)]
    pub ngram_size: Option<usize>,

    /// See [`super::Config::boundary_pattern`]
    #[serde(default)]
    pub boundary_pattern: Option<String>,

    /// See [`super::Config::filename_spacing_pattern`]
    #[serde(default)]
    pub filename_spacing_pattern: Option<String>,

    /// See [`super::Config::filename_match_threshold`]
    #[serde(default)]
    pub filename_match_threshold: Option<i64>,

    /// See [`super::Config::exclude`]
    #[serde(default)]
    pub exclude: Option<Vec<String>>,

    /// See [`super::Config::ignore_word_pairs`]
    #[serde(default)]
    pub ignore_word_pairs: Option<Vec<(String, String)>>,

    /// See [`super::Config::alias_to_filename`]. Stored as a `(from, to)` regex pair.
    #[serde(default)]
    pub alias_to_filename: Option<(String, String)>,

    /// See [`super::Config::filename_to_alias`]. Stored as a `(from, to)` regex pair.
    #[serde(default)]
    pub filename_to_alias: Option<(String, String)>,

    /// See [`super::Config::fix`]
    #[serde(default)]
    pub fix: Option<bool>,

    /// See [`super::Config::allow_dirty`]
    #[serde(default)]
    pub allow_dirty: Option<bool>,

    /// See [`super::Config::allow_staged`]
    #[serde(default)]
    pub allow_staged: Option<bool>,

    /// See [`super::Config::ignore_remaining`]
    #[serde(default)]
    pub ignore_remaining: Option<bool>,
}

impl Config {
    /// Read the partial config from a TOML file.
    pub fn new(path: &Path) -> Result<Self, NewConfigError> {
        let contents =
            std::fs::read_to_string(path).map_err(NewConfigError::FileDoesNotReadError)?;
        toml::from_str(&contents).map_err(NewConfigError::FileDoesNotParseError)
    }
}
