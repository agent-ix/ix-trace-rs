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
    /// The cargo package id, as in `cargo metadata`.
    pub package_id: String,
    /// The package manifest.
    pub manifest_path: PathBuf,
    /// The target's root source file, as cargo reports `src_path`.
    pub root_file: PathBuf,
}

fn cargo() -> Command {
    Command::new(std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
}

/// The result of `cargo test --no-run`: the binaries, and what cargo's build
/// messages say the binaries need to run.
#[derive(Debug, Default)]
pub struct Build {
    /// The test binaries.
    pub binaries: Vec<TestBinary>,
    /// Native library directories the build scripts linked against.
    pub native_dirs: Vec<PathBuf>,
}

/// Build the tests with `cargo test --no-run` and list the test binaries.
///
/// # Errors
///
/// Returns a cargo [`Error`] when cargo cannot run or the build fails.
pub fn build_tests(cargo_args: &[String]) -> Result<Build, Error> {
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
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(Build {
        binaries: parse_messages(&stdout),
        native_dirs: parse_native_dirs(&stdout),
    })
}

/// The `native=` and `dependency=` library paths of the build scripts.
#[must_use]
pub fn parse_native_dirs(stdout: &str) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for line in stdout.lines() {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if message["reason"] != "build-script-executed" {
            continue;
        }
        for path in message["linked_paths"].as_array().into_iter().flatten() {
            let Some(path) = path.as_str() else { continue };
            let dir = path.split_once('=').map_or(path, |(_, dir)| dir);
            let dir = PathBuf::from(dir);
            if !dirs.contains(&dir) {
                dirs.push(dir);
            }
        }
    }
    dirs
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

/// The process environment `cargo test` gives a test binary, rebuilt from the
/// build messages and `cargo metadata` so the binary that was inspected is the
/// binary that runs.
#[derive(Debug)]
pub struct Environment {
    cargo: String,
    library_dirs: Vec<PathBuf>,
    packages: Vec<serde_json::Value>,
}

impl Environment {
    /// Gather the library directories and package metadata for `build`.
    ///
    /// # Errors
    ///
    /// Returns a cargo [`Error`] when a target runner is configured, which this
    /// runner does not support, or when cargo or rustc cannot be queried.
    pub fn gather(cargo_args: &[String], build: &Build) -> Result<Self, Error> {
        reject_target_runner(cargo_args)?;
        let mut library_dirs: Vec<PathBuf> = Vec::new();
        for binary in &build.binaries {
            if let Some(deps) = binary.executable.parent() {
                let deps = deps.to_path_buf();
                if !library_dirs.contains(&deps) {
                    library_dirs.push(deps);
                }
            }
        }
        let rustc = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let mut libdir = Command::new(&rustc);
        libdir.args(["--print", "target-libdir"]);
        if let Some(target) = flag_value(cargo_args, "--target") {
            libdir.args(["--target", &target]);
        }
        let host = Command::new(&rustc)
            .args(["--print", "target-libdir"])
            .output()
            .map_err(|err| Error::io("cannot run rustc", err))?;
        let target = libdir
            .output()
            .map_err(|err| Error::io("cannot run rustc", err))?;
        for out in [target, host] {
            let dir = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
            if !dir.as_os_str().is_empty() && !library_dirs.contains(&dir) {
                library_dirs.push(dir);
            }
        }
        for dir in &build.native_dirs {
            if !library_dirs.contains(dir) {
                library_dirs.push(dir.clone());
            }
        }

        let mut metadata = cargo();
        metadata.args(["metadata", "--no-deps", "--format-version", "1"]);
        if let Some(path) = flag_value(cargo_args, "--manifest-path") {
            metadata.args(["--manifest-path", &path]);
        }
        let output = metadata
            .stderr(Stdio::inherit())
            .output()
            .map_err(|err| Error::io("cannot run cargo", err))?;
        if !output.status.success() {
            return Err(Error::cargo("`cargo metadata` failed"));
        }
        let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
            .map_err(|err| Error::cargo(format!("unreadable `cargo metadata`: {err}")))?;
        let packages = parsed["packages"].as_array().cloned().unwrap_or_default();
        Ok(Self {
            cargo: cargo_program(),
            library_dirs,
            packages,
        })
    }

    /// The test binary as a command: the package root as its directory, the
    /// variables cargo sets for the owning package, and the library search path.
    #[must_use]
    pub fn command(&self, binary: &TestBinary, harness_args: &[&str]) -> Command {
        let package_dir = binary
            .manifest_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let mut command = Command::new(&binary.executable);
        command
            .args(harness_args)
            .current_dir(&package_dir)
            .env("CARGO", &self.cargo)
            .env("CARGO_MANIFEST_DIR", &package_dir)
            .env("CARGO_CRATE_NAME", binary.target_name.replace('-', "_"));
        if binary.kind == "bin" {
            command.env("CARGO_BIN_NAME", &binary.target_name);
        }
        if let Some(package) = self
            .packages
            .iter()
            .find(|p| p["id"] == binary.package_id.as_str())
        {
            for (key, value) in package_variables(package) {
                command.env(key, value);
            }
        }
        let name = library_path_variable();
        let mut paths: Vec<PathBuf> = self.library_dirs.clone();
        if let Some(existing) = std::env::var_os(name) {
            paths.extend(std::env::split_paths(&existing));
        }
        if let Ok(joined) = std::env::join_paths(paths) {
            command.env(name, joined);
        }
        command
    }
}

fn cargo_program() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// The variable holding the dynamic library search path on this platform.
fn library_path_variable() -> &'static str {
    if cfg!(windows) {
        "PATH"
    } else if cfg!(target_os = "macos") {
        "DYLD_FALLBACK_LIBRARY_PATH"
    } else {
        "LD_LIBRARY_PATH"
    }
}

