---
id: FR-002
title: "implements attribute macro"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/US-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-002: implements attribute macro

## Description

The crate SHALL export a second attribute macro `implements`, distinct from
`trace`, that accepts the same argument grammar and expands to the annotated
item unchanged, so that a symbol declares the ids it **implements** — its scope
— separately from the ids it verifies.

## Behavior

- The macro SHALL return the annotated item unchanged whenever the argument list
  is well formed.
- The macro SHALL apply the argument grammar of
  [FR-003](./FR-003-argument-grammar.md), and its diagnostics SHALL name
  `implements` rather than `trace`, so a reader can tell which attribute
  rejected the argument.
- The two attributes SHALL be usable together on one item without either
  affecting the other.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-002-CON-1 | `implements` SHALL remain a separate attribute rather than an argument or flag on `trace` | Design | Test |

### FR-002-CON-1

The separation is the point, not a stylistic preference.

`verifies` is **evidence** and may back an acceptance criterion. `implements` is
**scope** and never may. The module contract keeps them in two lists and binds
them to complementary symbol kinds, so a marker declared as the wrong one binds
*nothing* rather than binding the wrong thing — a loud failure instead of a
quiet one.

A single attribute carrying a discriminator — `#[trace(kind = "implements", …)]`
— would put one typo between evidence and scope, and a typo in that position
produces a plausible, wrong coverage number rather than an error. Two attributes
move the distinction into the name, where the compiler resolves it.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-002-AC-1 | An item annotated with `#[implements(...)]` behaves exactly as if the attribute were absent, for both functions and non-function items | Test (TC-004) |
| FR-002-AC-2 | An item carrying both `#[trace(...)]` and `#[implements(...)]` is unchanged, and neither attribute disturbs the other | Test (TC-005) |
| FR-002-AC-3 | `implements` rejects a malformed argument on the same grammar as `trace`, and its diagnostic names `implements` | Test (TC-009) |

## Dependencies

- **Upstream**: [US-001](../usecase/US-001-tag-a-test-with-every-id-it-verifies.md).
- **Downstream**: [FR-003](./FR-003-argument-grammar.md) — the shared grammar and
  diagnostics.
