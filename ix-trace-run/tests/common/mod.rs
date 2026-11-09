//! Helpers shared by the fixture-driven tests: build a fixture crate, run the
//! runner over it, and pick cases out of the JUnit text.

#![allow(dead_code)]

use std::path::PathBuf;
use std::process::Command;

use ix_trace_run::cargo::{parse_messages, TestBinary};

/// The manifest of fixture `name`.
pub fn manifest(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .join("Cargo.toml")
}

/// One build directory for every fixture, so shared dependencies compile once.
pub fn target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("fixtures-target")
}

/// What one runner invocation produced.
pub struct Run {
    pub xml: String,
    pub stderr: String,
    pub code: Option<i32>,
    /// The directory the result was written to; it must hold nothing else.
    pub dir: tempfile::TempDir,
}

/// Run `ix-trace-run` over fixture `name`.
pub fn runner(name: &str, cargo_args: &[&str], test_args: &[&str]) -> Run {
    let dir = tempfile::tempdir().expect("temp dir");
    let output = dir.path().join("result.xml");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ix-trace-run"));
    command
        .arg("--output")
        .arg(&output)
        .arg("--manifest-path")
        .arg(manifest(name))
        .args(cargo_args)
        .env("CARGO_TARGET_DIR", target_dir());
    if !test_args.is_empty() {
        command.arg("--").args(test_args);
    }
    let out = command.output().expect("runner starts");
    Run {
        xml: std::fs::read_to_string(&output).unwrap_or_default(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code(),
        dir,
    }
}

/// `cargo test --no-run` over fixture `name`, returning its test binaries.
pub fn build(name: &str, cargo_args: &[&str]) -> Vec<TestBinary> {
    let out = Command::new("cargo")
        .args([
            "test",
            "--no-run",
            "--message-format",
            "json",
            "--manifest-path",
        ])
        .arg(manifest(name))
        .args(cargo_args)
        .env("CARGO_TARGET_DIR", target_dir())
        .output()
        .expect("cargo starts");
    assert!(
        out.status.success(),
        "fixture {name} must build: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    parse_messages(&String::from_utf8_lossy(&out.stdout))
}

/// The binary built from the target named `target`.
pub fn binary<'a>(binaries: &'a [TestBinary], target: &str) -> &'a TestBinary {
    binaries
        .iter()
        .find(|b| b.target_name == target)
        .unwrap_or_else(|| panic!("no test binary for target {target}"))
}

/// Every `<testcase>` element of `xml`, as text. The runner writes each
/// element either on one line or between a `<testcase ...>` line and a
/// `</testcase>` line.
pub fn cases(xml: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut open: Option<String> = None;
    for line in xml.lines() {
        match open.as_mut() {
            Some(text) => {
                text.push_str(line);
                if line.contains("</testcase>") {
                    found.extend(open.take());
                }
            }
            None if line.contains("<testcase ") => {
                if line.trim_end().ends_with("/>") {
                    found.push(line.to_owned());
                } else {
                    open = Some(line.to_owned());
                }
            }
            None => {}
        }
    }
    found
}

/// The `<testcase>` elements whose `name` is `name`.
pub fn cases_named(xml: &str, name: &str) -> Vec<String> {
    let needle = format!("name=\"{name}\"");
    cases(xml)
        .into_iter()
        .filter(|c| c.contains(&needle))
        .collect()
}

/// The single `<testcase>` named `name`.
pub fn case(xml: &str, name: &str) -> String {
    let mut found = cases_named(xml, name);
    assert_eq!(found.len(), 1, "expected one case named {name} in:\n{xml}");
    found.remove(0)
}
