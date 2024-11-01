//! Defines Rules and creates Reports based on those rules
//!
//! # Terminology
//! * A Rule is a thing like [`crate::rules::similar_filename::SimilarFilename`], which are all public structs which derive
//!   [`thiserror::Error`]
//!   and [`miette::Diagnostic`] inside [`crate::rules`]
//! * A Report is the result of a rule, like "These two filenames are similar".
//!   Reports all implement [`crate::rules::HasId`].

/// All reports should have a code that can be human readable
/// Codes's should also be useful to deduplicate errors before presenting them to the user
pub trait HasId {
    fn id(&self) -> String;
}

/// Implemented for all vectors of items that implement [`HasId`]
pub trait VecHasIdExtensions<T> {
    #[must_use]
    fn filter_by_excludes(self, excludes: Vec<String>) -> Self;
    #[must_use]
    fn dedupe_by_code(self) -> Self;
    #[must_use]
    fn contains_code(&self, code: &str) -> Vec<&T>;
}

/// Used for filtering out items that start with the exclude code
impl<T: HasId> VecHasIdExtensions<T> for Vec<T> {
    fn filter_by_excludes(mut self, excludes: Vec<String>) -> Self {
        self.retain(|item| {
            !excludes.iter().any(|exclude| {
                item.id()
                    .to_lowercase()
                    .starts_with(&exclude.to_lowercase())
            })
        });
        self
    }

    fn dedupe_by_code(mut self) -> Self {
        self.dedup_by(|a, b| a.id().to_lowercase() == b.id().to_lowercase());
        self
    }

    fn contains_code(&self, code: &str) -> Vec<&T> {
        self.iter().filter(|item| item.id() == code).collect()
    }
}

pub mod broken_wikilink;
pub mod duplicate_alias;
pub mod similar_filename;
