//! `ix-trace-run`: run a cargo test suite and write JUnit XML in which each
//! case carries the `trace` property of the ids recorded by `ix-trace-rs` with
//! its `emit` feature (FR-005, FR-006).

pub mod cargo;
pub mod junit;
pub mod logfile;
pub mod matching;
pub mod registry;
pub mod run;

use std::fmt;

/// Why a run could not produce a result file.
#[derive(Debug)]
pub enum Error {
    /// Cargo could not build the tests or locate the workspace.
    Cargo(String),
    /// A file operation failed.
    Io {
        /// What was being done.
        context: String,
        /// The underlying error.
        source: std::io::Error,
    },
    /// The command line is not understood.
    Usage(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Cargo(message) | Self::Usage(message) => f.write_str(message),
            Self::Io { context, source } => write!(f, "{context}: {source}"),
        }
    }
}

impl std::error::Error for Error {}
