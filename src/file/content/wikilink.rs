use std::{
    cell::RefCell,
    fmt::{Display, Formatter},
};

use bon::Builder;
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue, NodeWikiLink},
};
use miette::{SourceOffset, SourceSpan};

use crate::{
    file::name::Filename,
    sed::ReplacePair,
    visitor::{VisitError, Visitor},
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

#[derive(Debug, Default, Clone)]
pub struct WikilinkVisitor {
    pub wikilinks: Vec<Wikilink>,
}

impl WikilinkVisitor {
    pub const NODE_KINDS: &'static [&'static str] = &["wiki_link", "tag"];
    #[must_use]
    pub fn new() -> Self {
        Self {
            wikilinks: Vec::new(),
        }
    }
}

impl Visitor for WikilinkVisitor {
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        let data = node.data.borrow();
        if let NodeValue::WikiLink(NodeWikiLink { url }) = &data.value {
            self.wikilinks.push(
                Wikilink::builder()
                    .alias(Alias::new(url))
                    .span(SourceSpan::new(
                        SourceOffset::from_location(
                            source,
                            data.sourcepos.start.line,
                            data.sourcepos.start.column,
                        ),
                        data.sourcepos.end.column - data.sourcepos.start.column,
                    ))
                    .build(),
            );
        }
        Ok(())
    }
    fn finalize_file(
        &mut self,
        _source: &str,
        _path: &std::path::Path,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(())
    }
    fn finalize(
        &mut self,
        _exclude: &[crate::rules::ErrorCode],
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(())
    }
}
