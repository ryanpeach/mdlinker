use clap::Parser;
use std::path::PathBuf;

use crate::sed::{ReplacePair, ReplacePairError};

use super::Partial;

#[derive(Parser, Default)]
#[command(version, about, long_about = None)]
pub(super) struct Config {
    /// The directories to search in
    /// May provide more than one directory
    #[clap(short = 'd', long = "dir")]
    pub directories: Vec<PathBuf>,

    /// Path to a configuration file
    #[clap(short = 'c', long = "config", default_value = "mdlinker.toml")]
    #[allow(clippy::struct_field_names)]
    pub config_path: PathBuf,

    /// Size of the n-grams to generate from filenames
    /// Will generate n-grams UP TO and INCLUDING this size
    #[clap(short = 'n', long = "ngram")]
    pub ngram_size: Option<usize>,

    /// Regex pattern to stop n-gram generation on, like , or .
    #[clap(short = 'b', long = "bound")]
    pub boundary_pattern: Option<String>,

    /// Regex pattern for wikilinks
    #[clap(short = 'w', long = "wikilink")]
    pub wikilink_pattern: Option<String>,

    /// Regex pattern to split filenames on, like ___ or /
    #[clap(short = 's', long = "space")]
    pub filename_spacing_pattern: Option<String>,

    /// The minimum score to consider a match for filename ngrams
    #[clap(short = 'm', long = "score")]
    pub filename_match_threshold: Option<i64>,

    /// Exclude certain error codes
    /// If an error code **starts with** this string, it will be excluded
    #[clap(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,
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
        None
    }
    fn title_to_filepath(&self) -> Option<Result<Vec<Vec<ReplacePair>>, ReplacePairError>> {
        None
    }
}
