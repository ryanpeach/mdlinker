use std::fmt::{Display, Formatter};

use bon::Builder;
use miette::SourceSpan;
use tree_sitter::Node;

use crate::{
    config::Config,
    file::name::Filename,
    sed::{ReplacePair, ReplacePairCompilationError},
    visitor::Visitor,
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
    pub const NODE_KIND: &'static str = "wikilink";
    pub fn new() -> Self {
        Self {
            wikilinks: Vec::new(),
        }
    }
}

impl Visitor for WikilinkVisitor {
    fn visit(&mut self, node: &Node, source: &str) {
        let node_type = node.kind();
        if node_type == Self::NODE_KIND {
            let tag_text = node.utf8_text(source.as_bytes()).unwrap();
            let span = SourceSpan::new(
                node.start_byte().into(),
                node.end_byte() - node.start_byte(),
            );
            self.wikilinks.push(
                Wikilink::builder()
                    .alias(Alias::new(tag_text))
                    .span(span)
                    .build(),
            );
        }
    }
    fn finalize_file(
        &mut self,
        _source: &str,
        _path: &std::path::PathBuf,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(())
    }
    fn finalize(
        &mut self,
        _exclude: &Vec<crate::rules::ErrorCode>,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.wikilinks.clear();
        Ok(())
    }
}
