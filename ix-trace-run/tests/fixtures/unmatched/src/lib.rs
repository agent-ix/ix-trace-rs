//! Fixture: records that belong to no listed test, and a libtest-name clash.

use ix_trace_rs::trace;

#[path = "../other/outside.rs"]
mod outside;

#[test]
fn host() {
    #[trace("TC-400")]
    #[test]
    fn never_collected() {}
}

#[trace("TC-401")]
#[test]
fn fine() {}

pub struct Checker;

impl Checker {
    #[trace("TC-410")]
    pub fn check() {}
}

#[trace("TC-412")]
#[test]
fn check() {}
