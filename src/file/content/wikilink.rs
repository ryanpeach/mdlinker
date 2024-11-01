use bon::Builder;
use getset::Getters;
use miette::SourceSpan;
use regex::Regex;

#[derive(Builder, Getters, Clone)]
#[getset(get = "pub")]
pub struct Wikilink {
    alias: String,
    span: SourceSpan,
}

impl Wikilink {
    pub(super) fn get_wikilinks(contents: &str, wikilink_pattern: &Regex) -> Vec<Wikilink> {
        let mut wikilinks = Vec::new();

        for mat in wikilink_pattern.find_iter(contents) {
            wikilinks.push(
                Wikilink::builder()
                    .span(SourceSpan::new(mat.start().into(), mat.len()))
                    .alias(mat.as_str().to_owned())
                    .build(),
            );
        }
        wikilinks
    }
}
