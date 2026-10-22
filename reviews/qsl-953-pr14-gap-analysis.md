---
id: SR-003
title: "Gap analysis — QSL-953 #[string_edge] marker (PR #14)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/ix-trace-rs@13a8bdf71209cbad6c60990b444b60489e833eb6; spec/functional/FR-004-string-edge-attribute.md, spec/spec.md, spec/tests.md, tests/"
review_set: subset
---

## Summary

Ticket: QSL-953. Planless gap analysis of FR-004 (AC-1..3, CON-1) against the
computed matrix (`quoin matrix --json`) and against the Matrix Rules in
`spec/tests.md`. Plan completion: not assessed.

Computed matrix for FR-004:

- FR-004-AC-1 is tagged on
  `tests/string_edge.rs::string_edge_is_transparent_on_free_functions_and_methods`
  (TC-015).
- FR-004-AC-2 is tagged on
  `tests/compile_fail.rs::malformed_markers_are_rejected_at_compile_time`
  (TC-009, fixture `string_edge_with_args`).
- FR-004-AC-3 is tagged on
  `tests/string_edge.rs::string_edge_composes_with_the_test_attribute`
  (TC-016).
- FR-004-CON-1 is **untagged**, although it declares `Validation: Test`.

Results against the Matrix Rules:

- Rule 2 (one id, one symbol): each of TC-015, TC-016 and FR-004-AC-1..3 sits
  on exactly one symbol. TC-009 stays on its single runner.
- Rule 3 (every rejection branch has a compile-fail fixture): met. FR-004 has
  one rejection branch, and `string_edge_with_args` covers it.
- Rule 4 (non-vacuity): met by measurement. The four mutants in SR-002 were
  all killed.

The FR-004 statements match the code and the QSL FR-064 / QSL-953 contract:
no arguments, an exact message, the item retained, an identity expansion, no
dependency, and the `ix_trace_rs::string_edge` path.

## Verdict

**APPROVE**, with one medium and one low finding. Neither blocks the merge.
Both are spec/matrix bookkeeping. No code is defective.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | FR-004-CON-1 ("no runtime code, no static, no registration side effect, and no dependency"; Validation: Test) has no tagged test. `quoin matrix` reports it as `untagged`. That breaks Matrix Rule 1, which needs a trace or an explicit row saying why none exists. FR-003-CON-1 sets the pattern: TC-008 carries it. The dependency half is already asserted by `tests/dependency_posture.rs` (TC-014), and the transparency half by TC-015. Tag one of those tests with FR-004-CON-1, or add a row explaining the gap | spec/functional/FR-004-string-edge-attribute.md:46 |
| FND-002 | low | The TC-009 row in `spec/tests.md` adds the `string_edge_with_args` description but its Traces column still lists only FR-003/FR-002 ids. The test's `#[trace]` now carries FR-004-AC-2, so the hand-written row and the computed binding disagree | spec/tests.md:63 |

## Dispositions

| FND | outcome | sha/reason |
| --- | ------- | ---------- |
| FND-001 | fixed | caf3723 |
| FND-002 | fixed | caf3723 |
