//! The attribute must compile away to nothing — that is the whole contract.
//!
//! Rejection cases (non-literal arguments, an empty argument list) are compile
//! errors by construction, so they cannot be asserted from a passing test
//! without a `trybuild`-style harness. What *can* be asserted here is the
//! property downstream repos rely on: an annotated item behaves exactly as if
//! the attribute were not there.

use ix_trace_rs::{implements, trace};

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

#[trace("TC-001", "FR-001-AC-1", "FR-001-AC-2", "FR-001-AC-5")]
#[test]
fn attribute_is_transparent_on_functions() {
    assert_eq!(single_marker(), 7);
    assert_eq!(multiple_ids(), 9);
    assert_eq!(normalized_argument_list(), 11);
}

#[trace("TC-002", "FR-001-AC-3")]
#[test]
fn attribute_is_transparent_on_items() {
    let annotated = Annotated { value: 3 };
    assert_eq!(annotated.value, 3);
}

#[trace("TC-003", "FR-001-AC-4")]
#[test]
fn attribute_composes_with_the_test_attribute() {
    // The ordering downstream suites actually use: the marker sits above
    // `#[test]` and must not disturb harness registration.
    assert_eq!(single_marker(), 7);
}

// `implements` is a separate attribute, not a flag on `trace`, and must be
// equally transparent. Kept apart deliberately: `verifies` is evidence and may
// back a criterion, `implements` is scope and never may, and the module binds
// them to complementary symbol kinds — a shared attribute with a discriminator
// argument would put one typo between the two.
#[implements("FR-051-AC-4")]
fn implemented_marker() -> u8 {
    13
}

#[implements("FR-051-AC-4", "FR-051-AC-6")]
struct Scoped {
    value: u8,
}

#[trace("TC-004", "FR-002-AC-1")]
#[test]
fn implements_is_transparent_on_functions_and_items() {
    assert_eq!(implemented_marker(), 13);
    assert_eq!(Scoped { value: 5 }.value, 5);
}

// Both attributes on one item: neither disturbs the other, and the item is
// still the item.
#[trace("TC-744")]
#[implements("FR-051-AC-4")]
fn both_attributes() -> u8 {
    17
}

#[trace("TC-005", "FR-002-AC-2")]
#[test]
fn trace_and_implements_compose() {
    assert_eq!(both_attributes(), 17);
}

// agent-ix/ix-trace-rs#7: this program's object ids use underscores, not
// hyphens (`interface_004`), so the KIND segment itself carries digits and
// underscores. `interface_004-AC-1` must compile as written — the only
// spelling that used to compile, `interface-004-AC-1`, names an id the spec
// does not define. This is a compile-pass regression test: if `is_id_shaped`
// ever goes back to letters-only kinds, this file fails to build.
#[trace("interface_004-AC-1")]
fn interface_ac_backed() -> u8 {
    19
}

#[trace("TC-006", "FR-003-AC-1")]
#[test]
fn kind_segment_with_underscore_and_digit_compiles() {
    assert_eq!(interface_ac_backed(), 19);
}
