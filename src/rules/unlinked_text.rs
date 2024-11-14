use crate::{
    config::Config,
    file::{
        content::{
            front_matter::{remove_frontmatter_from_source, repair_span_due_to_frontmatter},
            wikilink::{Alias, WikilinkVisitor},
        },
        name::{get_filename, Filename},
    },
    sed::ReplacePair,
    visitor::{FinalizeError, VisitError, Visitor},
};
use bon::Builder;
use cached::proc_macro::cached;
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue},
};
use fancy_regex::Regex;
use hashbrown::HashMap;
use miette::{Diagnostic, NamedSource, Result, SourceOffset, SourceSpan};
use std::{
    backtrace::Backtrace,
    cell::RefCell,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::{
    dedupe_by_code, filter_by_excludes, ErrorCode, FixError, HasId, Report, ReportTrait,
    ThirdPassReport,
};

pub const CODE: &str = "content::alias::unlinked";

#[derive(Error, Debug, Diagnostic, Builder, Clone)]
#[error("Found text which could probably be put in a wikilink")]
#[diagnostic(code("content::alias::unlinked"))]
pub struct UnlinkedText {
    /// Used to identify the diagnostic and exclude it if needed
    id: ErrorCode,

    alias: Alias,

    #[source_code]
    src: NamedSource<String>,

    #[label("Alias")]
    pub span: SourceSpan,

    #[help]
    advice: String,
}

impl ReportTrait for UnlinkedText {
    /// Open the file, surround the span in [[ ]], then save it
    fn fix(&self, _config: &Config) -> Result<Option<()>, FixError> {
        let mut source = self.src.inner().clone();
        let start = self.span.offset();
        let end = start + self.span.len();
        source.insert_str(end, "]]");
        source.insert_str(start, "[[");
        std::fs::write(self.src.name(), source).map_err(|source| FixError::IOError {
            source,
            file: self.src.name().to_owned(),
            backtrace: Backtrace::force_capture(),
        })?;
        Ok(Some(()))
    }
}

impl PartialEq for UnlinkedText {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl PartialOrd for UnlinkedText {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl HasId for UnlinkedText {
    fn id(&self) -> ErrorCode {
        self.id.clone()
    }
}

#[derive(Debug)]
pub struct UnlinkedTextVisitor {
    pub alias_table: HashMap<Alias, PathBuf>,
    new_unlinked_texts: Vec<(Alias, SourceSpan)>,
    wikilink_visitor: WikilinkVisitor,
    pub unlinked_texts: Vec<UnlinkedText>,
}

impl UnlinkedTextVisitor {
    #[must_use]
    pub fn new(
        _all_files: &[PathBuf],
        _filename_to_alias: &ReplacePair<Filename, Alias>,
        alias_table: HashMap<Alias, PathBuf>,
    ) -> Self {
        Self {
            alias_table,
            wikilink_visitor: WikilinkVisitor::new(),
            unlinked_texts: Vec::new(),
            new_unlinked_texts: Vec::new(),
        }
    }
}

#[cached]
fn get_regex(alias: Alias) -> Regex {
    // Compile the regex and cache it based on the alias
    let pattern = format!(r"(?i)(?<![\w#]){alias}(?!\w)");
    Regex::new(&pattern).expect("The regex is just case insensitive string search")
}

impl Visitor for UnlinkedTextVisitor {
    fn name(&self) -> &'static str {
        "UnlinkedTextVisitor"
    }
    fn _visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        self.wikilink_visitor.visit(node, source)?;
        let data_ref = node.data.borrow();
        let data = &data_ref.value;
        let sourcepos = data_ref.sourcepos;
        let parent = node.parent();
        if let NodeValue::Text(text) = data {
            for alias in self.alias_table.keys() {
                // Make sure neither the character before or after is a letter
                // This makes sure you aren't matching a part of a word
                // This should also handle tags
                // Check the character before the match

                let re = get_regex(alias.clone());
                if let Ok(Some(found)) = re.find(text) {
                    // Get our span
                    let span = repair_span_due_to_frontmatter(
                        SourceSpan::new(
                            (SourceOffset::from_location(
                                remove_frontmatter_from_source(source, node),
                                sourcepos.start.line,
                                sourcepos.start.column,
                            )
                            .offset()
                                + found.start())
                            .into(),
                            alias.to_string().len(),
                        ),
                        node,
                    );

                    // Dont match inside wikilinks
                    if let Some(parent) = parent {
                        if let NodeValue::WikiLink(_) = parent.data.borrow().value {
                            // If this is already in a link, skip it
                            continue;
                        }
                    }

                    self.new_unlinked_texts.push((alias.clone(), span));
                }
            }
        }
        Ok(())
    }
    fn _finalize_file(
        &mut self,
        source: &str,
        path: &Path,
    ) -> std::result::Result<(), FinalizeError> {
        for (alias, span) in &mut self.new_unlinked_texts {
            let filename = get_filename(path);
            self.unlinked_texts.push(
                UnlinkedText::builder()
                    .id(ErrorCode::new(format!("{CODE}::{filename}::{alias}")))
                    .src(NamedSource::new(path.to_string_lossy(), source.to_string()))
                    .alias(alias.clone())
                    .span(*span)
                    .advice(format!(
                        "Consider wrapping it in a wikilink, like: [[{alias}]]"
                    ))
                    .build(),
            );
        }
        self.new_unlinked_texts.clear();
        self.wikilink_visitor.finalize_file(source, path)?;
        Ok(())
    }

    fn _finalize(&mut self, excludes: &[ErrorCode]) -> Result<Vec<Report>, FinalizeError> {
        // We can "take" this because we are putting it right back
        self.unlinked_texts = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.unlinked_texts),
            excludes,
        ));
        self.wikilink_visitor.finalize(excludes)?;
        Ok(self
            .unlinked_texts
            .iter()
            .map(|x| Report::ThirdPass(ThirdPassReport::UnlinkedText(x.clone())))
            .collect())
    }
}
