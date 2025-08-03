pub mod cli;
pub mod file;
use std::path::PathBuf;

use crate::{
    file::{
        content::wikilink::Alias,
        name::{Filename, FilenameLowercase},
    },
    rules::{ErrorCode, ReportTrait},
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
    #[allow(clippy::struct_field_names)]
    file_config: file::Config,
    #[allow(clippy::struct_field_names)]
    cli_config: cli::Config,
    /// See [`self::cli::Config::files`]
    #[builder(default=vec![])]
    pub files: Vec<PathBuf>,
    /// See [`self::cli::Config::root_directory`]
    #[builder(default=PathBuf::from("."))]
    pub new_files_directory: PathBuf,
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
    /// See [`self::cli::Config::ignore_remaining`]
    #[builder(default = false)]
    pub ignore_remaining: bool,
}

/// Things which implement the partial config trait
/// implement functions which return optionals
/// these can be unioned with one another
/// and then we can use that to create the final config
pub trait Partial {
    fn files(&self) -> Option<Vec<PathBuf>>;
    fn new_files_directory(&self) -> Option<PathBuf>;
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
    fn ignore_remaining(&self) -> Option<bool>;
}

/// Now we implement a combine function for patrial configs which
/// iterates over the partials and if they have a Some field they use that field in the final
/// config.
///
/// Note: This makes last elements in the input slice first priority
fn combine_partials(
    file_config: &file::Config,
    cli_config: &cli::Config,
) -> Result<Config, NewConfigError> {
    Ok(Config::builder()
        .file_config(file_config.clone())
        .cli_config(cli_config.clone())
        .maybe_ngram_size(cli_config.ngram_size().or(file_config.ngram_size()))
        .maybe_boundary_pattern(
            cli_config
                .boundary_pattern()
                .or(file_config.boundary_pattern()),
        )
        .maybe_filename_spacing_pattern(
            cli_config
                .filename_spacing_pattern()
                .or(file_config.filename_spacing_pattern()),
        )
        .maybe_filename_match_threshold(
            cli_config
                .filename_match_threshold()
                .or(file_config.filename_match_threshold()),
        )
        .maybe_exclude(cli_config.exclude().or(file_config.exclude()))
        .maybe_filename_to_alias({
            match (
                cli_config.filename_to_alias(),
                file_config.filename_to_alias(),
            ) {
                (Some(Ok(cli)), None) => Some(cli),
                (None, Some(Ok(file))) => Some(file),
                (Some(Ok(cli)), Some(Ok(_file))) => Some(cli),
                (_, Some(Err(e))) | (Some(Err(e)), _) => {
                    return Err(NewConfigError::ReplacePairCompilationError(e))
                }
                (None, None) => None,
            }
        })
        .maybe_alias_to_filename({
            match (
                cli_config.alias_to_filename(),
                file_config.alias_to_filename(),
            ) {
                (Some(Ok(cli)), None) => Some(cli),
                (None, Some(Ok(file))) => Some(file),
                (Some(Ok(cli)), Some(Ok(_file))) => Some(cli),
                (_, Some(Err(e))) | (Some(Err(e)), _) => {
                    return Err(NewConfigError::ReplacePairCompilationError(e))
                }
                (None, None) => None,
            }
        })
        .maybe_fix(cli_config.fix().or(file_config.fix()))
        .maybe_allow_dirty(cli_config.allow_dirty().or(file_config.allow_dirty()))
        .files(
            cli_config
                .files()
                .or(file_config.files())
                .expect("A default is set"),
        )
        .maybe_ignore_word_pairs(
            cli_config
                .ignore_word_pairs()
                .or(file_config.ignore_word_pairs()),
        )
        .maybe_ignore_remaining(
            cli_config
                .ignore_remaining()
                .or(file_config.ignore_remaining()),
        )
        .build())
}

impl Config {
    /// Creates a new [`Config`] from the default location at "mdlinker.toml"
    ///
    /// # Errors
    ///
    ///  - [`Error::FileDoesNotExistError`] - Config file does not exist
    ///  - [`Error::FileDoesNotParseError`] - Config file does not parse from toml into the
    ///    expected format
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
        let mut out = combine_partials(&file, &cli);

        // Match on a ref to out, so we do NOT move the config out of `out`
        if let Ok(ref mut config) = out {
            config.cli_config = cli;
            config.file_config = file;
        }

        // Now `out` is still valid (unchanged type), so we can return it
        out
    }

    /// Legacy directories function
    /// Gets all the directories into one vec
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    pub fn add_report_to_ignore(&mut self, report: &impl ReportTrait) {
        report.ignore(&mut self.file_config);
    }

    pub fn save_config(&self) -> Result<(), SaveConfigError> {
        let toml_str =
            toml::to_string(&self.file_config).map_err(|e| SaveConfigError::Toml { source: e })?;
        std::fs::write(self.cli_config.config_path.clone(), toml_str)
            .map_err(|e| SaveConfigError::Io { source: e })?;
        Ok(())
    }
}

#[derive(thiserror::Error, Debug, Diagnostic)]
pub enum SaveConfigError {
    #[error(transparent)]
    Io {
        #[backtrace]
        source: io::Error,
    },
    #[error(transparent)]
    Toml {
        #[backtrace]
        source: toml::ser::Error,
    },
}
