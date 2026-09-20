# ix-trace-rs

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/6qsdhSPE)

A no-op `#[trace(...)]` attribute, so a Rust test can name the spec ids it
verifies in a form a tool can read **without running anything**.

```rust
use ix_trace_rs::trace;

#[trace("FR-047-AC-1", "TC-707")]
#[test]
fn tc707_shape_classification() {
    // …
}
```

The attribute expands to the annotated item unchanged. That is the entire
implementation — the crate exists so the marker is a construct the compiler
accepts and a coverage tool can parse statically.

## Why it exists

`quire-rs` **FR-051** binds test symbols to spec trace ids and makes
framework-native markers the canonical trace form, replacing the textual
conventions (`// TC-041`, `Trace: FR-001`, ids embedded in test names) that a
grep-based workflow used before. Markers are parsed **statically** from source:
coverage never requires a test run. This crate is the Rust half of that
contract, tracked as external deliverable **EXT-4b** in quire-rs's Plan-001.
The sibling deliverables are a pytest plugin registering a `trace` marker
(EXT-4a) and a vitest/jest helper (EXT-4c).

Runtime queryability — running every test for a requirement, tagging JUnit
output — is a benefit of the marker form, not a requirement this crate serves.

## Contract

Arguments must be comma-separated **string literals**. Anything else is a
compile error: a marker a tool cannot read is worse than no marker, because it
looks like coverage while providing none.

```rust
#[trace("FR-001-AC-1")]              // ok
#[trace("FR-001-AC-1", "TC-041")]    // ok
#[trace(FR_001_AC_1)]                // compile error: not a string literal
#[trace()]                           // compile error: needs at least one id
```

The crate has **no dependencies** — not `syn`, not `quote`. The `proc_macro`
API alone validates the argument list and hands the item back, which keeps it
cheap to take as a dev-dependency in every repo that authors traced tests.

## Use

```toml
[dev-dependencies]
ix-trace-rs = "0.1"
```

Markers carry no runtime cost: they are gone by the time the compiler finishes
expanding them.

## Import it; do not path-qualify it

```rust
use ix_trace_rs::trace;

#[trace("TC-707", "FR-047-AC-1")]
#[test]
fn tc707_shape_classification() { /* … */ }
```

`#[ix_trace_rs::trace("TC-707")]` compiles and behaves identically — and binds
**nothing**. The module's canonical marker pattern is anchored on the literal
attribute name (`#\[trace\(…\)\]`), so a leading path does not match. Nothing
warns: the test passes, the id is visibly there in the source, and the row it
should back silently reads as unbacked.

This crate hit it in its own test suite. `TC-009` was a status lie until the
qualified form was replaced — coverage went 19/29 with one lie to 26/29 with
none, from that change alone.

## License

AGPL-3.0-or-later, matching the rest of the Agent IX Rust projects.

An earlier revision proposed `MIT OR Apache-2.0` on the argument that the macro
conveys none of its own source into a consumer, and that a permissive marker
crate costs adopters no `deny.toml` edit. That trade was declined: the project
licenses uniformly and this crate is not carved out.

Adopting repositories therefore add a crate-scoped exception — the same shape
this repo's own `deny.toml` uses, admitting AGPL for this crate alone while the
third-party allow-list stays permissive-only:

```toml
[licenses]
exceptions = [
    { allow = ["AGPL-3.0-or-later"], crate = "ix-trace-rs" },
]
```
