use ix_trace_rs::trace;

// The first argument is a real id; only the second is prose. The diagnostic
// must point at the second literal, not at the attribute.
#[trace("TC-707", "vague-response")]
fn annotated() {}

fn main() {}
