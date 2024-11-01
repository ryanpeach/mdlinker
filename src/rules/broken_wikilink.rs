use std::{collections::HashMap, path::PathBuf};

use miette::{miette, Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::{file::content::from_file, rules::duplicate_alias::DuplicateAlias};

use super::HasId;

pub const CODE: &str = "content::wikilink::broken";

#[derive(Error, Debug, Diagnostic)]
#[error("A wikilink does not have a corresponding page")]
#[diagnostic(code(CODE))]
pub struct BrokenWikilink {
    /// Used to identify the diagnostic and exclude it if needed
    id: String,

    #[source_code]
    src: NamedSource<String>,

    #[label("Wikilink")]
    wikilink: SourceSpan,

    #[help]
    advice: String,
}

impl PartialEq for BrokenWikilink {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl HasId for BrokenWikilink {
    fn id(&self) -> String {
        self.id.clone()
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
            let front_matter = from_file(file_path.clone()).map_err(|e| miette!(e))?;
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
