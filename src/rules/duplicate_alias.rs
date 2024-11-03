use crate::{
    file::{
        content::{front_matter::FrontMatterVisitor, wikilink::Alias},
        name::{get_filename, Filename},
    },
    ngrams::MissingSubstringError,
    sed::{ReplacePair, ReplacePairCompilationError},
    visitor::{FinalizeError, VisitError, Visitor},
};
use comrak::{arena_tree::Node, nodes::Ast};
use hashbrown::{HashMap, HashSet};
use miette::{Diagnostic, NamedSource, SourceOffset, SourceSpan};
use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};
use thiserror::Error;

use super::{dedupe_by_code, filter_by_excludes, ErrorCode, HasId};

pub const CODE: &str = "name::alias::duplicate";

#[derive(Error, Debug, Diagnostic)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code("name::alias::duplicate"))]
pub enum DuplicateAlias {
    FileNameContentDuplicate {
        /// Used to identify the diagnostic and exclude it if needed
        id: ErrorCode,

        /// The filename the alias contradicts with
        other_filename: Filename,

        /// The content of the file with the alias
        #[source_code]
        src: NamedSource<String>,

        /// The alias span in the content of the file with the alias
        #[label("Contradicts with the file named '{other_filename}' (case insensitive)")]
        alias: SourceSpan,

        /// Just some advice
        #[help]
        advice: String,
    },
    FileContentContentDuplicate {
        /// Used to identify the diagnostic and exclude it if needed
        id: ErrorCode,

        /// The filename which contains the other duplicate alias
        other_filename: Filename,

        /// The content of the file with the alias
        #[source_code]
        src: NamedSource<String>,

        /// The alias span in the content of the file with the
        #[label("Contradicts with aliases within '{other_filename}' (case insensitive)")]
        alias: SourceSpan,

        /// Put an exact copy but using the other file in src
        #[related]
        other: Vec<Self>,
    },
}

impl HasId for DuplicateAlias {
    fn id(&self) -> ErrorCode {
        match self {
            DuplicateAlias::FileNameContentDuplicate { id: code, .. }
            | DuplicateAlias::FileContentContentDuplicate { id: code, .. } => code.clone(),
        }
    }
}

impl PartialEq for DuplicateAlias {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl PartialOrd for DuplicateAlias {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id().partial_cmp(&other.id())
    }
}

#[derive(Debug)]
pub struct DuplicateAliasVisitor {
    /// Put an alias in get a file that contains that alias (or is named after the alias) out
    /// Also useful for telling you if you have seen this alias before
    pub alias_table: HashMap<Alias, PathBuf>,
    /// These are the duplicate alias diagnostics for miette
    pub duplicate_alias_errors: Vec<DuplicateAlias>,
    /// This is just the duplicate aliases themselves, useful for downstream tasks
    pub duplicate_aliases: HashSet<Alias>,
    /// Our main visitor, helps us get aliases from files, needs to be reset each file
    front_matter_visitor: FrontMatterVisitor,
    /// Just need to strore this for later to get aliases from filenames
    filename_to_alias: ReplacePair<Filename, Alias>,
}

impl DuplicateAliasVisitor {
    pub const NODE_KIND: &'static str = "alias";

    #[must_use]
    pub fn new(all_files: &Vec<PathBuf>, filename_to_alias: &ReplacePair<Filename, Alias>) -> Self {
        // First collect the files in the directories as aliases
        let mut alias_table = HashMap::new();
        for file in all_files {
            let filename = get_filename(file.as_path());
            let alias = Alias::from_filename(&filename, filename_to_alias);
            alias_table.insert(alias, file.clone());
        }
        Self {
            alias_table,
            duplicate_alias_errors: Vec::new(),
            duplicate_aliases: HashSet::new(),
            front_matter_visitor: FrontMatterVisitor::new(),
            filename_to_alias: filename_to_alias.clone(),
        }
    }
}

