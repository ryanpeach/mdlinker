use std::path::PathBuf;

use bon::Builder;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;
use tree_sitter::Node;

use crate::{
    file::{
        content::wikilink::{Alias, WikilinkVisitor},
        name::{get_filename, Filename},
    },
    sed::ReplacePair,
    visitor::{FinalizeError, Visitor},
};

use super::{
    dedupe_by_code, duplicate_alias::DuplicateAliasVisitor, filter_by_excludes, ErrorCode, HasId,
};

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
    pub duplicate_alias_visitor: DuplicateAliasVisitor,
    pub wikilinks_visitor: WikilinkVisitor,
    pub broken_wikilinks: Vec<BrokenWikilink>,
}

impl BrokenWikilinkVisitor {
    #[must_use]
    pub fn new(all_files: &Vec<PathBuf>, filename_to_alias: &ReplacePair<Filename, Alias>) -> Self {
        Self {
            duplicate_alias_visitor: DuplicateAliasVisitor::new(all_files, filename_to_alias),
            wikilinks_visitor: WikilinkVisitor::new(),
            broken_wikilinks: Vec::new(),
        }
    }
}

impl Visitor for BrokenWikilinkVisitor {
    fn visit(&mut self, node: &Node, source: &str) {
        match node.kind() {
            DuplicateAliasVisitor::NODE_KIND => self.duplicate_alias_visitor.visit(node, source),
            WikilinkVisitor::NODE_KIND => self.wikilinks_visitor.visit(node, source),
            _ => {}
        }
    }
    fn finalize_file(
        &mut self,
        source: &str,
        path: &PathBuf,
    ) -> std::result::Result<(), FinalizeError> {
        self.duplicate_alias_visitor.finalize_file(source, path)?;
        let lookup_table = &self.duplicate_alias_visitor.alias_table;
        let filename = get_filename(path.as_path()).lowercase();
        let wikilinks = self.wikilinks_visitor.wikilinks.clone();
        for wikilink in wikilinks {
            let alias = wikilink.alias;
            if !lookup_table.contains_key(&alias) {
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

    fn finalize(&mut self, excludes: &Vec<ErrorCode>) -> Result<(), FinalizeError> {
        // We can "take" this because we are putting it right back
        self.broken_wikilinks = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.broken_wikilinks),
            excludes,
        ));
        self.wikilinks_visitor.finalize(excludes)?;
        self.duplicate_alias_visitor.finalize(excludes)?;
        Ok(())
    }
}

// impl BrokenWikilink {
//     pub fn calculate(
//         files: &[PathBuf],
//         config: &Config,
//     ) -> Result<Vec<BrokenWikilink>, CalculateError> {
//     }
// }
