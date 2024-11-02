pub mod config;
pub mod file;
pub mod ngrams;
pub mod rules;
pub mod sed;

use bon::Builder;
use file::{get_files, name::ngrams};
use miette::{miette, Result};
use rules::{
    broken_wikilink::BrokenWikilink, duplicate_alias::DuplicateAlias,
    similar_filename::SimilarFilename,
};

use crate::rules::VecHasIdExtensions;

/// A miette diagnostic that controls the printout of errors to the user
/// Put a vector of all outputs in a new field with a #[related] macro above it
#[derive(Debug, Builder)]
pub struct OutputReport {
    pub similar_filenames: Vec<SimilarFilename>,
    pub duplicate_aliases: Vec<DuplicateAlias>,
    pub broken_wikilinks: Vec<BrokenWikilink>,
}

impl OutputReport {
    /// Get if this is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.similar_filenames.is_empty() && self.broken_wikilinks.is_empty()
    }
}

/// The main library function that takes a configuration and returns a Result
/// Comparable to running as an executable
///
/// # Errors
///
/// Even though this returns a Result, its `Ok()` type is also a mieette error IFF .`is_empty()` == false
/// The `Err()` type is a non-linter defined error, like a parsing error or regex error
///
/// Basically if this library fails, this returns an Err
/// but if this library runs, even if it finds linting violations, this returns an Ok
pub fn lib(config: &config::Config) -> Result<OutputReport> {
    // Compile our regex patterns
    let boundary_regex = regex::Regex::new(&config.boundary_pattern).map_err(|e| miette!(e))?;
    let filename_spacing_regex =
        regex::Regex::new(&config.filename_spacing_pattern).map_err(|e| miette!(e))?;

    let file_ngrams = ngrams(
        config.directories.clone(),
        config.ngram_size,
        &boundary_regex,
        &filename_spacing_regex,
    );

    // All our reports
    // NOTE: Always use `filter_by_excludes` and `dedupe_by_code` on the reports
    let similar_filenames =
        SimilarFilename::calculate(&file_ngrams, config.filename_match_threshold)
            .map_err(|e| miette!("From SimilarFilename: {e}"))?
            .filter_by_excludes(config.exclude.clone())
            .dedupe_by_code();

    let duplicate_aliases =
        DuplicateAlias::calculate(get_files(config.directories.clone()), config)
            .map_err(|e| miette!("From DuplicateAlias: {e}"))?
            .filter_by_excludes(config.exclude.clone())
            .dedupe_by_code();

    // Unfortunately we can't continue if we have duplicate aliases
    if !duplicate_aliases.is_empty() {
        log::debug!("Duplicate aliases found, skipping BrokenWikilink check");
        // Return
        return Ok(OutputReport::builder()
            .similar_filenames(similar_filenames)
            .duplicate_aliases(duplicate_aliases)
            .broken_wikilinks(vec![])
            .build());
    }

    let broken_wikilinks =
        BrokenWikilink::calculate(get_files(config.directories.clone()).as_slice(), config)
            .map_err(|e| miette!("From BrokenWikilink: {e}"))?
            .filter_by_excludes(config.exclude.clone())
            .dedupe_by_code();

    // Return
    Ok(OutputReport::builder()
        .similar_filenames(similar_filenames)
        .duplicate_aliases(duplicate_aliases)
        .broken_wikilinks(broken_wikilinks)
        .build())
}
