use clap::Parser;
use std::path::PathBuf;

use crate::{
    file::{
        content::wikilink::Alias,
        name::{Filename, FilenameLowercase},
    },
    rules::ErrorCode,
    sed::{ReplacePair, ReplacePairCompilationError},
};

use super::Partial;

#[derive(Parser, Default, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// The pages directory is the directory where pages are named for their alias
    /// and where new pages should be created when running --fix
    #[clap(short = 'p', long = "pages")]
    pub pages_directory: Option<PathBuf>,

    /// Other directories to search in
    #[clap(short = 'd', long = "dir")]
    pub other_directories: Vec<PathBuf>,

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

    /// Regex pattern to split filenames on, like ___ or /
    #[clap(short = 's', long = "space")]
    pub filename_spacing_pattern: Option<String>,

    /// The minimum score to consider a match for filename ngrams
    #[clap(short = 'm', long = "score")]
    pub filename_match_threshold: Option<i64>,

    /// Exclude certain error codes
    /// If an error code **starts with** this string, it will be excluded
    /// This accepts glob patterns
    #[clap(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,

    /// Whether or not to try to fix the errors
    #[clap(short = 'f', long = "fix")]
    pub fix: bool,

    /// Whether or not to allow fixing in a "dirty" git repo, meaning
    /// the git repo has uncommitted changes
    #[clap(long = "allow-dirty")]
    pub allow_dirty: bool,

    /// Ignore remaining errors by adding them to the config
    #[clap(long = "ignore-remaining")]
    pub ignore_remaining: bool,
}

impl Partial for Config {
    fn pages_directory(&self) -> Option<PathBuf> {
        self.pages_directory.clone()
    }
    fn other_directories(&self) -> Option<Vec<PathBuf>> {
        let out = self.other_directories.clone();
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
    fn filename_to_alias(
        &self,
    ) -> Option<Result<ReplacePair<Filename, Alias>, ReplacePairCompilationError>> {
        None
    }
    fn alias_to_filename(
        &self,
    ) -> Option<Result<ReplacePair<Alias, FilenameLowercase>, ReplacePairCompilationError>> {
        None
    }
    fn fix(&self) -> Option<bool> {
        Some(self.fix)
    }
    fn allow_dirty(&self) -> Option<bool> {
        Some(self.allow_dirty)
    }
    fn ignore_word_pairs(&self) -> Option<Vec<(String, String)>> {
        None
    }
    fn ignore_remaining(&self) -> Option<bool> {
        Some(self.ignore_remaining)
    }
}
