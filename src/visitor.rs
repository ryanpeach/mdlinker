//! A module for implementing a visitor pattern for ASTs in tree-sitter
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    rc::Rc,
};

use comrak::{arena_tree::Node, nodes::Ast, parse_document, Arena, ExtensionOptions, Options};
use log::{debug, trace};
use std::backtrace;
use thiserror::Error;

use crate::rules::{duplicate_alias::NewDuplicateAliasError, ErrorCode, Report};

#[derive(Error, Debug)]
pub enum VisitError {
    #[error("Error deserializing the node")]
    FrontMatterDeserializeError {
        #[from]
        #[backtrace]
        source: serde_yaml::Error,
    },

    #[error("Error making patterns from aliases")]
    AhoBuildError {
        #[from]
        #[backtrace]
        source: aho_corasick::BuildError,
    },
}

#[derive(Error, Debug)]
pub enum FinalizeError {
    #[error(transparent)]
    NewDuplicateAliasError {
        #[from]
        #[backtrace]
        source: NewDuplicateAliasError,
    },
}

/// A trait for implementing an AST visitor pattern
pub trait Visitor {
    /// The function that is called when visiting a node
    /// WARNING: Don't overwrite this, its already written for you.
    /// Implement [`Self::_visit`] instead
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        trace!(
            "{:?} visiting node type: {:?}",
            self.name(),
            node.data.borrow().value
        );
        #[allow(clippy::used_underscore_items)]
        self._visit(node, source)
    }

    /// Optional function that runs after every file
    /// WARNING: Don't overwrite this, its already written for you.
    /// Implement [`Self::_finalize_file`] instead
    fn finalize_file(&mut self, source: &str, path: &Path) -> Result<(), FinalizeError> {
        trace!("{:?} finalizing file {:?}", self.name(), path.display());
        #[allow(clippy::used_underscore_items)]
        self._finalize_file(source, path)
    }

    /// Optional function for doing something after visiting all nodes
    /// You have to run this yourself in lib, its not done in any of the funtions in this file for you
    /// WARNING: Don't overwrite this, its already written for you.
    /// Implement [`Self::_finalize`] instead
    fn finalize(&mut self, exclude: &[ErrorCode]) -> Result<Vec<Report>, FinalizeError> {
        trace!("{:?} finalizing", self.name());
        #[allow(clippy::used_underscore_items)]
        self._finalize(exclude)
    }

    fn _visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError>;

    fn _finalize_file(&mut self, _source: &str, _path: &Path) -> Result<(), FinalizeError>;

    fn _finalize(&mut self, _exclude: &[ErrorCode]) -> Result<Vec<Report>, FinalizeError>;

    /// Get a unique name for the visitor
    fn name(&self) -> &str;
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Error reading the file {file:?}")]
    IoError {
        file: PathBuf,
        #[backtrace]
        source: std::io::Error,
    },
    #[error("Multibyte characters found in the file {file:?}")]
    MultibyteError {
        file: PathBuf,
        backtrace: backtrace::Backtrace,
    },
    #[error("Error parsing the source code for file {file:?} using tree-sitter")]
    TreeSitter {
        file: PathBuf,
        backtrace: backtrace::Backtrace,
    },
    #[error("Error finalizing the file {file:?}")]
    FinalizeError {
        file: PathBuf,
        #[backtrace]
        source: FinalizeError,
    },
    #[error("Error visiting the file {file:?}")]
    VisitError {
        file: PathBuf,
        #[backtrace]
        source: VisitError,
    },
}

/// Parse the source code and visit all the nodes using tree-sitter
#[allow(clippy::result_large_err)]
pub fn parse(path: &PathBuf, visitors: Vec<Rc<RefCell<dyn Visitor>>>) -> Result<(), ParseError> {
    debug!("Parsing file {:?}", path.display());
    let source = std::fs::read_to_string(path).map_err(|source| ParseError::IoError {
        file: path.clone(),
        source,
    })?;

    // Check for multibyte characters
    if source.chars().count() != source.len() {
        return Err(ParseError::MultibyteError {
            file: path.clone(),
            backtrace: backtrace::Backtrace::force_capture(),
        });
    }

    // Parse the source code
    let arena = Arena::new();
    let options = ExtensionOptions::builder()
        .front_matter_delimiter("---".to_string())
        .wikilinks_title_after_pipe(true)
        .build();
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
        visitor_cell
            .visit(root, &source)
            .map_err(|source| ParseError::VisitError {
                file: path.clone(),
                source,
            })?;
    }

    // Pass the node to all the visitors
    for node in root.descendants() {
        for visitor in visitors.clone() {
            let mut visitor_cell = (*visitor).borrow_mut();
            visitor_cell
                .visit(node, &source)
                .map_err(|source| ParseError::VisitError {
                    file: path.clone(),
                    source,
                })?;
        }
    }

    for visitor in visitors {
        let mut visitor_cell = (*visitor).borrow_mut();
        visitor_cell
            .finalize_file(&source, path)
            .map_err(|source| ParseError::FinalizeError {
                file: path.clone(),
                source,
            })?;
    }

    // The visitors are modified in place, no need to return anything
    Ok(())
}
