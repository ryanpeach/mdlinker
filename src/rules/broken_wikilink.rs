use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use crate::{
    file::{
        content::wikilink::{Alias, WikilinkVisitor},
        name::{get_filename, Filename},
    },
    sed::ReplacePair,
    visitor::{FinalizeError, VisitError, Visitor},
};
use bon::Builder;
use comrak::{arena_tree::Node, nodes::Ast};
use hashbrown::HashMap;
use log::debug;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use super::{dedupe_by_code, filter_by_excludes, ErrorCode, HasId};

pub const CODE: &str = "content::wikilink::broken";

#[derive(Error, Debug, Diagnostic, Builder)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code("content::wikilink::broken"))]
pub struct BrokenWikilink {
    /// Used to identify the diagnostic and exclude it if needed
    id: ErrorCode,

    #[source_code]
    src: NamedSource<String>,

    #[label("Wikilink")]
    wikilink: SourceSpan,

    #[help]
    advice: String,
}

impl PartialEq for BrokenWikilink {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for BrokenWikilink {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl HasId for BrokenWikilink {
    fn id(&self) -> ErrorCode {
        self.id.clone()
    }
}

#[derive(Debug)]
pub struct BrokenWikilinkVisitor {
    pub alias_table: HashMap<Alias, PathBuf>,
    pub wikilinks_visitor: WikilinkVisitor,
    pub broken_wikilinks: Vec<BrokenWikilink>,
}

impl BrokenWikilinkVisitor {
    #[must_use]
    pub fn new(
        _all_files: &[PathBuf],
        _filename_to_alias: &ReplacePair<Filename, Alias>,
        alias_table: HashMap<Alias, PathBuf>,
    ) -> Self {
        Self {
            alias_table,
            wikilinks_visitor: WikilinkVisitor::new(),
            broken_wikilinks: Vec::new(),
        }
    }
}

impl Visitor for BrokenWikilinkVisitor {
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        self.wikilinks_visitor.visit(node, source)?;
        Ok(())
    }
    fn finalize_file(
        &mut self,
        source: &str,
        path: &Path,
    ) -> std::result::Result<(), FinalizeError> {
        let filename = get_filename(path).lowercase();
        let wikilinks = self.wikilinks_visitor.wikilinks.clone();
        for wikilink in wikilinks {
            let alias = wikilink.alias;
            if !self.alias_table.contains_key(&alias) {
                debug!("Broken wikilink: {}", alias);
                self.broken_wikilinks.push(
                    BrokenWikilink::builder()
                        .id(format!("{CODE}::{filename}::{alias}").into())
                        .src(NamedSource::new(path.to_string_lossy(), source.to_string()))
                        .wikilink(wikilink.span)
                        .advice(format!(
                            "Create a page or alias for '{alias}' (case insensitive)"
                        ))
                        .build(),
                );
            }
        }

        self.wikilinks_visitor.finalize_file(source, path)?;
        Ok(())
    }

    fn finalize(&mut self, excludes: &[ErrorCode]) -> Result<(), FinalizeError> {
        // We can "take" this because we are putting it right back
        self.broken_wikilinks = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.broken_wikilinks),
            excludes,
        ));
        self.wikilinks_visitor.finalize(excludes)?;
        Ok(())
    }
}
