use std::{collections::HashMap, path::PathBuf};

use bon::Builder;
use miette::{miette, Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

use crate::{
    config::Config,
    file::{content::from_file, name::get_filename},
    rules::duplicate_alias::DuplicateAlias,
};

use super::HasId;

pub const CODE: &str = "content::wikilink::broken";

#[derive(Error, Debug, Diagnostic, Builder)]
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
    pub fn calculate(files: Vec<PathBuf>, config: &Config) -> Result<Vec<BrokenWikilink>> {
        // First we need to collect all the file names and and aliases and collect a lookup table
        // relating the string and the path to the file
        // We may hit a duplicate alias
        // In this case the developer forgot to run the duplicate aliases rule first
        let mut lookup_table = HashMap::<String, PathBuf>::new();
        for file_path in &files {
            lookup_table.insert(get_filename(file_path.as_path()), file_path.clone());
            let front_matter = from_file(file_path.clone(), config.wikilink_pattern().clone())
                .map_err(|e| miette!(e))?
                .front_matter;
            for alias in front_matter.aliases {
                if let Some(out) = lookup_table.insert(alias.clone(), file_path.clone()) {
                    return match DuplicateAlias::new(&alias, &out, file_path) {
                        Ok(duplicatealias) => Err(miette!(duplicatealias)),
                        Err(e) => Err(miette!(e)),
                    };
                }
            }
        }

        // Now we will take each file, get its wikilinks, and check that their alias is in the
        // lookup_table. If not, we will add a BrokenWikilink to out
        let mut out = Vec::new();
        for file_path in &files {
            let mut file_content = None;
            let wikilinks = from_file(file_path.clone(), config.wikilink_pattern().clone())
                .map_err(|e| miette!(e))?
                .wikilinks;
            let filename = get_filename(file_path.as_path());
            for wikilink in wikilinks {
                let alias = wikilink.alias();
                if !lookup_table.contains_key(alias) {
                    if file_content.is_none() {
                        file_content =
                            Some(std::fs::read_to_string(file_path).map_err(|e| miette!(e))?);
                    }
                    out.push(
                        BrokenWikilink::builder()
                            .id(format!("{CODE}::{filename}::{alias}"))
                            .src(NamedSource::new(
                                file_path.to_string_lossy(),
                                file_content
                                    .as_ref()
                                    .expect("file content exists")
                                    .to_string(),
                            ))
                            .wikilink(*wikilink.span())
                            .advice(format!("Create a page for {alias}"))
                            .build(),
                    );
                }
            }
        }
        Ok(out)
    }
}
