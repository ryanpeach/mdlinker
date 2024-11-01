//! A module for sed like replacements
//! Eventually actually replicating sed became to hard, so now
//! I'm just using tuples of regex patterns

use std::path::PathBuf;

use bon::Builder;
use getset::Getters;
use regex::Regex;
use thiserror::Error;

#[derive(thiserror::Error, Debug, Builder, Getters)]
#[error("{path} does not contain the ngram {ngram}")]
#[getset(get = "pub")]
pub struct MissingSubstringError {
    path: PathBuf,
    ngram: String,
}

#[derive(Error, Debug)]
pub enum ReplacePairError {
    #[error("The 'from' pattern is not a valid regex")]
    FromError(regex::Error),
    #[error("The 'to' pattern is not valid regex")]
    ToError(regex::Error),
}

/// A struct that holds a pair of regex patterns
pub struct ReplacePair {
    /// The pattern to search for
    from: Regex,
    /// The pattern to replace with
    /// Can use capture groups from the 'from' pattern
    to: Regex,
}

impl ReplacePair {
    /// Create a new `ReplacePair` from two regex patterns as strings
    /// Will return errors if the patterns are not valid regex
    pub fn new(from: &str, to: &str) -> Result<Self, ReplacePairError> {
        // Compile the 'from' pattern into a Regex object
        let from_regex = Regex::new(from).map_err(ReplacePairError::FromError)?;
        let to_regex = Regex::new(to).map_err(ReplacePairError::ToError)?;
        // The 'to' pattern is a literal replacement string
        Ok(ReplacePair {
            from: from_regex,
            to: to_regex,
        })
    }

    /// Apply replacement to an input string, and return the resultant string
    #[must_use]
    pub fn apply(&self, input: &str) -> String {
        self.from.replace_all(input, self.to.as_str()).to_string()
    }
}
