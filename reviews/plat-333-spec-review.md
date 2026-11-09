---
id: SR-004
title: "Spec review — PLAT-333 Rust trace-mark emitter (PR #15)"
type: SpecReview
analysis: base
scope: "agent-ix/ix-trace-rs@7836a1f7dccfab7a27fed5845f561a5b539041a4; spec/functional/FR-004-emission-mode.md, spec/functional/FR-005-emission-run.md, spec/functional/FR-001-trace-attribute.md, spec/non-functional/NFR-001-dependency-posture.md, spec/spec.md, spec/tests.md, README.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Reviewed `git diff origin/main...HEAD` (head 7836a1f) against the merged
emitter contract E-1..E-10 of quoin FR-120 and ADR-100 (quoin origin/main), and against the
quire-rs Rust extractor (`src/symbols/rust.rs`, origin/main, read-only).

Contract mapping, as written: E-1 (mark only), E-3 (declared ids), E-4 (outcome from the test
process), E-5 (process isolation, single writer), E-6 (opt-in `emit`, byte-identical default,
FR-004-CON-1/2 and AC-1), E-8 (single write, no sidecar, FR-005-CON-2 and AC-8) and E-10
(delegated to quire-rs FR-051-AC-32) are addressed. E-2/E-9 (symbol equals the extractor's
`qualified_name`) is not met for any test outside a crate-root file (FND-001). E-7 is stated
(FR-005-CON-1) but the chosen process-per-test design makes it an unmeasured feasibility risk
(FND-004). The macro crate stays dependency-free (NFR-001 edit). No vendoring, compatibility
layer or ceremony was introduced.

The "tagged test inside a function body exits non-zero" outcome is acceptable: libtest never
collects a `#[test]` declared inside a function body, so the test never runs and failing the run
is the correct signal (the rationale text is inaccurate; see SR-008).

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
- `FR-004 behavior (sym)` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004 behavior (exp)` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004 behavior (nextest)` (examined): spec/functional/FR-004-emission-mode.md
- `FR-005 behavior (run)` (examined): spec/functional/FR-005-emission-run.md
- `FR-005 behavior (case)` (examined): spec/functional/FR-005-emission-run.md

## Verdict

**REQUEST CHANGES.** Two high findings: the registered symbol is the libtest name, which differs
from the extractor's `qualified_name` for every unit test in a non-root source file, so E-2/E-9
fail for most real tests; and emit-mode expansion is specified for every item although FR-001
permits `#[trace]` on structs and methods, which then fails to compile or fails the run. Three
medium findings on the unspecified registry read, the E-7 cost risk and the unevidenced nextest
rejection. Mechanism choice D itself (opt-in registry plus a runner that writes JUnit) is a sound
fit for E-1, E-3, E-4, E-5, E-6 and E-8.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | high | Registered symbol is the libtest name, not the extractor's qualified_name, for any test outside a crate-root file (E-2/E-9). The quire-rs extractor walks each file from an empty container (src/symbols/rust.rs `parse` -> `walk(root, None, ...)`), so a test in `mod tests` of `src/foo/bar.rs` has qualified_name `tests::t` with path `src/foo/bar.rs`, while `module_path!()` minus the crate gives the libtest name `foo::bar::tests::t`. The same holds for `tests/x/main.rs` submodules. FR-004 claims the two agree for module-only chains; they agree only in lib.rs/main.rs/bin roots and single-file integration tests, which is all FR-004-AC-2/AC-3 and quire-rs FR-051-AC-31's fixture cover. FR-005 then writes the libtest name as name/classname, violating E-2/E-9 for most unit tests. The registry must keep the libtest name as the join key for --list and report the file-relative chain (module_path minus the module path of the file that file!() names) as name; add an AC with a tagged test in a non-root source file. | spec/functional/FR-004-emission-mode.md:58-65 |
| FND-002 | high | Emit-mode expansion is specified for every annotated item, but FR-001 allows #[trace] on structs and methods: compile error in impl blocks, run failure elsewhere. FR-001-AC-3 makes `#[trace]` valid on a struct and FR-001 places no restriction to test functions. FR-004 says that with `emit` on the expansion is always the item followed by a registry entry. A registry static after a method inside an `impl` block is not valid Rust, so enabling `emit` breaks compilation of code that is valid in inert mode; on a struct, helper fn or any non-test item the entry registers a symbol no `--list` contains, so FR-005-AC-6 makes the runner exit non-zero. The spec needs a rule for which items register (for example only fns carrying a test attribute visible to the macro, with the remaining cases stated) and an AC for a non-test item under `emit`. | spec/functional/FR-004-emission-mode.md:50-55 |
| FND-003 | medium | How the runner reads a stock libtest binary's link-time registry is unspecified. A stock libtest binary's `main` is libtest; it offers no hook to dump a link-time slice. The runner must either parse the object file's linker section (per-platform section names, an object-parsing dependency in ix-trace-run) or have a hidden test injected that prints the registry (which then appears in --list and in the JUnit as an extra case, contradicting FR-004-AC-7's 'neither registered nor affected'). Mechanism D's feasibility rests on this step, two implementers would diverge, and no AC constrains it. State the mechanism and its visible effects. | spec/functional/FR-005-emission-run.md:28-32 |
| FND-004 | medium | Process-per-test makes the E-7 <5% bound an unmeasured feasibility risk; a single-process alternative was not evaluated. Stock cargo test runs each binary's tests as threads in one process; FR-005 spawns one process per test (quire-rs main has about 1343 `#[test]`/`#[rstest]`/`#[tokio::test]` attributes), re-running process start-up and any lazily initialised shared fixture per test. Whether that stays under 5% is unknown and the spec records no measurement or fallback. Running each binary once and taking per-test outcomes from libtest's own per-test status lines on stable was not considered and would keep the stock cost. At minimum record a spike measurement before committing to process-per-test, or name the alternative. | spec/functional/FR-005-emission-run.md:58 |
| FND-005 | medium | nextest rejection is asserted as fact without evidence, although E-9 explicitly names a confirmed nextest feature as an option. E-9 leaves the choice to PLAT-333 between a custom harness and 'a nextest feature confirmed to provide it'. FR-004 states nextest's JUnit 'has no carrier for source-declared per-test properties' with no version, doc or test cited; the PR body itself calls the claim unverified. The claim is probably true for nextest's own JUnit, but it decides an architecture choice and must be checked and cited (nextest version, config surface examined). It also does not consider nextest as the executor under ix-trace-run (nextest already runs process-per-test with timeouts, retries and signal handling, and emits a structured event stream the runner could consume and write JUnit from itself without a post-run rewrite). | spec/functional/FR-004-emission-mode.md:44-46 |

