//! NFR-001: the crate must stay free of runtime dependencies.
//!
//! Source inspection is the only way to reach this — no runtime path can
//! observe a dependency that is not there, and the property is about the
//! manifest rather than about behaviour. It is asserted here rather than left
//! to review because the cost this bounds is paid by ~150 consuming
//! repositories, one clean build at a time.

#[ix_trace_rs::trace("TC-014", "NFR-001-AC-1", "NFR-001-AC-2")]
#[test]
fn tc014_the_crate_declares_no_runtime_dependencies() {
    let manifest = include_str!("../Cargo.toml");

    // `[dependencies]` must be present and empty. Absent would also be zero
    // dependencies, but the empty section is what makes the intent legible to
    // the next person tempted to add one.
    let (_, after) = manifest
        .split_once("\n[dependencies]")
        .expect("Cargo.toml declares a [dependencies] section");
    let body = after
        .split_once("\n[")
        .map_or(after, |(body, _)| body)
        .trim();
    assert!(
        body.is_empty(),
        "ix-trace-rs has taken a runtime dependency, which every consumer then \
         builds (NFR-001-AC-1):\n{body}"
    );

    // And the implementation must not have reached for a parsing/quoting crate
    // by another route.
    let lib = include_str!("../src/lib.rs");
    for crate_name in ["syn", "quote", "proc_macro2", "regex"] {
        assert!(
            !lib.contains(&format!("use {crate_name}")),
            "src/lib.rs imports `{crate_name}`; validation must use the \
             `proc_macro` API alone (NFR-001-AC-2)"
        );
    }
}
