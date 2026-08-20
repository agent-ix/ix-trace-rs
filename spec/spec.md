---
type: master-requirements
name: ix-trace-rs
org: agent-ix
component_type: rust-proc-macro-crate
implementation_language: rust
tags:
  - traceability
  - proc-macro
depends_on: []
standards_alignment:
  - iso-iec-ieee-29148
relationships:
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "depends_on"
    cardinality: "1:1"
security_critical: false
---

# Master Requirements Specification

## Purpose

This document specifies `ix-trace-rs`, a proc-macro crate providing the
`#[trace(...)]` and `#[implements(...)]` attributes. The attributes let a Rust
symbol name the specification ids it answers for, in a form the compiler
rejects when malformed and a coverage tool can read without running anything.

The crate exists because the marker it provides was already declared canonical
and did not compile. `spec-artifacts-process` declares `rust-trace-attribute`
as the canonical Rust trace marker, `bind_symbol` prefers it over every legacy
comment form, and the engine emits rewrite suggestions pointing authors at it —
while `rustc` answers `error: cannot find attribute 'trace' in this scope`,
because no crate defined it.

The cost of that gap is measurable. In `agent-ix/quire-cli`, matrix rows read
96/105 backed while acceptance criteria read **2/119**, because the comment
form the corpus actually writes binds only its first id:

```rust
// IT-033 (FR-011-AC-1, US-005-AC-2): prose   <- binds IT-033, drops two
```

An attribute has no such failure mode: every argument is a separate literal, and
one that is not an id is a compile error rather than prose that silently
verifies nothing.

## Scope

### In Scope

- The `#[trace(...)]` and `#[implements(...)]` attribute macros, their argument
  grammar, and their compile-time diagnostics.
- The guarantee that both expand to the annotated item unchanged.
- The crate's dependency posture, which governs what adopting it costs a
  consumer.

### Out of Scope

- **Which ids exist.** The kind vocabulary is declared by the module
  (`spec-artifacts-process`); a second copy compiled into this crate would drift
  from it. This crate checks shape, never membership.
- **Reading the specification.** Validating an id against `spec/tests.md` at
  compile time is a later stage, and brings a `build.rs` rerun-if-changed
  dependency that belongs with it rather than here.
- **Recording execution.** Registering an annotated test in a link-time slice so
  *backed* can mean "ran and passed" rather than "is annotated" is a later stage.
- **Replacing the static scan.** `extract_tree` reads ~150 repositories in any
  language without building them, which is why corpus-wide sweeps are possible.
  A macro requires compilation. This crate adds a second, stronger tier; it
  retires nothing.
- **Symbols that are not items.** `fuzz_target!(|data: &[u8]| { … })` declares
  no `fn` for an attribute to attach to. The legacy comment forms must survive
  for that case.

## System Overview

### System Description

A dependency-free proc-macro crate. Each attribute receives two token streams,
validates the argument list against the id grammar by hand over
`proc_macro::TokenTree`, and returns the annotated item unchanged. On rejection
it prepends a `compile_error!` whose tokens carry the span of the offending
literal, so the diagnostic underlines the argument at fault rather than the
whole attribute.

Consumers add it as a dev-dependency. Because dev-dependencies are not
transitive, nothing the crate uses for its own tests reaches a downstream build.

### Interfaces

The public surface is two attribute macros and nothing else: `trace` and
`implements`. Both take a comma-separated list of string literals and expand to
their input.

## Requirements Architecture

### Requirement Groups

- **Functional** — the two attributes ([FR-001](./functional/FR-001-trace-attribute.md),
  [FR-002](./functional/FR-002-implements-attribute.md)) and the argument
  grammar and diagnostics they share
  ([FR-003](./functional/FR-003-argument-grammar.md)).
- **Non-functional** — the dependency posture that decides adoption cost
  ([NFR-001](./non-functional/NFR-001-dependency-posture.md)).
- **Stakeholder** — the need the crate answers
  ([StR-001](./stakeholder/StR-001-compiler-checked-trace-markers.md)), reached
  through [US-001](./usecase/US-001-tag-a-test-with-every-id-it-verifies.md).

### Traceability Approach

Every acceptance criterion is traced from the Test Matrix (`spec/tests.md`) to a
test carrying the id. The crate marks its own tests with its own attribute:
if the marker cannot express this crate's traceability, it should not be asked
to express anyone else's.

## References

- ISO/IEC/IEEE 29148:2018 — requirements engineering, the shape these artifacts
  follow.
- `ix://agent-ix/quire-rs/FR-051` — the trace-tag grammar declaring
  `rust-trace-attribute` canonical, and this crate's reason for existing.
- `agent-ix/quire-rs#191` — the ticket specifying the staged plan this crate
  implements the first stage of.
- `agent-ix/quire-cli#43` — where the 2/119 criterion-coverage figure was
  measured, and the comment-form workaround that does not travel.