/// The `CARGO_PKG_*` variables cargo sets, from one `cargo metadata` package.
fn package_variables(package: &serde_json::Value) -> Vec<(String, String)> {
    let text = |key: &str| package[key].as_str().unwrap_or("").to_owned();
    let version = text("version");
    let (core, pre) = version
        .split_once('-')
        .map_or((version.as_str(), ""), |(c, p)| (c, p));
    let core = core.split_once('+').map_or(core, |(c, _)| c);
    let mut parts = core.split('.');
    let mut part = || parts.next().unwrap_or("").to_owned();
    let (major, minor, patch) = (part(), part(), part());
    let authors = package["authors"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(":")
        })
        .unwrap_or_default();
    let variables = vec![
        ("CARGO_PKG_NAME", text("name")),
        ("CARGO_PKG_VERSION", version.clone()),
        ("CARGO_PKG_VERSION_MAJOR", major),
        ("CARGO_PKG_VERSION_MINOR", minor),
        ("CARGO_PKG_VERSION_PATCH", patch),
        ("CARGO_PKG_VERSION_PRE", pre.to_owned()),
        ("CARGO_PKG_AUTHORS", authors),
        ("CARGO_PKG_DESCRIPTION", text("description")),
        ("CARGO_PKG_HOMEPAGE", text("homepage")),
        ("CARGO_PKG_REPOSITORY", text("repository")),
        ("CARGO_PKG_LICENSE", text("license")),
        ("CARGO_PKG_LICENSE_FILE", text("license_file")),
        ("CARGO_PKG_README", text("readme")),
        ("CARGO_PKG_RUST_VERSION", text("rust_version")),
    ];
    variables
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect()
}

/// The value of `--flag value` or `--flag=value` among `args`.
fn flag_value(args: &[String], flag: &str) -> Option<String> {
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next().cloned();
        }
        if let Some(value) = arg.strip_prefix(&format!("{flag}=")) {
            return Some(value.to_owned());
        }
    }
    None
}

/// Fail when a target runner is configured: `cargo test` would run the binary
/// through it, and this runner executes the binary directly.
fn reject_target_runner(cargo_args: &[String]) -> Result<(), Error> {
    let unsupported = || {
        Error::cargo(
            "a target runner (`target.<triple>.runner` in cargo config or a \
             CARGO_TARGET_<TRIPLE>_RUNNER variable) is configured; ix-trace-run runs test \
             binaries directly and does not support target runners",
        )
    };
    if std::env::vars_os().any(|(key, _)| {
        let key = key.to_string_lossy();
        key.starts_with("CARGO_TARGET_") && key.ends_with("_RUNNER")
    }) {
        return Err(unsupported());
    }
    let mut args = cargo_args.iter();
    while let Some(arg) = args.next() {
        let value = if arg == "--config" {
            args.next()
        } else {
            arg.strip_prefix("--config=").map(|_| arg)
        };
        if value.is_some_and(|v| v.contains("runner")) {
            return Err(unsupported());
        }
    }
    let mut dirs: Vec<PathBuf> = Vec::new();
    let start = flag_value(cargo_args, "--manifest-path")
        .map(PathBuf::from)
        .and_then(|p| std::fs::canonicalize(p).ok())
        .and_then(|p| p.parent().map(Path::to_path_buf))
        .or_else(|| std::env::current_dir().ok());
    let mut dir = start;
    while let Some(current) = dir {
        dirs.push(current.join(".cargo"));
        dir = current.parent().map(Path::to_path_buf);
    }
    if let Some(home) = std::env::var_os("CARGO_HOME") {
        dirs.push(PathBuf::from(home));
    } else if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".cargo"));
    }
    for dir in dirs {
        for name in ["config.toml", "config"] {
            let Ok(text) = std::fs::read_to_string(dir.join(name)) else {
                continue;
            };
            let Ok(config) = text.parse::<toml::Table>() else {
                continue;
            };
            let has_runner = config
                .get("target")
                .and_then(toml::Value::as_table)
                .is_some_and(|targets| {
                    targets
                        .values()
                        .any(|t| t.as_table().is_some_and(|t| t.contains_key("runner")))
                });
            if has_runner {
                return Err(unsupported());
            }
        }
    }
    Ok(())
}

impl TestBinary {
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
