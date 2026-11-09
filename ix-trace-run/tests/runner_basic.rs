//! FR-006: the runner over a fixture crate with passing, failing, ignored and
//! concurrent tests.

mod common;

use std::process::Command;
use std::sync::OnceLock;

use common::{binary, build, case, cases, cases_named, runner, Run};
use ix_trace_rs::trace;

fn basic() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| runner("basic", &["--features", "emit"], &[]))
}

#[trace("TC-020", "FR-006-AC-1")]
#[test]
fn tc020_records_are_read_from_the_binary_alone() {
    let bins = build("basic", &["--features", "emit"]);
    let exe = &binary(&bins, "basic").executable;
    let out = Command::new(exe).arg("--list").output().expect("lists");
    let listed = String::from_utf8_lossy(&out.stdout)
        .matches(": test")
        .count();
    let run = basic();
    // Every case belongs to a listed test: no case exists for a record.
    let in_basic = run
        .xml
        .split("<testsuite name=\"basic\"")
        .nth(1)
        .and_then(|s| s.split("</testsuite>").next())
        .expect("a suite for the lib target");
    assert_eq!(
        in_basic.matches("<testcase ").count(),
        listed,
        "{}",
        run.xml
    );
    assert!(
        !run.xml.contains("name=\"plain\""),
        "a non-test record became a case"
    );
    assert!(
        !run.xml.contains("name=\"assoc\""),
        "a non-test record became a case"
    );
}

#[trace("FR-006-AC-2")]
#[test]
fn a_passing_tagged_test_has_name_equal_to_classname_file_and_trace() {
    let c = case(&basic().xml, "passes");
    assert!(c.contains("classname=\"passes\" name=\"passes\""), "{c}");
    assert!(c.contains("file=\"src/lib.rs\""), "{c}");
    assert!(
        c.contains("<property name=\"trace\" value=\"TC-101,FR-101-AC-1\"/>"),
        "{c}"
    );
    assert!(!c.contains("<failure") && !c.contains("<skipped"), "{c}");
}

#[trace("FR-006-AC-3")]
#[test]
fn a_nested_test_reports_its_file_relative_name_and_a_top_level_one_its_bare_name() {
    let nested = case(&basic().xml, "tests::t");
    assert!(nested.contains("file=\"src/foo/bar.rs\""), "{nested}");
    assert!(nested.contains("value=\"TC-110,FR-110-AC-1\""), "{nested}");
    let top = case(&basic().xml, "x_top");
    assert!(top.contains("file=\"tests/x.rs\""), "{top}");
}

#[trace("FR-006-AC-4")]
#[test]
fn a_panicking_tagged_test_has_a_failure_and_its_trace() {
    let c = case(&basic().xml, "panics");
    assert!(c.contains("<failure"), "{c}");
    assert!(c.contains("value=\"TC-102\""), "{c}");
}

#[trace("FR-006-AC-5")]
#[test]
fn ignored_is_skipped_with_its_trace_and_should_panic_passes() {
    let ignored = case(&basic().xml, "ignored");
    assert!(
        ignored.contains("<skipped/>") && ignored.contains("value=\"TC-103\""),
        "{ignored}"
    );
    let should_panic = case(&basic().xml, "should_panic_passes");
    assert!(!should_panic.contains("<failure"), "{should_panic}");
}

#[trace("FR-006-AC-6")]
#[test]
fn concurrent_tests_keep_their_own_ids_once() {
    for (name, id) in [("slow_a", "TC-105"), ("slow_b", "TC-106")] {
        let c = case(&basic().xml, name);
        assert_eq!(c.matches("<property").count(), 1, "{c}");
        assert!(c.contains(&format!("value=\"{id}\"")), "{c}");
    }
}

#[trace("FR-005-AC-11")]
#[test]
fn a_test_attribute_above_trace_still_gets_its_ids() {
    let c = case(&basic().xml, "test_above_trace");
    assert!(c.contains("value=\"TC-108\""), "{c}");
}

#[trace("FR-006-AC-14")]
#[test]
fn a_test_forging_stdout_changes_no_outcome() {
    let xml = &basic().xml;
    assert!(!case(xml, "forges_stdout").contains("<failure"));
    assert!(cases_named(xml, "other").is_empty() && cases_named(xml, "self").is_empty());
    assert!(!case(xml, "passes").contains("<failure"));
}

#[trace("FR-006-AC-17")]
#[test]
fn a_multiline_ignore_reason_is_one_skipped_case_and_later_tests_keep_outcomes() {
    let xml = &basic().xml;
    assert!(case(xml, "multiline_ignore").contains("<skipped/>"));
    let after = case(xml, "after_multiline");
    assert!(
        !after.contains("<failure") && !after.contains("<skipped"),
        "{after}"
    );
}

#[trace("FR-006-AC-18")]
#[test]
fn a_binary_without_a_trace_section_gets_a_notice_and_the_tests_exit_status() {
    let run = runner("basic", &[], &[]);
    assert!(
        run.stderr.contains("has no trace section"),
        "{}",
        run.stderr
    );
    assert!(!run.xml.contains("<property"), "{}", run.xml);
    // The suite has a panicking test, so its own status is a failure.
    assert_eq!(run.code, Some(1));
    assert!(case(&run.xml, "panics").contains("<failure"));
    assert!(cases(&run.xml).len() > 5);
}

#[trace("FR-006-AC-11")]
#[test]
fn tc021_a_clean_run_writes_one_file_and_leaves_no_log() {
    let tmp = tempfile::tempdir().expect("temp dir");
    let out = tempfile::tempdir().expect("temp dir");
    let result = out.path().join("result.xml");
    let status = Command::new(env!("CARGO_BIN_EXE_ix-trace-run"))
        .arg("--output")
        .arg(&result)
        .arg("--manifest-path")
        .arg(common::manifest("unified"))
        .arg("--workspace")
        .env("CARGO_TARGET_DIR", common::target_dir())
        .env("TMPDIR", tmp.path())
        .status()
        .expect("runner starts");
    assert!(status.success());
    let files: Vec<_> = std::fs::read_dir(out.path()).unwrap().collect();
    assert_eq!(files.len(), 1);
    let left: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
    assert!(left.is_empty(), "temporary files left behind: {left:?}");
    let xml = std::fs::read_to_string(&result).unwrap();
    assert!(case(&xml, "from_a").contains("value=\"TC-5a1\""));
    assert!(case(&xml, "from_b").contains("value=\"TC-5b1\""));
}
