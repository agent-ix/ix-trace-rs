---
id: FR-006
title: "The Rust run writes JUnit with trace marks"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ix-trace-rs/FR-005"
    type: "extends"
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
---

# FR-006: The Rust run writes JUnit with trace marks

## Description

The runner `ix-trace-run` (a binary crate of this repository, the only one with
dependencies) SHALL write the run's final JUnit result file in which each case
carries the `trace` property of [quoin FR-120](ix://agent-ix/quoin/FR-120) for the
ids recorded by [FR-005](./FR-005-emission-mode.md). This is the run half of the
emitter contract and the harness choice E-9 leaves to PLAT-333.

## Behavior

- **Records.** The runner SHALL locate each test binary from
  `cargo test --no-run --message-format json`, read its trace section (named per object format by FR-005) from the
  file with an object-file reader, and execute no injected code. The binary's
  `--list` output is therefore identical to a build without `emit`. A binary with
  no trace section (an unsupported target, or no tagged test) yields a stderr
  notice naming it and no marks, and does not by itself change the exit status.
- **Execution.** The runner SHALL run each test binary once, in the default
  parallel mode and without `--nocapture`. The binary it executes SHALL be the file
  it read the records from, taken from the build messages of the one
  `cargo test --no-run`; it SHALL NOT invoke `cargo test` again, because a second
  invocation can unify features differently and build another binary. It SHALL give
  the process the environment `cargo test` gives it, rebuilt from the build messages
  and `cargo metadata`: the package root as working directory, `CARGO`,
  `CARGO_MANIFEST_DIR`, the `CARGO_PKG_*` variables of the owning package, and a
  dynamic library search path (`LD_LIBRARY_PATH`, `DYLD_FALLBACK_LIBRARY_PATH` or
  `PATH`) made of the binary's `deps` directory, the compiler's library directory and
  the native library directories of the build scripts. A configured target runner
  (`target.<triple>.runner` in cargo config, a `CARGO_TARGET_<TRIPLE>_RUNNER`
  variable or a `--config` with a runner) is unsupported and the runner SHALL stop
  with an error that says so. It passes libtest's
  `--logfile <temporary path>`. Outcomes SHALL come from that log file, which
  the harness writes and test output cannot reach, and never from stdout. Its
  line forms are `ok <name>`, `failed <name>`, `ignored <name>` and
  `ignored: <reason> <name>`, where `<name>` is the listed test name (a
  `should_panic` test is `ok` when it panics); a line is read as a name only when
  its remainder equals a listed name exactly. `ok` is `pass`, `failed` is `fail`
  (a panic is `failed`, E-4) and `ignored` is `skip`. The logfile option is
  accepted by stable libtest and marked deprecated in its help; its removal is
  a reason to revisit this by amendment.
- **Unlistable binaries.** A target with `harness = false` SHALL be run as is,
  without libtest flags, and yields no cases (FR-006-AC-20). A libtest binary whose
  `--list` fails, and a test argument that `--list` rejects, SHALL be reported on
  stderr and the runner SHALL exit non-zero (FR-006-AC-21).
- **Log lines.** Every listed test SHALL have exactly one log line, whatever the
  exit status. A duplicate line makes the test `fail` and the runner exit
  non-zero, never `pass`. A test with no log line takes the abnormal-exit path
  below, including a test that calls `process::exit(0)`: a missing line never
  counts as a normal exit and never as `pass`. A reason may contain a newline,
  which splits the line: after an `ignored:` line the runner SHALL read
  continuation lines as part of the reason until a line ends with a space and a
  listed test name not yet seen, and that name is the test. An exit status 0 with a `failed`
  line, or a non-zero status with no `failed` line, is an inconsistency: the runner
  records the log's outcomes, reports the conflict on stderr and exits non-zero.
- **Cleanup.** The temporary log file is the only other file a run creates; it
  lies outside the result path, is read only by the runner and is removed before
  the runner exits.
- **Abnormal exit.** When a binary exits abnormally (a signal, `abort`, a
  process exit inside a test) or any listed test has no log line, the runner SHALL run every listed test that has no
  log line alone in a fresh process with `--exact`; the outcome is read from that
  run's log file: `ok` is `pass` and `ignored` is `skip`; a `failed` line, no log
  line, any non-zero status or a signal is `fail`, and a run that exits 0 with no
  log line is reported on stderr. No listed test is absent from the result (E-4).
- **Matching.** A record belongs to a listed test when the record's libtest name
  (`module_path!()` without its first segment, joined by `::` to the identifier)
  equals the listed name exactly, whether or not the record's `test` flag is set,
  and exactly one record has that libtest name. When two or more records have one
  libtest name, the runner SHALL mark no test with any of them, report the
  ambiguity on stderr naming the records' files and ids, and exit non-zero (E-3).
  A record with the `parametrized` flag also
  belongs to every listed name that is that libtest name followed by `::` and one
  or more further segments. No other test receives a record's ids (E-3).
- **Reported symbol.** For each case that a record belongs to, the runner SHALL
  report, as `name` and `classname`, the file-relative chain the extractor reports: the libtest name with the module
  prefix of the source file removed. The prefix of a file is derived from its
  path relative to the directory of its target's root file (as `cargo` reports
  `src_path`): the root file itself, `lib.rs`, `main.rs` and `tests/x.rs` have an
  empty prefix; `foo/bar.rs` has `foo::bar`; `foo/mod.rs` has `foo`. A test in
  `mod tests` of `src/foo/bar.rs` has libtest name `foo::bar::tests::t` and
  reported name `tests::t`. A parametrized case reports its source function's
  file-relative name. When a libtest name does not begin with its file's derived
  prefix (for example under `#[path]`) the runner SHALL treat the record as
  unmatched (below) rather than guess. A record with the `test` flag clear that
  matches no listed test is ignored without a report. A case that no record
  belongs to is reported under its libtest name, with no `file` and no `trace`
  property; it carries no ids, so it needs no join to a declared symbol.
