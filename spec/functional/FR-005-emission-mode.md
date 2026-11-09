---
id: FR-005
title: "Emission mode writes a trace record into the test binary"
type: FR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/StR-001"
    type: "implements"
    cardinality: "1:1"
  - target: "ix://agent-ix/ix-trace-rs/FR-001"
    type: "extends"
  - target: "ix://agent-ix/quoin/FR-120"
    type: "implements"
---

# FR-005: Emission mode writes a trace record into the test binary

## Description

While the cargo feature `emit` of `ix-trace-rs` is enabled, `#[trace(...)]` on a
function SHALL embed one trace record in the compiled test binary, so that
the run can write the declared ids into its own result output
([FR-006](./FR-006-emission-run.md)). While `emit` is not enabled the attribute
is the transparent marker of [FR-001](./FR-001-trace-attribute.md) and nothing in
this requirement applies.

This requirement and FR-006 are this repository's satisfaction of the emitter
contract (E-1..E-10) of [quoin FR-120](ix://agent-ix/quoin/FR-120) and
[quoin ADR-100](ix://agent-ix/quoin/ADR-100). FR-005 is the macro half.

## Mechanism decision

The ticket (PLAT-333) offered four mechanisms. Each is judged against the
emitter contract:

| Mechanism | Verdict | Reason |
| --- | --- | --- |
| A. Test prints `##trace:` line into `<system-out>` | Rejected | E-1 allows no carrier but the `<property name="trace">` element, so A needs an amendment to FR-120 and a second reader in the quoin adapter; a line in captured stdout is also forgeable by any test that prints it. |
| B. `libtest-mimic` custom harness (`harness = false`) | Rejected as framed | `harness = false` removes every `#[test]` function the binary does not re-declare to the new harness: `#[rstest]`, `#[tokio::test]` and every untagged test vanish, which breaks E-2 (parametrized cases) and shrinks the suite silently. |
| C. Sidecar file merged after the run | Rejected | E-8 forbids it. |
| D. Trace record embedded in the stock libtest binary, plus a runner that writes the JUnit file | Chosen | Libtest still discovers and runs every `#[test]`, rstest and `tokio::test` included; the record carries `symbol -> ids` as inert data; the runner (FR-006) reads libtest's own per-test results and writes the JUnit file itself. Meets E-1, E-4, E-5, E-8 and E-9. |

`cargo-nextest` as the result writer is not adopted. Its JUnit documentation
(<https://nexte.st/docs/machine-readable/junit/>, read for this requirement
against the docs that mention release 0.9.143; nextest is not installed in this
corpus and no local run was made) lists the elements it writes per `<testcase>` —
`failure`, `flakyFailure`, `flakyError`, `skipped`, `system-out`, `system-err` —
and no `<property>` fed from source or configuration, so it cannot meet E-9
alone. That is a finding from documentation, not a measurement; a nextest
release that adds a per-test property carrier is a reason to revisit this
decision by amendment. Nextest as the executor under `ix-trace-run` was
considered and not chosen: its structured event stream is an experimental,
opt-in output, where libtest's own text results are stable.

## Behavior

- **Record carrier.** While `emit` is enabled, `#[trace]` on a `fn` item SHALL
  place one record in a linker section of the binary as a `static` of plain bytes
  marked `#[used]` (no pointers, so no relocation), built from `concat!` of the
  ids, the identifier, `module_path!()` and `file!()`. The expansion SHALL
  reference no crate other than `core`. The runner (FR-006) reads the section
  from the test binary file; nothing in the binary runs, prints or registers at
  run time, and no test is added to the binary.
- **Section names.** `#[trace]` SHALL name the record section per object format: ELF `ixtrace`
  (a C identifier, so the linker also defines start and stop symbols), Mach-O
  `__DATA,__ixtrace` (the section name is at most 16 characters), and PE/COFF
  `.ixtrace` (at most 8 characters, the most a PE image keeps).
- **Supported targets.** Records are supported on Linux (ELF), macOS (Mach-O) and
  Windows MSVC (PE/COFF) test binaries. `wasm` targets are unsupported: they have
  no such sections and `emit` there is rejected with a `compile_error!`. A target
  or toolchain is supported only after FR-005-AC-10 passes on it; retention of a
  `#[used]` static against section garbage collection depends on the linker and
  toolchain (it is verified on rustc 1.98.1 for x86_64 Linux and is unverified for
  the crate's MSRV 1.75 or for older GNU ld), so no other combination is claimed.
- **Placement.** The record `static` SHALL be an item inside the annotated
  function's body, so it is valid wherever the function is.
- **Which items.** `#[trace]` on any `fn` item SHALL embed a record, and on any
  other item (a `struct`, an `enum`) SHALL expand to the item unchanged, so
  enabling `emit` cannot break a build that compiles without it.
- **Test flag.** The record carries a `test` flag, set when the macro sees a
  test attribute after itself: `#[test]`, an attribute whose last path segment is
  `test` (such as `tokio::test`), or `rstest`. Rustc consumes a `#[test]` written
  above `#[trace]` before the macro runs, so that order yields a record with the
  flag clear. The flag changes only which record the runner may report as
  unmatched (FR-006); it never changes whether a record's ids attach to a listed
  test, because the runner joins by libtest name.
- **Record fields.** The record holds the ids exactly as the argument literals
  gave them, in order (E-3: no id is inferred from a name), the function
  identifier, the `module_path!()` value, the `file!()` value, the `test` flag
  and a `parametrized` flag set when the function carries `rstest`.
- **Attribute order.** `#[test]` works in either order with `#[trace]`; with
  `#[test]` above, the record has the `test` flag clear and a `#[test]` declared in
  a function body then cannot be reported unmatched. `#[tokio::test]` works in
  either order. For `#[rstest]`, authors SHALL write `#[trace]` above it, so the
  macro registers the source function once with the `parametrized` flag. With
  `#[rstest]` above `#[trace]`, rstest copies `#[trace]` onto each generated case
  function and each case registers under its own libtest name; ids still attach
  to the right case, but the reported symbol is the case's, so E-2 is not met and
  the order is unsupported. The runner warns about it (FR-006-AC-13).
- **Feature unification.** Cargo unifies `emit` across every package of one
  build, so a crate that never opted in can see `emit` on. Because the expansion
  needs nothing but `core` and the runner needs nothing linked into the test
  binary, such a crate still compiles and its tests still run unchanged; it
  merely carries records nobody reads.
- **Not a test.** A `#[test]` declared inside a function body is never collected
  by libtest (rustc `unnameable_test_items`), so it never runs. Its record has the
  `test` flag set and matches no listed test; the runner reports it unmatched
  (FR-006-AC-9). The non-zero exit is the correct signal, and no symbol is
  invented for it.

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-005-CON-1 | While `emit` is not enabled, the expansion of `#[trace]` and `#[implements]` SHALL be byte-identical to the expansion before `emit` existed, adding no dependency, section or token to any build (E-6). | Design | Test |
| FR-005-CON-2 | `emit` SHALL be an explicit cargo feature of `ix-trace-rs`, off by default and never implied by the presence of the attribute (E-6). | Design | Inspection |
| FR-005-CON-3 | `#[implements(...)]` SHALL register nothing in any mode, because scope is not evidence. | Design | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-005-AC-1 | With `emit` disabled, expanding `#[trace("FR-001-AC-1")] #[test] fn t() {}` yields exactly the input tokens, and the test binary has no trace section. | Test |
| FR-005-AC-2 | With `emit` enabled, a tagged `#[test]` carrying two ids in `mod tests` yields one record in the binary, with both ids in declared order, the identifier, `module_path!()` and `file!()`. | Test |
| FR-005-AC-3 | With `emit` enabled, `#[trace]` on a `struct` yields the item unchanged with no record; on an `impl` method and on a function with no test attribute it yields a record with the `test` flag clear; each crate compiles. | Test |
| FR-005-AC-4 | With `emit` enabled, a tagged `#[rstest]` function written below `#[trace]` yields one record with the `parametrized` flag; with `#[rstest]` written above `#[trace]` each generated case yields its own record without it. | Test |
| FR-005-AC-5 | With `emit` enabled, a tagged `#[tokio::test]` yields one record with `#[trace]` written above it and one with `#[trace]` written below it. | Test |
| FR-005-AC-6 | With `emit` enabled, a rejected `#[trace]` argument list produces the same `compile_error!` as without `emit` (FR-003). | Test |
| FR-005-AC-7 | With `emit` enabled, an item carrying only `#[implements(...)]` yields no record. | Test |
| FR-005-AC-8 | With `emit` enabled, an untagged test yields no record and runs under libtest exactly as without `emit`. | Test |
| FR-005-AC-9 | With `emit` enabled for one package of a build in which a second package has `#[trace]` tests and no dependency on `ix-trace-run`, the second package compiles and its tests pass. | Test |
| FR-005-AC-10 | The records of a default `cargo test` build with `emit` enabled survive linking: every tagged test of a fixture crate has its record in the produced test binary, and the check runs on each supported platform (Linux, macOS, Windows MSVC) before that platform is claimed. | Test |
| FR-005-AC-11 | With `emit` enabled, a tagged `#[test]` written above `#[trace]` yields one record with the `test` flag clear, and the runner attaches its ids to the listed test of that name. | Test |
| FR-005-AC-12 | With `emit` enabled for a `wasm` target, the build fails with a `compile_error!` naming the unsupported target; the section names of FR-005 are those of each object format. | Test |

## Dependencies

- **Upstream**: [FR-001](./FR-001-trace-attribute.md), [FR-003](./FR-003-argument-grammar.md),
  quoin FR-120 (emitter contract).
- **Downstream**: [FR-006](./FR-006-emission-run.md) reads the records.
