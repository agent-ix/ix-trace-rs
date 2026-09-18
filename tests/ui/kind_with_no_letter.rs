use ix_trace_rs::trace;

// agent-ix/ix-trace-rs#7 relaxed the KIND segment to allow digits and
// underscores (`interface_004`), but a KIND with no ASCII letter at all is
// still not an id — it is a bare number, indistinguishable from the digit
// segment that must follow it.
#[trace("004-AC-1")]
fn annotated() {}

fn main() {}
