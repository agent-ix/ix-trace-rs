use ix_trace_rs::trace;

#[trace("TC-802", "TC-801")]
#[trace("TC-803", "TC-801")]
#[test]
fn stacked() {}

#[trace("TC-802", "TC-801")]
#[test]
fn single() {}

#[trace("TC-804")]
#[test]
#[trace("TC-805")]
fn interleaved() {}
