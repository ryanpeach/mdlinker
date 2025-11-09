#[ctor::ctor]
fn init_logger() {
    env_logger::init();
}

#[path = "logseq/broken_wikilink.rs"]
mod broken_wikilink;
#[path = "logseq/duplicate_alias.rs"]
mod duplicate_alias;
#[path = "logseq/similar_filename.rs"]
mod similar_filename;
#[path = "logseq/unlinked_text.rs"]
mod unlinked_text;
