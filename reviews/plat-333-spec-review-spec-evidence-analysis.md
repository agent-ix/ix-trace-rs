---
id: SR-007
title: "Evidence analysis — PLAT-333 emitter spec (PR #15)"
type: SpecReview
analysis: evidence
scope: "agent-ix/ix-trace-rs@7836a1f7dccfab7a27fed5845f561a5b539041a4; spec/functional/FR-004-emission-mode.md, spec/functional/FR-005-emission-run.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Checked each new AC and constraint for a verification method that can fail
and for a TC row in spec/tests.md. Every FR-004/FR-005 AC maps to one of TC-017..TC-021. The
Test-method ACs are each falsifiable by a fixture crate (registry contents, JUnit shape,
outcomes, concurrency, single write). FR-004-CON-2 and FR-005-CON-2 are Inspection, which fits.

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
- `TC-021` (examined): spec/tests.md
- `FR-004-mech-D` (examined): spec/functional/FR-004-emission-mode.md

## Verdict

**APPROVE with one medium and one low finding.** The cost criterion's measurement is
undefined, and `tokio::test`, which the mechanism decision relies on, has no criterion.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | FR-005-AC-9 / CON-1 cost measurement is undefined, and the test lives in ix-trace-rs but needs a quire-rs checkout. The AC does not say whether compile time is included, warm or cold build cache, how many runs form the median, the worker count, or which quire-rs revision. Two runs could pass and fail the same implementation. TC-021 is a Benchmark row in this repo over another repo's suite, with no stated location or invocation. Define the measurement (test execution only, prebuilt binaries, N runs, same worker count) and say where the benchmark runs. | spec/functional/FR-005-emission-run.md:73 |
| FND-002 | low | Mechanism D's rejection of B rests on tokio::test support, but no AC or TC exercises a tagged #[tokio::test]. The mechanism table cites `tokio::test` being kept as a reason to choose D, and E-2 asks for the common shapes, but only rstest has criteria (FR-004-AC-4, FR-005-AC-5). Add a tagged `#[tokio::test]` (attribute above and below `#[trace]`) to TC-018/TC-019. | spec/functional/FR-004-emission-mode.md:42 |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-003 | low | The 105% baseline is stock `cargo test`, which also runs doctests the runner does not; the two modes do not run the same tests. With one process per binary as stock, 105% is realistic (the extra cost is reading sections and writing JUnit). But `cargo test` runs doctests, which the runner does not execute, so the ratio can pass by dropping work. Compare against `cargo test --tests` (or the exact targets the runner runs). | spec/functional/FR-006-emission-run.md:92 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | round 1, e6f4900 |
| FND-002 | fixed | round 1, e6f4900 |
