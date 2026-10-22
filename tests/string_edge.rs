//! `#[string_edge]` is a pure marker: the annotated item behaves exactly as if
//! the attribute were absent. The rejection half (an argument) is a compile
//! error, covered by the `tests/ui/string_edge_with_args.rs` fixture.

use ix_trace_rs::{string_edge, trace};

#[string_edge]
fn free_edge(kind: &str) -> bool {
    kind == "a"
}

struct Dispatcher;

impl Dispatcher {
    #[string_edge]
    fn method_edge(&self, kind: &str) -> u8 {
        kind.len() as u8
    }
}

#[trace("TC-015", "FR-004-AC-1")]
#[test]
fn string_edge_is_transparent_on_free_functions_and_methods() {
    assert!(free_edge("a"));
    assert!(!free_edge("b"));
    assert_eq!(Dispatcher.method_edge("abc"), 3);
}

#[trace("TC-016", "FR-004-AC-3")]
#[string_edge]
#[test]
fn string_edge_composes_with_the_test_attribute() {
    // Importable as `ix_trace_rs::string_edge` (the `use` above) and stackable
    // with `#[test]`: the harness still discovers and runs this function.
    assert!(free_edge("a"));
}
