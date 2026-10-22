---
id: FR-004
title: "string_edge attribute macro"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/US-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-004: string_edge attribute macro

## Description

The crate SHALL export a function attribute macro `string_edge` that takes no
arguments and expands to the annotated item unchanged. It marks a function as a
place where a string is allowed to select semantics, so a source scan can find
every such edge mechanically instead of by convention. It is a marker only and
has no runtime behaviour.

## Inputs

- An empty attribute argument list.
- The annotated item.

## Outputs

- The annotated item, unchanged, when no argument is given.
- The annotated item preceded by `compile_error!("#[string_edge] takes no
  arguments")` when any argument is given.

## Behavior

- The macro SHALL return the annotated item unchanged when it carries no
  arguments.
- The macro SHALL reject any argument with the compile error
  `#[string_edge] takes no arguments`, and SHALL retain the annotated item in
  the expansion so that one stray argument produces one error rather than a
  cascade of unresolved-name errors at every call site.
- The macro SHALL be importable as `ix_trace_rs::string_edge`.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-004-CON-1 | The expansion SHALL add no runtime code, no static, and no registration side effect, and the attribute SHALL add no dependency | Design | Test (TC-014) |

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-004-AC-1 | A free function and a method carrying `#[string_edge]` behave exactly as if the attribute were absent | Test (TC-015) |
| FR-004-AC-2 | `#[string_edge(anything)]` fails to compile with the message `#[string_edge] takes no arguments`, and the annotated item is retained (no further error arises from its call sites) | Test (TC-009) |
| FR-004-AC-3 | `ix_trace_rs::string_edge` is importable and the attribute composes with `#[test]` | Test (TC-016) |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-tag-a-test-with-every-id-it-verifies.md).
- **Related**: [NFR-001](../non-functional/NFR-001-dependency-posture.md) — the
  attribute adds no dependency.
