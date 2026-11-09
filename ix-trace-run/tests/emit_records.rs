//! FR-005: the records `#[trace]` embeds with the `emit` feature.

mod common;

use std::process::Command;

use common::{binary, build, manifest, target_dir};
use ix_trace_rs::trace;
use ix_trace_run::registry::{read_records, Record};
use object::{Object, ObjectSection};

fn records(bins: &[ix_trace_run::cargo::TestBinary], target: &str) -> Vec<Record> {
    read_records(&binary(bins, target).executable)
        .expect("the binary is readable")
        .expect("the binary has a trace section")
}

fn with_ids<'a>(records: &'a [Record], ids: &[&str]) -> Vec<&'a Record> {
    records.iter().filter(|r| r.ids == ids).collect()
}

fn list(bin: &ix_trace_run::cargo::TestBinary) -> String {
    let out = Command::new(&bin.executable)
        .arg("--list")
        .output()
        .expect("lists");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

#[trace("TC-017", "FR-005-AC-1", "FR-005-CON-1")]
#[test]
fn tc017_without_emit_the_binary_has_no_trace_section() {
    let bins = build("basic", &[]);
    let found = read_records(&binary(&bins, "basic").executable).expect("readable");
    assert!(found.is_none(), "default build carries a trace section");
}

#[trace("FR-005-AC-6")]
#[test]
fn a_rejected_argument_list_gives_the_same_error_with_emit_on_and_off() {
    let errors = |features: &[&str]| -> Vec<String> {
        let out = Command::new("cargo")
            .args(["build", "--tests", "--manifest-path"])
            .arg(manifest("badargs"))
            .args(features)
            .env("CARGO_TARGET_DIR", target_dir())
            .output()
            .expect("cargo starts");
        assert!(!out.status.success(), "prose must not compile");
        String::from_utf8_lossy(&out.stderr)
            .lines()
            .filter(|l| l.starts_with("error"))
            .map(str::to_owned)
            .collect()
    };
    let off = errors(&[]);
    let on = errors(&["--features", "emit"]);
    assert!(
        off.iter()
            .any(|l| l.contains("`hello world` is not a spec id")),
        "{off:?}"
    );
    assert_eq!(off, on);
}

#[trace("FR-005-AC-7", "FR-005-CON-3")]
#[test]
fn implements_registers_nothing_in_any_mode() {
    let bins = build("basic", &["--features", "emit"]);
    let found = records(&bins, "basic");
    assert!(with_ids(&found, &["TC-150"]).is_empty());
    assert!(found.iter().all(|r| r.ident != "implemented"));
}

#[trace("TC-018", "FR-005-AC-2")]
#[test]
fn tc018_a_tagged_test_yields_one_record_with_ids_in_order() {
    let bins = build("basic", &["--features", "emit"]);
    let found = records(&bins, "basic");
    let hits = with_ids(&found, &["TC-110", "FR-110-AC-1"]);
    assert_eq!(hits.len(), 1);
    let record = hits[0];
    assert_eq!(record.ident, "t");
    assert_eq!(record.module_path, "basic::foo::bar::tests");
    assert_eq!(record.file, "src/foo/bar.rs");
    assert!(record.test && !record.parametrized);
}

#[trace("FR-005-AC-3")]
#[test]
fn only_functions_with_a_body_and_no_receiver_get_a_record() {
    let bins = build("basic", &["--features", "emit"]);
    let found = records(&bins, "basic");
    assert!(
        with_ids(&found, &["FR-900-AC-1"]).is_empty(),
        "struct got a record"
    );
    for (ids, ident) in [(["TC-140"], "assoc"), (["TC-144"], "plain")] {
        let hits = with_ids(&found, &ids);
        assert_eq!(hits.len(), 1, "{ident}");
        assert_eq!(hits[0].ident, ident);
        assert!(!hits[0].test, "{ident} has no test attribute");
    }
}

#[trace("FR-005-AC-13")]
#[test]
fn a_method_with_a_receiver_and_a_bodyless_declaration_get_no_record() {
    let bins = build("basic", &["--features", "emit"]);
    let found = records(&bins, "basic");
    for id in ["TC-141", "TC-142", "TC-143"] {
        assert!(with_ids(&found, &[id]).is_empty(), "{id} got a record");
    }
}

#[trace("FR-005-AC-8")]
#[test]
fn an_untagged_test_has_no_record_and_the_listing_is_unchanged() {
    let on = build("basic", &["--features", "emit"]);
    let off = build("basic", &[]);
    assert!(records(&on, "basic").iter().all(|r| r.ident != "untagged"));
    assert_eq!(list(binary(&on, "basic")), list(binary(&off, "basic")));
}

#[trace("TC-019", "FR-005-AC-10")]
#[test]
fn tc019_every_tagged_test_keeps_its_record_through_linking() {
    let bins = build("basic", &["--features", "emit"]);
    let found = records(&bins, "basic");
    let idents: Vec<&str> = found.iter().map(|r| r.ident.as_str()).collect();
    for ident in [
        "passes",
        "panics",
        "ignored",
        "should_panic_passes",
        "slow_a",
        "slow_b",
        "forges_stdout",
        "test_above_trace",
        "plain_tagged",
        "t",
    ] {
        assert!(idents.contains(&ident), "no record for {ident}: {idents:?}");
    }
    assert_eq!(
        read_records(&binary(&bins, "x").executable)
            .unwrap()
            .unwrap()
            .len(),
        1
    );

    // The section carries the ELF name on Linux. macOS and Windows MSVC are
    // not available to this suite and are not claimed.
    if cfg!(target_os = "linux") {
        let data = std::fs::read(&binary(&bins, "basic").executable).unwrap();
        let file = object::File::parse(&*data).unwrap();
        assert!(file.sections().any(|s| s.name() == Ok("ixtrace")));
    }
}

#[trace("FR-005-AC-9")]
#[test]
fn a_package_that_never_opted_in_still_builds_and_passes_under_unification() {
    let out = Command::new("cargo")
        .args(["test", "--workspace", "--manifest-path"])
        .arg(manifest("unified"))
        .env("CARGO_TARGET_DIR", target_dir())
        .output()
        .expect("cargo starts");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[trace("FR-005-AC-12")]
#[test]
fn an_unsupported_target_compiles_and_carries_no_record() {
    let libdir = Command::new("rustc")
        .args([
            "--print",
            "target-libdir",
            "--target",
            "wasm32-unknown-unknown",
        ])
        .output()
        .expect("rustc starts");
    let installed = String::from_utf8_lossy(&libdir.stdout);
    if !std::path::Path::new(installed.trim()).exists() {
        eprintln!("SKIPPED: wasm32-unknown-unknown is not installed");
        return;
    }
    let bins = build(
        "basic",
        &["--features", "emit", "--target", "wasm32-unknown-unknown"],
    );
    let wasm = std::fs::read(&binary(&bins, "basic").executable).unwrap();
    let needle = [b"\x02IXTR".as_slice(), b"1"].concat();
    assert!(
        !wasm.windows(needle.len()).any(|w| w == needle),
        "wasm binary carries a record"
    );
}
