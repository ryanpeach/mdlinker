use std::{path::PathBuf, str::FromStr};

use mdlinker::{
    config, lib,
    rules::{broken_wikilink, duplicate_alias},
};

use mdlinker::rules::VecHasCodeExtensions;

fn get_report() -> mdlinker::OutputReport {
    let config = config::Config::builder()
        .directories(vec![PathBuf::from_str("./assets/pages").unwrap()])
        .build();

    lib(&config).expect("Shouldn't be a problem with the linter itself")
}

/// This passes because the link is valid
#[test]
fn lorem_exist_and_is_wikilink() {
    let report = get_report();
    assert!(!report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::lorem", broken_wikilink::CODE)));
}

/// This fails because the link is invalid
#[test]
fn ipsum_does_not_exist_and_is_wikilink() {
    let report = get_report();
    assert!(report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::ipsum", broken_wikilink::CODE)));
}

/// This passes because there is no link
#[test]
fn dolor_does_not_exist_and_is_not_wikilink() {
    let report = get_report();
    assert!(!report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::dolor", broken_wikilink::CODE)));
}

/// This passes because the link is valid
/// This is also a tag so testing that tags work
#[test]
fn sit_does_not_exist_and_is_tag() {
    let report = get_report();
    assert!(report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::sit", broken_wikilink::CODE)));
}

/// This fails because the link is invalid
/// This is a multi word tag, just testing those work
#[test]
fn amet_does_not_exist_and_is_tag() {
    let report = get_report();
    assert!(report
        .broken_wikilinks
        .contains_code(&format!("{}::2024_11_01::amet", broken_wikilink::CODE)));
}

/// Test that we detect the sit and lorem duplicate aliases
#[test]
fn sit_lorem_duplicate_aliases() {
    let report = get_report();
    assert!(report
        .duplicate_aliases
        .contains_code(&format!("{}::sit", duplicate_alias::CODE)));
    assert!(report
        .duplicate_aliases
        .contains_code(&format!("{}::lorem", duplicate_alias::CODE)));
}
