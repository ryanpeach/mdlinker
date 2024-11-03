use rust_embed::{Embed, EmbeddedFile};

enum SCMFiles {
    Logseq,
}

#[derive(Embed)]
#[folder = "src/file/treesitter/assets"]
#[include = "*.scm"]
/// The tresitter grammars included in the library
struct TreeSitterGrammars;

impl TreeSitterGrammars {
    pub fn get_scm(&self, scm: SCMFiles) -> EmbeddedFile {
        match scm {
            SCMFiles::Logseq => {
                Self::get("logseq.scm").expect("This file is packed with the library")
            }
        }
    }
}
