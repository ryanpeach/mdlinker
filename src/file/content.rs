use std::path::PathBuf;

use wikilink::{Alias, Wikilink};

use crate::sed::ReplacePairCompilationError;

use super::Error;

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
