//! The rejection half of the contract.
//!
//! `tests/integration.rs` can only assert that a *well-formed* marker compiles
//! away to nothing — a malformed one is a compile error by construction, so no
//! passing test can observe it from inside the same crate. These cases run the
//! compiler and compare its diagnostic against a recorded `.stderr`, which is
//! also how the **span** is checked: a `compile_error!` whose tokens carry the
//! call site instead of the offending literal underlines the wrong thing, and
//! the recorded output would not match.

#[ix_trace_rs::trace("TC-009", "FR-003-AC-4", "FR-003-AC-5", "FR-003-AC-6", "FR-002-AC-3")]
#[test]
fn malformed_markers_are_rejected_at_compile_time() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
