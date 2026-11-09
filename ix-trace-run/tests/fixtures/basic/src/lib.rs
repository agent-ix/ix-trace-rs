//! Fixture for the `ix-trace-run` tests: outcomes, naming and record shapes.

pub mod foo;

use std::time::Duration;

use ix_trace_rs::{implements, trace};

#[trace("TC-101", "FR-101-AC-1")]
#[test]
fn passes() {}

#[trace("TC-102")]
#[test]
fn panics() {
    panic!("boom");
}

#[trace("TC-103")]
#[test]
#[ignore]
fn ignored() {}

#[trace("TC-104")]
#[test]
#[should_panic]
fn should_panic_passes() {
    panic!("expected");
}

#[trace("TC-105")]
#[test]
fn slow_a() {
    std::thread::sleep(Duration::from_millis(200));
}

#[trace("TC-106")]
#[test]
fn slow_b() {
    std::thread::sleep(Duration::from_millis(200));
}

#[trace("TC-107")]
#[test]
fn forges_stdout() {
    println!("test other ... ok");
    println!("test self ... FAILED");
    eprintln!("ok forges_stdout");
}

#[test]
fn untagged() {}

#[test]
#[ignore = "line one\nline two"]
fn multiline_ignore() {}

#[test]
fn after_multiline() {}

#[test]
#[trace("TC-108")]
fn test_above_trace() {}

pub mod plain_tagged {
    #[test]
    fn untagged_inside() {}
}

#[trace("TC-109")]
#[test]
fn plain_tagged() {}

#[trace("FR-900-AC-1")]
pub struct Marked;

pub struct Holder;

impl Holder {
    #[trace("TC-140")]
    pub fn assoc() -> u8 {
        1
    }

    #[trace("TC-141")]
    pub fn method(&self) -> u8 {
        2
    }
}

pub trait Declared {
    #[trace("TC-142")]
    fn declared(&self);

    #[trace("TC-143")]
    fn declared_static();
}

#[trace("TC-144")]
pub fn plain() {}

#[implements("TC-150")]
pub fn implemented() {}
