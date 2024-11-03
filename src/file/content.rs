use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use front_matter::FrontMatterVisitor;
use tree_sitter::Parser;
use wikilink::{Alias, Wikilink, WikilinkVisitor};

use crate::{
    sed::{ReplacePair, ReplacePairCompilationError},
    visitor::{parse, Visitor},
};

use super::{
    name::{get_filename, Filename},
    Error,
};

pub mod front_matter;
pub mod wikilink;

#[derive(Clone)]
pub struct FromFile {
    pub path: PathBuf,
    pub aliases: Vec<Alias>,
    pub wikilinks: Vec<Wikilink>,
}

#[derive(Error, Debug)]
pub enum FromFileError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error("Failed to create alias from filename {path}: {e}")]
    AliasFromFilenameError {
        e: ReplacePairCompilationError,
        path: String,
    },
}
