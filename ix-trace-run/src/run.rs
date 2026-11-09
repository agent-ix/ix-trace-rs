//! Running the suite and writing the result file (FR-006).

use std::path::{Path, PathBuf};

use crate::cargo::{self, Environment, TestBinary};
use crate::junit::{self, Case, Suite};
use crate::logfile::{self, Outcome};
use crate::matching::{match_records, Layout};
use crate::registry::{self, Record};
use crate::Error;

/// The command line: `ix-trace-run [--output FILE] [CARGO ARGS] [-- TEST ARGS]`.
#[derive(Debug, Eq, PartialEq)]
pub struct Options {
    /// The result file.
    pub output: PathBuf,
    /// Arguments for `cargo test --no-run`.
    pub cargo_args: Vec<String>,
    /// Arguments for each test binary, after `--`.
    pub test_args: Vec<String>,
}

impl Options {
    /// Parse the arguments after the program name.
    ///
    /// # Errors
    ///
    /// Returns a usage [`Error`] when `--output` has no value.
    pub fn parse(args: &[String]) -> Result<Self, Error> {
        let mut output = PathBuf::from("junit.xml");
        let mut cargo_args = Vec::new();
        let mut test_args = Vec::new();
        let mut iter = args.iter();
        while let Some(arg) = iter.next() {
            if arg == "--" {
                test_args = iter.by_ref().cloned().collect();
            } else if arg == "--output" || arg == "-o" {
                let value = iter
                    .next()
                    .ok_or_else(|| Error::usage(format!("{arg} needs a path")))?;
                output = PathBuf::from(value);
            } else if let Some(value) = arg.strip_prefix("--output=") {
                output = PathBuf::from(value);
            } else {
                cargo_args.push(arg.clone());
            }
        }
        Ok(Self {
            output,
            cargo_args,
            test_args,
        })
    }
}

/// Build, run and write the result file. Returns whether the run is clean:
/// every test passed or was skipped and nothing was reported on stderr as a
/// fault.
///
/// # Errors
///
/// Returns an [`Error`] when the build fails or the result file cannot be
/// written.
pub fn run(options: &Options) -> Result<bool, Error> {
    let build = cargo::build_tests(&options.cargo_args)?;
    let environment = Environment::gather(&options.cargo_args, &build)?;
    let binaries = build.binaries;
    let workspace_root = cargo::workspace_root(&options.cargo_args)?;
    let mut clean = true;
    let mut suites = Vec::new();
    for binary in &binaries {
        let layout = Layout {
            workspace_root: workspace_root.clone(),
            root_file: binary.root_file.clone(),
        };
        let (suite, binary_clean) = run_binary(binary, &layout, options, &environment)?;
        clean &= binary_clean;
        suites.extend(suite);
    }
    clean &= suites
        .iter()
        .all(|suite| suite.cases.iter().all(|c| c.outcome != Outcome::Fail));
    std::fs::write(&options.output, junit::render(&suites)).map_err(|source| {
        Error::io(format!("cannot write {}", options.output.display()), source)
    })?;
    Ok(clean)
}

fn notice(message: &str) {
    eprintln!("ix-trace-run: {message}");
}

/// Run one test binary once and build its suite. The flag is false when the
/// run reported a fault beyond a failing test.
fn run_binary(
    binary: &TestBinary,
    layout: &Layout,
    options: &Options,
    env: &Environment,
) -> Result<(Option<Suite>, bool), Error> {
    let name = binary.executable.display();
    let test_args: Vec<&str> = options.test_args.iter().map(String::as_str).collect();
    if binary.harness_disabled() {
        // Its `main` is not libtest: running it with `--list` would run it.
        notice(&format!(
            "notice: {name} sets harness = false; it is run as is and has no cases"
        ));
        let status = env
            .command(binary, &test_args)
            .status()
            .map_err(|source| Error::io(format!("cannot run {name}"), source))?;
        return Ok((None, status.success()));
    }

    let records = load_records(binary);
    let all = match list(binary, env, &[]) {
        Ok(all) => all,
        Err(detail) => {
            notice(&format!(
                "error: {name} cannot list its tests, so none of them ran: {detail}"
            ));
            return Ok((None, false));
        }
    };
    let selected = if test_args.is_empty() {
        all.clone()
    } else {
        match list(binary, env, &test_args) {
            Ok(selected) => selected,
            Err(detail) => {
                notice(&format!(
                    "error: {name} rejected the test arguments `{}` given with `--list`: {detail}",
                    test_args.join(" ")
                ));
                return Ok((None, false));
            }
        }
    };

    let mut clean = true;
    let mut outcomes = Vec::new();
    if !selected.is_empty() {
        let (found, ok) = execute(binary, &selected, options, env)?;
        clean &= ok;
        outcomes = found;
    }

    let matching = match_records(&records, &all, layout);
    for warning in &matching.warnings {
        notice(&format!("warning: {warning}"));
    }
    for group in &matching.ambiguities {
        clean = false;
        notice(&format!(
            "ambiguous trace records for libtest name `{}`; none was applied: {}",
            group[0].libtest_name(),
            describe(group)
        ));
    }
    for record in &matching.unmatched {
        clean = false;
        notice(&format!(
            "trace record belongs to no listed test: {}",
            describe(std::slice::from_ref(record))
        ));
    }

    let cases = outcomes
        .into_iter()
        .map(|(test, outcome, failure)| match matching.marks.get(&test) {
            Some(mark) => Case {
                name: mark.symbol.clone(),
                file: Some(mark.file.clone()),
                trace: Some(mark.ids.clone()),
                outcome,
                failure,
            },
            None => Case {
                name: test,
                file: None,
                trace: None,
                outcome,
                failure,
            },
        })
        .collect();
    Ok((
        Some(Suite {
            name: binary.target_name.clone(),
            cases,
        }),
        clean,
    ))
}