impl Visitor for DuplicateAliasVisitor {
    fn visit(&mut self, node: &Node<RefCell<Ast>>, source: &str) -> Result<(), VisitError> {
        self.front_matter_visitor.visit(node, source)?;
        Ok(())
    }
    fn finalize_file(&mut self, source: &str, path: &Path) -> Result<(), FinalizeError> {
        // We can "take" the aliases from the front_matter_visitor since we are going to clear them
        let aliases = std::mem::take(&mut self.front_matter_visitor.aliases);
        for alias in aliases {
            // This inserts the alias into the table and returns the previous value if it existed
            // If it did exist, we have a duplicate
            // If it did not exist, we have a new alias in our table
            if let Some(out) = self.alias_table.insert(alias.clone(), path.into()) {
                self.duplicate_aliases.insert(alias.clone());
                self.duplicate_alias_errors.push(DuplicateAlias::new(
                    &alias,
                    path,
                    Some(source),
                    &out,
                    None,
                    &self.filename_to_alias,
                )?);
            }
        }

        // Call finalize_file on the other visitors
        self.front_matter_visitor.finalize_file(source, path)?;
        Ok(())
    }
    fn finalize(&mut self, excludes: &[ErrorCode]) -> Result<(), FinalizeError> {
        // We can "take" the duplicate from the front_matter_visitor since we are going to put them
        // right back in after some cleaning
        self.duplicate_alias_errors = dedupe_by_code(filter_by_excludes(
            std::mem::take(&mut self.duplicate_alias_errors),
            excludes,
        ));
        self.front_matter_visitor.finalize(excludes)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum NewDuplicateAliasError {
    #[error(transparent)]
    MissingSubstringError(#[from] MissingSubstringError),
    #[error(transparent)]
    ReplacePairError(#[from] ReplacePairCompilationError),
    #[error("The file {filename} contains its own alias {alias}")]
    AliasAndFilenameSame { filename: Filename, alias: Alias },
}
//
// #[derive(Error, Debug)]
// pub enum CalculateError {
//     #[error(transparent)]
//     MissingSubstringError(#[from] MissingSubstringError),
//     #[error(transparent)]
//     ReplacePairError(#[from] ReplacePairCompilationError),
//     #[error(transparent)]
//     FileError(#[from] file::Error),
//     #[error(transparent)]
//     NewDuplicateAliasError(#[from] NewDuplicateAliasError),
//     #[error(transparent)]
//     IoError(#[from] std::io::Error),
// }
//
impl DuplicateAlias {
    /// Create a new diagnostic
    /// based on the two filenames and their similar ngrams
    ///
    /// File1 [`alias`] has been determined to be in file2
    ///
    ///
    pub fn new(
        alias: &Alias,
        file1_path: &Path,
        file1_content: Option<&str>,
        file2_path: &Path,
        file2_content: Option<&str>,
        filename_to_alias: &ReplacePair<Filename, Alias>,
    ) -> Result<Self, NewDuplicateAliasError> {
        assert!(!alias.to_string().is_empty());
        // Boundary conditions
        if file1_path == file2_path {
            return Err(NewDuplicateAliasError::AliasAndFilenameSame {
                filename: get_filename(file1_path),
                alias: alias.clone(),
            });
        }

        // Create the unique id
        let id = format!("{CODE}::{alias}");

        let file1_content = match file1_content {
            None => &std::fs::read_to_string(file1_path).expect("File reported as existing"),
            Some(content) => content,
        };
        let file2_content = match file2_content {
            None => &std::fs::read_to_string(file2_path).expect("File reported as existing"),
            Some(content) => content,
        };

        if Alias::from_filename(&get_filename(file1_path), filename_to_alias) == *alias {
            // Find the alias
            let file2_content_found = file2_content
                .to_lowercase()
                .find(&alias.to_string())
                .ok_or_else(|| MissingSubstringError {
                    path: file2_path.to_path_buf(),
                    ngram: alias.to_string(),
                    backtrace: std::backtrace::Backtrace::capture(),
                })?;

            // Generate the spans relative to the NamedSource
            let file2_content_span = SourceSpan::new(
                SourceOffset::from(file2_content_found),
                alias.to_string().len(),
            );

            Ok(DuplicateAlias::FileNameContentDuplicate {
                id: id.into(),
                other_filename: get_filename(file1_path),
                src: NamedSource::new(file2_path.to_string_lossy(), file2_content.to_string()),
                alias: file2_content_span,
                advice: format!("Delete the alias from {}", file2_path.to_string_lossy()),
            })
        } else if Alias::from_filename(&get_filename(file2_path), filename_to_alias) == *alias {
            Self::new(
                alias,
                file2_path,
                Some(file2_content),
                file1_path,
                Some(file1_content),
                filename_to_alias,
            )
        } else {
            // Find the alias
            let file1_content_found = file1_content
                .to_lowercase()
                .find(&alias.to_string())
                .ok_or_else(|| MissingSubstringError {
                    path: file1_path.to_path_buf(),
                    ngram: alias.to_string(),
                    backtrace: std::backtrace::Backtrace::capture(),
                })?;
            let file2_content_found = file2_content
                .to_lowercase()
                .find(&alias.to_string())
                .ok_or_else(|| MissingSubstringError {
                    path: file2_path.to_path_buf(),
                    ngram: alias.to_string(),
                    backtrace: std::backtrace::Backtrace::capture(),
                })?;

            // Generate the spans relative to the NamedSource
            let file1_content_span = SourceSpan::new(
                SourceOffset::from(file1_content_found),
                alias.to_string().len(),
            );
            let file2_content_span = SourceSpan::new(
                SourceOffset::from(file2_content_found),
                alias.to_string().len(),
            );

            Ok(DuplicateAlias::FileContentContentDuplicate {
                id: id.clone().into(),
                other_filename: get_filename(file2_path),
                src: NamedSource::new(file1_path.to_string_lossy(), file1_content.to_string()),
                alias: file1_content_span,
                other: vec![DuplicateAlias::FileContentContentDuplicate {
                    id: id.into(),
                    other_filename: get_filename(file1_path),
                    src: NamedSource::new(file2_path.to_string_lossy(), file2_content.to_string()),
                    alias: file2_content_span,
                    other: vec![],
                }],
            })
        }
    }
}
