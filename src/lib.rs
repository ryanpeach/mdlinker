#![feature(error_generic_member_access)]

pub mod config;
pub mod file;
pub mod ngrams;
pub mod rules;
pub mod sed;
pub mod visitor;

use std::{backtrace::Backtrace, cell::RefCell, rc::Rc};

use file::{get_files, name::ngrams};
use log::info;
use miette::{Diagnostic, Result};
use ngrams::MissingSubstringError;
use rules::{
    broken_wikilink::BrokenWikilinkVisitor, duplicate_alias::DuplicateAliasVisitor,
    similar_filename::SimilarFilename, Report, ReportTrait, ThirdPassRule,
};
use strum::IntoEnumIterator;
use thiserror::Error;
use visitor::{parse, FinalizeError, ParseError, Visitor};

use crate::rules::VecHasIdExtensions;

/// A miette diagnostic that controls the printout of errors to the user
/// Put a vector of all outputs in a new field with a #[related] macro above it
pub struct OutputReport {
    pub reports: Vec<Report>,
}

impl OutputReport {
    /// Get if this is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reports.is_empty()
    }
    #[must_use]
    pub fn broken_wikilinks(&self) -> Vec<rules::broken_wikilink::BrokenWikilink> {
        self.reports
            .iter()
            .filter_map(|x| match x {
                Report::ThirdPass(rules::ThirdPassReport::BrokenWikilink(x)) => Some(x.clone()),
                _ => None,
            })
            .collect()
    }
    #[must_use]
    pub fn unlinked_texts(&self) -> Vec<rules::unlinked_text::UnlinkedText> {
        self.reports
            .iter()
            .filter_map(|x| match x {
                Report::ThirdPass(rules::ThirdPassReport::UnlinkedText(x)) => Some(x.clone()),
                _ => None,
            })
            .collect()
    }
    #[must_use]
    pub fn duplicate_aliases(&self) -> Vec<rules::duplicate_alias::DuplicateAlias> {
        self.reports
            .iter()
            .filter_map(|x| match x {
                Report::DuplicateAlias(x) => Some(x.clone()),
                _ => None,
            })
            .collect()
    }
    #[must_use]
    pub fn similar_filenames(&self) -> Vec<rules::similar_filename::SimilarFilename> {
        self.reports
            .iter()
            .filter_map(|x| match x {
                Report::SimilarFilename(x) => Some(x.clone()),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug, Error, Diagnostic)]
pub enum OutputErrors {
    #[error(transparent)]
    RegexError(#[from] regex::Error),
    #[error(transparent)]
    MissingSubstringError(#[from] MissingSubstringError),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    FinalizeError(#[from] FinalizeError),
    #[error(transparent)]
    FixError(#[from] rules::FixError),
}

use git2::{Error, Repository, StatusOptions};

fn is_repo_dirty(repo: &Repository) -> Result<bool, Error> {
    let mut options = StatusOptions::new();
    options
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .exclude_submodules(true)
        .include_unmodified(false)
        .include_ignored(false);

    let statuses = repo.statuses(Some(&mut options))?;
    Ok(!statuses.is_empty())
}

/// Runs [`check`] in a loop until no more fixes can be made
fn fix(config: &config::Config) -> Result<OutputReport, OutputErrors> {
    // Check if the git repo is dirty
    match git2::Repository::open_from_env() {
        Ok(git) => match is_repo_dirty(&git) {
            Ok(is_dirty) => {
                if !config.allow_dirty && is_dirty {
                    return Err(OutputErrors::FixError(rules::FixError::DirtyRepo {
                        backtrace: Backtrace::force_capture(),
                    }));
                }
            }
            Err(e) => {
                return Err(OutputErrors::FixError(rules::FixError::GitError {
                    source: e,
                    backtrace: Backtrace::force_capture(),
                }));
            }
        },
        Err(e) => {
            return Err(OutputErrors::FixError(rules::FixError::GitError {
                source: e,
                backtrace: Backtrace::force_capture(),
            }));
        }
    }

    let mut output_report = check(config)?;
    let mut counter = 0;
    loop {
        info!("Fixing iteration {}", counter);
        let mut any_fixes = false;
        for report in output_report.reports.clone() {
            if let Some(()) = match report {
                Report::DuplicateAlias(report) => report.fix(config)?,
                Report::SimilarFilename(report) => report.fix(config)?,
                Report::ThirdPass(rules::ThirdPassReport::BrokenWikilink(report)) => {
                    report.fix(config)?
                }
                Report::ThirdPass(rules::ThirdPassReport::UnlinkedText(report)) => {
                    report.fix(config)?
                }
            } {
                any_fixes = true;
            }
        }

        if !any_fixes {
            break;
        }

        output_report = check(config)?;
        counter += 1;
    }

    Ok(output_report)
}

fn check(config: &config::Config) -> Result<OutputReport, OutputErrors> {
    // Compile our regex patterns
    let boundary_regex = regex::Regex::new(&config.boundary_pattern)?;
    let filename_spacing_regex = regex::Regex::new(&config.filename_spacing_pattern)?;

    let all_files = get_files(&config.directories());
    let file_ngrams = ngrams(
        &all_files,
        config.ngram_size,
        &boundary_regex,
        &filename_spacing_regex,
    );

    let mut reports: Vec<Report> = vec![];

    // First pass
    // Just over filenames
    // NOTE: Always use `filter_by_excludes` and `dedupe_by_code` on the reports
    let similar_filenames = SimilarFilename::calculate(
        &file_ngrams,
        config.filename_match_threshold,
        &filename_spacing_regex,
    )?
    .finalize(&config.exclude);
    reports.extend(
        similar_filenames
            .iter()
            .map(|x| Report::SimilarFilename(x.clone())),
    );

    // Second pass
    // This gives us metadata we need for all other rules from the content of files
    //  The duplicate alias visitor has to run first to get the table of aliases
    let duplicate_alias_visitor = Rc::new(RefCell::new(DuplicateAliasVisitor::new(
        &all_files,
        &config.filename_to_alias,
    )));
    for file in &all_files {
        let visitors: Vec<Rc<RefCell<dyn Visitor>>> = vec![duplicate_alias_visitor.clone()];
        parse(file, visitors)?;
    }
    let mut duplicate_alias_visitor: DuplicateAliasVisitor =
        Rc::try_unwrap(duplicate_alias_visitor)
            .expect("parse is done")
            .into_inner();
    reports.extend(duplicate_alias_visitor.finalize(&config.exclude)?);

    // Third Pass
    let mut visitors: Vec<Rc<RefCell<dyn Visitor>>> = vec![];
    for rule in ThirdPassRule::iter() {
        visitors.push(match rule {
            ThirdPassRule::UnlinkedText => Rc::new(RefCell::new(
                rules::unlinked_text::UnlinkedTextVisitor::new(
                    &all_files,
                    &config.filename_to_alias,
                    duplicate_alias_visitor.alias_table.clone(),
                ),
            )),
            ThirdPassRule::BrokenWikilink => Rc::new(RefCell::new(BrokenWikilinkVisitor::new(
                &all_files,
                &config.filename_to_alias,
                duplicate_alias_visitor.alias_table.clone(),
            ))),
        });
    }

    for file in &all_files {
        parse(file, visitors.clone())?;
    }

    for visitor in visitors {
        let mut visitor_cell = (*visitor).borrow_mut();
        reports.extend(visitor_cell.finalize(&config.exclude)?);
    }

    Ok(OutputReport { reports })
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
pub fn lib(config: &config::Config) -> Result<OutputReport, OutputErrors> {
    if config.fix {
        fix(config)
    } else {
        check(config)
    }
}
