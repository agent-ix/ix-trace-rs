---
id: SR-015
title: "Evidence analysis of PLAT-333 TC-022 flip (ix-trace-rs PR 20)"
type: SpecReview
analysis: evidence
scope: "agent-ix/ix-trace-rs; spec/tests.md, ix-trace-run/tests/cost_quire_rs.rs (PR 20 diff only, head f349cfc)"
review_set: subset
---

## Summary

Ticket: PLAT-333. Every number in the new Verification Status paragraph was checked against the raw measurement logs (run 3 output; the two PLAT-333 measurement notes for runs 1 and 2) and against the test's bound.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | "load average 7-14" does not match the recorded readings: run 1 was 11-14, run 2 was 1.3 at start, 10.4 mid-run and 14.6 at the end, and run 3's log records load 1 at start and no in-run reading; the range should be stated from the readings (for example "up to 14.6 during the runs") | spec/tests.md:101 |

## Verdict

Mergeable. The test `tc022_the_runner_adds_under_five_percent_over_quire_rs` asserts `runner / stock < LIMIT` with `LIMIT = 1.05`, median of at least five alternating rounds, `--test-threads=4` by default for both arms, binaries prebuilt by `cargo test --no-run`, revision printed: it matches FR-006-AC-12 and CON-1. The figures all check out: run 1 92.77/83.79 = 90.3%; run 2 87.60/84.30 = 96.2%; run 3's medians from its five rounds are stock 123.38 s and runner 127.64 s, 103.5%, revision 5a2a933..., test `ok`. 4 threads, five rounds, release build and the 105% limit are all correct. "no run had a quiet host" holds, because load rose during runs 1 and 2. For run 3 the log has no in-run reading, so that clause is unverified for run 3, but an error there would understate the evidence, not overstate it.
