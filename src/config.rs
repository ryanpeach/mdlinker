mod cli;
mod file;
use std::path::PathBuf;

use crate::{
    file::{
        content::wikilink::Alias,
        name::{Filename, FilenameLowercase},
    },
    sed::{ReplacePair, ReplacePairError},
};
use bon::Builder;
use clap::Parser;
use std::io;
use thiserror;
use toml;

/// Errors derived from config file reads
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The config file at {path} does not exist")]
    FileDoesNotExistError { path: PathBuf },
    #[error("Failed to read the config file")]
    FileDoesNotReadError(#[from] io::Error),
    #[error("The config file does not have expected values")]
    FileDoesNotParseError(#[from] toml::de::Error),
}

/// Config which contains both the cli and the config file
/// Used to reconcile the two
#[derive(Builder)]
pub struct Config {
    /// See [`self::cli::Config::directories`]
    #[builder(default=vec![PathBuf::from(".")])]
    pub directories: Vec<PathBuf>,
    /// See [`self::cli::Config::ngram_size`]
    #[builder(default = 2)]
    pub ngram_size: usize,
    /// See [`self::cli::Config::boundary_pattern`]
    #[builder(default=r"___".to_owned())]
    pub boundary_pattern: String,
    /// See [`self::cli::Config::wikilink_pattern`]
    #[builder(default=r"#?\[\[(.*?)]]|#([A-Za-z0-9_]+)".to_owned())]
    pub wikilink_pattern: String,
    /// See [`self::cli::Config::filename_spacing_pattern`]
    #[builder(default=r"-|_|\s".to_owned())]
    pub filename_spacing_pattern: String,
    /// See [`self::cli::Config::filename_match_threshold`]
    #[builder(default = 2)]
    pub filename_match_threshold: i64,
    /// See [`self::cli::Config::exclude`]
    #[builder(default=vec![])]
    pub exclude: Vec<String>,
    /// See [`self::file::Config::filename_to_alias`]
    #[builder(default=Ok(ReplacePair::new(r"___", r"/").expect("Constant")))]
    pub filename_to_alias: Result<ReplacePair<Filename, Alias>, ReplacePairError>,
    /// See [`self::file::Config::alias_to_filename`]
    #[builder(default=Ok(ReplacePair::new(r"/", r"___").expect("Constant")))]
    pub alias_to_filename: Result<ReplacePair<Alias, FilenameLowercase>, ReplacePairError>,
}

/// Things which implement the partial config trait
/// implement functions which return optionals
/// these can be unioned with one another
/// and then we can use that to create the final config
pub trait Partial {
    fn directories(&self) -> Option<Vec<PathBuf>>;
    fn ngram_size(&self) -> Option<usize>;
    fn boundary_pattern(&self) -> Option<String>;
    fn wikilink_pattern(&self) -> Option<String>;
    fn filename_spacing_pattern(&self) -> Option<String>;
    fn filename_match_threshold(&self) -> Option<i64>;
    fn exclude(&self) -> Option<Vec<String>>;
    fn filename_to_alias(&self) -> Option<Result<ReplacePair<Filename, Alias>, ReplacePairError>>;
    fn alias_to_filename(
        &self,
    ) -> Option<Result<ReplacePair<Alias, FilenameLowercase>, ReplacePairError>>;
}

/// Now we implement a combine function for patrial configs which
/// iterates over the partials and if they have a Some field they use that field in the final
/// config.
///
/// Note: This makes last elements in the input slice first priority
fn combine_partials(partials: &[&dyn Partial]) -> Config {
    Config::builder()
        .maybe_directories(partials.iter().find_map(|p| p.directories()))
        .maybe_ngram_size(partials.iter().find_map(|p| p.ngram_size()))
        .maybe_boundary_pattern(partials.iter().find_map(|p| p.boundary_pattern()))
        .maybe_wikilink_pattern(partials.iter().find_map(|p| p.wikilink_pattern()))
        .maybe_filename_spacing_pattern(partials.iter().find_map(|p| p.filename_spacing_pattern()))
        .maybe_filename_match_threshold(partials.iter().find_map(|p| p.filename_match_threshold()))
        .maybe_exclude(partials.iter().find_map(|p| p.exclude()))
        .maybe_filename_to_alias(partials.iter().find_map(|p| p.filename_to_alias()))
        .maybe_alias_to_filename(partials.iter().find_map(|p| p.alias_to_filename()))
        .build()
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
        let cli = cli::Config::parse();

        // If the config file doesn't exist, and it's not the default, error out
        let file = if cli.config_path.is_file() {
            match file::Config::new(&cli.config_path) {
                Ok(file) => file,
                Err(report) => Err(report)?,
            }
        } else {
            Err(Error::FileDoesNotExistError {
                path: cli.config_path.clone(),
            })?
        };

        // CLI has priority over file by being last
        Ok(combine_partials(&[&file, &cli]))
    }
}
