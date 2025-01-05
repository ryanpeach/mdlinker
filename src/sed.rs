//! A module for sed like replacements
//! Eventually actually replicating sed became to hard, so now
//! I'm just using tuples of regex patterns

use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ReplacePairCompilationError {
    #[error("The 'from' pattern is not a valid regex")]
    FromError(regex::Error),
    #[error("The 'to' pattern is not valid regex")]
    ToError(regex::Error),
}

/// A struct that holds a pair of regex patterns
#[derive(Clone, Debug)]
pub struct ReplacePair<T, U>
where
    T: ToString + From<String>,
    U: ToString + From<String>,
{
    /// The pattern to search for
    from: Regex,
    /// The pattern to replace with
    /// Can use capture groups from the 'from' pattern
    to: Regex,
    /// The type of string coming in
    _t: std::marker::PhantomData<T>,
    /// The type of string coming out
    _u: std::marker::PhantomData<U>,
}

impl<T, U> From<ReplacePair<T, U>> for (String, String)
where
    T: ToString + From<String>,
    U: ToString + From<String>,
{
    fn from(val: ReplacePair<T, U>) -> Self {
        (val.from.as_str().to_owned(), val.to.as_str().to_owned())
    }
}

impl<T, U> ReplacePair<T, U>
where
    T: ToString + From<String>,
    U: ToString + From<String>,
{
    /// Create a new `ReplacePair` from two regex patterns as strings
    /// Will return errors if the patterns are not valid regex
    pub fn new(from: &str, to: &str) -> Result<Self, ReplacePairCompilationError> {
        // Compile the 'from' pattern into a Regex object
        let from_regex = Regex::new(from).map_err(ReplacePairCompilationError::FromError)?;
        let to_regex = Regex::new(to).map_err(ReplacePairCompilationError::ToError)?;
        // The 'to' pattern is a literal replacement string
        Ok(ReplacePair {
            from: from_regex,
            to: to_regex,
            _t: std::marker::PhantomData,
            _u: std::marker::PhantomData,
        })
    }

    /// Apply replacement to an input string, and return the resultant string
    #[must_use]
    pub fn apply(&self, input: &T) -> U {
        let out = self
            .from
            .replace_all(&input.to_string(), self.to.as_str())
            .to_string();
        out.into()
    }
}
