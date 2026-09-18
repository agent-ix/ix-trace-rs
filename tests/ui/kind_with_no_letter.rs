use ix_trace_rs::trace;

// agent-ix/ix-trace-rs#7 relaxed the KIND segment to allow digits and
// underscores (`interface_004`), but a KIND with no ASCII letter at all is
// still not an id — it is a bare number, indistinguishable from the digit
// segment that must follow it.
#[trace("004-AC-1")]
fn annotated() {}

// agent-ix/ix-trace-rs#7 review: the KIND segment must BEGIN with an ASCII
// letter — matching quire-rs's own `is_object_id` (src/semantic/target.rs)
// exactly — not merely contain one anywhere in it.
#[trace("_interface_004-AC-1")]
fn leading_underscore() {}

#[trace("4interface-AC-1")]
fn leading_digit() {}

#[trace("1_x-AC-1")]
fn leading_digit_short_kind() {}

fn main() {}
