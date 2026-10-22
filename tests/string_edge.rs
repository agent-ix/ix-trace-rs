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

struct Edge;

impl std::str::FromStr for Edge {
    type Err = ();

    #[string_edge]
    fn from_str(s: &str) -> Result<Self, ()> {
        if s == "edge" {
            Ok(Edge)
        } else {
            Err(())
        }
    }
}

#[string_edge]
fn generic_edge<'a, T: AsRef<str> + ?Sized>(kind: &'a T, fallback: &'a str) -> &'a str {
    if kind.as_ref().is_empty() {
        fallback
    } else {
        kind.as_ref()
    }
}

#[trace("TC-015", "FR-004-AC-1")]
#[test]
fn string_edge_is_transparent_on_free_functions_and_methods() {
    assert!(free_edge("a"));
    assert!(!free_edge("b"));
    assert_eq!(Dispatcher.method_edge("abc"), 3);
    assert!("edge".parse::<Edge>().is_ok());
    assert!("other".parse::<Edge>().is_err());
    assert_eq!(generic_edge("x", "z"), "x");
    assert_eq!(generic_edge("", "z"), "z");

    #[string_edge]
    fn main() -> u8 {
        8
    }
    assert_eq!(main(), 8);
}

#[trace("TC-016", "FR-004-AC-3")]
#[string_edge]
#[test]
fn string_edge_composes_with_the_test_attribute() {
    // Importable as `ix_trace_rs::string_edge` (the `use` above) and stackable
    // with `#[test]`: the harness still discovers and runs this function.
    assert!(free_edge("a"));
}
