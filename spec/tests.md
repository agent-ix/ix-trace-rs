---
id: TM-001
title: "ix-trace-rs Test Matrix"
type: TestMatrix
---

# Test Matrix

## Overview

This matrix maps every acceptance criterion in `ix-trace-rs/spec/` to the test
that verifies it. Ids use the two declared evidence prefixes — `TC-XXX` for a
test case, `IT-XXX` for an integration test — and the `Type` column says what
kind of evidence each row is.

The crate marks its own tests with its own attribute wherever it can. That is
not decoration: if `#[trace(...)]` cannot express this crate's traceability, it
has no business expressing anyone else's.

> **One place it cannot.** A proc-macro crate cannot invoke its own attribute
> inside its own `src/`, so the three unit tests in `src/lib.rs` carry the
> legacy comment form instead (TC-006..TC-008). The limitation is the crate's,
> not the marker's, and it is stated here rather than left for a reader to
> discover as an inconsistency.

## Matrix Rules

1. **Coverage Rule** — every acceptance criterion has at least one trace, or an
   explicit row saying why it has none.
2. **One id, one symbol** — no trace id appears on two different tests. A
   collision is invisible to `quire coverage`: the row binds, the count is
   right, and deleting either test leaves the row green on the strength of the
   other (`agent-ix/quire-cli#45`).
3. **Rejection rule** — every rejection branch of the argument grammar has a
   compile-fail fixture, since a rejection cannot be observed from a passing
   test inside the same crate.
