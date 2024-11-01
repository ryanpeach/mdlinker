mod cli;
mod file;
use std::path::PathBuf;

use crate::sed::{ReplacePair, ReplacePairError};
use bon::Builder;
use clap::Parser;
use getset::Getters;
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
#[derive(Getters, Builder)]
#[getset(get = "pub")]
pub struct Config {
    #[builder(default=vec![PathBuf::from(".")])]
    directories: Vec<PathBuf>,
    #[builder(default = 2)]
    ngram_size: usize,
    #[builder(default=r"\s".to_owned())]
    boundary_pattern: String,
    #[builder(default=r"___|__|-|_|\s".to_owned())]
    filename_spacing_pattern: String,
    #[builder(default = 2)]
    filename_match_threshold: i64,
    #[builder(default=vec![])]
    exclude: Vec<String>,
    #[builder(default=Ok(vec![vec![
        ReplacePair::new(r"([A-Za-z0-1_-]+).md", r"\[\[$1\]\]").expect("Constant"),
        ReplacePair::new(r"___", r"/").expect("Constant"),
    ]]))]
    filepath_to_title: Result<Vec<Vec<ReplacePair>>, ReplacePairError>,
    #[builder(default=Ok(vec![vec![
        ReplacePair::new(r"\[\[(.*?)\]\]", r"$1.md").expect("Constant"),
        ReplacePair::new(r"/", r"___").expect("Constant"),
        ReplacePair::new(r"(.*)", r"../pages/$1").expect("Constant"),
    ]]))]
    title_to_filepath: Result<Vec<Vec<ReplacePair>>, ReplacePairError>,
}

/// Things which implement the partial config trait
/// implement functions which return optionals
/// these can be unioned with one another
/// and then we can use that to create the final config
pub trait Partial {
    fn directories(&self) -> Option<Vec<PathBuf>>;
    fn ngram_size(&self) -> Option<usize>;
    fn boundary_pattern(&self) -> Option<String>;
    fn filename_spacing_pattern(&self) -> Option<String>;
    fn filename_match_threshold(&self) -> Option<i64>;
    fn exclude(&self) -> Option<Vec<String>>;
    fn filepath_to_title(&self) -> Option<Result<Vec<Vec<ReplacePair>>, ReplacePairError>>;
    fn title_to_filepath(&self) -> Option<Result<Vec<Vec<ReplacePair>>, ReplacePairError>>;
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
        .maybe_filename_spacing_pattern(partials.iter().find_map(|p| p.filename_spacing_pattern()))
        .maybe_filename_match_threshold(partials.iter().find_map(|p| p.filename_match_threshold()))
        .maybe_exclude(partials.iter().find_map(|p| p.exclude()))
        .maybe_filepath_to_title(partials.iter().find_map(|p| p.filepath_to_title()))
        .maybe_title_to_filepath(partials.iter().find_map(|p| p.title_to_filepath()))
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
