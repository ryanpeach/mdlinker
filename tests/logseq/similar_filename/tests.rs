use log::info;
use mdlinker::{config, lib};
use std::{path::PathBuf, str::FromStr};

/// [`foo.md`](./assets/logseq/pages/foo.md) and [`foo___bar.md`](./assets/logseq/pages/foo___bar.md) should not conflict
/// because the word `foo` in `foo/bar` is just a properly used group name.
#[test]
fn groups_first_element_same() {
    info!("groups_first_element_same");
    let config = config::Config::builder()
        .pages_directory(
            PathBuf::from_str("./tests/logseq/similar_filename/assets/pages")
                .expect("This is a constant"),
        )
        .filename_match_threshold(80)
        .build();

    if let Err(e) = lib(&config) {
        panic!("There should have been no error. Found: {e}");
    }
}

// #[test]
// fn some_valid_match() {
//     unimplemented!()
// }
