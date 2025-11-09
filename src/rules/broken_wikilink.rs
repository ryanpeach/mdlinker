use std::{
    backtrace::Backtrace,
    cell::RefCell,
    path::{self, Path, PathBuf},
};

use crate::{
    config::Config,
    file::{
        content::wikilink::{Alias, WikilinkVisitor},
        name::{get_filename, FilenameLowercase},
    },
    visitor::{FinalizeError, VisitError, Visitor},
};
use bon::{bon, Builder};
use comrak::{arena_tree::Node, nodes::Ast};
use getset::Getters;
use hashbrown::HashMap;
use log::{trace, warn};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use std::rc::Rc;
use thiserror::Error;

use super::{
    dedupe_by_code, filter_by_excludes, ErrorCode, FixError, Report, ReportTrait, ThirdPassReport,
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
    fn id(&self) -> ErrorCode {
        self.id.clone()
    }
    /// Create a new file called the text under the span
    fn fix(&self, config: &Config) -> Result<Option<()>, FixError> {
        trace!(
            "Fixing BrokenWikilink {} in {}",
            self.alias,
            self.src.name()
        );
        let filename = format!("{}.md", FilenameLowercase::from_alias(&self.alias, config));
        let path = config.new_files_directory.join(filename);
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

#[derive(Debug, Getters)]
pub struct BrokenWikilinkVisitor {
    alias_table: Rc<HashMap<Alias, PathBuf>>,
    additional_aliases_table: HashMap<Alias, PathBuf>,
    wikilinks_visitor: WikilinkVisitor,
    broken_wikilinks: Vec<BrokenWikilink>,
}

#[bon]
impl BrokenWikilinkVisitor {
    #[builder]
    pub fn new(all_files: &[PathBuf], alias_table: Rc<HashMap<Alias, PathBuf>>) -> Self {
        // In this case we need to add all the filepaths in all their forms to the alias as well
        // to handle wikilinks which are more like filenames and filepaths
        let mut additional_aliases_table: HashMap<Alias, PathBuf> = HashMap::new();
        for filepath in all_files {
            let filepath_as_list: Vec<String> = filepath
                .iter()
                .map(|x| x.to_string_lossy().to_string())
                .collect();
            for i in 1..=filepath_as_list.len() {
                let with_extension = filepath_as_list[filepath_as_list.len() - i..]
                    .join(path::MAIN_SEPARATOR.to_string().as_str());
                let without_extension = with_extension
                    .split('.')
                    .next()
                    .unwrap_or(&with_extension)
                    .to_string();
                additional_aliases_table
                    .entry(Alias::new(&with_extension))
                    .or_insert_with(|| filepath.clone());
                additional_aliases_table
                    .entry(Alias::new(&without_extension))
                    .or_insert_with(|| filepath.clone());
            }
        }
        Self {
            alias_table,
            additional_aliases_table,
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
            let id = format!("{CODE}::{filename}::{alias}");
            if !self.alias_table.contains_key(&alias)
                && !self.additional_aliases_table.contains_key(&alias)
            {
                if alias.as_str().starts_with('.') || alias.as_str().contains("..") {
                    warn!(
                        "Skipping broken wikilink '{}' in {} because it looks like a relative path",
                        alias,
                        path.to_string_lossy()
                    );
                    continue;
                }
                self.broken_wikilinks.push(
                    BrokenWikilink::builder()
                        .advice(format!(
                            "Create a page or alias on an existing page for '{alias}' (case insensitive), or fix the wikilinks spelling.\nid: {id:?}"
                        ))
                        .id(id.into())
                        .src(NamedSource::new(path.to_string_lossy(), source.to_string()))
                        .wikilink(wikilink.span)
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
