//! Defines Rules and creates Reports based on those rules
//!
//! # Terminology
//! * A Rule is a thing like [`crate::rules::similar_filename::SimilarFilename`], which are all public structs which derive
//!   [`thiserror::Error`]
//!   and [`miette::Diagnostic`] inside [`crate::rules`]
//! * A Report is the result of a rule, like "These two filenames are similar".
//!   Reports all implement [`crate::rules::HasId`].

use derive_more::derive::{Constructor, From, Into};
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
    fn finalize(self, excludes: &Vec<ErrorCode>) -> Self;
}

fn filter_by_excludes<T: HasId>(mut this: Vec<T>, excludes: &Vec<ErrorCode>) -> Vec<T> {
    this.retain(|item| {
        !excludes.iter().any(|exclude| {
            item.id()
                .0
                .to_lowercase()
                .starts_with(&exclude.0.to_lowercase())
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
    fn finalize(self, excludes: &Vec<ErrorCode>) -> Self {
        dedupe_by_code(filter_by_excludes(self, excludes))
    }
}

pub mod broken_wikilink;
pub mod duplicate_alias;
pub mod similar_filename;
