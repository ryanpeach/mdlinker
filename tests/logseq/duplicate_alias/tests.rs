use lazy_static::lazy_static;
use mdlinker::rules::duplicate_alias;

use mdlinker::rules::duplicate_alias::DuplicateAlias;
use mdlinker::rules::filter_code;

use crate::common::get_report;
use log::{debug, info};

use itertools::Itertools;

lazy_static! {
    static ref PATHS: Vec<String> = vec![
        "./tests/logseq/duplicate_alias/assets/pages".to_string(),
        "./tests/logseq/duplicate_alias/assets/journals".to_string()
    ];
}

#[test]
fn number_of_duplicate_alias() {
    info!("number_of_duplicate_alias");
    let report = get_report(PATHS.as_slice());
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    assert_eq!(report.duplicate_aliases().len(), 3);
}

#[test]
fn filename_alias_relation() {
    info!("filename_alias_relation");
    let report = get_report(PATHS.as_slice());
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::lorem", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn filecontent_filecontent_relation() {
    info!("filecontent_filecontent_relation");
    let report = get_report(PATHS.as_slice());
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::dolor", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn duplicate_ipsum() {
    info!("duplicate_ipsum_span");
    let report = get_report(PATHS.as_slice());
    for duplicate_alias in &report.duplicate_aliases() {
        debug!("{duplicate_alias:#?}");
    }
    let duplicate = filter_code(
        report.duplicate_aliases(),
        &format!("{}::ipsum", duplicate_alias::CODE).into(),
    )
    .into_iter()
    .at_most_one()
    .unwrap();
    assert!(duplicate.is_some());
}

#[test]
fn duplicate_ipsum_span() {
    info!("duplicate_ipsum");
    let report = get_report(PATHS.as_slice());
    let err_list = filter_code(
        report.duplicate_aliases(),
        &format!("{}::ipsum", duplicate_alias::CODE).into(),
    );
    let err = err_list.iter().exactly_one().unwrap();
    match err {
        DuplicateAlias::FileNameContentDuplicate { alias, .. }
        | DuplicateAlias::FileContentContentDuplicate { alias, .. } => {
            assert_eq!(alias.offset(), 11);
            assert_eq!(alias.len(), 5);
        }
    }
}
