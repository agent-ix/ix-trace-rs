//! FR-005-AC-4: `#[rstest]` below `#[trace]`.

mod common;

use common::{binary, build};
use ix_trace_rs::trace;
use ix_trace_run::registry::read_records;

#[trace("FR-005-AC-4")]
#[test]
fn rstest_below_trace_is_one_parametrized_record_and_after_the_cases_one_per_case() {
    let bins = build("rstest_tokio", &["--features", "emit"]);
    let records = read_records(&binary(&bins, "rstest_tokio").executable)
        .unwrap()
        .unwrap()
        .records;
    let param: Vec<_> = records.iter().filter(|r| r.ids == ["TC-200"]).collect();
    assert_eq!(param.len(), 1);
    assert!(param[0].parametrized && param[0].test);

    let bins = build("misuse", &["--features", "emit"]);
    let records = read_records(&binary(&bins, "misuse").executable)
        .unwrap()
        .unwrap()
        .records;
    let cases: Vec<_> = records.iter().filter(|r| r.ids == ["TC-204"]).collect();
    assert_eq!(cases.len(), 2, "{cases:?}");
    assert!(cases
        .iter()
        .all(|r| !r.parametrized && r.ident.starts_with("case_")));
}
