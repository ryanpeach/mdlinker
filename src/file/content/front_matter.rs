use std::{cell::RefCell, path::Path};

use crate::{
    rules::ErrorCode,
    visitor::{VisitError, Visitor},
};
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue},
};
use serde::Deserialize;

use super::wikilink::Alias;

#[derive(Deserialize, Debug, Default)]
#[serde(default)]
pub struct YamlFrontMatter {
    #[serde(default)]
    pub alias: String,
}

#[derive(Debug, Default, Clone)]
pub struct FrontMatterVisitor {
    /// The aliases of the file
    pub aliases: Vec<Alias>,
}

impl FrontMatterVisitor {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Visitor for FrontMatterVisitor {
    fn name(&self) -> &str {
        "FrontMatterVisitor"
    }
    fn _visit(&mut self, node: &Node<RefCell<Ast>>, _source: &str) -> Result<(), VisitError> {
        if let NodeValue::FrontMatter(text) = &node.data.borrow().value {
            // Strip off first and last line for --- delimeters
            let lines: Vec<&str> = text.trim().lines().collect();
            let trimmed_lines = &lines[1..lines.len() - 1];
            let text = trimmed_lines.join("\n");
            if text.is_empty() {
                return Ok(());
            }
            let YamlFrontMatter { alias } = serde_yaml::from_str::<YamlFrontMatter>(&text)?;
            if alias.is_empty() {
                return Ok(());
            }
            for alias in alias.split(',') {
                self.aliases.push(Alias::new(alias.trim()));
            }
        }
        Ok(())
    }
    fn _finalize_file(
        &mut self,
        _source: &str,
        _path: &Path,
    ) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
    fn _finalize(&mut self, _exclude: &[ErrorCode]) -> Result<(), crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(())
    }
}
