use std::fmt::{Display, Formatter};

use bon::Builder;
use getset::Getters;
use itertools::Itertools;
use miette::SourceSpan;

use crate::{
    config::Config,
    file::name::Filename,
    sed::{RegexError, ReplacePairError},
};

/// A linkable string, like that in a wikilink, or its corresponding filename
/// Aliases are always lowercase
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Alias(String);

impl Alias {
    #[must_use]
    pub fn new(alias: &str) -> Self {
        Self(alias.to_lowercase())
    }
}

impl Display for Alias {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Alias {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl Alias {
    pub fn from_filename(filename: &Filename, config: &Config) -> Result<Alias, ReplacePairError> {
        match config.filename_to_alias.clone() {
            Ok(pair) => Ok(pair.apply(filename)),
            Err(e) => Err(e),
        }
    }
}

#[derive(Builder, Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct Wikilink {
    alias: Alias,
    span: SourceSpan,
}

impl Wikilink {
    pub(super) fn get_wikilinks(
        contents: &str,
        wikilink_pattern: &str,
    ) -> Result<Vec<Wikilink>, RegexError> {
        let mut wikilinks = Vec::new();
        let wikilink_pattern =
            regex::Regex::new(wikilink_pattern).map_err(RegexError::CompileError)?;
        for mat in wikilink_pattern.captures_iter(contents) {
            let capture0 = mat.get(0).expect("0 always exists");
            let Ok(alias) = mat.iter().skip(1).flatten().exactly_one() else {
                return Err(RegexError::CaptureError {
                    pattern: wikilink_pattern.to_string(),
                    mat: mat.get(0).expect("0 always exists").as_str().to_string(),
                });
            };
            wikilinks.push(
                Wikilink::builder()
                    .span(SourceSpan::new(capture0.start().into(), capture0.len()))
                    .alias(Alias::new(alias.as_str()))
                    .build(),
            );
        }
        Ok(wikilinks)
    }
}
