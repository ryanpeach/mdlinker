pub mod config;
pub mod file;
pub mod ngrams;
pub mod rules;
pub mod sed;
pub mod visitor;

use std::{cell::RefCell, rc::Rc};

use bon::Builder;
use file::{get_files, name::ngrams};
use miette::{miette, Result};
use rules::{
    broken_wikilink::{BrokenWikilink, BrokenWikilinkVisitor},
    duplicate_alias::DuplicateAlias,
    similar_filename::SimilarFilename,
};
use visitor::{parse, Visitor};

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

    let all_files = get_files(&config.directories);
    let file_ngrams = ngrams(
        &all_files,
        config.ngram_size,
        &boundary_regex,
        &filename_spacing_regex,
    );

    // All our reports
    // NOTE: Always use `filter_by_excludes` and `dedupe_by_code` on the reports
    let similar_filenames = SimilarFilename::calculate(
        &file_ngrams,
        config.filename_match_threshold,
        &filename_spacing_regex,
    )
    .map_err(|e| miette!("From SimilarFilename: {e}"))?
    .finalize(&config.exclude);

    let broken_wikilinks_visitor = Rc::new(RefCell::new(BrokenWikilinkVisitor::new(
        &all_files,
        &config.filename_to_alias,
    )));
    for file in all_files {
        let visitors: Vec<Rc<RefCell<dyn Visitor>>> = vec![broken_wikilinks_visitor.clone()];
        parse(&file, visitors).map_err(|e| miette!(e))?;
    }
    let mut broken_wikilinks_visitor: BrokenWikilinkVisitor =
        Rc::try_unwrap(broken_wikilinks_visitor)
            .expect("visitors vector went out of scope")
            .into_inner();
    broken_wikilinks_visitor
        .finalize(&config.exclude)
        .map_err(|e| miette!(e))?;

    Ok(OutputReport::builder()
        .similar_filenames(similar_filenames)
        .duplicate_aliases(
            broken_wikilinks_visitor
                .duplicate_alias_visitor
                .duplicate_alias_errors,
        )
        .broken_wikilinks(broken_wikilinks_visitor.broken_wikilinks)
        .build())
}
