mod cli;
mod file;
use std::path::PathBuf;

use crate::{
    file::{
        content::wikilink::Alias,
        name::{Filename, FilenameLowercase},
    },
    rules::ErrorCode,
    sed::{ReplacePair, ReplacePairCompilationError},
};
use bon::Builder;
use clap::Parser;
use miette::Diagnostic;
use std::io;
use thiserror;
use toml;

/// Errors derived from config file reads
#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum NewConfigError {
    #[error("The config file at {path} does not exist")]
    FileDoesNotExistError { path: PathBuf },
    #[error("Failed to read the config file")]
    FileDoesNotReadError(#[from] io::Error),
    #[error("The config file does not have expected values")]
    FileDoesNotParseError(#[from] toml::de::Error),
    #[error("ReplacePair compilation error")]
    ReplacePairCompilationError(#[from] ReplacePairCompilationError),
    #[error("Pages directory missing")]
    #[help("Please provide a pages directory argument in either your cli or config file")]
    PagesDirectoryMissing,
}

/// Config which contains both the cli and the config file
/// Used to reconcile the two
#[derive(Builder)]
pub struct Config {
    /// See [`self::cli::Config::pages_directory`]
    pub pages_directory: PathBuf,
    /// See [`self::cli::Config::other_directories`]
    #[builder(default=vec![])]
    pub other_directories: Vec<PathBuf>,
    /// See [`self::cli::Config::ngram_size`]
    #[builder(default = 2)]
    pub ngram_size: usize,
    /// See [`self::cli::Config::boundary_pattern`]
    #[builder(default=r"___".to_owned())]
    pub boundary_pattern: String,
    /// See [`self::cli::Config::filename_spacing_pattern`]
    #[builder(default=r"-|_|\s".to_owned())]
    pub filename_spacing_pattern: String,
    /// See [`self::cli::Config::filename_match_threshold`]
    #[builder(default = 100)]
    pub filename_match_threshold: i64,
    /// See [`self::cli::Config::exclude`]
    #[builder(default=vec![])]
    pub exclude: Vec<ErrorCode>,
    /// See [`self::file::Config::filename_to_alias`]
    #[builder(default=ReplacePair::new(r"___", r"/").expect("Constant"))]
    pub filename_to_alias: ReplacePair<Filename, Alias>,
    /// See [`self::file::Config::alias_to_filename`]
    #[builder(default=ReplacePair::new(r"/", r"___").expect("Constant"))]
    pub alias_to_filename: ReplacePair<Alias, FilenameLowercase>,
    /// See [`self::cli::Config::fix`]
    #[builder(default = false)]
    pub fix: bool,
    /// See [`self::cli::Config::allow_dirty`]
    #[builder(default = false)]
    pub allow_dirty: bool,
    /// See [`self::file::Config::ignore_word_pairs`]
    #[builder(default = vec![])]
    pub ignore_word_pairs: Vec<(String, String)>,
}

/// Things which implement the partial config trait
/// implement functions which return optionals
/// these can be unioned with one another
/// and then we can use that to create the final config
pub trait Partial {
    fn pages_directory(&self) -> Option<PathBuf>;
    fn other_directories(&self) -> Option<Vec<PathBuf>>;
    fn ngram_size(&self) -> Option<usize>;
    fn boundary_pattern(&self) -> Option<String>;
    fn filename_spacing_pattern(&self) -> Option<String>;
    fn filename_match_threshold(&self) -> Option<i64>;
    fn exclude(&self) -> Option<Vec<ErrorCode>>;
    fn filename_to_alias(
        &self,
    ) -> Option<Result<ReplacePair<Filename, Alias>, ReplacePairCompilationError>>;
    fn alias_to_filename(
        &self,
    ) -> Option<Result<ReplacePair<Alias, FilenameLowercase>, ReplacePairCompilationError>>;
    fn fix(&self) -> Option<bool>;
    fn allow_dirty(&self) -> Option<bool>;
    fn ignore_word_pairs(&self) -> Option<Vec<(String, String)>>;
}

/// Now we implement a combine function for patrial configs which
/// iterates over the partials and if they have a Some field they use that field in the final
/// config.
///
/// Note: This makes last elements in the input slice first priority
fn combine_partials(partials: &[&dyn Partial]) -> Result<Config, NewConfigError> {
    Ok(Config::builder()
        .maybe_ngram_size(partials.iter().find_map(|p| p.ngram_size()))
        .maybe_boundary_pattern(partials.iter().find_map(|p| p.boundary_pattern()))
        .maybe_filename_spacing_pattern(partials.iter().find_map(|p| p.filename_spacing_pattern()))
        .maybe_filename_match_threshold(partials.iter().find_map(|p| p.filename_match_threshold()))
        .maybe_exclude(partials.iter().find_map(|p| p.exclude()))
        .maybe_filename_to_alias(match partials.iter().find_map(|p| p.filename_to_alias()) {
            Some(Ok(pair)) => Some(pair),
            Some(Err(e)) => return Err(NewConfigError::ReplacePairCompilationError(e)),
            None => None,
        })
        .maybe_alias_to_filename(match partials.iter().find_map(|p| p.alias_to_filename()) {
            Some(Ok(pair)) => Some(pair),
            Some(Err(e)) => return Err(NewConfigError::ReplacePairCompilationError(e)),
            None => None,
        })
        .maybe_fix(partials.iter().find_map(|p| p.fix()))
        .maybe_allow_dirty(partials.iter().find_map(|p| p.allow_dirty()))
        .pages_directory(
            partials
                .iter()
                .find_map(|p| p.pages_directory())
                .ok_or(NewConfigError::PagesDirectoryMissing)?,
        )
        .maybe_other_directories(partials.iter().find_map(|p| p.other_directories()))
        .maybe_ignore_word_pairs(partials.iter().find_map(|p| p.ignore_word_pairs()))
        .build())
}

impl Config {
    /// Creates a new [`Config`] from the default location at "mdlinker.toml"
    ///
    /// # Errors
    ///
    /// - [`Error::FileDoesNotExistError`] - Config file does not exist
    /// - [`Error::FileDoesNotParseError`] - Config file does not parse from toml into the
    ///     expected format
    ///
    pub fn new() -> Result<Self, NewConfigError> {
        let cli = cli::Config::parse();

        // If the config file doesn't exist, and it's not the default, error out
        let file = if cli.config_path.is_file() {
            match file::Config::new(&cli.config_path) {
                Ok(file) => file,
                Err(report) => Err(report)?,
            }
        } else {
            Err(NewConfigError::FileDoesNotExistError {
                path: cli.config_path.clone(),
            })?
        };

        // CLI has priority over file by being last
        combine_partials(&[&file, &cli])
    }

    /// Legacy directories function
    /// Gets all the directories into one vec
    #[must_use]
    pub fn directories(&self) -> Vec<PathBuf> {
        let mut out = vec![self.pages_directory.clone()];
        out.extend(self.other_directories.clone());
        out
    }
}
