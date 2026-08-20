---
id: FR-001
title: "trace attribute macro"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/US-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/quire-rs/FR-051"
    type: "consumes"
    cardinality: "1:1"
---

# FR-001: trace attribute macro

## Description

The crate SHALL export an attribute macro `trace` that accepts a comma-separated
list of specification ids and expands to the annotated item unchanged, so that a
symbol declares the ids it **verifies** in a form the compiler accepts and a
static reader can extract without executing anything.

## Inputs

- The attribute argument list: one or more string literals, comma-separated,
  with an optional trailing comma.
- The annotated item.

## Outputs

- The annotated item, byte-for-byte the input token stream, when the argument
  list is well formed.
- The annotated item preceded by a `compile_error!` invocation otherwise
  ([FR-003](./FR-003-argument-grammar.md)).

## Behavior

- The macro SHALL return the annotated item unchanged whenever the argument list
  is well formed.
- The macro SHALL NOT inspect, rewrite, reorder, or execute the annotated item.
- The macro SHALL accept the attribute above or below `#[test]`, leaving test
  harness registration undisturbed.
- The macro SHALL accept an argument list wrapped in an invisible group, so that
  an id supplied by an enclosing macro expansion is not rejected.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-001-CON-1 | The expansion SHALL add no runtime code, no static, and no registration side effect — the crate is a marker, and a marker that emits code changes what it annotates | Design | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-001-AC-1 | A function carrying one well-formed id returns the value its body computes — the annotated and unannotated forms produce equal results | Test (TC-001) |
| FR-001-AC-2 | A function carrying several comma-separated ids returns the value its body computes; more than one id is accepted without altering the item | Test (TC-001) |
| FR-001-AC-3 | A `struct` carrying the attribute is constructible and its fields hold the values assigned, so the attribute applies to items that are not functions | Test (TC-002) |
| FR-001-AC-4 | A `#[test]` function carrying the attribute above `#[test]` is discovered by the harness and runs to completion | Test (TC-003) |
| FR-001-AC-5 | An argument list ending in a comma compiles and the annotated function returns its computed value | Test (TC-001) |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-tag-a-test-with-every-id-it-verifies.md);
  quire-rs [FR-051](ix://agent-ix/quire-rs/FR-051) declares
  `rust-trace-attribute` and the pattern that reads it.
- **Downstream**: [FR-003](./FR-003-argument-grammar.md) supplies the argument
  grammar and the diagnostics this attribute emits on rejection.
