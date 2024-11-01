pub mod config;
pub mod error;
pub mod file;
pub mod ngrams;
pub mod sed;

use file::name::SimilarFilenames;
use miette::{miette, Diagnostic, Result};

use crate::error::VecHasIdExtensions;
use thiserror::Error;

/// A miette diagnostic that controls the printout of errors to the user
/// Put a vector of all outputs in a new field with a #[related] macro above it
#[derive(Debug, Error, Diagnostic)]
#[error("Output Report")]
struct OutputReport {
    #[related]
    similar_filenames: Vec<SimilarFilenames>,
}

impl OutputReport {
    /// Create a new output report
    pub fn new(similar_filenames: Vec<SimilarFilenames>) -> Self {
        Self { similar_filenames }
    }
}

/// The main library function that takes a configuration and returns a Result
/// Comparable to running as an executable
pub fn lib(config: &config::Config) -> Result<()> {
    // Compile our regex patterns
    let boundary_regex = regex::Regex::new(config.boundary_pattern()).map_err(|e| miette!(e))?;
    let filename_spacing_regex =
        regex::Regex::new(config.filename_spacing_pattern()).map_err(|e| miette!(e))?;

    let file_ngrams = file::name::ngrams(
        config.directories().clone(),
        *config.ngram_size(),
        &boundary_regex,
        &filename_spacing_regex,
    );

    // Calculate the similarity between filenames
    let matches = SimilarFilenames::calculate(&file_ngrams, *config.filename_match_threshold())
        .map_err(|e| miette!(e))?
        .filter_by_excludes(config.exclude().clone())
        .dedupe_by_id();

    // Return
    if matches.is_empty() {
        Ok(())
    } else {
        log::error!("Found {} similar filenames", matches.len());
        Err(OutputReport::new(matches).into())
    }
}
