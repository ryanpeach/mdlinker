use std::path::PathBuf;

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
    pub fn calculate(files: &[PathBuf], config: &Config) -> Result<Vec<BrokenWikilink>> {
        let (lookup_table, _) =
            DuplicateAlias::get_alias_to_path_table_and_duplicates(files.into(), config)?;

        // Now we will take each file, get its wikilinks, and check that their alias is in the
        // lookup_table. If not, we will add a BrokenWikilink to out
        let mut out = Vec::new();
        for file_path in files {
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
                            .advice(format!(
                                "Create a page or alias for '{alias}' (case insensitive)"
                            ))
                            .build(),
                    );
                }
            }
        }
        Ok(out)
    }
}
