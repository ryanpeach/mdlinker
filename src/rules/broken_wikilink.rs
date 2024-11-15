use std::{
    backtrace::Backtrace,
    cell::RefCell,
    path::{Path, PathBuf},
};

use crate::{
    config::Config,
    file::{
        content::wikilink::{Alias, WikilinkVisitor},
        name::{get_filename, Filename, FilenameLowercase},
    },
    sed::ReplacePair,
    visitor::{FinalizeError, VisitError, Visitor},
};
use bon::Builder;
use comrak::{arena_tree::Node, nodes::Ast};
use hashbrown::HashMap;
use log::trace;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use super::{
    dedupe_by_code, filter_by_excludes, ErrorCode, FixError, HasId, Report, ReportTrait,
    ThirdPassReport,
};

pub const CODE: &str = "content::wikilink::broken";

#[derive(Error, Debug, Diagnostic, Builder, Clone)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code("content::wikilink::broken"))]
pub struct BrokenWikilink {
    /// Used to identify the diagnostic and exclude it if needed
    id: ErrorCode,

    alias: Alias,

    #[source_code]
    src: NamedSource<String>,

    #[label("Wikilink")]
    pub wikilink: SourceSpan,

    #[help]
    advice: String,
}

impl ReportTrait for BrokenWikilink {
    /// Create a new file called the text under the span
    fn fix(&self, config: &Config) -> Result<Option<()>, FixError> {
        trace!(
            "Fixing BrokenWikilink {} in {}",
            self.alias,
            self.src.name()
        );
        let filename = format!("{}.md", FilenameLowercase::from_alias(&self.alias, config));
        let path = config.pages_directory.join(filename);
        std::fs::write(path.clone(), "").map_err(|source| FixError::IOError {
            source,
            backtrace: Backtrace::force_capture(),
            file: path.to_string_lossy().to_string(),
        })?;
        Ok(Some(()))
    }
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
    fn name(&self) -> &'static str {
        "BrokenWikilinkVisitor"
    }
    fn _visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        self.wikilinks_visitor.visit(node, source)?;
        Ok(())
    }
    fn _finalize_file(
        &mut self,
        source: &str,
        path: &Path,
    ) -> std::result::Result<(), FinalizeError> {
        let filename = get_filename(path).lowercase();
        let wikilinks = self.wikilinks_visitor.wikilinks.clone();
        for wikilink in wikilinks {
            let alias = wikilink.alias;
            if !self.alias_table.contains_key(&alias) {
                self.broken_wikilinks.push(
                    BrokenWikilink::builder()
                        .id(format!("{CODE}::{filename}::{alias}").into())
                        .src(NamedSource::new(path.to_string_lossy(), source.to_string()))
                        .wikilink(wikilink.span)
                        .advice(format!(
                            "Create a page or alias on an existing page for '{alias}' (case insensitive), or fix the wikilinks spelling"
                        ))
                        .alias(alias)
                        .build(),
                );
            }
        }

        self.wikilinks_visitor.finalize_file(source, path)?;
        Ok(())
    }

    fn _finalize(&mut self, excludes: &[ErrorCode]) -> Result<Vec<Report>, FinalizeError> {
        // We can "take" this because we are putting it right back
        self.broken_wikilinks = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.broken_wikilinks),
            excludes,
        ));
        self.wikilinks_visitor.finalize(excludes)?;
        Ok(self
            .broken_wikilinks
            .iter()
            .map(|x| Report::ThirdPass(ThirdPassReport::BrokenWikilink(x.clone())))
            .collect())
    }
}
