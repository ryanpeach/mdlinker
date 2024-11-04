use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use crate::{
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
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue},
};
use hashbrown::HashMap;
use miette::{Diagnostic, NamedSource, Result, SourceOffset, SourceSpan};
use thiserror::Error;

use super::{dedupe_by_code, filter_by_excludes, ErrorCode, HasId};

pub const CODE: &str = "content::alias::unlinked";

#[derive(Error, Debug, Diagnostic, Builder)]
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
            let lowercase_text = text.to_lowercase();
            for alias in self.alias_table.keys() {
                if let Some(found) = lowercase_text.find(&alias.to_string()) {
                    // Make sure neither the character before or after is a letter
                    // This makes sure you aren't matching a part of a word
                    // This should also handle tags
                    if found > 0
                        && !text
                            .chars()
                            .nth(found - 1)
                            .expect("found is greater than 0")
                            .is_whitespace()
                    {
                        continue;
                    }
                    if found + alias.to_string().len() < text.len()
                        && text
                            .chars()
                            .nth(found + alias.to_string().len())
                            .expect("Already checked that found + alias.len() < text.len()")
                            .is_alphabetic()
                    {
                        continue;
                    }

                    // Get our span
                    let span = repair_span_due_to_frontmatter(
                        SourceSpan::new(
                            (SourceOffset::from_location(
                                remove_frontmatter_from_source(source, node),
                                sourcepos.start.line,
                                sourcepos.start.column,
                            )
                            .offset()
                                + found)
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

    fn _finalize(&mut self, excludes: &[ErrorCode]) -> Result<(), FinalizeError> {
        // We can "take" this because we are putting it right back
        self.unlinked_texts = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.unlinked_texts),
            excludes,
        ));
        self.wikilink_visitor.finalize(excludes)?;
        Ok(())
    }
}
