use std::{collections::HashMap, path::PathBuf};

use miette::{miette, Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::{file::content::front_matter::FrontMatter, rules::duplicate_alias::DuplicateAlias};

use super::HasCode;

pub const CODE: &str = "content::wikilink::broken";

#[derive(Error, Debug, Diagnostic)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code(CODE))]
pub struct BrokenWikilink {
    /// Used to identify the diagnostic and exclude it if needed
    code: String,

    #[source_code]
    filepaths: NamedSource<String>,

    #[label("Wikilink")]
    wikilink: SourceSpan,

    #[help]
    most_similar: String,
}

impl PartialEq for BrokenWikilink {
    fn eq(&self, other: &Self) -> bool {
        self.code == other.code
    }
}

impl HasCode for BrokenWikilink {
    fn code(&self) -> String {
        self.code.clone()
    }
}

impl BrokenWikilink {
    pub fn calculate(files: Vec<PathBuf>) -> Result<Vec<BrokenWikilink>> {
        // First we need to collect all the file names and and aliases and collect a lookup table
        // relating the string and the path to the file
        // We may hit a duplicate alias
        // In this case the developer forgot to run the duplicate aliases rule first
        let mut lookup_table = HashMap::<String, PathBuf>::new();
        for file_path in files {
            let front_matter =
                FrontMatter::from_file(&file_path).expect("This file was reported as existing");
            for alias in front_matter.aliases {
                if let Some(out) = lookup_table.insert(alias.clone(), file_path.clone()) {
                    return match DuplicateAlias::new(&alias, &out, &file_path) {
                        Ok(duplicatealias) => Err(miette!(duplicatealias)),
                        Err(e) => Err(miette!(e)),
                    };
                }
            }
        }

        unimplemented!()
    }
}
