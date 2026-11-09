//! FR-005-AC-4: `#[rstest]` below `#[trace]`.

mod common;

use common::{binary, build};
use ix_trace_rs::trace;
use ix_trace_run::registry::read_records;

#[trace("FR-005-AC-4")]
#[test]
fn rstest_below_trace_is_one_parametrized_record() {
    let bins = build("rstest_tokio", &["--features", "emit"]);
    let records = read_records(&binary(&bins, "rstest_tokio").executable)
        .unwrap()
        .unwrap();
    let param: Vec<_> = records.iter().filter(|r| r.ids == ["TC-200"]).collect();
    assert_eq!(param.len(), 1);
    assert!(param[0].parametrized && param[0].test);
}
