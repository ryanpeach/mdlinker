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
use merge::Merge;
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
    #[error("Mutually exclusive options: {0} and {1}")]
    MutuallyExclusiveError(String, String),
    #[error("No files were passed to the config")]
    NoFilesPassedError,
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
    pub original_file_globs: Option<Vec<String>>,
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
    /// See [`self::cli::Config::allow_staged`]
    #[builder(default = false)]
    pub allow_staged: bool,
    /// See [`self::file::Config::ignore_word_pairs`]
    #[builder(default = vec![])]
    pub ignore_word_pairs: Vec<(String, String)>,
    /// See [`self::cli::Config::ignore_remaining`]
    #[builder(default = false)]
    pub ignore_remaining: bool,
}

/// Expand a list of file globs/paths into concrete, existing [`PathBuf`]s.
///
/// Entries containing `*` are treated as glob patterns; everything else is treated as
/// a literal path and kept only if it exists. Invalid globs and missing files are
/// reported to stderr and skipped.
fn expand_files(globs: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for file in globs {
        if file.contains('*') {
            match glob::glob(file) {
                Ok(paths) => {
                    for path in paths {
                        match path {
                            Ok(path) => out.push(path),
                            Err(e) => eprintln!("Error processing glob '{file}': {e}"),
                        }
                    }
                }
                Err(e) => eprintln!("Error parsing glob pattern '{file}': {e}"),
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
    out
}

/// Build a [`ReplacePair`] from an optional `(from, to)` regex tuple.
///
/// `None` and an empty `("", "")` pair both mean "unset" (so the master default is
/// used); a half-empty pair is a compilation error, matching the previous behaviour.
fn build_replace_pair<T, U>(
    pair: Option<(String, String)>,
) -> Result<Option<ReplacePair<T, U>>, NewConfigError>
where
    T: ToString + From<String>,
    U: ToString + From<String>,
{
    let Some((from, to)) = pair else {
        return Ok(None);
    };
    match (from.is_empty(), to.is_empty()) {
        (true, true) => Ok(None),
        (false, false) => Ok(Some(ReplacePair::new(&from, &to)?)),
        (true, false) => Err(ReplacePairCompilationError::ToError(regex::Error::Syntax(
            "To is empty".to_string(),
        ))
        .into()),
        (false, true) => Err(ReplacePairCompilationError::FromError(regex::Error::Syntax(
            "From is empty".to_string(),
        ))
        .into()),
    }
}

/// Merge the CLI and file partials (CLI wins), then apply defaults, transforms, and
/// validation to produce the final [`Config`].
///
/// This is the single place a setting's default/transform/validation lives — adding a
/// setting means adding a field to [`file::Config`] and one `.maybe_*` line here.
fn resolve(file_config: file::Config, cli_config: cli::Config) -> Result<Config, NewConfigError> {
    // CLI has priority: its `Some` values are kept, and only its `None` holes are
    // filled from the file. See `merge::Merge for Option`.
    let mut merged = cli_config.to_partial();
    merged.merge(file_config.clone());

    let files = expand_files(&merged.files.clone().unwrap_or_default());
    if files.is_empty() {
        return Err(NewConfigError::NoFilesPassedError);
    }

    // allow_dirty and allow_staged are mutually exclusive
    if merged.allow_dirty.unwrap_or(false) && merged.allow_staged.unwrap_or(false) {
        return Err(NewConfigError::MutuallyExclusiveError(
            "allow_dirty".to_string(),
            "allow_staged".to_string(),
        ));
    }

    Ok(Config::builder()
        .file_config(file_config)
        .cli_config(cli_config)
        .files(files)
        .maybe_original_file_globs(merged.files.clone())
        .maybe_new_files_directory(merged.new_files_directory.clone())
        .maybe_ngram_size(merged.ngram_size)
        .maybe_boundary_pattern(merged.boundary_pattern.clone())
        .maybe_filename_spacing_pattern(merged.filename_spacing_pattern.clone())
        .maybe_filename_match_threshold(merged.filename_match_threshold)
        .maybe_exclude(
            merged
                .exclude
                .clone()
                .map(|codes| codes.into_iter().map(ErrorCode::new).collect()),
        )
        .maybe_filename_to_alias(build_replace_pair(merged.filename_to_alias.clone())?)
        .maybe_alias_to_filename(build_replace_pair(merged.alias_to_filename.clone())?)
        .maybe_fix(merged.fix)
        .maybe_allow_dirty(merged.allow_dirty)
        .maybe_allow_staged(merged.allow_staged)
        .maybe_ignore_word_pairs(merged.ignore_word_pairs.clone())
        .maybe_ignore_remaining(merged.ignore_remaining)
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
            file::Config::new(&cli.config_path)?
        } else {
            return Err(NewConfigError::FileDoesNotExistError {
                path: cli.config_path,
            });
        };

        // CLI has priority over the file; see `resolve`
        resolve(file, cli)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: `filename_to_alias` must be built from the `filename_to_alias`
    /// field, not `alias_to_filename`. A previous copy-paste bug read the wrong field,
    /// so setting a distinct `filename_to_alias` in a config was silently ignored. No
    /// fixture set these fields, so the defaults (which are exact inverses) hid the bug.
    #[test]
    fn filename_to_alias_uses_its_own_field() {
        // Distinct pairs so the two directions cannot be confused: filename_to_alias
        // maps "___" -> "/", alias_to_filename maps "/" -> "-".
        let file = file::Config {
            files: Some(vec!["Cargo.toml".to_string()]),
            filename_to_alias: Some(("___".to_string(), "/".to_string())),
            alias_to_filename: Some(("/".to_string(), "-".to_string())),
            ..Default::default()
        };
        let config = resolve(file, cli::Config::default()).expect("resolve should succeed");

        // If the old bug were present, filename_to_alias would be built from the
        // alias_to_filename pair ("/" -> "-") and leave "a___b" unchanged.
        let alias = config
            .filename_to_alias
            .apply(&Filename("a___b".to_string()));
        assert_eq!(alias.to_string(), "a/b");
    }

    /// The CLI takes priority over the file, and file values fill the holes the CLI
    /// leaves. This exercises the `merge` precedence that `resolve` relies on.
    #[test]
    fn cli_overrides_file_and_file_fills_holes() {
        let file = file::Config {
            files: Some(vec!["Cargo.toml".to_string()]),
            ngram_size: Some(2),
            boundary_pattern: Some("from-file".to_string()),
            ..Default::default()
        };
        let cli = cli::Config {
            ngram_size: Some(1),
            ..Default::default()
        };
        let config = resolve(file, cli).expect("resolve should succeed");

        assert_eq!(config.ngram_size, 1, "CLI value must win");
        assert_eq!(
            config.boundary_pattern, "from-file",
            "file value must fill the hole the CLI left"
        );
    }
}