fn describe(records: &[Record]) -> String {
    records
        .iter()
        .map(|r| {
            format!(
                "{}::{} [{}] in {}",
                r.module_path,
                r.ident,
                r.ids.join(","),
                r.file
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn load_records(binary: &TestBinary) -> Vec<Record> {
    let name = binary.executable.display();
    match registry::read_records(&binary.executable) {
        Ok(Some(registry)) => {
            if registry.malformed > 0 {
                notice(&format!(
                    "warning: {name} has {} trace record(s) that could not be read (a format \
                     version mismatch between ix-trace-rs and ix-trace-run?); their tests carry no marks",
                    registry.malformed
                ));
            }
            registry.records
        }
        Ok(None) => {
            notice(&format!(
                "notice: {name} has no trace section; its cases carry no marks"
            ));
            Vec::new()
        }
        Err(err) => {
            notice(&format!("notice: {name}: {err}; its cases carry no marks"));
            Vec::new()
        }
    }
}

/// The names `--list` prints, or the reason it failed.
fn list(binary: &TestBinary, env: &Environment, extra: &[&str]) -> Result<Vec<String>, String> {
    let mut args = vec!["--list"];
    args.extend_from_slice(extra);
    let output = env
        .command(binary, &args)
        .output()
        .map_err(|err| format!("cannot start cargo: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{}: {}", output.status, stderr.trim()));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text
        .lines()
        .filter_map(|line| line.strip_suffix(": test"))
        .map(str::to_owned)
        .collect())
}

type Outcomes = Vec<(String, Outcome, Option<String>)>;

/// Run the binary once, then every test without a log line alone. Returns the
/// outcome of each selected test and whether the run was free of faults.
fn execute(
    binary: &TestBinary,
    selected: &[String],
    options: &Options,
    env: &Environment,
) -> Result<(Outcomes, bool), Error> {
    let name = binary.executable.display();
    let dir = tempfile::tempdir()
        .map_err(|source| Error::io(format!("cannot create a log directory for {name}"), source))?;
    let log_path = dir.path().join("libtest.log");
    let log_arg = log_path.to_string_lossy().into_owned();
    let mut harness_args = vec!["--logfile", log_arg.as_str()];
    harness_args.extend(options.test_args.iter().map(String::as_str));
    let status = env
        .command(binary, &harness_args)
        .status()
        .map_err(|source| Error::io(format!("cannot run {name}"), source))?;
    let text = std::fs::read(&log_path).map(|b| String::from_utf8_lossy(&b).into_owned());
    let log = logfile::parse(&text.unwrap_or_default(), selected);

    let mut clean = true;
    let any_missing = selected.iter().any(|name| !log.outcomes.contains_key(name));
    if status.success() && log.saw_failed {
        clean = false;
        notice(&format!(
            "{name} exited 0 but its log has a failed test; the log's outcomes are recorded"
        ));
    } else if !status.success() && !log.saw_failed && !any_missing && log.duplicates.is_empty() {
        clean = false;
        notice(&format!(
            "{name} exited with {status} but its log has no failed test; the log's outcomes are recorded"
        ));
    }

    let rerun_flags: Vec<&str> = options
        .test_args
        .iter()
        .map(String::as_str)
        .filter(|arg| matches!(*arg, "--include-ignored" | "--ignored"))
        .collect();
    let mut outcomes: Outcomes = Vec::with_capacity(selected.len());
    for test in selected {
        if log.duplicates.contains(test) {
            clean = false;
            notice(&format!(
                "{name}: `{test}` has more than one log line; recorded as failed"
            ));
            outcomes.push((
                test.clone(),
                Outcome::Fail,
                Some("duplicate log line".to_owned()),
            ));
        } else if let Some(&outcome) = log.outcomes.get(test) {
            outcomes.push((test.clone(), outcome, None));
        } else {
            let (outcome, message) = rerun(binary, test, env, &rerun_flags, dir.path())?;
            outcomes.push((test.clone(), outcome, message));
        }
    }
    Ok((outcomes, clean))
}

/// Run one test alone in a fresh process. Its outcome is what that run's log
/// says; a run that writes no log line for the test, exits non-zero or dies is
/// a failure, never a pass.
fn rerun(
    binary: &TestBinary,
    test: &str,
    env: &Environment,
    flags: &[&str],
    dir: &Path,
) -> Result<(Outcome, Option<String>), Error> {
    let log_path = dir.join("rerun.log");
    // A stale file from an earlier rerun must not answer for this one.
    let _ = std::fs::remove_file(&log_path);
    let log_arg = log_path.to_string_lossy().into_owned();
    let mut args = vec!["--exact", test, "--logfile", log_arg.as_str()];
    args.extend_from_slice(flags);
    let status = env.command(binary, &args).status().map_err(|source| {
        Error::io(
            format!("cannot run {}", binary.executable.display()),
            source,
        )
    })?;
    let text = std::fs::read(&log_path).map(|b| String::from_utf8_lossy(&b).into_owned());
    let log = logfile::parse(&text.unwrap_or_default(), &[test.to_owned()]);
    if !status.success() {
        return Ok((
            Outcome::Fail,
            Some(format!("ran alone after an abnormal exit: {status}")),
        ));
    }
    match log.outcomes.get(test) {
        Some(&outcome) if !log.duplicates.contains(test) => Ok((outcome, None)),
        _ => {
            notice(&format!(
                "`{test}` exited 0 when run alone but wrote no log line, so it did not finish \
                 normally; recorded as failed"
            ));
            Ok((Outcome::Fail, Some("no log line when run alone".to_owned())))
        }
    }
}