4. **Non-vacuity rule** — a recorded compiler diagnostic proves what the
   compiler said, not that the suite would notice a regression. The shape rule
   is short-circuited and the suite re-run to confirm it fails (FR-003-AC-7).

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|----|----|----|--------|
| FR-001 trace attribute | AC-1..5 | TC-001 (single + multiple ids, trailing comma), TC-002 (non-function items), TC-003 (composes with `#[test]`) | ✅ |
| FR-002 implements attribute | AC-1..3 | TC-004 (transparency), TC-005 (both attributes on one item), TC-009 (rejects prose, names `implements`) | ✅ |
| FR-003 argument grammar | AC-1..7 | TC-006 (real id forms accepted), TC-007 (prose rejected), TC-008 (vocabulary not checked), TC-009 (compile-fail suite: prose, non-literal, empty list, empty string, second-argument span) | 🚧 |
| FR-004 string_edge attribute | AC-1..3, CON-1 | TC-015 (transparent on free functions and methods), TC-009 (rejects an argument, item retained), TC-016 (composes with `#[test]`), TC-014 (CON-1: no dependency added) | ✅ |
| FR-005 emission mode | AC-1..13; CON-1..3 | TC-017 (inert default, bad arguments, `implements` registers nothing), TC-018 (records: ids, non-qualifying items, rstest order, tokio, untagged), TC-019 (unification, link retention) | 🚧 |
| FR-006 emission run | AC-1..19; CON-1..2 | TC-020 (JUnit shape, non-root file naming, outcomes, tokio), TC-021 (concurrency, rstest matching, abnormal exit, unmatched record, single write), TC-022 (cost on quire-rs) | 🚧 |
| NFR-001 dependency posture | AC-1..3 | TC-014 (`[dependencies]` empty, no parsing crate imported) | ✅ |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|----|----|----|----|----|----|
| TC-001 | An item annotated with one id, and one annotated with several, behave exactly as if the attribute were absent; a trailing comma is accepted (`integration::attribute_is_transparent_on_functions`) | Unit | P0 | FR-001-AC-1, FR-001-AC-2, FR-001-AC-5 | ✅ |
| TC-002 | The attribute applies to a `struct` as well as a function, leaving it unchanged (`integration::attribute_is_transparent_on_items`) | Unit | P1 | FR-001-AC-3 | ✅ |
| TC-003 | `#[trace(...)]` above `#[test]` leaves the test discoverable and runnable by the harness — the ordering downstream suites actually write (`integration::attribute_composes_with_the_test_attribute`) | Unit | P0 | FR-001-AC-4 | ✅ |
| TC-004 | `#[implements(...)]` is transparent on both functions and non-function items (`integration::implements_is_transparent_on_functions_and_items`) | Unit | P0 | FR-002-AC-1 | ✅ |
| TC-005 | An item carrying both attributes is unchanged and neither disturbs the other (`integration::trace_and_implements_compose`) | Unit | P1 | FR-002-AC-2 | ✅ |
| TC-006 | Every id form the corpus writes is accepted — bare `TC-707`, criterion `FR-047-AC-1`, mixed-case kind `StR-004-AC-2`, constraint `FR-003-CON-1`, alphanumeric suffix `TC-001a`, underscored digit-bearing kind `interface_004-AC-1` (`is_id_shaped` unit tests) | Unit | P0 | FR-003-AC-1 | ✅ |
| TC-007 | Prose and malformed ids are rejected: `hello world`, `TC`, `TC-`, `-707`, `FR-AC-1`, `non-canonical`, `vague-response`, `TC 707`, `FR_047`, and a `KIND` segment that does not begin with an ASCII letter: `004-AC-1`, `_004-AC-1`, `_interface_004-AC-1`, `4interface-AC-1`, `1_x-AC-1` (`is_id_shaped` unit tests) | Unit | P0 | FR-003-AC-2 | ✅ |
| TC-008 | A well-shaped id whose kind no module declares is accepted, proving the macro checks shape and not vocabulary (`is_id_shaped` unit tests) | Unit | P0 | FR-003-AC-3, FR-003-CON-1 | ✅ |
| TC-009 | The compile-fail suite: prose argument, non-literal argument, empty argument list, empty string, a prose *second* argument (span lands on it, not the attribute), `implements` rejecting prose with its own name in the message, and a `KIND` segment that does not begin with an ASCII letter (`004-AC-1`, plus FR-003-AC-2's `_interface_004-AC-1`/`4interface-AC-1`/`1_x-AC-1`) (`compile_fail::malformed_markers_are_rejected_at_compile_time`, eight `tests/ui/` fixtures including `kind_with_no_letter`); also `#[string_edge(anything)]` is rejected with `#[string_edge] takes no arguments` and its item retained (`string_edge_with_args`) | Compile | P0 | FR-003-AC-4, FR-003-AC-5, FR-003-AC-6, FR-002-AC-3, FR-003-AC-2, FR-004-AC-2 | ✅ |
| TC-015 | `#[string_edge]` is transparent on a free function and a method, a trait-impl method, a generic and lifetime-bearing function, and a nested `fn main` (`string_edge::string_edge_is_transparent_on_free_functions_and_methods`) | Unit | P0 | FR-004-AC-1 | ✅ |
| TC-016 | `#[string_edge]` is importable as `ix_trace_rs::string_edge` and composes with `#[test]` (`string_edge::string_edge_composes_with_the_test_attribute`) | Unit | P1 | FR-004-AC-3 | ✅ |
| TC-013 | 🚧 The compile-fail suite is non-vacuous: short-circuiting `is_id_shaped` to `return true` fails the cases that pass a malformed id to it. Demonstrated during development, reverted; **no automated trace** — an automated form would mutate the crate's own source and rebuild the proc-macro, which costs more confidence than it adds | Manual | P1 | FR-003-AC-7 | 🚧 |
| TC-014 | `[dependencies]` is present and empty, and `src/lib.rs` imports no parsing or quoting crate — the cost this bounds is paid by ~150 consuming repositories (`dependency_posture::tc014_*`) | Unit | P0 | NFR-001-AC-1, NFR-001-AC-2, FR-004-CON-1 | ✅ |
| TC-017 | With `emit` off the expansion equals the input and the binary has no trace section; with `emit` on a bad argument list gives the same `compile_error!` and `#[implements]` yields no record | Unit | P0 | FR-005-AC-1, FR-005-AC-6, FR-005-AC-7, FR-005-CON-1, FR-005-CON-3 | 🚧 |
| TC-018 | With `emit` on, a tagged test yields one record with ids in order, identifier, module path and file; a struct yields none, a method with a `self` receiver yields none, an associated fn without a receiver and a non-test fn yield an unflagged record, and all compile; `#[test]` written above `#[trace]` still gets its ids; rstest below and above `#[trace]`, `#[tokio::test]` in both orders, and an untagged test behave as stated | Integration | P0 | FR-005-AC-2, FR-005-AC-3, FR-005-AC-4, FR-005-AC-5, FR-005-AC-8, FR-005-AC-11 | 🚧 |
| TC-019 | With `emit` on for one package, a second package without `ix-trace-run` still compiles and passes; every tagged test of a fixture keeps its record through linking on each supported platform, and a `wasm` crate builds with no section | Integration | P0 | FR-005-AC-9, FR-005-AC-10, FR-005-AC-12, FR-005-AC-13 | 🚧 |
| TC-020 | The runner over a fixture crate reads records from the binary alone and writes JUnit from libtest's `--logfile` (a test forging stdout changes nothing; a missing or duplicate line fails): pass, panic, ignored, `should_panic` and `tokio::test` cases carry `trace` with the right outcome; a test in `mod tests` of `src/foo/bar.rs` reports `tests::t` | Integration | P0 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-4, FR-006-AC-5, FR-006-AC-10, FR-006-AC-14, FR-006-AC-15, FR-006-AC-17, FR-006-AC-18 | 🚧 |
| TC-021 | Concurrent tests keep their own ids; rstest cases report the source name and a same-named module's untagged test gets no mark; an aborting binary loses no test; the rstest-above warning is printed; a duplicate-name ambiguity marks nothing; `process::exit(0)` is re-run; an unmatched record is reported with non-zero exit; one output file is written once and plain `cargo test` writes no mark | Integration | P0 | FR-006-AC-6, FR-006-AC-7, FR-006-AC-8, FR-006-AC-9, FR-006-AC-11, FR-006-AC-13, FR-006-AC-16, FR-006-AC-19 | 🚧 |
| TC-022 | The runner's median wall-clock over the quire-rs suite, binaries prebuilt, warm cache, equal thread count, five alternating runs, stays under 105% of `cargo test --tests`; the quire-rs revision is recorded in the output | Benchmark | P1 | FR-006-AC-12, FR-006-CON-1 | 🚧 |

## Stakeholder Validation

| ID | Traces To | Validation | Status |
|----|----|----|----|
| StR-001-VC-1 | US-001; FR-001, FR-003 | Demonstration — an author annotates a test with several ids and all of them bind. Confirmed by the adoption measurement rather than by a unit test: the conversion in a consuming repository is where "all of them bind" becomes observable | 🚧 |
| StR-001-VC-2 | FR-003 | Demonstration — covered by TC-009, which fails the build and names the offending argument | ✅ |
| StR-001-VC-3 | FR-001, FR-002 | Demonstration — the canonical marker compiles; every test in this repository annotated with it is the standing evidence | ✅ |

## Verification Status

GREEN for the v1 surface, with two rows honestly pending.

`FR-003` is marked 🚧 rather than ✅ because AC-7 has no automated trace, and a
row that claimed one would be exactly the defect this crate exists to reduce.
`StR-001-VC-1` is pending until the marker is adopted in a consuming repository
and the criterion count is measured before and after — the claim is about
coverage improving, and that cannot be verified from inside this crate.

Two capabilities are out of scope for v1 and are not counted as gaps here:
checking an id against `spec/tests.md` at compile time, and recording that an
annotated test actually ran. Both are staged in `agent-ix/quire-rs#191`.