- **File attribute.** Each `<testcase>` SHALL carry `file`, the record's `file!()`
  value normalized to a repo-relative, `/`-separated path, and one
  `<property name="trace" value="ID1,ID2"/>` child when a record belongs to it.
  A case with no record carries no `trace` property.
- **Unmatched records.** The runner SHALL report on stderr every record with the
  `test` flag set that belongs to no listed test, and SHALL exit non-zero, so a tagged test that cannot
  run produces no silent absence.
- **One writer.** The runner SHALL write the result file once, after every binary
  finishes, and SHALL rewrite nothing afterward (E-8). The runner creates no other file that outlives the run.
- **Misuse warning.** The runner SHALL warn on stderr, without failing, when two
  or more records of one file carry identical ids and identifiers beginning
  `case_`, the shape of a path-qualified `#[ix_trace_rs::trace]` below `#[rstest]`
  (FR-005). The warning is best effort: rstest `#[values]` matrices generate
  functions named after their arguments rather than `case_`, and those misuse
  cases get no warning. A bare `#[trace]` below `#[rstest]` leaves no record, so it
  cannot be warned about.
- **Plain runs.** Running the same test binaries with plain `cargo test` is
  unchanged.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-006-CON-1 | While emission is enabled, the runner SHALL add under 5% (target under 2%) to the median wall-clock time of the suite under stock `cargo test` (E-7), measured as FR-006-AC-12 states. | Performance | Test |
