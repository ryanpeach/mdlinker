use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Default)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// The directories to search in
    /// May provide more than one directory
    #[clap(short = 'd', long = "dir")]
    pub directories: Vec<PathBuf>,

    /// Path to a configuration file
    #[clap(short = 'c', long = "config", default_value = "mdlinker.toml")]
    pub config_path: PathBuf,

    /// Size of the n-grams to generate from filenames
    /// Will generate n-grams UP TO and INCLUDING this size
    #[clap(short = 'n', long = "ngram")]
    pub ngram_size: Option<usize>,

    /// Regex pattern to stop n-gram generation on, like , or .")
    #[clap(short = 'b', long = "bound")]
    pub boundary_pattern: Option<String>,

    /// Regex pattern to split filenames on, like _ or -")
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
