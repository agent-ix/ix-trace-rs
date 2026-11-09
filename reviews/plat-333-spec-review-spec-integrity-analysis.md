---
id: SR-005
title: "Spec integrity analysis — PLAT-333 emitter spec (PR #15)"
type: SpecReview
analysis: integrity
scope: "agent-ix/ix-trace-rs@7836a1f7dccfab7a27fed5845f561a5b539041a4; spec/functional/FR-004-emission-mode.md, spec/functional/FR-005-emission-run.md, spec/functional/FR-001-trace-attribute.md, spec/spec.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Checked id uniqueness, cross-references, matrix rows and consistency between
the new FR-004/FR-005 and the edited FR-001, spec.md and tests.md. `quire validate` passes on
every changed requirement file; `spec/tests.md` fails one TestMatrix column assertion
(`Coverage Status` vs `Status`) that is pre-existing on main and outside this diff. `quire matrix`
was run over the PR tree. TC-017..TC-021 do not collide with existing TC ids (max TC-016).

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
- `FR-001` (examined): spec/functional/FR-001-trace-attribute.md
- `TC-017` (examined): spec/tests.md

## Verdict

**REQUEST CHANGES.** The new requirement reuses the id `FR-004`, already held by
`FR-004-string-edge-attribute.md`; the computed matrix already shows the string_edge tests
falsely backing the new FR-004-AC-1..3 and CON-1. One medium contradiction (FR-001 and spec.md
still promise an unconditional identity expansion) and one low inconsistency.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | high | Id collision: FR-004 is already the string_edge requirement; the matrix now counts string_edge tests as evidence for the emitter ACs. `spec/functional/FR-004-string-edge-attribute.md` (merged in #14) already has `id: FR-004` with FR-004-AC-1..3 and FR-004-CON-1. The new file reuses FR-004, so `quire matrix` on the PR tree reports emitter FR-004-AC-1 tagged by tests/string_edge.rs:47, AC-2 by tests/compile_fail.rs:22, AC-3 by tests/string_edge.rs:66 and CON-1 by tests/dependency_posture.rs:13: green status for behaviour that does not exist. tests.md now has two `FR-004` rows and spec.md links FR-004 to two files. Renumber the emitter requirements (FR-005 and FR-006) and every reference to them here and in quire-rs#612. | spec/functional/FR-004-emission-mode.md:2 |
| FND-002 | medium | FR-001 statement, Outputs and Behavior and spec.md In Scope still promise an unconditional identity expansion; only CON-1 was qualified. With `emit` on, FR-004 makes the expansion the item plus a registry entry. FR-001's statement ('expands to the annotated item unchanged'), Outputs ('byte-for-byte the input token stream') and Behavior ('SHALL return the annotated item unchanged') and spec.md line 56 ('The guarantee that all expand to the annotated item unchanged') remain unconditional, so FR-001 and FR-004 contradict each other. Qualify each with 'while `emit` is not enabled' as FR-001-CON-1 now is. | spec/functional/FR-001-trace-attribute.md:19,31,38 |
| FND-003 | low | FR-005-AC-5 says rstest cases carry the source function name; the Behavior says they carry the entry's symbol (tests::<name>). FR-005 Behavior and FR-004 say each parametrized case carries the entry's symbol (module chain plus function name, e.g. `tests::name`), but AC-5 says `name` equal to 'the source function name'. A test asserting AC-5 literally checks the wrong value. Say 'the entry's symbol'. | spec/functional/FR-005-emission-run.md:69 |

## New findings (disposition pass 2)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-004 | low | FR-006 'One writer' still says the runner creates no other file, contradicting the new Cleanup bullet and CON-2. Cleanup and FR-006-CON-2 now allow a temporary log file removed before exit, but the One writer bullet was not updated and still says 'SHALL create no other file'. Qualify it the same way. | spec/functional/FR-006-emission-run.md:80-81 |
| FND-005 | low | 'Any fn item SHALL embed a record' inside its body is impossible for a bodyless fn (trait method declaration, extern fn). Placement puts the record inside the function body, but `fn f();` in a trait or an `extern` block has no body. FR-001 allows `#[trace]` there today. State that a bodyless fn expands unchanged with no record. | spec/functional/FR-005-emission-mode.md:76-78 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | round 1, e6f4900 |
| FND-002 | fixed | round 1, e6f4900 |
| FND-003 | fixed | round 1, e6f4900 |
