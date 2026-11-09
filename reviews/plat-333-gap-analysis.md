---
id: SR-010
title: "Gap analysis — PLAT-333 emit feature and ix-trace-run runner (PR #17)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/ix-trace-rs@9e3584b91a36b3b11514f0c0dc40ec62ddd95fa1; spec/functional/FR-001,FR-005,FR-006, spec/non-functional/NFR-001, spec/tests.md TC-017..022, src/, ix-trace-run/; quoin FR-120 emitter contract E-1..E-10 (agent-ix/quoin origin/main)"
review_set: subset
---

## Summary

Ticket: PLAT-333. This is a planless gap analysis of FR-005, FR-006, FR-001
and NFR-001, and of TC-017..TC-022, against the computed matrix
(`quoin matrix --repo . --json`, quoin 0.29.1, quire 0.36.2). It also checks
the emitter contract E-1..E-10 of quoin FR-120. Plan completion: not assessed.

Computed matrix: every FR-005 AC (1..13) and FR-006 AC (1..11 and 13..19) is
tagged on a test. Three items are not:

- FR-006-AC-12 and FR-006-CON-1 are untagged.
- FR-005-CON-2 and FR-006-CON-2 are Inspection items with no symbol.
- FR-001-CON-1 is untagged. That predates this PR.

Emitter contract status:

- **Met:** E-1 (property carrier), E-3 (declared ids only), E-5 (once, unmixed),
  E-6 (inert when off) and E-8 (single writer).
- **Met except for parametrized cases:** E-2. rstest written below `#[trace]`
  works. rstest written above it does not (FND-001).
- **Weakened:** E-4, by the outcome given to a rerun that writes no log line
  (SR-009 FND-001, FND-003 here).
- **Not measured:** E-7 (FR-006-AC-12).
- **Out of this repository:** E-10, which is quire-rs FR-051-AC-33.

The coder disclosed these gaps, and each was checked:

- FR-006-AC-12 (E-7 cost) was not run. Deferring it is legitimate, because TC-022 stays 🚧.
- FR-005-AC-10 was verified on Linux only. TC-019 stays 🚧, which is
  correct.
- The second clause of FR-005-AC-4 fails. This is a spec correction, not a code
  fix (FND-001).
- `harness = false` and `LD_LIBRARY_PATH` were real problems, and both are
  recorded in SR-009 (FND-003, FND-004).

## Verdict

**REQUEST CHANGES**: three medium findings and three low findings. The fixes are
two spec corrections, which belong in this PR because the spec merged in #15,
plus two test and status corrections.

The rstest behaviour was measured on rstest 0.26.1 with probe crates:

- **Bare `#[trace("X")]` below `#[rstest]` (between, before or after `#[case]`
  lines):** rstest consumes it as its own argument-tracing `#[trace]`
  attribute. The binary has no record, and the build only warns
  `unused import: ix_trace_rs::trace`.
- **Path-qualified `#[ix_trace_rs::trace("X")]` after the `#[case]` lines:**
  copied onto each case, giving `case_1` and `case_2` records with the flag
  clear. This is the shape the FR-006 misuse warning detects.
- **The same form before the first `#[case]`:** attached to `case_1` only.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | FR-005-AC-4 clause 2, the FR-005 "Attribute order" text, and the FR-006 misuse warning (AC-13) assume rstest copies `#[trace]` onto each case. rstest 0.26.1 instead consumes a bare `#[trace]` as its own attribute, so the ids silently vanish with no record and no warning. Only the path-qualified spelling behaves as specified. This is a spec correction (proposed text in the posted comment), not a code fix: the macro never sees the attribute, and the runner has no record to warn on | spec/functional/FR-005-emission-mode.md:102-105,132; spec/functional/FR-006-emission-run.md:92-97,122 |
| FND-002 | medium | FR-006-AC-15 ("a listed test with two log lines yields a failing case and a non-zero runner exit") is bound only to the `logfile::parse` unit test, which asserts the `duplicates` set. Deleting the duplicate branch in `run::execute` would leave the suite green, so neither the failing case nor the exit code is verified. Likewise FR-006-AC-13's "does not change its exit status" is asserted nowhere | ix-trace-run/src/logfile.rs:111-120; ix-trace-run/src/run.rs:293-299 |
| FND-003 | medium | FR-006 contradicts itself. "Log lines" says "a missing line never counts as a normal exit and never as `pass`", but "Abnormal exit" says the alone rerun's "exit status 0 is `pass`". The code and the FR-006-AC-16 test follow the second reading, which conflicts with quoin FR-120 E-4 (SR-009 FND-001). Correct the Abnormal-exit sentence (proposed text in the posted comment) | spec/functional/FR-006-emission-run.md:46-48,57-60 |
| FND-004 | low | spec/tests.md flips TC-020 and TC-021 to ✅ too early. TC-020 claims "a missing or duplicate line fails", which is only parser-tested (FND-002). TC-021 claims "the rstest-above warning is printed", which is tested only on synthetic records and cannot occur for the bare spelling (FND-001) | spec/tests.md:73-74 |
| FND-005 | low | Some clauses are only partly tested. FR-005-AC-1's first clause ("yields exactly the input tokens") is checked only through the absence of a section. FR-005-AC-12's second clause (section name per object format) is checked on Linux only. FR-006-AC-11's second clause ("plain `cargo test` writes no `trace` property") is vacuous, because plain `cargo test` writes no JUnit file | ix-trace-run/tests/emit_records.rs:159-165,307-334; ix-trace-run/tests/runner_basic.rs:147-171 |
| FND-006 | low | FR-006 "Reported symbol" applies the file-relative chain to every case, but cases with no record are written with the raw libtest name and no `file` (for example `plain_tagged::untagged_inside`). quoin's store join (FR-030-AC-18) then sees module-qualified names it cannot join. Either narrow the spec to cases that have a record, or say that untagged cases bind nothing and need no join | ix-trace-run/src/run.rs:158-164; spec/functional/FR-006-emission-run.md:70-80 |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-007 | low | The new FR-006 "Unlistable binaries" behaviour (two SHALLs: run `harness = false` as is; a `--list` failure or a rejected argument is reported with a non-zero exit) has no acceptance criterion, so its tests bind to nothing in the matrix | spec/functional/FR-006-emission-run.md:45-48 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | 25363df: the FR-005 Attribute order text, AC-4 and the FR-006 Misuse warning now describe rstest consuming a bare `#[trace]`; the `misuse` fixture proves the per-case records |
| FND-002 | fixed | 25363df: the `dup_log` fixture proves AC-15 end to end (a failing case and exit 1); AC-13's unchanged exit is asserted on the `misuse` fixture |
| FND-003 | fixed | 25363df: Abnormal exit reads the rerun's log; no log line is `fail` with a notice; AC-16 is reworded to match |
| FND-004 | fixed | 25363df: TC-020 and TC-021 now rest on end-to-end tests; TC-019 and TC-022 stay pending |
| FND-005 | accepted-no-change | The partial clauses remain partial, but a Known limits paragraph in spec/tests.md now states each one (AC-1 by absence of a section, AC-10 and AC-12 on Linux only), and FR-006-AC-11's vacuous clause was removed |
| FND-006 | fixed | 25363df: Reported symbol is narrowed to cases that have a record; untagged cases are stated to need no join |
