//! Running the suite and writing the result file (FR-006).

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use crate::cargo::{self, TestBinary};
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
    /// Returns [`Error::Usage`] when `--output` has no value.
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
                    .ok_or_else(|| Error::Usage(format!("{arg} needs a path")))?;
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
    let binaries = cargo::build_tests(&options.cargo_args)?;
    let workspace_root = cargo::workspace_root(&options.cargo_args)?;
    let mut clean = true;
    let mut suites = Vec::new();
    for binary in &binaries {
        let layout = Layout {
            workspace_root: workspace_root.clone(),
            root_file: binary.root_file.clone(),
        };
        let (suite, binary_clean) = run_binary(binary, &layout, &options.test_args)?;
        clean &= binary_clean;
        suites.extend(suite);
    }
    clean &= suites
        .iter()
        .all(|suite| suite.cases.iter().all(|c| c.outcome != Outcome::Fail));
    std::fs::write(&options.output, junit::render(&suites)).map_err(|source| Error::Io {
        context: format!("cannot write {}", options.output.display()),
        source,
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
    test_args: &[String],
) -> Result<(Option<Suite>, bool), Error> {
    let name = binary.executable.display();
    let records = load_records(binary);

    let Some(all) = list(binary, &[])? else {
        // Not a libtest binary: run it as cargo would and keep its status.
        let status = command(binary)
            .args(test_args)
            .status()
            .map_err(|source| Error::Io {
                context: format!("cannot run {name}"),
                source,
            })?;
        return Ok((None, status.success()));
    };
    let selected = if test_args.is_empty() {
        all.clone()
    } else {
        list(binary, test_args)?.unwrap_or_default()
    };

    let mut clean = true;
    let mut cases_by_name = Vec::new();
    if !selected.is_empty() {
        let (outcomes, ok) = execute(binary, &selected, test_args)?;
        clean &= ok;
        cases_by_name = outcomes;
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

    let cases = cases_by_name
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
        Ok(Some(records)) => records,
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

fn command(binary: &TestBinary) -> Command {
    let mut command = Command::new(&binary.executable);
    command
        .current_dir(&binary.package_dir)
        .env("CARGO_MANIFEST_DIR", &binary.package_dir);
    command
}

/// The names `--list` prints, or `None` when the binary is not a libtest one.
fn list(binary: &TestBinary, extra: &[String]) -> Result<Option<Vec<String>>, Error> {
    let output = command(binary)
        .arg("--list")
        .args(extra)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|source| Error::Io {
            context: format!("cannot run {}", binary.executable.display()),
            source,
        })?;
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(Some(
        text.lines()
            .filter_map(|line| line.strip_suffix(": test"))
            .map(str::to_owned)
            .collect(),
    ))
}

type Outcomes = Vec<(String, Outcome, Option<String>)>;

/// Run the binary once, then every test without a log line alone. Returns the
/// outcome of each selected test and whether the run was free of faults.
fn execute(
    binary: &TestBinary,
    selected: &[String],
    test_args: &[String],
) -> Result<(Outcomes, bool), Error> {
    let io = |context: &str| {
        let context = format!("{context} {}", binary.executable.display());
        move |source| Error::Io { context, source }
    };
    let dir = tempfile::tempdir().map_err(io("cannot create a log directory for"))?;
    let log_path = dir.path().join("libtest.log");
    let status = command(binary)
        .arg("--logfile")
        .arg(&log_path)
        .args(test_args)
        .status()
        .map_err(io("cannot run"))?;
    let text = std::fs::read(&log_path).map(|b| String::from_utf8_lossy(&b).into_owned());
    let log = logfile::parse(&text.unwrap_or_default(), selected);

    let mut clean = true;
    let missing: Vec<&String> = selected
        .iter()
        .filter(|name| !log.outcomes.contains_key(*name))
        .collect();
    if status.success() && log.saw_failed {
        clean = false;
        notice(&format!(
            "{} exited 0 but its log has a failed test; the log's outcomes are recorded",
            binary.executable.display()
        ));
    } else if !status.success()
        && !log.saw_failed
        && missing.is_empty()
        && log.duplicates.is_empty()
    {
        clean = false;
        notice(&format!(
            "{} exited with {status} but its log has no failed test; the log's outcomes are recorded",
            binary.executable.display()
        ));
    }

    let rerun_args: Vec<&String> = test_args
        .iter()
        .filter(|arg| matches!(arg.as_str(), "--include-ignored" | "--ignored"))
        .collect();
    let mut outcomes: Outcomes = Vec::with_capacity(selected.len());
    for name in selected {
        if log.duplicates.contains(name) {
            clean = false;
            outcomes.push((
                name.clone(),
                Outcome::Fail,
                Some("duplicate log line".to_owned()),
            ));
        } else if let Some(&outcome) = log.outcomes.get(name) {
            outcomes.push((name.clone(), outcome, None));
        } else {
            let (outcome, message) = rerun(binary, name, &rerun_args, dir.path())?;
            outcomes.push((name.clone(), outcome, message));
        }
    }
    Ok((outcomes, clean))
}

/// Run one test alone in a fresh process.
fn rerun(
    binary: &TestBinary,
    name: &str,
    extra: &[&String],
    dir: &Path,
) -> Result<(Outcome, Option<String>), Error> {
    let log_path = dir.join("rerun.log");
    let status: ExitStatus = command(binary)
        .args(["--exact", name, "--logfile"])
        .arg(&log_path)
        .args(extra)
        .status()
        .map_err(|source| Error::Io {
            context: format!("cannot run {}", binary.executable.display()),
            source,
        })?;
    if !status.success() {
        return Ok((
            Outcome::Fail,
            Some(format!("ran alone after an abnormal exit: {status}")),
        ));
    }
    let text = std::fs::read(&log_path).map(|b| String::from_utf8_lossy(&b).into_owned());
    let log = logfile::parse(&text.unwrap_or_default(), &[name.to_owned()]);
    let outcome = log.outcomes.get(name).copied().unwrap_or(Outcome::Pass);
    Ok((
        if outcome == Outcome::Skip {
            Outcome::Skip
        } else {
            Outcome::Pass
        },
        None,
    ))
}
