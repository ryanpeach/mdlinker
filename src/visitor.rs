//! A module for implementing a visitor pattern for ASTs in tree-sitter
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use comrak::{
    arena_tree::Node, nodes::Ast, parse_document, Arena, ExtensionOptionsBuilder, Options,
};
use log::debug;
use thiserror::Error;

use crate::rules::{duplicate_alias::NewDuplicateAliasError, ErrorCode};

#[derive(Error, Debug)]
pub enum VisitError {
    #[error("Error deserializing the node")]
    FrontMatterDeserializeError {
        #[from]
        source: serde_yaml::Error,
        #[backtrace]
        backtrace: std::backtrace::Backtrace,
    },
}

#[derive(Error, Debug)]
pub enum FinalizeError {
    #[error(transparent)]
    NewDuplicateAliasError(#[from] NewDuplicateAliasError),
}

/// A trait for implementing an AST visitor pattern
pub trait Visitor {
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError>;

    /// Optional function that runs after every file
    fn finalize_file(&mut self, _source: &str, _path: &Path) -> Result<(), FinalizeError>;

    /// Optional function for doing something after visiting all nodes
    /// You have to run this yourself in lib, its not done in any of the funtions in this file for you
    fn finalize(&mut self, _exclude: &[ErrorCode]) -> Result<(), FinalizeError>;
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Error parsing the source code using tree-sitter")]
    TreeSitter,
    #[error(transparent)]
    FinalizeError(#[from] FinalizeError),
    #[error(transparent)]
    VisitError(#[from] VisitError),
}

/// Parse the source code and visit all the nodes using tree-sitter
pub fn parse(path: &PathBuf, visitors: Vec<Rc<RefCell<dyn Visitor>>>) -> Result<(), ParseError> {
    let source = std::fs::read_to_string(path)?;

    // Parse the source code
    let arena = Arena::new();
    let options = ExtensionOptionsBuilder::default()
        .front_matter_delimiter(Some("---".to_string()))
        .wikilinks_title_before_pipe(true)
        .build()
        .expect("Constant");
    let root = parse_document(
        &arena,
        &source,
        &Options {
            extension: options,
            ..Default::default()
        },
    );

    // Visit the root
    for visitor in visitors.clone() {
        let mut visitor_cell = (*visitor).borrow_mut();
        visitor_cell.visit(root, &source)?;
    }

    // Pass the node to all the visitors
    for node in root.descendants() {
        debug!("{:?}", node);
        for visitor in visitors.clone() {
            let mut visitor_cell = (*visitor).borrow_mut();
            visitor_cell.visit(node, &source)?;
        }
    }

    for visitor in visitors {
        let mut visitor_cell = (*visitor).borrow_mut();
        visitor_cell.finalize_file(&source, path)?;
    }

    // The visitors are modified in place, no need to return anything
    Ok(())
}
