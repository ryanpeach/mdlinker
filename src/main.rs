use mdlinker::config;
use mdlinker::error::VecHasIdExtensions;
use mdlinker::file;
use mdlinker::file::name::SimilarFilenames;
use miette::miette;
use miette::Diagnostic;
use thiserror::Error;

use miette::Result;

#[derive(Debug, Error, Diagnostic)]
#[error("Output Report")]
struct OutputReport {
    #[related]
    similar_filenames: Vec<SimilarFilenames>,
}

impl OutputReport {
    pub fn new(similar_filenames: Vec<SimilarFilenames>) -> Self {
        Self { similar_filenames }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    // Load the configuration
    let config = config::Config::new().map_err(|e| miette!(e))?;

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
