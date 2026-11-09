---
id: SR-008
title: "Failure-domain analysis — PLAT-333 emitter spec (PR #15)"
type: SpecReview
analysis: failure-domain
scope: "agent-ix/ix-trace-rs@7836a1f7dccfab7a27fed5845f561a5b539041a4; spec/functional/FR-004-emission-mode.md, spec/functional/FR-005-emission-run.md, spec/non-functional/NFR-001-dependency-posture.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Looked for unstated failure modes, identity confusion and environment
edges in the emit/run design: cargo feature resolution, parametrized name mapping, attribute
order, nested items, duplicate registration, crash/signal outcomes. Signal and non-zero exit
are covered (fail); `#[should_panic]` is safe because the outcome comes from the exit status;
concurrency is isolated per process.

Units examined:

- `FR-004` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-1` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-2` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-3` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-4` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-5` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-6` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-AC-7` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-CON-1` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-CON-2` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-CON-3` (examined): spec/functional/FR-004-emission-mode.md
- `FR-005` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-1` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-2` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-3` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-4` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-5` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-6` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-7` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-8` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-AC-9` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-CON-1` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-CON-2` (examined): spec/functional/FR-005-emission-run.md
- `FR-001-CON-1` (examined): spec/functional/FR-001-trace-attribute.md
- `NFR-001` (examined): spec/non-functional/NFR-001-dependency-posture.md
- `FR-004 behavior (param)` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004 behavior (body)` (examined): spec/functional/FR-004-emission-mode.md
- `FR-005 behavior (case)` (examined): spec/functional/FR-005-emission-run.md

## Verdict

**REQUEST CHANGES on three medium findings** (feature unification, suffix-mapping identity
confusion, rstest attribute order), plus one low.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | Cargo feature unification turns emit on for every crate using #[trace] in the same build, which then fails to resolve ix_trace_run. `emit` is a feature of a proc-macro crate. Cargo unifies features of a host dependency across all packages built together, so one workspace member enabling `ix-trace-rs/emit` makes `#[trace]` emit registry code in every other member, including ones without an `ix-trace-run` dependency, and in non-test builds where a dev-dependency is absent (`#[trace]` on non-test items): an unresolved path compile error. The spec should state the failure and either require the consumer's own feature to gate both together or make the expansion degrade explicitly; add an AC for a second crate in the same build. | spec/functional/FR-004-emission-mode.md:81 |
| FND-002 | medium | Stripping a '::' suffix to find the entry attributes ids to an untagged test in a same-named module (E-3); the number of segments stripped is unstated. Rust allows `fn foo` and `mod foo` side by side. With `#[trace] fn foo` registered as `tests::foo`, an untagged `tests::foo::bar` in `mod foo` matches 'the symbol with a ::-delimited case suffix removed' and receives foo's ids: ids the source never declared for that test (E-3). The rule also does not say whether one or several trailing segments are stripped (rstest `#[values]` matrices nest more than one level). Mark entries as parametrized (the macro can see `#[rstest]` when above it) and map suffixes only for those, stating the depth. | spec/functional/FR-005-emission-run.md:37-43 |
| FND-003 | medium | Attribute order is unstated: #[rstest] above #[trace] re-applies #[trace] to each generated case, registering per-case symbols. Outer attributes expand first. With `#[trace]` above `#[rstest]` the macro sees the source fn and registers it once. With `#[rstest]` above, rstest's expansion copies the remaining attributes onto each generated case fn inside `mod name`, so `#[trace]` registers `tests::name::case_1`, ... instead of `tests::name`, contradicting AC-4 and changing the reported symbol. FR-001 promises either order works for `#[test]`; state the rule for rstest (require `#[trace]` outermost, or handle both) and cover both orders. | spec/functional/FR-004-emission-mode.md:91 |
| FND-004 | low | Rationale for the in-function-body case is inaccurate; the real reason is that libtest never runs such a test. A `#[test]` declared inside a function body is not collected by libtest at all (rustc `unnameable_test_items`), so it never runs; that, not only the symbol mismatch, is why reporting it as unmatched and exiting non-zero is the right outcome. The behaviour is acceptable; correct the stated reason so a later reader does not 'fix' it by inventing a symbol. | spec/functional/FR-004-emission-mode.md:69-72 |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-005 | low | The unsupported #[rstest]-above-#[trace] order produces case-level symbols silently; nothing reports the misuse. The order is documented as unsupported, but a run with it still writes the case's own name as the symbol with no diagnostic, so the E-2 miss is invisible. The ids still bind (FR-120 binds ids regardless of the symbol join), so impact is small; a stderr warning when a record's libtest name is itself a parametrized-looking case of an rstest module, or a documented lint, would surface it. | spec/functional/FR-005-emission-mode.md:75-82 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | round 1, e6f4900 |
| FND-002 | fixed | round 1, e6f4900 |
| FND-003 | fixed | round 1, e6f4900 |
| FND-004 | fixed | round 1, e6f4900 |
