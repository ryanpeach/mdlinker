use mdlinker::rules::broken_wikilink;

use mdlinker::rules::VecHasCodeExtensions;

use crate::common::get_report;

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    let report = get_report();
    assert!(report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::lorem", broken_wikilink::CODE))
        .is_empty());
}

/// This fails because the link is invalid
#[test]
fn ipsum_does_not_exist_and_is_wikilink() {
    let report = get_report();
    assert!(!report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::ipsum", broken_wikilink::CODE))
        .is_empty());
}

/// This passes because there is no link
#[test]
fn dolor_does_not_exist_and_is_not_wikilink() {
    let report = get_report();
    assert!(report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::dolor", broken_wikilink::CODE))
        .is_empty());
}

/// This passes because the link is valid
/// This is also a tag so testing that tags work
#[test]
fn sit_does_not_exist_and_is_tag() {
    let report = get_report();
    assert!(!report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::sit", broken_wikilink::CODE))
        .is_empty());
}

/// This fails because the link is invalid
/// This is a multi word tag, just testing those work
#[test]
fn amet_does_not_exist_and_is_tag() {
    let report = get_report();
    assert!(!report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::amet", broken_wikilink::CODE))
        .is_empty());
}

#[test]
fn miette_formatting() {
    unimplemented!()
}
