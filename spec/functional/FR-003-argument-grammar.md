---
id: FR-003
title: "Argument grammar and compile-time diagnostics"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/StR-001"
    type: "implements"
    cardinality: "1:1"
---

# FR-003: Argument grammar and compile-time diagnostics

## Description

Both attribute macros SHALL reject an argument list that is not a comma-separated
sequence of id-shaped string literals, emitting a `compile_error!` whose span
covers the offending argument, so that a marker a tool cannot read fails the
build instead of looking like coverage while providing none.

## Inputs

- The attribute argument token stream.

## Outputs

- Acceptance: the annotated item, unchanged.
- Rejection: a `compile_error!` naming the rejecting attribute and the reason,
  spanned to the token at fault.

## Behavior

- The macro SHALL accept `"lit"` (`,` `"lit"`)* with an optional trailing comma.
- The macro SHALL reject a non-literal argument, naming the token found.
- The macro SHALL reject an empty argument list, naming an example of a
  well-formed one.
- The macro SHALL reject an empty string literal.
- The macro SHALL reject a literal that is not id-shaped, where **id-shaped**
  means a `KIND` of ASCII letters, digits, and underscores that contains **at
  least one ASCII letter**, followed by one or more hyphen-separated segments
  that are each non-empty and alphanumeric, **at least one of which contains
  an ASCII digit**.
- If any argument is rejected, then the macro SHALL span the diagnostic to that
  argument rather than to the attribute or the call site.
- The macro SHALL still emit the annotated item alongside the `compile_error!`,
  so that rejection produces one diagnostic rather than a cascade of
  unresolved-name errors from the item's absence.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-003-CON-1 | The macro SHALL NOT encode which kinds exist, and SHALL NOT check that an id appears in any matrix | Design | Test |
| FR-003-CON-2 | Validation SHALL use only the `proc_macro` API, adding no runtime dependency | Design | Inspection |

### FR-003-CON-1

Shape, never membership. `ZZ-001` and `MADEUP-042-AC-9` are well-shaped and
SHALL be accepted, even though no module declares those kinds.

The kind vocabulary belongs to `spec-artifacts-process`. A second copy compiled
into this crate becomes a second declaration of the same thing, and the two
drift — silently, because nothing compares them.

This is not a hypothetical risk. That same module declares the matrix `Status`
column twice: a validation regex admitting `⚠️`, and a `traceability.status`
block that gives `⚠️` no class. The test written to keep the two honest asserts
only that every classed marker is admitted, never that every admitted marker is
classed, so the gap passed its own gate. 261 rows across 31 repositories were
sitting in it when it was found (`agent-ix/spec-artifacts-process#52`).

Checking membership requires reading the specification, which is a later stage
with its own rebuild-dependency problem. Keeping this crate to shape means the
vocabulary has exactly one home.

**Stricter is the same defect as looser.** The rule is calibrated against the
real corpus, not reasoned from the pattern, because two plausible tightenings
each rejected real ids:

| Tightening | Rejected | Found in |
| --- | --- | --- |
| the number must be the **second** segment | `TC-CB-01`, `IT-EDGE-008`, `TC-EC-01` — 308 distinct ids | a shape the module's `TestMatrix` id_pattern explicitly admits |
| the digit-bearing segment must **begin** with its digit | `FR-S003`, `FR-M6`, `FR-SP001` | golden-path security, ix-cli |
| the `KIND` segment must be letters-only | `interface_004-AC-1`, `interface_004-AC-2` | object ids use underscores rather than hyphens, so the object's numeral sits inside the kind segment itself (agent-ix/ix-trace-rs#7) |

Both were caught by feeding every id-shaped token in `~/dev` through the macro
and compiling. 6804 of 6931 are accepted. The remainder are documentation
placeholders (`FR-XXX`, `NFR-NNN-AC-N`), prose compounds (`FR-backed`,
`FR-to-IT`), and roughly ninety digit-free test ids of the form
`TC-CAT-APPS-FILTER` / `TC-REG-LAST-WRITE-WINS`.

That last group is worth naming rather than absorbing: those ids do not match
the module's own `TestMatrix` id_pattern either, which mandates `-\d+`. Rejecting
them keeps this crate aligned with the declared grammar rather than stricter
than it — but the repositories using that convention cannot adopt the marker
until their ids conform, and that is a corpus problem, not a macro one.

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-003-AC-1 | The id forms the corpus writes are all accepted, including a bare `TC-707`, a criterion `FR-047-AC-1`, a mixed-case kind `StR-004-AC-2`, a constraint `FR-003-CON-1`, an alphanumeric suffix `TC-001a`, an alphabetic segment before the number `TC-CB-01` / `IT-EDGE-008`, a digit inside rather than leading a segment `FR-S003` / `FR-M6`, and an underscored, digit-bearing `KIND` segment `interface_004-AC-1` | Test (TC-006) |
| FR-003-AC-2 | Prose and malformed ids are rejected, including `hello world`, `TC`, `TC-`, `-707`, `non-canonical`, `vague-response`, `TC 707`, `FR_047`, and a `KIND` segment with no ASCII letter in it at all (`004-AC-1`, `_004-AC-1`) | Test (TC-007) |
| FR-003-AC-3 | A well-shaped id whose kind no module declares is **accepted**, confirming the macro checks shape and not vocabulary (FR-003-CON-1) | Test (TC-008) |
| FR-003-AC-4 | A prose argument fails the build with a diagnostic naming the argument and the expected shape | Test (TC-009) |
| FR-003-AC-5 | A non-literal argument, an empty argument list, and an empty string each fail the build with their own diagnostic | Test (TC-009) |
| FR-003-AC-6 | When the **second** of two arguments is at fault, the diagnostic underlines that second literal, not the attribute and not the first argument | Test (TC-009) |
| FR-003-AC-7 | Weakening the shape rule causes the compile-failure suite to fail, so the suite gates the rule rather than merely recording today's output | Demonstration |

### FR-003-AC-7

`Demonstration`, not `Test`, and deliberately so.

A `trybuild` suite compares recorded compiler output. Recorded output proves the
compiler said this yesterday; it does not prove the suite would notice if the
rule stopped working. The distinction matters here more than usual, because this
whole crate exists to stop markers that look like evidence from being none.

The demonstration performed during development: `is_id_shaped` was short-circuited
to `return true`, and the suite re-run. Three of the six cases —
`prose_argument`, `second_id_is_prose`, `implements_rejects_prose_too` — failed;
the other three passed, correctly, because they exercise the literal-shape,
empty-list and empty-string branches rather than the id-shape branch. The
short-circuit was then reverted.

Writing this as an automated test would mean a test that mutates the crate's own
source and rebuilds the proc-macro — fragile in a way that would cost more
confidence than it adds. It is recorded here as a demonstration with its result,
and its matrix row is marked pending rather than claiming a trace it does not
have.

## Dependencies

- **Upstream**: [StR-001](../stakeholder/StR-001-compiler-checked-trace-markers.md).
- **Downstream**: [FR-001](./FR-001-trace-attribute.md) and
  [FR-002](./FR-002-implements-attribute.md) both apply this grammar.
