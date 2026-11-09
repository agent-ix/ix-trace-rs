//! Fixture: a binary that aborts and a test that exits the process.

use ix_trace_rs::trace;

#[trace("TC-300")]
#[test]
fn aborts() {
    std::process::abort();
}

#[trace("TC-301")]
#[test]
fn exits_zero() {
    std::process::exit(0);
}

#[trace("TC-302")]
#[test]
fn survivor_a() {}

#[trace("TC-303")]
#[test]
fn survivor_b() {}
