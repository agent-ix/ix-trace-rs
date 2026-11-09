//! Locating the test binaries of a build (FR-006).

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::Error;

/// A test binary and where its sources live.
#[derive(Clone, Debug)]
pub struct TestBinary {
    /// The compiled test executable.
    pub executable: PathBuf,
    /// The target name.
    pub target_name: String,
    /// The target's root source file, as cargo reports `src_path`.
    pub root_file: PathBuf,
    /// The directory of the package manifest: the working directory cargo
    /// gives the binary.
    pub package_dir: PathBuf,
}

fn cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

/// Build the tests with `cargo test --no-run` and list the test binaries.
///
/// # Errors
///
/// Returns [`Error::Cargo`] when cargo cannot run or the build fails.
pub fn build_tests(cargo_args: &[String]) -> Result<Vec<TestBinary>, Error> {
    let output = cargo()
        .args(["test", "--no-run", "--message-format", "json"])
        .args(cargo_args)
        .stderr(Stdio::inherit())
        .output()
        .map_err(|err| Error::Cargo(format!("cannot run cargo: {err}")))?;
    if !output.status.success() {
        return Err(Error::Cargo(format!(
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
        let (Some(executable), Some(name), Some(src), Some(manifest)) = (
            message["executable"].as_str(),
            message["target"]["name"].as_str(),
            message["target"]["src_path"].as_str(),
            message["manifest_path"].as_str(),
        ) else {
            continue;
        };
        let manifest = PathBuf::from(manifest);
        binaries.push(TestBinary {
            executable: PathBuf::from(executable),
            target_name: name.to_owned(),
            root_file: PathBuf::from(src),
            package_dir: manifest.parent().map(PathBuf::from).unwrap_or_default(),
        });
    }
    binaries
}

/// The workspace root: the directory of the workspace manifest.
///
/// # Errors
///
/// Returns [`Error::Cargo`] when cargo cannot locate the workspace.
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
        .map_err(|err| Error::Cargo(format!("cannot run cargo: {err}")))?;
    if !output.status.success() {
        return Err(Error::Cargo("`cargo locate-project` failed".to_owned()));
    }
    let manifest = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim());
    manifest
        .parent()
        .map(PathBuf::from)
        .ok_or_else(|| Error::Cargo("cargo reported no workspace manifest".to_owned()))
}
