use clap::Parser;
use std::path::PathBuf;

use super::file::Config as PartialConfig;

#[derive(Parser, Default, Clone)]
#[command(version, about, long_about = None)]
pub struct Config {
    /// Globs or paths to relevant files
    #[clap()]
    pub files: Vec<PathBuf>,

    /// A location to store new files in created by --fix
    /// for the [`super::rules::broken_wikilink::BrokenWikilink`] rule
    #[clap(short = 'N', long = "newf")]
    pub new_files_directory: Option<PathBuf>,

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
    /// the git repo has uncommitted changes and those changes will be overwritten by this apps
    /// changes without user involvement. WARNING: Dangerous
    /// Mutually exclusive with --allow-staged
    #[clap(long = "allow-dirty")]
    pub allow_dirty: bool,

    /// Whether or not to allow fixing in a "dirty" git repo where everything is "staged", meaning
    /// the git repo has uncommitted changes, but they still won't be changed by edits unless the
    /// user stages the changes.
    /// Mutually exclusive with --allow-dirty
    #[clap(long = "allow-staged")]
    pub allow_staged: bool,

    /// Ignore remaining errors by adding them to the config
    #[clap(long = "ignore-remaining")]
    pub ignore_remaining: bool,
}

impl Config {
    /// Map the CLI arguments into a [`PartialConfig`] so they can be merged with the
    /// file config. Fields the CLI does not expose (the sed pairs, `ignore_word_pairs`)
    /// stay `None`. Empty positional/multi-value args map to `None` so an unset CLI
    /// value does not clobber a file value during the merge. Boolean flags always map
    /// to `Some`, so a CLI-set flag wins and the file cannot override it — matching the
    /// previous behaviour.
    #[must_use]
    pub fn to_partial(&self) -> PartialConfig {
        PartialConfig {
            files: if self.files.is_empty() {
                None
            } else {
                Some(
                    self.files
                        .iter()
                        .map(|p| p.to_string_lossy().into_owned())
                        .collect(),
                )
            },
            new_files_directory: self.new_files_directory.clone(),
            ngram_size: self.ngram_size,
            boundary_pattern: self.boundary_pattern.clone(),
            filename_spacing_pattern: self.filename_spacing_pattern.clone(),
            filename_match_threshold: self.filename_match_threshold,
            exclude: if self.exclude.is_empty() {
                None
            } else {
                Some(self.exclude.clone())
            },
            ignore_word_pairs: None,
            alias_to_filename: None,
            filename_to_alias: None,
            fix: Some(self.fix),
            allow_dirty: Some(self.allow_dirty),
            allow_staged: Some(self.allow_staged),
            ignore_remaining: Some(self.ignore_remaining),
        }
    }
}
