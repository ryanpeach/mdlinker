#[ctor::ctor]
fn init_logger() {
    env_logger::init();
}

#[path = "obsidian/path_wikilink.rs"]
mod path_wikilink;

#[path = "obsidian/pipe_wikilink.rs"]
mod pipe_wikilink;
