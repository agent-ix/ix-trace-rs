---
id: SR-009
title: "Code review — PLAT-333 emit feature and ix-trace-run runner (PR #17)"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-trace-rs@9e3584b91a36b3b11514f0c0dc40ec62ddd95fa1; src/emit.rs, src/lib.rs, Cargo.toml, .gitignore, ix-trace-run/**, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Code review, with the rust-review and rust-style lane folded
in, of PR #17. The PR adds an opt-in `emit` feature to the macro crate, which
puts a `#[used]` plain-bytes record in a linker section. It also adds a new
workspace member, `ix-trace-run`, which reads those records from the test
binary file, runs each binary once with libtest `--logfile`, and writes one
JUnit file. The scope is `git diff origin/main...HEAD`.

The PR was re-measured on Rust 1.98.1 (the pin from PLAT-176, #16), in a scratch
worktree where the PR head was merged locally with `origin/main`:

- `cargo test --workspace --features emit`: exit 0.
- `cargo test --workspace`: exit 0. The ix-trace-run suites are 9 unit tests, 10
  emit_records, 1 rstest_records, 11 runner_basic and 6 runner_faults tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0,
  but warns that the MSRV differs (FND-006).
- `cargo fmt --all --check`: exit 0.
- `cargo deny check licenses`: `licenses ok`.
- `scripts/check_unsafe_comments.sh`: clean.
- The wasm32-unknown-unknown target is installed, so the FR-005-AC-12 test
  really ran. It did not take its SKIPPED branch.

## Verdict

**REQUEST CHANGES**: one high finding, three medium findings and five low findings.

What is right, checked rather than assumed:

- **With `emit` off, the default expansion is unchanged.** `register` returns
  the item, and `#[implements]` passes `|_, item| item`. No dependency is added:
  `emit = []` and `[dependencies]` stays empty. The runner is a separate
  package, and `cargo package` excludes nested packages.
- **Carrier and gating are correct.** The record is a `static [u8; N]` with
  `#[used]`, built from `concat!` with `core` paths only, inside `const _` at
  the start of the body, after any inner attributes. `link_section` is set per
  OS, giving ELF `ixtrace`, Mach-O `__DATA,__ixtrace` and PE `.ixtrace`. The
  whole item is under
  `cfg(any(linux, macos, all(windows, msvc)))`. The ELF section was confirmed
  with `readelf`, and a wasm build carries no record.
- **`unsafe` and `forbid(unsafe_code)` are not a real break.** Four probe
  crates were built with `emit` on rustc 1.98.1: edition 2021 with
  `#![forbid(unsafe_code)]`, edition 2021 with `#![deny(unsafe_code)]`, edition
  2024 with no lint, and edition 2024 with `#![forbid(unsafe_code)]`. All four
  compiled, passed and carried the `ixtrace` section. Clippy with
  `-D warnings -W clippy::pedantic -W rust-2024-compatibility` was also clean.
  quire-rs, whose `src/lib.rs` is `forbid(unsafe_code)` under edition 2021, is
  therefore not broken. The claim that forbid crates may reject the attribute
  did not reproduce.
- **Runner basics hold.** Records come from the file through `object`, and
  nothing in the binary runs. Each binary runs once with `--logfile` in a
  tempdir that is removed on drop. A line counts only when its name exactly
  matches a listed test. A duplicate line gives `fail` and exit 1. Missing
  tests are rerun alone with `--exact`. Ambiguous libtest names mark nothing
  and exit non-zero. Unmatched flagged records are reported with exit 1. The
  output file is written once, with `name == classname`, `file`, and
  `<property name="trace">` as a direct child of `<testcase>` (quoin FR-120 E-1).
- **Tests can fail.** The runner tests assert concrete `trace` values per case,
  so a build that emitted no records fails `runner_basic` and `runner_faults`.

On symbol naming, the runner takes the libtest name minus the module prefix of
the file. That matches quire-rs `src/symbols/rust.rs`, which walks each file
from an empty container, for top-level, `mod tests` and rstest-source names.
`#[path]` modules are treated as unmatched.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | high | A test that is rerun alone and still writes no log line is recorded `pass` on exit status 0 (`unwrap_or(Outcome::Pass)`). So a test that calls `process::exit(0)` part-way through, skipping its assertions, becomes passing evidence for its ids, and the runner prints no stderr notice. Libtest never reported `ok` for it, which breaks quoin FR-120 E-4 ("SHALL NOT write an outcome the runner did not report") and FR-006's own "a missing line never counts as a normal exit and never as `pass`". The fixture test `a_test_that_exits_zero_is_rerun_alone_and_never_an_unreported_pass` asserts the pass. Fix: a rerun with no log line is `fail`, with a notice, and correct the spec text (SR-010 FND-003) | ix-trace-run/src/run.rs:327-343 |
| FND-002 | medium | The new crate is outside every gate. `make lint`/`make test` and `.github/workflows/ci.yml` run `cargo clippy --all-targets` and `cargo test` at the root, which select only the root package. None of ix-trace-run's tests, which hold every FR-005/FR-006 test, run in `make ci` or CI. Least-touch fix, with no workflow edit: `default-members = [".", "ix-trace-run"]` under `[workspace]` | Cargo.toml:26-27; Makefile:34; .github/workflows/ci.yml:29-32 |
| FND-003 | medium | A failing `--list` is treated as "not a libtest binary", which has three effects. (a) `harness = false` targets are executed with `--list`, which runs their whole `main`. A passing one is then taken as a zero-test libtest binary and never run properly, and a failing one runs twice (reproduced). (b) A libtest binary that cannot start produces no `<testsuite>`, so its tagged tests are absent from the JUnit file with no runner notice (reproduced with `-C prefer-dynamic`: `error while loading shared libraries: libstd-….so`). That breaks E-4's "never absent". (c) A test argument that `--list` rejects gives `unwrap_or_default()`, an empty selection, so the binary is silently not run | ix-trace-run/src/run.rs:103-118, 218-230 |
| FND-004 | medium | The process environment is not the one `cargo test` gives. `command()` sets only `CARGO_MANIFEST_DIR`, with no cargo dylib search path (`LD_LIBRARY_PATH`/`DYLD_FALLBACK_LIBRARY_PATH`/`PATH`, plus the sysroot lib dir), and no `CARGO` or `CARGO_PKG_*`. `target.<triple>.runner` is also ignored. Binaries that link dylibs or use `prefer-dynamic` fail under the runner and pass under `cargo test` (reproduced), which breaks FR-006's "as stock `cargo test` does" | ix-trace-run/src/run.rs:209-215 |
| FND-005 | low | The registry reader silently drops a record that does not parse (wrong field count, flags length, non-UTF-8) or that has another format tag. An emitter/reader version skew, or a record cut by the section, then loses that test's ids with no notice | ix-trace-run/src/registry.rs:257-273, 282-292 |
| FND-006 | low | `rust-version = "1.75"` in the new crate conflicts with the repository pin of 1.98.1 (`clippy.toml` msrv, root `rust-version`). After merging with main, clippy warns "the MSRV in `clippy.toml` and `Cargo.toml` differ" | ix-trace-run/Cargo.toml:5 |
| FND-007 | low | rust-style: the new crate has no `#![forbid(unsafe_code)]` and no workspace safety lints. Its `Error` is a hand-rolled enum without a stable `Copy` code enum or a `thiserror` envelope | ix-trace-run/src/lib.rs:1-39 |
| FND-008 | low | New tests carry no trace attribute although the repository traces tests (rust-style testing): logfile `a_reason_with_a_newline_…` (FR-006-AC-17) and `a_line_naming_no_listed_test_is_not_read` (FR-006-AC-14), matching `the_symbol_drops_the_module_prefix_of_the_file` (FR-006-AC-3), `a_name_outside_its_files_prefix_is_unmatched` (FR-006-AC-9), `two_records_with_one_libtest_name_mark_nothing` (FR-006-AC-19) and `a_parametrized_record_marks_its_cases_…` (FR-006-AC-7), and the registry parse test | ix-trace-run/src/logfile.rs:122,135; ix-trace-run/src/matching.rs:380,394,402,412; ix-trace-run/src/registry.rs:307 |
| FND-009 | low | Fixture crates take `rstest = "0.26"` and `tokio = "1"` from crates.io, and their lockfiles are gitignored. The suite needs the network and floats to new releases, and the rstest attribute behaviour it depends on (SR-010 FND-001) can change underneath it | .gitignore:12-13; ix-trace-run/tests/fixtures/rstest_tokio/Cargo.toml:10-12 |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-010 | high | The FND-004 fix runs each binary through `cargo test --package <id> <selector>`, after removing the caller's `--workspace`/`-p`/target flags. That changes feature unification, so cargo builds and executes a different binary from the one whose records were read. Reproduced on the `unified` fixture: `--no-run --workspace` built `b-c421729ce7e7a4f9`, but the runner ran `b-4082d85133be9902` (b without the unified `emit`). The listed tests, the `cfg(feature)`-gated tests and the outcomes then come from a build the user did not request, while the records come from another. A workspace run like quire-rs can also rebuild per package. The new FR-006 "Execution" text mandates this design. Keep one build and one identity, for example with a single `cargo test` that uses ix-trace-run as the target runner (`--config target.<triple>.runner`) so cargo supplies the env, or execute the inspected executable with the env cargo would give it | ix-trace-run/src/cargo.rs:132-182; ix-trace-run/src/run.rs:236-253; spec/functional/FR-006-emission-run.md:33-35 |
| FND-011 | low | The three new runner tests carry no trace attribute: `a_harness_false_target_is_run_as_is_and_never_given_list`, `a_test_argument_that_list_rejects_is_a_loud_error` and `test_processes_get_cargos_dylib_search_path` | ix-trace-run/tests/runner_faults.rs:144-189 |
| FND-012 | low | E-7 cost risk. Every libtest binary now costs at least two cargo invocations (`--list` and the run), plus one more with test arguments and one per alone-rerun. Each cargo no-op startup measured about 40 ms on a fixture, before any FND-010 rebuild. That is unmeasured over quire-rs's roughly 55 test binaries, so TC-022 must be run before FR-006-CON-1 is claimed | ix-trace-run/src/run.rs:113-140 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | 25363df: a rerun with no log line is `fail` with a stderr notice; the fixture test asserts `<failure>` and the notice |
| FND-002 | fixed | 25363df: `default-members = [".", "ix-trace-run"]`; plain `cargo test` at the root now runs the ix-trace-run suites |
| FND-003 | fixed | 25363df: `harness = false` is read from the manifest and never given `--list`; a `--list` failure and a rejected test argument are loud errors with a non-zero exit; tests for both |
| FND-004 | fixed | 25363df: processes run under `cargo test`, and the prefer-dynamic test passes; the fix introduces FND-010 |
| FND-005 | fixed | 25363df: `Registry.malformed` counts unreadable and other-version entries and the runner warns |
| FND-006 | fixed | 25363df: `rust-version = "1.98.1"`; clippy no longer warns about the MSRV |
| FND-007 | fixed | 25363df: `[lints.rust] unsafe_code = "forbid"`; a `thiserror` `Error` with a `Copy` `ErrorCode` |
| FND-008 | fixed | 25363df: the seven unit tests carry trace attributes |
| FND-009 | fixed | 25363df: fixture lockfiles are committed and pin rstest and tokio; the network is needed once, as disclosed in Known limits |
