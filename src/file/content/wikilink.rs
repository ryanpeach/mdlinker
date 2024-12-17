use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
};

use crate::{
    file::name::Filename,
    rules::Report,
    sed::ReplacePair,
    visitor::{VisitError, Visitor},
};
use bon::Builder;
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue, NodeWikiLink},
};
use miette::{SourceOffset, SourceSpan};
use regex::Regex;

/// A linkable string, like that in a wikilink, or its corresponding filename
/// Aliases are always lowercase
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Alias(String);

impl Alias {
    #[must_use]
    pub fn new(alias: &str) -> Self {
        Self(alias.to_lowercase())
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    #[must_use]
    pub fn char_len(&self) -> usize {
        self.0.chars().count()
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
    #[must_use]
    pub fn from_filename(
        filename: &Filename,
        filename_to_alias: &ReplacePair<Filename, Alias>,
    ) -> Alias {
        filename_to_alias.apply(filename)
    }
}

#[derive(Builder, Clone, Debug)]
pub struct Wikilink {
    pub alias: Alias,
    pub span: SourceSpan,
}

#[derive(Debug, Clone)]
pub struct WikilinkVisitor {
    pub wikilinks: Vec<Wikilink>,
    tag_pattern: Regex,
}

impl Default for WikilinkVisitor {
    fn default() -> Self {
        Self {
            wikilinks: Vec::new(),
            tag_pattern: Regex::new(r"#([A-Za-z0-9_/-]+)").expect("Constant"),
        }
    }
}

impl WikilinkVisitor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}
impl Visitor for WikilinkVisitor {
    fn name(&self) -> &'static str {
        "WikilinkVisitor"
    }
    fn _visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        let data_ref = node.data.borrow();
        let data = &data_ref.value;
        let sourcepos = data_ref.sourcepos;
        let mut get_tags = |text: &str| {
            for captures in self.tag_pattern.captures_iter(text) {
                let alias = Alias::new(
                    captures
                        .get(1)
                        .expect("Otherwise the regex wouldn't match")
                        .as_str(),
                );
                let capture_start_byte = captures
                    .get(1)
                    .expect("The regex has 2 capture groups")
                    .start();
                let sourcepos_start_offset_bytes =
                    SourceOffset::from_location(text, sourcepos.start.line, sourcepos.start.column)
                        .offset();
                let span = SourceSpan::new(
                    (sourcepos_start_offset_bytes + capture_start_byte).into(),
                    alias.char_len(),
                );
                self.wikilinks
                    .push(Wikilink::builder().alias(alias.clone()).span(span).build());
            }
        };
        match data {
            NodeValue::Text(text) => {
                get_tags(text);
            }
            NodeValue::WikiLink(NodeWikiLink { url }) => {
                self.wikilinks.push(
                    Wikilink::builder()
                        .alias(Alias::new(url))
                        .span(SourceSpan::new(
                            SourceOffset::from_location(
                                source,
                                sourcepos.start.line,
                                sourcepos.start.column,
                            ),
                            url.len() + 4,
                        ))
                        .build(),
                );
            }
            x => {
                if let Some(text) = x.text() {
                    get_tags(text);
                }
            }
        }
        Ok(())
    }
    fn _finalize_file(
        &mut self,
        _source: &str,
        _path: &std::path::Path,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(())
    }
    fn _finalize(
        &mut self,
        _exclude: &[crate::rules::ErrorCode],
    ) -> Result<Vec<Report>, crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(vec![])
    }
}