## New findings (disposition pass 1)

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-006 | high | With #[test] written above #[trace] the macro never sees a test attribute, so the test gets no record and silently loses its evidence; FR-005 claims #[test] works in either order. Reproduced on rustc 1.98.1: an attribute macro below `#[test]` receives `fn below() { ... }` with no `#[test]` and no marker in its input (rustc consumes `#[test]` first), while one above receives `#[test] fn above()`. Under the qualifying-function rule the below case writes no record, so the test runs untagged, gets no `trace` property, and nothing reports it (only records can be unmatched). FR-005 says '`#[tokio::test]` and `#[test]` work in either order' and FR-001 still promises either order; quire-rs main has 12 `#[test]`-above-`#[trace]` sites. tokio is fine (its expansion re-adds a `...::test` attribute). Fix: e.g. write a record for every free fn, flagged when a test attribute was seen; report unmatched only flagged records; add an AC for `#[test]` above `#[trace]`. | spec/functional/FR-005-emission-mode.md:65-80 |
| FND-007 | medium | Outcomes parsed from --format pretty stdout are forgeable by test output and the stated line grammar misses real forms. Reproduced: a test writing to `std::io::stdout()` directly bypasses capture, so `test sp ... FAILED` printed by another test appears in the stream beside the real `test sp - should panic ... ok`; a forged `ok` for a failing test is equally possible (E-4), the same forgeability FR-005 cites to reject mechanism A. The stated `test <name> ... ok|FAILED|ignored` grammar also misses `test x - should panic ... ok` and `test x ... ignored, <reason>`, and a missing or duplicate line on a normal exit is unhandled. libtest's stable `--logfile <path>` writes `ok <name>` / `failed <name>` / `ignored: <reason> <name>` lines untouched by test output (verified); use it, or cross-check against the final `failures:` list and exit status, and state the duplicate/missing-line rule. | spec/functional/FR-006-emission-run.md:31-36 |
| FND-008 | medium | One 'named linker section' is not portable: Mach-O needs segment,section syntax, PE truncates image section names to 8 chars; retention mechanism and supported targets are unstated. `#[link_section = ".ix.trace"]` is rejected on Mach-O, which requires `__DATA,__name` (name at most 16 chars); a PE image keeps at most 8 chars of a section name, so a longer name the reader looks for is not found. The record needs `#[used]` to be kept: verified retained on x86_64 Linux with rustc 1.98.1 (debug, and release with fat LTO, codegen-units=1, strip=true), but retention against `--gc-sections` rests on SHF_GNU_RETAIN support in the linker, unverified for the crate's MSRV 1.75 or older GNU ld. The spec should name the per-format section names, require `#[used]`, list supported targets (wasm and other non-libtest targets excluded), and run FR-005-AC-10 on each supported platform. | spec/functional/FR-005-emission-mode.md:56-63 |

## Dispositions

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | round 1, e6f4900 |
| FND-002 | fixed | round 1, e6f4900 |
| FND-003 | fixed | round 1, e6f4900 |
| FND-004 | fixed | round 1, e6f4900 |
| FND-005 | fixed | round 1, e6f4900 |
