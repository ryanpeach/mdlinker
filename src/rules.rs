//! Defines Rules and creates Reports based on those rules
//!
//! # Terminology
//! * A [`Rule`] is a thing like [`crate::rules::similar_filename::SimilarFilename`], which are all public structs which derive
//!   [`thiserror::Error`]
//!   and [`miette::Diagnostic`] inside [`crate::rules`]. It is the "type itself" not a specific instance of that type. In `strum` a similar concept is called a "Discriminant".
//! * A [`Report`] is the result of a rule, like "These two filenames are similar".
//!   Some [`Report`]s are [`Fixable`], meaning they can be auto-handled with the cli argument
//!   `--fix`
//!   Reports all implement [`crate::rules::HasId`].

use std::backtrace::Backtrace;

use crate::config::file::Config as FileConfig;
use derive_more::derive::{Constructor, From, Into};
use glob::Pattern;
use miette::Diagnostic;
use strum_macros::{EnumDiscriminants, EnumIter};
use thiserror::Error;

use crate::config::Config;

#[derive(Debug, EnumDiscriminants, Clone)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(Rule))]
pub enum Report {
    SimilarFilename(similar_filename::SimilarFilename),
    DuplicateAlias(duplicate_alias::DuplicateAlias),
    ThirdPass(ThirdPassReport),
}

#[derive(Debug, EnumDiscriminants, Clone)]
#[strum_discriminants(derive(EnumIter))]
#[strum_discriminants(name(ThirdPassRule))]
pub enum ThirdPassReport {
    BrokenWikilink(crate::rules::broken_wikilink::BrokenWikilink),
    UnlinkedText(crate::rules::unlinked_text::UnlinkedText),
}

/// A Reports error code, usually like `asdf::asdf::asdf`
/// Uniquely identifies a violation of a rule, and can be deduped by Eq
#[derive(Debug, Constructor, PartialEq, Eq, PartialOrd, Ord, Clone, From, Into)]
pub struct ErrorCode(pub String);

#[must_use]
pub fn filter_code<T: ReportTrait>(errors: Vec<T>, code: &ErrorCode) -> Vec<T> {
    errors
        .into_iter()
        .filter(|item| item.id().0.starts_with(&code.0))
        .collect()
}

/// Implemented for all vectors of items that implement [`HasId`]
pub trait VecHasIdExtensions<T>
where
    T: ReportTrait + PartialOrd,
{
    #[must_use]
    fn finalize(self, excludes: &[ErrorCode]) -> Self;
}

fn filter_by_excludes<T: ReportTrait>(mut this: Vec<T>, excludes: &[ErrorCode]) -> Vec<T> {
    this.retain(|item| {
        !excludes.iter().any(|exclude| {
            Pattern::new(&exclude.0.to_lowercase())
                .map(|pattern| pattern.matches(&item.id().0.to_lowercase()))
                .unwrap_or(false)
        })
    });
    this
}

fn dedupe_by_code<T: ReportTrait + PartialOrd>(mut this: Vec<T>) -> Vec<T> {
    // Make sure things with
    // a higher "value" are first before deduping
    this.sort_by(|b, a| a.partial_cmp(b).expect("This never fails"));
    this.dedup_by(|a, b| a.id().0.to_lowercase() == b.id().0.to_lowercase());
    this
}

/// Used for filtering out items that start with the exclude code
impl<T: ReportTrait + PartialOrd> VecHasIdExtensions<T> for Vec<T> {
    #[must_use]
    fn finalize(self, excludes: &[ErrorCode]) -> Self {
        dedupe_by_code(filter_by_excludes(self, excludes))
    }
}

/// Returned by [`ReportTrait::fix`]
#[derive(Error, Debug, Diagnostic)]
pub enum FixError {
    #[error("The git repo is dirty")]
    #[help("Please commit or stash your changes")]
    DirtyRepo {
        #[backtrace]
        backtrace: Backtrace,
    },
    #[error("There was an error checking the git status: {source}")]
    GitError {
        source: git2::Error,
        #[backtrace]
        backtrace: Backtrace,
    },
    #[error("There was an IOError on file {file}: {source}")]
    IOError {
        source: std::io::Error,
        #[backtrace]
        backtrace: Backtrace,
        file: String,
    },
}

pub trait ReportTrait {
    /// All reports should have a code that can be human readable
    /// Codes's should also be useful to deduplicate errors before presenting them to the user
    fn id(&self) -> ErrorCode;

    /// Returns a [`FixError`] if it tried to fix things but failed
    /// Returns [`Some`] if it fixed things
    /// Returns [`None`] if it did not even try to fix things
    fn fix(&self, config: &Config) -> Result<Option<()>, FixError>;

    /// Adds the id to the config file as an ignore
    /// This has a default implementation
    fn ignore(&self, config: &mut FileConfig) {
        config.exclude.push(self.id().0);
    }
}

pub mod broken_wikilink;
pub mod duplicate_alias;
pub mod similar_filename;
pub mod unlinked_text;
