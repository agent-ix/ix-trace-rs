---
id: NFR-001
title: "Zero runtime dependencies"
type: NFR
quality_attribute: maintainability
relationships:
  - target: "ix://agent-ix/ix-trace-rs/FR-001"
    type: "constrains"
  - target: "ix://agent-ix/ix-trace-rs/FR-003"
    type: "constrains"
---

# NFR-001: Zero runtime dependencies

## Statement

The crate SHALL declare no runtime dependencies, so that adopting the marker
across roughly 150 repositories adds exactly one crate to each consumer's build
graph.

## Rationale

The marker is worth having only if it is cheap enough to put everywhere. A
proc-macro that pulls `syn` and `quote` costs every adopting repository a
compile of both, on every clean build, for an attribute that expands to nothing.

Those crates exist to parse and re-emit Rust. This one does neither: it inspects
a flat list of literals and hands the item back untouched. The `proc_macro` API
alone is enough, and the token walking it needs will not change when the Rust
grammar does, because it never looks at the annotated item.

The trade is deliberate — a few dozen lines of hand-written token walking here,
in one place, against a dependency in ~150 build graphs.

Development dependencies are a separate matter and are permitted. Cargo does not
propagate them transitively, so the compile-fail harness this crate uses for its
own tests costs an adopter nothing.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|--------|--------|-----------|--------|
| Runtime dependencies declared in `[dependencies]` | 0 | 0 — any entry fails | Automated assertion over `Cargo.toml` (TC-014) |
| Parsing or quoting crates imported by `src/lib.rs` | 0 | 0 — `syn`, `quote`, `proc_macro2` or `regex` fails | Automated assertion over the library source (TC-014) |
| Crates added to a consumer's build graph by adopting | 1 | 1 — the crate itself and nothing further | Follows from the two above; dev-dependencies are not transitive |

## Verification

TC-014 asserts both metrics directly, in-process, over the manifest and the
library source. Source inspection is the appropriate method here rather than a
weaker choice: no runtime path can observe a dependency that is not there, and
the property is about the manifest rather than about behaviour.

The assertion was confirmed non-vacuous by adding `syn = "2"` to
`[dependencies]` and re-running: the test fails and names the offending entry.
The edit was reverted.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| NFR-001-AC-1 | `[dependencies]` in `Cargo.toml` is present and empty | Test (TC-014) |
| NFR-001-AC-2 | `src/lib.rs` imports no parsing or quoting crate; validation uses the `proc_macro` API alone | Test (TC-014) |
| NFR-001-AC-3 | Development dependencies do not reach consumers, so the compile-fail harness adds nothing to an adopter's graph | Inspection |

## Dependencies

- **Upstream**: [StR-001](../stakeholder/StR-001-compiler-checked-trace-markers.md)
  — the stakeholder constraint that adoption cost effectively nothing.
