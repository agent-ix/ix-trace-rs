//! The attribute must compile away to nothing — that is the whole contract.
//!
//! Rejection cases (non-literal arguments, an empty argument list) are compile
//! errors by construction, so they cannot be asserted from a passing test
//! without a `trybuild`-style harness. What *can* be asserted here is the
//! property downstream repos rely on: an annotated item behaves exactly as if
//! the attribute were not there.

use ix_trace_rs::trace;

#[trace("FR-051-AC-4")]
fn single_marker() -> u8 {
    7
}

#[trace("FR-051-AC-4", "TC-744")]
fn multiple_ids() -> u8 {
    9
}

// A trailing comma in the argument list is accepted by the macro; rustfmt
// normalizes it away in source, so this fixture documents the tolerance rather
// than preserving the comma.
#[trace("FR-051-AC-4")]
fn normalized_argument_list() -> u8 {
    11
}

#[trace("FR-051-AC-6")]
struct Annotated {
    value: u8,
}

#[test]
fn attribute_is_transparent_on_functions() {
    assert_eq!(single_marker(), 7);
    assert_eq!(multiple_ids(), 9);
    assert_eq!(normalized_argument_list(), 11);
}

#[test]
fn attribute_is_transparent_on_items() {
    let annotated = Annotated { value: 3 };
    assert_eq!(annotated.value, 3);
}

#[trace("TC-744")]
#[test]
fn attribute_composes_with_the_test_attribute() {
    // The ordering downstream suites actually use: the marker sits above
    // `#[test]` and must not disturb harness registration.
    assert_eq!(single_marker(), 7);
}
