//! Fixture: rstest and tokio tests under `#[trace]`.

use ix_trace_rs::trace;
use rstest::rstest;

#[trace("TC-200")]
#[rstest]
#[case(1)]
#[case(2)]
#[case(3)]
fn param(#[case] n: u32) {
    assert_ne!(n, 2);
}

#[trace("TC-201")]
#[tokio::test]
async fn tokio_above() {}

#[tokio::test]
#[trace("TC-202")]
async fn tokio_below() {}

#[trace("TC-203")]
#[test]
fn plain_tagged() {}

mod plain_tagged {
    #[test]
    fn untagged_inside() {}
}
