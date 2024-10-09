mod cli;
mod default;
mod file;
use std::path::PathBuf;

use crate::sed::{ReplacePair, ReplacePairError};
use clap::Parser;
use thiserror;
use toml;
use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The config file at {path} does not exist")]
    FileDoesNotExistError{
        path: PathBuf
    },
    #[error("Failed to read the config file")]
    FileDoesNotReadError(#[from] io::Error),
    #[error("The config file does not have expected values")]
    FileDoesNotParseError(#[from] toml::de::Error)
}

/// Config which contains both the cli and the config file
/// Used to reconcile the two
pub struct Config {
    cli: cli::Cli,
    file: file::Config,
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
    pub fn new() -> Result<Self, Error> {
        let cli = cli::Cli::parse();

        // If the config file doesn't exist, and it's not the default, error out
        if cli.config_path.is_file() {
            match file::Config::new(&cli.config_path) {
                Ok(file) => Ok(Self { cli, file }),
                Err(report) => Err(report)
            }
        } else {
            Err(Error::FileDoesNotExistError{ path: cli.config_path })
        } 
    }

    #[must_use]
    pub fn directories(&self) -> Vec<PathBuf> {
        let mut out = self.cli.directories.clone();
        out.extend(self.file.directories.clone());
        if out.is_empty() {
            default::directories()
        } else {
            out
        }
    }

    #[must_use]
    pub fn ngram_size(&self) -> usize {
        self.cli
            .ngram_size
            .unwrap_or_else(|| self.file.ngram_size.unwrap_or_else(default::ngram_size))
    }

    #[must_use]
    pub fn boundary_pattern(&self) -> String {
        self.cli.boundary_pattern.clone().unwrap_or_else(|| {
            self.file
                .boundary_pattern
                .clone()
                .unwrap_or_else(default::boundary_pattern)
        })
    }

    #[must_use]
    pub fn filename_spacing_pattern(&self) -> String {
        self.cli
            .filename_spacing_pattern
            .clone()
            .unwrap_or_else(|| {
                self.file
                    .filename_spacing_pattern
                    .clone()
                    .unwrap_or_else(default::filename_spacing_pattern)
            })
    }

    #[must_use]
    pub fn filename_match_threshold(&self) -> i64 {
        self.cli.filename_match_threshold.unwrap_or_else(|| {
            self.file
                .filename_match_threshold
                .unwrap_or_else(default::filename_match_threshold)
        })
    }

    #[must_use]
    pub fn exclude(&self) -> Vec<String> {
        let mut out = self.cli.exclude.clone();
        out.extend(self.file.exclude.clone());
        if out.is_empty() {
            default::exclude()
        } else {
            out
        }
    }

    /// Converts filepaths to titles using the from and to regexes defined in the 
    /// [`file::Config`]`.filepath_to_title` option.
    ///
    /// # Errors
    ///
    /// - [`ReplacePairError`] In the case of a regex error.
    pub fn filepath_to_title(&self) -> Result<Vec<Vec<ReplacePair>>, ReplacePairError> {
        let mut out: Vec<Vec<ReplacePair>> = vec![];
        for outer in self.file.filepath_to_title.clone() {
            for inner in outer {
                let mut temp: Vec<ReplacePair> = vec![];
                let (from, to) = inner;
                temp.push(ReplacePair::new(&from, &to)?);
                out.push(temp);
            }
        }
        if out.is_empty() {
            out = default::filepath_to_title();
        }
        Ok(out)
    }

    /// Converts titles to filepaths using the from and to regexes defined in the 
    /// [`file::Config`]`.title_to_filepath` option.
    ///
    /// The opposite of [`Config::filepath_to_title`]
    ///
    /// # Errors
    ///
    /// - [`ReplacePairError`] In the case of a regex error.
    pub fn title_to_filepath(&self) -> Result<Vec<Vec<ReplacePair>>, ReplacePairError> {
        let mut out: Vec<Vec<ReplacePair>> = vec![];
        for outer in self.file.title_to_filepath.clone() {
            for inner in outer {
                let mut temp: Vec<ReplacePair> = vec![];
                let (from, to) = inner;
                temp.push(ReplacePair::new(&from, &to)?);
                out.push(temp);
            }
        }
        if out.is_empty() {
            out = default::title_to_filepath();
        }
        Ok(out)
    }
}
