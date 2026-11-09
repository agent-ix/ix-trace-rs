//! `ix-trace-run`: run a cargo test suite and write JUnit XML in which each
//! case carries the `trace` property of the ids recorded by `ix-trace-rs` with
//! its `emit` feature (FR-005, FR-006).

pub mod cargo;
pub mod junit;
pub mod logfile;
pub mod matching;
pub mod registry;
pub mod run;

/// What kind of failure an [`Error`] is. Callers branch on this, never on the
/// message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorCode {
    /// Cargo could not build the tests or locate the workspace.
    Cargo,
    /// A file operation or a process start failed.
    Io,
    /// The command line is not understood.
    Usage,
}

/// Why a run could not produce a result file.
#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct Error {
    /// The failure kind.
    pub code: ErrorCode,
    message: String,
    #[source]
    source: Option<std::io::Error>,
}

impl Error {
    /// A cargo failure.
    #[must_use]
    pub fn cargo(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Cargo,
            message: message.into(),
            source: None,
        }
    }

    /// A command-line mistake.
    #[must_use]
    pub fn usage(message: impl Into<String>) -> Self {
        Self {
            code: ErrorCode::Usage,
            message: message.into(),
            source: None,
        }
    }

    /// An I/O failure while doing `context`.
    #[must_use]
    pub fn io(context: impl std::fmt::Display, source: std::io::Error) -> Self {
        Self {
            code: ErrorCode::Io,
            message: format!("{context}: {source}"),
            source: Some(source),
        }
    }
}
