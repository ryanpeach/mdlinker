use std::path::Path;

use serde::Deserialize;

use crate::{rules::ErrorCode, visitor::Visitor};

use super::wikilink::Alias;

#[derive(Deserialize, Debug)]
pub struct YamlFrontMatter {
    pub alias: String,
}

#[derive(Debug, Default, Clone)]
pub struct FrontMatterVisitor {
    /// The aliases of the file
    pub aliases: Vec<Alias>,
}

impl FrontMatterVisitor {
    pub const NODE_KIND: &'static str = "document";
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Visitor for FrontMatterVisitor {
    fn visit(&mut self, node: &tree_sitter::Node, source: &str) {
        let node_type = node.kind();
        if node_type == Self::NODE_KIND {
            let tag_text = node
                .utf8_text(source.as_bytes())
                .expect("The text must exist... right?"); // TODO: Investigate
            if let Ok(YamlFrontMatter { alias }) = serde_yaml::from_str::<YamlFrontMatter>(tag_text)
            {
                for alias in alias.split(',') {
                    self.aliases.push(Alias::new(alias.trim()));
                }
            }; // TODO: Return miette! error with filename
        }
    }
    fn finalize_file(
        &mut self,
        _source: &str,
        _path: &Path,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
    fn finalize(&mut self, _exclude: &[ErrorCode]) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
}