| FR-006-CON-2 | A run SHALL write the result file and, apart from the temporary log file removed before exit, nothing else; no sidecar is merged and no post-run step rewrites names or ids (E-8). | Design | Inspection |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-006-AC-1 | The runner reads records from the test binary file alone: the binary's `--list` output under the runner equals its output under stock libtest, and no case exists for a record. | Test |
| FR-006-AC-2 | A passing tagged test yields a `<testcase>` with `name` equal to `classname`, a `file` attribute, no failure child and a `trace` property holding its ids. | Test |
| FR-006-AC-3 | A tagged test in `mod tests` of a non-root source file `src/foo/bar.rs` reports `name` `tests::t` and `file` `src/foo/bar.rs`, and a test at the top level of `tests/x.rs` reports its bare name. | Test |
| FR-006-AC-4 | A tagged test that panics yields a `<failure>` child and its `trace` property; it is never absent. | Test |
| FR-006-AC-5 | A tagged `#[ignore]` test yields `<skipped/>` and its `trace` property; a tagged `should_panic` test that panics yields a passing case. | Test |
| FR-006-AC-6 | Two tagged tests that run concurrently with distinct ids yield two cases whose properties hold only their own ids, each once. | Test |
| FR-006-AC-7 | Every case of a tagged `#[rstest]` function written below `#[trace]` reports the source function's file-relative name and its ids, one failing case yields a failing entry for that symbol, and an untagged test in a module of the same name as a non-parametrized tagged function receives no `trace` property. | Test |
| FR-006-AC-8 | When a test binary aborts inside one test, every listed test without a log line is re-run alone and appears in the result with an outcome; none is absent. | Test |
| FR-006-AC-9 | A record that belongs to no listed test, including a `#[test]` declared inside a function body and a test whose libtest name does not begin with its file's derived prefix, is reported on stderr and makes the runner exit non-zero. | Test |
| FR-006-AC-10 | A tagged `#[tokio::test]` yields one case with its ids and its outcome. | Test |
| FR-006-AC-11 | A run leaves exactly one output file, which is not modified after it is first written, and its temporary log file is gone. | Test |
| FR-006-AC-13 | Two records in one file with identical ids and identifiers `case_1` and `case_2` make the runner print a warning on stderr and not change its exit status; the warning is not required for `#[values]` matrices. | Test |
| FR-006-AC-14 | A test that writes `test other ... ok` or `test self ... FAILED` to stdout directly does not change any case's outcome. | Test |
| FR-006-AC-15 | A listed test with two log lines yields a failing case and a non-zero runner exit; an `ignored: <reason> <name>` line yields a skipped case. | Test |
| FR-006-AC-16 | A test that calls `process::exit(0)` has no log line, is re-run alone, and appears in the result as `fail` with a stderr notice, never as `pass`. | Test |
| FR-006-AC-17 | A test marked `#[ignore = "line one\nline two"]` yields one skipped case under its own name, and the tests listed after it keep their own outcomes. | Test |
| FR-006-AC-18 | A binary with no trace section yields a stderr notice and no marks, and the run's exit status is that of its tests. | Test |
| FR-006-AC-19 | A method `#[trace("X")] fn check` without a receiver and a free `#[test] fn check` in one module give two records with one libtest name: neither test receives `X`, and the ambiguity is reported with a non-zero exit. | Test |
| FR-006-AC-20 | A target declared `harness = false` is executed once with no libtest flag, produces no case, and its exit status is the run's contribution for it. | Test |
| FR-006-AC-21 | A test argument that the binary's `--list` rejects produces a stderr error naming the argument and a non-zero exit, and no test of that binary is silently skipped. | Test |
| FR-006-AC-22 | The executed test binary path equals the path the records were read from, including in a workspace where feature unification differs between invocations, and a binary built with `-C prefer-dynamic` finds its libraries. | Test |
| FR-006-AC-23 | With a target runner configured, the runner stops with an error stating that target runners are unsupported and writes no result file. | Test |
| FR-006-AC-12 | Over the quire-rs suite, with test binaries prebuilt by one `cargo test --no-run` (compile time excluded), a warm cache, the same `--test-threads` value for both modes, and the quire-rs revision recorded in the benchmark output, the median of at least five alternating runs of the runner is under 105% of the median of the same runs of `cargo test --tests` (stock without doctests, which the runner does not run). | Test |

## Dependencies

- **Upstream**: [FR-005](./FR-005-emission-mode.md), quoin FR-120.
- **Downstream**: `quoin evidence record` reads the file; quire-rs FR-051-AC-33
  demonstrates the chain (E-10).
