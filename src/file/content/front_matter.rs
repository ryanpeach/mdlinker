use std::{cell::RefCell, path::Path};

use crate::{
    rules::{ErrorCode, Report},
    visitor::{VisitError, Visitor},
};
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue},
};
use log::debug;
use miette::SourceSpan;
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

fn get_frontmatter_from_any_node(node: &Node<RefCell<Ast>>) -> Option<String> {
    let mut node = node; // Its not the node which is now mutable, but you can change this value
                         // First check ourselves
    if let NodeValue::FrontMatter(text) = &node.data.borrow().value {
        return Some(text.clone());
    }

    // Then go to the root "Document" node
    while let Some(this_node) = node.parent() {
        node = this_node;
        if let NodeValue::Document = node.data.borrow().value {
            break;
        }
    }

    // Down one is elther NodeValue::FrontMatter or Something else
    // If its frontmatter, return it
    if let Some(child) = &node.first_child() {
        if let NodeValue::FrontMatter(text) = &child.data.borrow().value {
            return Some(text.clone());
        }
    }
    None
}

/// Spans get messed up because frontmatter is not considered part of the nodes sourcespan
/// This adds the frontmatter length to the span offset
pub fn repair_span_due_to_frontmatter(span: SourceSpan, node: &Node<RefCell<Ast>>) -> SourceSpan {
    let frontmatter = get_frontmatter_from_any_node(node);
    if let Some(frontmatter) = frontmatter {
        debug!("Frontmatter: {:?}", frontmatter);
        SourceSpan::new((span.offset() + frontmatter.len()).into(), span.len())
    } else {
        span
    }
}

/// Remove the frontmatter from the source
pub fn remove_frontmatter_from_source(source: &str, node: &Node<RefCell<Ast>>) -> String {
    let frontmatter = get_frontmatter_from_any_node(node);
    if let Some(frontmatter) = frontmatter {
        source[frontmatter.len()..].to_string()
    } else {
        source.to_string()
    }
}

impl Visitor for FrontMatterVisitor {
    fn name(&self) -> &'static str {
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
    fn _finalize(
        &mut self,
        _exclude: &[ErrorCode],
    ) -> Result<Vec<Report>, crate::visitor::FinalizeError> {
        self.aliases.clear();
        Ok(vec![])
    }
}
