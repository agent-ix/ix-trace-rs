//! Fixture: `#[rstest]` above a path-qualified `#[trace]`, which rstest copies
//! onto each case. Every case passes.

use rstest::rstest;

#[rstest]
#[case(1)]
#[case(2)]
#[ix_trace_rs::trace("TC-204")]
fn misuse(#[case] _n: u32) {}
