use std::path::PathBuf;

use crate::{rules::ErrorCode, visitor::Visitor};

use super::wikilink::Alias;

#[derive(Debug, Default, Clone)]
pub struct FrontMatterVisitor {
    /// The aliases of the file
    pub aliases: Vec<Alias>,
}

impl FrontMatterVisitor {
    pub const NODE_KIND: &'static str = "alias";
    pub fn new() -> Self {
        Self::default()
    }
}

impl Visitor for FrontMatterVisitor {
    fn visit(&mut self, node: &tree_sitter::Node, source: &str) {
        let node_type = node.kind();
        if node_type == Self::NODE_KIND {
            let tag_text = node.utf8_text(source.as_bytes()).unwrap();
            self.aliases.push(Alias::new(tag_text));
        }
    }
    fn finalize_file(
        &mut self,
        _source: &str,
        _path: &PathBuf,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
    fn finalize(&mut self, _exclude: &Vec<ErrorCode>) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
}
