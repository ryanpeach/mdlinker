use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use crate::{
    file::{
        content::wikilink::Alias,
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
    span: SourceSpan,

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
            unlinked_texts: Vec::new(),
            new_unlinked_texts: Vec::new(),
        }
    }
}

impl Visitor for UnlinkedTextVisitor {
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        let data_ref = node.data.borrow();
        let data = &data_ref.value;
        let sourcepos = data_ref.sourcepos;
        let mut get_tags = |text: &str| {
            let lowercase_source = text.to_lowercase();
            for alias in self.alias_table.keys() {
                if let Some(found) = lowercase_source.find(&alias.to_string()) {
                    self.new_unlinked_texts.push((
                        alias.clone(),
                        SourceSpan::new(
                            (SourceOffset::from_location(
                                source,
                                sourcepos.start.line,
                                sourcepos.start.column,
                            )
                            .offset()
                                + found)
                                .into(),
                            alias.to_string().len(),
                        ),
                    ));
                }
            }
        };
        match data {
            NodeValue::Text(text) => {
                get_tags(text);
            }
            _ => {}
        }
        Ok(())
    }
    fn finalize_file(
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
                    .advice(format!("[[{alias}]]"))
                    .build(),
            );
        }
        self.new_unlinked_texts.clear();
        Ok(())
    }

    fn finalize(&mut self, excludes: &[ErrorCode]) -> Result<(), FinalizeError> {
        // We can "take" this because we are putting it right back
        self.unlinked_texts = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.unlinked_texts),
            excludes,
        ));
        Ok(())
    }
}
