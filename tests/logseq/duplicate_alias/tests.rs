use mdlinker::rules::duplicate_alias;

use mdlinker::rules::VecHasIdExtensions;

use crate::common::get_report;

use itertools::Itertools;

#[test]
fn filename_alias_relation() {
    let report = get_report();
    report
        .duplicate_aliases
        .contains_code(&format!("{}::lorem", duplicate_alias::CODE))
        .iter()
        .at_most_one()
        .unwrap();
    unimplemented!()
}

#[test]
fn filecontent_filecontent_relation() {
    let report = get_report();
    report
        .duplicate_aliases
        .contains_code(&format!("{}::dolors", duplicate_alias::CODE))
        .iter()
        .at_most_one()
        .unwrap();
    unimplemented!()
}
