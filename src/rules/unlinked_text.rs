use crate::{
    config::Config,
    file::{
        content::wikilink::{Alias, WikilinkVisitor},
        name::{get_filename, Filename},
    },
    sed::ReplacePair,
    visitor::{FinalizeError, VisitError, Visitor},
};
use aho_corasick::AhoCorasick;
use bon::Builder;
use comrak::{
    arena_tree::Node,
    nodes::{Ast, NodeValue},
};
use hashbrown::HashMap;
use log::trace;
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
    /// TODO: Be able to handle this in parallel with other reports
    fn fix(&self, _config: &Config) -> Result<Option<()>, FixError> {
        let file = self.src.name().to_owned();
        trace!("Fixing unlinked text: {:?}", file);
        let mut source = self.src.inner().clone();
        let start = self.span.offset();
        let end = start + self.span.len();
        if end >= source.len() {
            source.push_str("]]"); // Append to the end if `end` is out of bounds
        } else {
            source.insert_str(end, "]]"); // Insert at `end` if within bounds
        }
        source.insert_str(start, "[[");
        std::fs::write(self.src.name(), source).map_err(|source| FixError::IOError {
            source,
            file,
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

/// Checks if the match at the given start and end indices is a whole word match.
fn is_whole_word_match(text: &str, start: usize, end: usize) -> bool {
    is_start_boundary(text, start) && is_end_boundary(text, end) && !is_start_hashtag(text, start)
}

/// Checks if the character before the start index is a word boundary.
fn is_start_boundary(text: &str, start: usize) -> bool {
    if start == 0 {
        true
    } else {
        text[..start]
            .chars()
            .next_back()
            .is_none_or(char::is_whitespace)
    }
}

/// Checks if the character before the start index is a word boundary.
fn is_start_hashtag(text: &str, start: usize) -> bool {
    if start == 0 {
        false
    } else {
        text[..start].ends_with('#')
    }
}

/// Checks if the character after the end index is a word boundary.
fn is_end_boundary(text: &str, end: usize) -> bool {
    if end == text.len() {
        true
    } else {
        text[end..].chars().next().is_none_or(char::is_whitespace)
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
            let patterns: Vec<String> = self
                .alias_table
                .keys()
                .map(std::string::ToString::to_string)
                .collect();
            let ac = AhoCorasick::builder()
                .ascii_case_insensitive(true)
                .build(&patterns)?;
            // Make sure neither the character before or after is a letter
            // This makes sure you aren't matching a part of a word
            // This should also handle tags
            // Check the character before the match
            for found in ac.find_iter(text) {
                if !is_whole_word_match(text, found.start(), found.end()) {
                    continue;
                }
                let alias = Alias::new(&patterns[found.pattern().as_usize()]);
                if "lorem" == alias.to_string() {
                    println!("Found lorem");
                }
                let sourcepos_start_offset_bytes =
                    SourceOffset::from_location(text, sourcepos.start.line, sourcepos.start.column)
                        .offset();
                let byte_length = found.end() - found.start();
                let offset_bytes = sourcepos_start_offset_bytes + found.start();
                let span = SourceSpan::new(offset_bytes.into(), byte_length);

                // Dont match inside wikilinks
                if let Some(parent) = parent {
                    if let NodeValue::WikiLink(_) = parent.data.borrow().value {
                        // If this is already in a link, skip it
                        continue;
                    }
                }

                self.new_unlinked_texts.push((alias, span));
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
            let id = format!("{CODE}::{filename}::{alias}");
            self.unlinked_texts.push(
                UnlinkedText::builder()
                    .advice(format!(
                        "Consider wrapping it in a wikilink, like: [[{alias}]]\nNOTE: If running in --fix, you may need to run fix more than once to fix all unlinked text errors.\n      I recommend doing this one at a time.\nREF: https://github.com/ryanpeach/mdlinker/issues/44\nid: {id:?}"
                    ))
                    .id(id.into())
                    .src(NamedSource::new(path.to_string_lossy(), source.to_string()))
                    .alias(alias.clone())
                    .span(*span)
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
