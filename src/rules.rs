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
pub struct ErrorCode(String);

/// All reports should have a code that can be human readable
/// Codes's should also be useful to deduplicate errors before presenting them to the user
pub trait HasId {
    fn id(&self) -> ErrorCode;
}

#[must_use]
pub fn filter_code<T: HasId>(errors: Vec<T>, code: &ErrorCode) -> Vec<T> {
    errors
        .into_iter()
        .filter(|item| &item.id() == code)
        .collect()
}

/// Implemented for all vectors of items that implement [`HasId`]
pub trait VecHasIdExtensions<T>
where
    T: HasId + PartialOrd,
{
    #[must_use]
    fn finalize(self, excludes: &[ErrorCode]) -> Self;
}

fn filter_by_excludes<T: HasId>(mut this: Vec<T>, excludes: &[ErrorCode]) -> Vec<T> {
    this.retain(|item| {
        !excludes.iter().any(|exclude| {
            Pattern::new(&exclude.0.to_lowercase())
                .map(|pattern| pattern.matches(&item.id().0.to_lowercase()))
                .unwrap_or(false)
        })
    });
    this
}

fn dedupe_by_code<T: HasId + PartialOrd>(mut this: Vec<T>) -> Vec<T> {
    // Make sure things with
    // a higher "value" are first before deduping
    this.sort_by(|b, a| a.partial_cmp(b).expect("This never fails"));
    this.dedup_by(|a, b| a.id().0.to_lowercase() == b.id().0.to_lowercase());
    this
}

/// Used for filtering out items that start with the exclude code
impl<T: HasId + PartialOrd> VecHasIdExtensions<T> for Vec<T> {
    #[must_use]
    fn finalize(self, excludes: &[ErrorCode]) -> Self {
        dedupe_by_code(filter_by_excludes(self, excludes))
    }
}

#[derive(Error, Debug, Diagnostic)]
pub enum FixError {
    #[error("The git repo is dirty")]
    #[help("Please commit or stash your changes")]
    DirtyRepo,
    #[error(transparent)]
    GitError(#[from] git2::Error),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}

pub trait ReportTrait {
    /// Returns a [`FixError`] if it tried to fix things but failed
    /// Returns [`Some`] if it fixed things
    /// Returns [`None`] if it did not even try to fix things
    fn fix(&self, config: &Config) -> Result<Option<()>, FixError>;
}

pub mod broken_wikilink;
pub mod duplicate_alias;
pub mod similar_filename;
pub mod unlinked_text;
