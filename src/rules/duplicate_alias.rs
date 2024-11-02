use std::{collections::HashMap, path::PathBuf};

use miette::{miette, Diagnostic, NamedSource, Result, SourceOffset, SourceSpan};
use thiserror::Error;

use crate::{
    config::Config,
    file::{content::from_file, name::get_filename},
    sed::MissingSubstringError,
};

use super::HasId;

pub const CODE: &str = "name::alias::duplicate";

#[derive(Error, Debug, Diagnostic)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code(CODE))]
pub enum DuplicateAlias {
    FileNameContentDuplicate {
        /// Used to identify the diagnostic and exclude it if needed
        id: String,

        /// The filename the alias contradicts with
        other_filename: String,

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
        id: String,

        /// The filename which contains the other duplicate alias
        other_filename: String,

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
    fn id(&self) -> String {
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

impl DuplicateAlias {
    /// Create a new diagnostic
    /// based on the two filenames and their similar ngrams
    ///
    pub fn new(
        alias: &str,
        file1_path: &PathBuf,
        file2_path: &PathBuf,
    ) -> Result<Self, MissingSubstringError> {
        assert_ne!(file1_path, file2_path);
        // Create the unique id
        let id = format!("{CODE}::{alias}");

        if get_filename(file1_path) == alias {
            let file2_content =
                std::fs::read_to_string(file2_path).expect("File reported as existing");
            // Find the alias
            let file2_content_found = file2_content.find(alias).ok_or_else(|| {
                MissingSubstringError::builder()
                    .path(file2_path.clone())
                    .ngram(alias.to_string())
                    .build()
            })?;

            // Generate the spans relative to the NamedSource
            let file2_content_span =
                SourceSpan::new(SourceOffset::from(file2_content_found), alias.len());

            Ok(DuplicateAlias::FileNameContentDuplicate {
                id,
                other_filename: get_filename(file1_path),
                src: NamedSource::new(file2_path.to_string_lossy(), file2_content),
                alias: file2_content_span,
                advice: format!("Delete the alias from {}", file2_path.to_string_lossy()),
            })
        } else if get_filename(file2_path) == alias {
            // This is the same as above just with path 1 and 2 flipped
            Self::new(alias, file2_path, file1_path)
        } else {
            let file1_content =
                std::fs::read_to_string(file1_path).expect("File reported as existing");
            let file2_content =
                std::fs::read_to_string(file2_path).expect("File reported as existing");

            // Find the alias
            let file1_content_found = file1_content.find(alias).ok_or_else(|| {
                MissingSubstringError::builder()
                    .path(file1_path.clone())
                    .ngram(alias.to_string())
                    .build()
            })?;
            let file2_content_found = file2_content.find(alias).ok_or_else(|| {
                MissingSubstringError::builder()
                    .path(file2_path.clone())
                    .ngram(alias.to_string())
                    .build()
            })?;

            // Generate the spans relative to the NamedSource
            let file1_content_span =
                SourceSpan::new(SourceOffset::from(file1_content_found), alias.len());
            let file2_content_span =
                SourceSpan::new(SourceOffset::from(file2_content_found), alias.len());

            Ok(DuplicateAlias::FileContentContentDuplicate {
                id: id.clone(),
                other_filename: file2_path.to_string_lossy().to_string(),
                src: NamedSource::new(file1_path.to_string_lossy(), file1_content),
                alias: file1_content_span,
                other: vec![DuplicateAlias::FileContentContentDuplicate {
                    id,
                    other_filename: file1_path.to_string_lossy().to_string(),
                    src: NamedSource::new(file2_path.to_string_lossy(), file2_content),
                    alias: file2_content_span,
                    other: vec![],
                }],
            })
        }
    }

    pub fn calculate(files: Vec<PathBuf>, config: &Config) -> Result<Vec<DuplicateAlias>> {
        Ok(Self::get_alias_to_path_table_and_duplicates(files, config)?.1)
    }

    /// This is a helper function for both [`crate::rules::broken_wikilink::BrokenWikilink`] and [`Self::calculate`]
    pub fn get_alias_to_path_table_and_duplicates(
        files: Vec<PathBuf>,
        config: &Config,
    ) -> Result<(HashMap<String, PathBuf>, Vec<DuplicateAlias>)> {
        // First we need to collect all the file names and and aliases and collect a lookup table
        // relating the string and the path to the file
        // We may hit a duplicate alias, if so we need to collect all of them and stop
        let mut lookup_table = HashMap::<String, PathBuf>::new();
        let mut duplicates: Vec<DuplicateAlias> = Vec::new();
        for file_path in files {
            let filename = get_filename(file_path.as_path());
            lookup_table.insert(filename, file_path.clone());
            let front_matter = from_file(file_path.clone(), config.wikilink_pattern().clone())
                .map_err(|e| miette!(e))?
                .front_matter;
            for alias in front_matter.aliases {
                if let Some(out) = lookup_table.insert(alias.clone(), file_path.clone()) {
                    duplicates.push(
                        DuplicateAlias::new(&alias, &out, &file_path).map_err(|e| miette!(e))?,
                    );
                }
            }
        }
        Ok((lookup_table, duplicates))
    }
}
