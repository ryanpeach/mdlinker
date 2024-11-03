//! A module for implementing a visitor pattern for ASTs in tree-sitter
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use thiserror::Error;
use tree_sitter::Node;
use tree_sitter_md::{MarkdownCursor, MarkdownParser};

use crate::rules::{duplicate_alias::NewDuplicateAliasError, ErrorCode};

#[derive(Error, Debug)]
pub enum FinalizeError {
    #[error(transparent)]
    NewDuplicateAliasError(#[from] NewDuplicateAliasError),
}

/// A trait for implementing an AST visitor pattern
pub trait Visitor {
    fn visit(&mut self, node: &Node, source: &str);

    /// Optional function that runs after every file
    fn finalize_file(&mut self, _source: &str, _path: &Path) -> Result<(), FinalizeError>;

    /// Optional function for doing something after visiting all nodes
    /// You have to run this yourself in lib, its not done in any of the funtions in this file for you
    fn finalize(&mut self, _exclude: &[ErrorCode]) -> Result<(), FinalizeError>;
}

/// Recursive function for visiting nodes
fn visit_node(cursor: &mut MarkdownCursor, source: &str, visitors: &Vec<Rc<RefCell<dyn Visitor>>>) {
    // Traverse the tree
    loop {
        let node = cursor.node();

        // Pass the node to all the visitors
        for visitor in visitors.clone() {
            let mut visitor_cell = (*visitor).borrow_mut();
            visitor_cell.visit(&node, source);
        }

        // If the current node has children, visit them
        if cursor.goto_first_child() {
            visit_node(cursor, source, visitors);
            cursor.goto_parent();
        }

        // Move to the next sibling node
        if !cursor.goto_next_sibling() {
            break;
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error parsing the source code using tree-sitter")]
    TreeSitter,
    #[error(transparent)]
    FinalizeError(#[from] FinalizeError),
}

/// Parse the source code and visit all the nodes using tree-sitter
pub fn parse(path: &PathBuf, visitors: Vec<Rc<RefCell<dyn Visitor>>>) -> Result<(), ParseError> {
    let source = std::fs::read_to_string(path)?;

    // Parse the source code
    let tree = MarkdownParser::default()
        .parse(&source.clone().into_bytes(), None)
        .ok_or(ParseError::TreeSitter)?;

    // Create a tree cursor for traversal
    let mut cursor = tree.walk();

    // Visit all the nodes starting from the root
    visit_node(&mut cursor, &source, &visitors.clone());

    for visitor in visitors {
        let mut visitor_cell = (*visitor).borrow_mut();
        visitor_cell.finalize_file(&source, path)?;
    }

    // The visitors are modified in place, no need to return anything
    Ok(())
}
