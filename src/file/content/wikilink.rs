use bon::Builder;
use getset::Getters;
use itertools::Itertools;
use miette::SourceSpan;

use crate::sed::RegexError;

#[derive(Builder, Getters, Clone, Debug)]
#[getset(get = "pub")]
pub struct Wikilink {
    alias: String,
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
                    .alias(alias.as_str().to_owned().to_lowercase())
                    .build(),
            );
        }
        Ok(wikilinks)
    }
}
