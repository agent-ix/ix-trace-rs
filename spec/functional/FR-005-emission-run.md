---
id: FR-005
title: "The Rust run writes JUnit with trace marks"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ix-trace-rs/FR-004"
    type: "extends"
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
---

# FR-005: The Rust run writes JUnit with trace marks

## Description

The companion crate `ix-trace-run` SHALL provide a runner that executes a
consumer's `emit`-enabled test binaries under libtest and itself writes the final
JUnit result file, in which each case carries the `trace` property of
[quoin FR-120](ix://agent-ix/quoin/FR-120) for the ids registered by
[FR-004](./FR-004-emission-mode.md). This is the run half of the emitter
contract, and the harness choice E-9 leaves to PLAT-333.

## Behavior

- The runner SHALL list the tests of each test binary with libtest's own `--list`,
  read the binary's registry, run each listed test to completion in its own
  process with libtest's `--exact`, and take the case's outcome from that
  process: exit status 0 is `pass`, a panic or any other non-zero exit or signal
  is `fail` (E-4), and a test the binary lists as ignored is `skip`.
- Tests SHALL run concurrently up to a bounded worker count defaulting to the
  available parallelism. Each worker returns its test's result to a single
  writer; no state is shared between tests, so a case's ids are never mixed with
  another's (E-5).
- Each `<testcase>` SHALL carry `name` and `classname` both equal to the symbol
  (the libtest name, FR-004), a `file` attribute holding the registered `file!()`
  value normalized to a repo-relative, `/`-separated path, and one
  `<property name="trace" value="ID1,ID2"/>` child when the symbol, or the symbol
  with a `::`-delimited case suffix removed, is registered. Each case of a
  parametrized entry carries the entry's ids and the entry's symbol as `name`; a
  failing case is a failing entry for the symbol.
- A case with no registry entry SHALL carry no `trace` property.
- The runner SHALL report a registered symbol that matches no listed test on stderr
  and exit non-zero, so a tagged test the runner could not
  join cannot silently produce no evidence.
- The runner SHALL write the result file itself and exactly once, after all
  workers finish. It creates no other file and rewrites nothing afterward
  (E-8).
- Running the same test binaries with plain `cargo test` is unchanged and writes
  no `trace` property.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-005-CON-1 | While emission is enabled the runner SHALL add under 5% (target under 2%) to the median wall-clock time of the suite under stock `cargo test`, measured on quire-rs (E-7). | Design | Test |
| FR-005-CON-2 | The result file SHALL be the only artifact a run writes; no sidecar is merged and no post-run step rewrites names or ids (E-8). | Design | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | A passing tagged test yields a `<testcase>` with `name` equal to `classname` equal to `tests::<name>`, a `file` attribute, no failure child and a `trace` property holding its ids. | Test |
| FR-005-AC-2 | A tagged test that panics yields a `<failure>` child and its `trace` property; it is never absent. | Test |
| FR-005-AC-3 | A tagged `#[ignore]` test yields `<skipped/>` and its `trace` property. | Test |
| FR-005-AC-4 | Two tagged tests run concurrently with distinct ids yield two cases whose properties contain only their own ids, and each appears once. | Test |
| FR-005-AC-5 | Every case of a tagged `#[rstest]` function carries `name` equal to the source function name and that function's ids, and one failing case yields a failing entry for the symbol. | Test |
| FR-005-AC-6 | A registered symbol that matches no listed test makes the runner report it and exit non-zero. | Test |
| FR-005-AC-7 | With emission disabled, `cargo test` on the same fixture runs unchanged and writes no `trace` property. | Test |
| FR-005-AC-8 | The runner creates exactly one output file per invocation, and that file is not modified after it is first written. | Test |
| FR-005-AC-9 | On quire-rs the median wall-clock of the runner over its suite stays within the FR-005-CON-1 bounds of stock `cargo test`. | Test |

## Dependencies

- **Upstream**: [FR-004](./FR-004-emission-mode.md), quoin FR-120.
- **Downstream**: `quoin evidence record` reads the file; quire-rs FR-051-AC-32
  demonstrates the chain (E-10).
