---
id: SR-012
title: "Gap analysis — PLAT-333 TC-019 on three platforms (PR #19)"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/ix-trace-rs@fdf0a9bc25299ca834f47fce186d0faa2be0635b; spec/tests.md, spec/functional/FR-005-emission-mode.md, spec/functional/FR-006-emission-run.md, ix-trace-run/tests/emit_records.rs, ix-trace-run/src/registry.rs, .github/workflows/ci.yml"
review_set: subset
---

## Summary

Ticket: PLAT-333. This is a planless gap analysis of what PR #19 claims for
FR-005 and FR-006, checked against the computed matrix (`quoin matrix --json`,
quoin 0.29.1), the tests and CI run 42646173333 (head b5a95a7, green on
ubuntu, macos and windows). Plan completion was not assessed.

Matrix: every FR-005 AC has a binding test. FR-005-AC-10 and TC-019 are on
`tc019_every_tagged_test_keeps_its_record_through_linking`, and FR-005-AC-12 is
on `an_unsupported_target_compiles_and_carries_no_record`. FR-006-AC-3 (`file`
`src/foo/bar.rs`) is on `runner_basic.rs` and `matching.rs` tests. FR-005-CON-2
is Inspection-only.

Claims checked against what runs:

- FR-005-AC-10 on all three legs is backed. The CI log shows TC-019 `ok` on
  Linux, macOS (aarch64-apple-darwin) and Windows MSVC (x86_64-pc-windows-msvc),
  all with rustc 1.98.1. The test checks ten named records in `basic` and one
  record in `x`, which is real non-vacuous evidence.
- The FR-005-AC-12 section-name assertion is explicit on Linux only, as the
  spec says. The new sentence says that on macOS and Windows "the runner finds
  the records by those section names". `read_records` accepts any of
  `ixtrace`, `__ixtrace` and `.ixtrace` on every platform
  (`registry.rs:91`), so a green macOS or Windows leg does not show which name
  was used (FND-001).
- The FR-005-AC-12 `wasm` clause never runs in CI (FND-002).
- The FR-006 File attribute: Windows backslashes reach JUnit as `/`. TC-018
  (`emit_records.rs:87`) and FR-006-AC-3 pass on the Windows leg. SR-011
  FND-001 covers the POSIX side effect.
- The FR-005 row and TC-019 flipping to passing is justified for AC-10. For
  AC-12 it rests on the Linux assertion and on local wasm runs.

Local check: `cargo test -p ix-trace-run --lib --test emit_records` passed. This
host has `wasm32-unknown-unknown` installed, so the wasm test ran for real here.

Units examined:

- `FR-005-AC-10` (examined)
- `FR-005-AC-12` (examined)
- `FR-005` row and `TC-019` row of `spec/tests.md` (examined)
- the known-limits paragraph of `spec/tests.md` (examined)
- `FR-006` File attribute bullet (examined)
- `FR-006-AC-3` (context_only)

## Verdict

**REQUEST CHANGES** on one medium finding. Each fix is a few lines.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | FR-005-AC-12 says "on each supported target the section name is that of its object format". TC-019 asserts the name only under `cfg!(target_os = "linux")`. The new known-limits sentence and test comment say that on macOS (`__DATA,__ixtrace`) and Windows (`.ixtrace`) "the runner finds the records by those section names". That is not shown: `read_records` matches all three names on every OS, so a Mach-O binary with an ELF-named section would also pass. The macOS and Windows legs exist now, so assert `__ixtrace` under `target_os = "macos"` and `.ixtrace` under `windows` in the same block. That makes AC-12's second clause verified on all three legs, as the ✅ on TC-019 implies. Otherwise reword the sentence to say these names are not asserted | ix-trace-run/tests/emit_records.rs:156-162; spec/tests.md:91-94; ix-trace-run/src/registry.rs:91 |
| FND-002 | low | The CI `test` job installs no `wasm32-unknown-unknown` target. `an_unsupported_target_compiles_and_carries_no_record` then prints SKIPPED (hidden without `--nocapture`), returns early and reports `ok` on all three legs. The FR-005-AC-12 wasm clause is never run in CI, and the skip shows as a pass. The spec does disclose that it runs only where the target is installed. Add `targets: wasm32-unknown-unknown` to the test job's toolchain step, at least on the ubuntu leg, so it is checked in CI | .github/workflows/ci.yml:43-44; ix-trace-run/tests/emit_records.rs:183-197 |
