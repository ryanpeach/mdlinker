use lazy_static::lazy_static;
use mdlinker::rules::unlinked_text;

use mdlinker::rules::filter_code;

use crate::common::get_report;

lazy_static! {
    static ref PATHS: Vec<String> = vec![
        "./tests/logseq/unlinked_texts/assets/pages/".to_string(),
        "./tests/logseq/unlinked_texts/assets/journals/".to_string()
    ];
}

#[test]
fn number_of_unlinked_texts() {
    let report = get_report(PATHS.as_slice());
    for unlinked_texts in &report.unlinked_texts {
        println!("{unlinked_texts:?}");
    }
    assert_eq!(report.unlinked_texts.len(), 1);
}

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    let report = get_report(PATHS.as_slice());
    for broken_wikilink in &report.broken_wikilinks {
        println!("{broken_wikilink:?}");
    }
    assert!(filter_code(
        report.broken_wikilinks,
        &format!("{}::2024_11_01::lorem", unlinked_text::CODE).into()
    )
    .is_empty());
}

/// This fails because the link is invalid
#[test]
fn ipsum_does_not_exist_and_is_wikilink() {
    let report = get_report(PATHS.as_slice());
    for broken_wikilink in &report.broken_wikilinks {
        println!("{broken_wikilink:?}");
    }
    assert!(!filter_code(
        report.broken_wikilinks,
        &format!("{}::2024_11_01::ipsum", unlinked_text::CODE).into()
    )
    .is_empty());
}
