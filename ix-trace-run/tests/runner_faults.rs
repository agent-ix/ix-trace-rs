//! FR-006: the runner over fixtures that abort, exit, mis-order attributes or
//! leave records without a test.

mod common;

use std::sync::OnceLock;

use common::{case, cases, cases_named, runner, Run};
use ix_trace_rs::trace;

#[trace("TC-021", "FR-006-AC-8")]
#[test]
fn tc021_an_aborting_binary_loses_no_test() {
    let run = abnormal();
    for (name, id) in [
        ("aborts", "TC-300"),
        ("survivor_a", "TC-302"),
        ("survivor_b", "TC-303"),
    ] {
        let c = case(&run.xml, name);
        assert!(c.contains(&format!("value=\"{id}\"")), "{c}");
    }
    assert!(case(&run.xml, "aborts").contains("<failure"));
    assert_ne!(run.code, Some(0));
}

fn abnormal() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| runner("abnormal", &["--features", "emit"], &[]))
}

#[trace("FR-006-AC-16")]
#[test]
fn a_test_that_exits_zero_is_rerun_alone_and_never_an_unreported_pass() {
    let run = abnormal();
    assert_eq!(cases(&run.xml).len(), 4, "{}", run.xml);
    let c = case(&run.xml, "exits_zero");
    assert!(
        c.contains("value=\"TC-301\"") && !c.contains("<failure"),
        "{c}"
    );
}

fn unmatched() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| runner("unmatched", &["--features", "emit"], &[]))
}

#[trace("FR-006-AC-9")]
#[test]
fn a_record_without_a_listed_test_is_reported_and_fails_the_run() {
    let run = unmatched();
    assert_eq!(run.code, Some(1));
    assert!(
        run.stderr.contains("TC-400"),
        "fn-body test: {}",
        run.stderr
    );
    assert!(
        run.stderr.contains("TC-420"),
        "off-prefix test: {}",
        run.stderr
    );
    assert!(case(&run.xml, "fine").contains("value=\"TC-401\""));
    assert!(!case(&run.xml, "outside::off_tree").contains("<property"));
}

#[trace("FR-006-AC-19")]
#[test]
fn two_records_with_one_libtest_name_mark_neither_test() {
    let run = unmatched();
    assert!(run.stderr.contains("ambiguous"), "{}", run.stderr);
    assert!(
        run.stderr.contains("TC-410") && run.stderr.contains("TC-412"),
        "{}",
        run.stderr
    );
    let check = case(&run.xml, "check");
    assert!(!check.contains("<property"), "{check}");
}

fn rstest_tokio() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| runner("rstest_tokio", &["--features", "emit"], &[]))
}

#[trace("FR-006-AC-7")]
#[test]
fn rstest_cases_report_the_source_function_and_a_same_named_module_gets_no_mark() {
    let run = rstest_tokio();
    let params = cases_named(&run.xml, "param");
    assert_eq!(params.len(), 3, "{}", run.xml);
    assert!(params.iter().all(|c| c.contains("value=\"TC-200\"")));
    assert_eq!(params.iter().filter(|c| c.contains("<failure")).count(), 1);
    assert!(case(&run.xml, "plain_tagged").contains("value=\"TC-203\""));
    let inside = case(&run.xml, "plain_tagged::untagged_inside");
    assert!(!inside.contains("<property"), "{inside}");
}

#[trace("FR-006-AC-10", "FR-005-AC-5")]
#[test]
fn a_tokio_test_yields_one_case_with_its_ids_in_either_order() {
    let run = rstest_tokio();
    for (name, id) in [("tokio_above", "TC-201"), ("tokio_below", "TC-202")] {
        let c = case(&run.xml, name);
        assert!(
            c.contains(&format!("value=\"{id}\"")) && !c.contains("<failure"),
            "{c}"
        );
    }
}
