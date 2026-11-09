//! Locating the test binaries of a build and running them the way
//! `cargo test` does (FR-006).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::Error;

/// A test binary and where its sources live.
#[derive(Clone, Debug)]
pub struct TestBinary {
    /// The compiled test executable, read for its trace records.
    pub executable: PathBuf,
    /// The target name.
    pub target_name: String,
    /// The target kind cargo reports first: `lib`, `bin`, `test`, `bench`...
    pub kind: String,
    /// The cargo package id, accepted by `--package`.
    pub package_id: String,
    /// The package manifest.
    pub manifest_path: PathBuf,
    /// The target's root source file, as cargo reports `src_path`.
    pub root_file: PathBuf,
}

fn cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

/// Build the tests with `cargo test --no-run` and list the test binaries.
///
/// # Errors
///
/// Returns a cargo [`Error`] when cargo cannot run or the build fails.
pub fn build_tests(cargo_args: &[String]) -> Result<Vec<TestBinary>, Error> {
    let output = cargo()
        .args(["test", "--no-run", "--message-format", "json"])
        .args(cargo_args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| Error::io("cannot run cargo", err))?;
    if !output.status.success() {
        return Err(Error::cargo(format!(
            "`cargo test --no-run` failed: {}",
            output.status
        )));
    }
    Ok(parse_messages(&String::from_utf8_lossy(&output.stdout)))
}

/// Read the test executables out of cargo's JSON messages.
#[must_use]
pub fn parse_messages(stdout: &str) -> Vec<TestBinary> {
    let mut binaries = Vec::new();
    for line in stdout.lines() {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if message["reason"] != "compiler-artifact" || message["profile"]["test"] != true {
            continue;
        }
        let (Some(executable), Some(name), Some(kind), Some(src), Some(manifest), Some(id)) = (
            message["executable"].as_str(),
            message["target"]["name"].as_str(),
            message["target"]["kind"][0].as_str(),
            message["target"]["src_path"].as_str(),
            message["manifest_path"].as_str(),
            message["package_id"].as_str(),
        ) else {
            continue;
        };
        binaries.push(TestBinary {
            executable: PathBuf::from(executable),
            target_name: name.to_owned(),
            kind: kind.to_owned(),
            package_id: id.to_owned(),
            manifest_path: PathBuf::from(manifest),
            root_file: PathBuf::from(src),
        });
    }
    binaries
}

impl TestBinary {
    /// The `cargo test` selector for this one target.
    fn selector(&self) -> Vec<String> {
        let flag = match self.kind.as_str() {
            "bin" => "--bin",
            "test" => "--test",
            "bench" => "--bench",
            "example" => "--example",
            _ => return vec!["--lib".to_owned()],
        };
        vec![flag.to_owned(), self.target_name.clone()]
    }

    /// Whether the manifest declares `harness = false` for this target. Such
    /// a binary has its own `main` and does not understand libtest flags.
    #[must_use]
    pub fn harness_disabled(&self) -> bool {
        let Ok(text) = std::fs::read_to_string(&self.manifest_path) else {
            return false;
        };
        let Ok(manifest) = text.parse::<toml::Table>() else {
            return false;
        };
        let table = match self.kind.as_str() {
            "bin" => "bin",
            "test" => "test",
            "bench" => "bench",
            "example" => "example",
            _ => "lib",
        };
        let entries: Vec<&toml::Table> = match manifest.get(table) {
            Some(toml::Value::Array(list)) => {
                list.iter().filter_map(toml::Value::as_table).collect()
            }
            Some(toml::Value::Table(one)) => vec![one],
            _ => Vec::new(),
        };
        entries.iter().any(|entry| {
            let named = entry.get("name").and_then(toml::Value::as_str);
            let applies = table == "lib" || named == Some(self.target_name.as_str());
            applies && entry.get("harness").and_then(toml::Value::as_bool) == Some(false)
        })
    }

    /// A `cargo test` command that runs only this target, with `cargo_args`
    /// minus their target selectors, and `harness_args` after `--`. Cargo gives
    /// the process the environment and runner `cargo test` gives it.
    #[must_use]
    pub fn command(&self, cargo_args: &[String], harness_args: &[&str]) -> Command {
        let mut command = cargo();
        command
            .arg("test")
            .args(without_selectors(cargo_args))
            .args(["--package", &self.package_id])
            .args(self.selector())
            .arg("--")
            .args(harness_args);
        command
    }
}

/// `cargo_args` without the flags that choose packages or targets, which this
/// runner chooses itself.
#[must_use]
pub fn without_selectors(cargo_args: &[String]) -> Vec<String> {
    const WITH_VALUE: [&str; 7] = [
        "-p",
        "--package",
        "--exclude",
        "--bin",
        "--test",
        "--example",
        "--bench",
    ];
    const ALONE: [&str; 9] = [
        "--workspace",
        "--all",
        "--lib",
        "--bins",
        "--tests",
        "--examples",
        "--benches",
        "--all-targets",
        "--doc",
    ];
    let mut kept = Vec::new();
    let mut args = cargo_args.iter();
    while let Some(arg) = args.next() {
        if ALONE.contains(&arg.as_str()) {
            continue;
        }
        if WITH_VALUE.contains(&arg.as_str()) {
            args.next();
            continue;
        }
        let name = arg.split('=').next().unwrap_or(arg);
        if arg.contains('=') && WITH_VALUE.contains(&name) {
            continue;
        }
        kept.push(arg.clone());
    }
    kept
}

/// The workspace root: the directory of the workspace manifest.
///
/// # Errors
///
/// Returns a cargo [`Error`] when cargo cannot locate the workspace.
pub fn workspace_root(cargo_args: &[String]) -> Result<PathBuf, Error> {
    let mut command = cargo();
    command.args(["locate-project", "--workspace", "--message-format", "plain"]);
    let mut args = cargo_args.iter();
    while let Some(arg) = args.next() {
        if arg == "--manifest-path" {
            if let Some(path) = args.next() {
                command.args(["--manifest-path", path]);
            }
        } else if arg.starts_with("--manifest-path=") {
            command.arg(arg);
        }
    }
    let output = command
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| Error::io("cannot run cargo", err))?;
    if !output.status.success() {
        return Err(Error::cargo("`cargo locate-project` failed"));
    }
    let manifest = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    manifest
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| Error::cargo("cargo reported no workspace manifest"))
}
