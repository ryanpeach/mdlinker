use std::{collections::HashMap, path::PathBuf};

use bon::Builder;
use miette::{miette, Diagnostic, NamedSource, Result, SourceOffset, SourceSpan};
use thiserror::Error;

use crate::{file::content::from_file, sed::MissingSubstringError};

use super::HasCode;

pub const CODE: &str = "name::alias::duplicate";

#[derive(Error, Debug, Diagnostic, Builder)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code(CODE))]
pub struct DuplicateAlias {
    /// Used to identify the diagnostic and exclude it if needed
    code: String,

    #[source_code]
    filepaths: NamedSource<String>,

    #[label("This bit here")]
    instance1: SourceSpan,

    #[label("That bit there")]
    instance2: SourceSpan,

    #[help]
    advice: String,
}

impl PartialEq for DuplicateAlias {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl HasCode for DuplicateAlias {
    fn code(&self) -> String {
        self.code.clone()
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

        if file1_path.to_string_lossy().contains(alias) {
            let file2_content =
                std::fs::read_to_string(file2_path).expect("File reported as existing");
            // Assemble the source
            let source = format!(
                "{}\n```{}\n{}\n```",
                file1_path.to_string_lossy(),
                file2_path.to_string_lossy(),
                file2_content
            );
            let filepaths = NamedSource::new("Filepaths", source);

            // Find the alias
            let file1_path_found = file1_path.to_string_lossy().find(alias).ok_or_else(|| {
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
            let file1_path_span =
                SourceSpan::new(SourceOffset::from(file1_path_found), alias.len());
            let file2_content_span = SourceSpan::new(
                SourceOffset::from(
                    file1_path.to_string_lossy().len()
                        + 4
                        + file2_path.to_string_lossy().len()
                        + 1
                        + file2_content_found,
                ),
                alias.len(),
            );

            Ok(DuplicateAlias::builder()
                .code(id)
                .filepaths(filepaths)
                .instance1(file1_path_span)
                .instance2(file2_content_span)
                .advice("Check the files for duplicate aliases".to_string())
                .build())
        } else if file2_path.to_string_lossy().contains(alias) {
            // This is the same as above just with path 1 and 2 flipped
            Self::new(alias, file2_path, file1_path)
        } else {
            let file1_content =
                std::fs::read_to_string(file1_path).expect("File reported as existing");
            let file2_content =
                std::fs::read_to_string(file2_path).expect("File reported as existing");

            // Assemble the source
            let source = format!(
                "```{}\n{}\n```\n```{}\n{}\n```",
                file1_path.to_string_lossy(),
                file1_content,
                file2_path.to_string_lossy(),
                file2_content
            );
            let filepaths = NamedSource::new("Filepaths", source);

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
            let file1_content_span = SourceSpan::new(
                SourceOffset::from(
                    3 + file1_path.to_string_lossy().len() + 1 + file1_content_found,
                ),
                alias.len(),
            );
            let file2_content_span = SourceSpan::new(
                SourceOffset::from(
                    3 + file1_path.to_string_lossy().len()
                        + 1
                        + file1_content.len()
                        + 4
                        + 3
                        + file2_path.to_string_lossy().len()
                        + 1
                        + file2_content_found,
                ),
                alias.len(),
            );

            Ok(DuplicateAlias::builder()
                .code(id)
                .filepaths(filepaths)
                .instance1(file1_content_span)
                .instance2(file2_content_span)
                .advice("Check the files for duplicate aliases".to_string())
                .build())
        }
    }

    pub fn calculate(files: Vec<PathBuf>) -> Result<Vec<DuplicateAlias>> {
        // First we need to collect all the file names and and aliases and collect a lookup table
        // relating the string and the path to the file
        // We may hit a duplicate alias, if so we need to collect all of them and stop
        let mut lookup_table = HashMap::<String, PathBuf>::new();
        let mut duplicates: Vec<DuplicateAlias> = Vec::new();
        for file_path in files {
            let front_matter = from_file(file_path.clone()).map_err(|e| miette!(e))?;
            for alias in front_matter.aliases {
                if let Some(out) = lookup_table.insert(alias.clone(), file_path.clone()) {
                    duplicates.push(
                        DuplicateAlias::new(&alias, &out, &file_path).map_err(|e| miette!(e))?,
                    );
                }
            }
        }
        Ok(duplicates)
    }
}
