---
id: FR-004
title: "Emission mode registers declared ids with the run"
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

# FR-004: Emission mode registers declared ids with the run

## Description

When the cargo feature `emit` is enabled, `#[trace(...)]` SHALL, in addition to
returning the annotated item, register the ids it declares and the symbol it
annotates in a link-time registry of the test binary, so that the run can write
those ids into its own result output ([FR-005](./FR-005-emission-run.md)). When
`emit` is not enabled the attribute is the transparent marker of
[FR-001](./FR-001-trace-attribute.md) and nothing in this requirement applies.

This requirement is this repository's satisfaction of the emitter contract
(E-1..E-10) of
[quoin FR-120](ix://agent-ix/quoin/FR-120) and
[quoin ADR-100](ix://agent-ix/quoin/ADR-100) for the Rust macro half; the run half
is FR-005.

## Mechanism decision

The ticket (PLAT-333) offered four mechanisms. Each is judged against the
emitter contract:

| Mechanism | Verdict | Reason |
| --- | --- | --- |
| A. Test prints `##trace:` line into `<system-out>` | Rejected | E-1 allows no carrier but the `<property name="trace">` element, so A needs an amendment to FR-120 and a second reader in the quoin adapter; a line in captured stdout is also forgeable by any test that prints it. |
| B. `libtest-mimic` custom harness (`harness = false`) | Rejected as framed | `harness = false` removes every `#[test]` function the binary does not re-declare to the new harness: `#[rstest]`, `#[tokio::test]` and every untagged test vanish, which breaks E-2 (parametrized cases) and shrinks the suite silently. |
| C. Sidecar file merged after the run | Rejected | E-8 forbids it. |
| D. Link-time registry plus a runner over the stock libtest binary | Chosen | Libtest still discovers and runs every `#[test]`, rstest and `tokio::test` included; the registry carries `symbol -> ids`; the run (FR-005) reads each test's outcome from the process libtest ran it in and writes the JUnit file itself. Meets E-1, E-4, E-5, E-8 and E-9. |

`cargo-nextest` is not used for the result file: its JUnit output has no carrier
for source-declared per-test properties, so it cannot meet E-9 alone. Should it
gain one, the choice is revisited by amendment.

## Behavior

- With `emit` enabled, the expansion SHALL be the annotated item followed by one
  registry entry holding the declared ids, the item's identifier, the value of
  `module_path!()`, and the value of `file!()`. The registry type and its
  linking support live in the companion crate `ix-trace-run` of this repository,
  which the consumer lists as a dev-dependency; `ix-trace-rs` itself keeps
  `[dependencies]` empty (NFR-001).
- The ids registered SHALL be exactly the string literals of the attribute
  argument list, in order. No id is inferred from an identifier or a path (E-3).
- The registered symbol SHALL be the libtest test name of the function: the
  `module_path!()` value with its first segment (the crate name) removed, joined
  by `::` to the function name. A function in `mod tests` is `tests::name`; a
  function at the top level of an integration test file `tests/x.rs` is `name`;
  neither the crate name nor the file stem appears. This is the symbol
  `quire coverage --json` reports as `qualified_name` for a function whose
  container chain is modules only, and the join is verified by FR-051-AC-31 of
  quire-rs against real extractor output.
- A parametrized test (`#[rstest]` or equivalent) SHALL register the source
  function's name; the run (FR-005) maps every libtest case name that begins
  with that name followed by `::` to the entry.
- A `#[trace]` test declared inside a function body is outside this contract:
  the macro cannot see the enclosing function, so the symbol would disagree with
  the extractor's chain. The runner SHALL report such a test as an
  unmatched registration (FR-005-AC-6) rather than recorded under a wrong symbol.
- The registry SHALL be written by the macro expansion alone. No test code writes
  to it at run time and no file is created (E-8).

## Constraints

| ID | Constraint | Type | Validation |
|----|------------|------|------------|
| FR-004-CON-1 | With `emit` disabled the expansion of `#[trace]` and `#[implements]` SHALL be byte-identical to the expansion before `emit` existed, adding no dependency to any build, so the compiled non-test output of a consumer is unchanged (E-6). | Design | Test |
| FR-004-CON-2 | `emit` SHALL be an explicit cargo feature of `ix-trace-rs`, never enabled by default and never implied by the presence of the attribute (E-6). | Design | Inspection |
| FR-004-CON-3 | `#[implements(...)]` SHALL register nothing in any mode; scope is not evidence. | Design | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-004-AC-1 | With `emit` disabled, expanding `#[trace("FR-001-AC-1")] #[test] fn t() {}` yields exactly the input tokens, and a consumer test build links no `ix-trace-run` item. | Test |
| FR-004-AC-2 | With `emit` enabled, a tagged `#[test]` carrying two ids in `mod tests` appears in the registry once, with both ids in declared order and the symbol `tests::<name>`. | Test |
| FR-004-AC-3 | With `emit` enabled, a tagged test at the top level of `tests/x.rs` registers the symbol `<name>` and a `file!()` path ending in `tests/x.rs`. | Test |
| FR-004-AC-4 | With `emit` enabled, a tagged `#[rstest]` function registers the source function name once, and none of its libtest case names is registered separately. | Test |
| FR-004-AC-5 | With `emit` enabled, a `#[trace]` argument list is still checked by FR-003, and a rejected list produces the same `compile_error!` as without `emit`. | Test |
| FR-004-AC-6 | With `emit` enabled, an item carrying only `#[implements(...)]` registers nothing. | Test |
| FR-004-AC-7 | With `emit` enabled, an untagged test is neither registered nor affected: it still runs under libtest and appears in the run's result file with no `trace` property. | Test |

## Dependencies

- **Upstream**: [FR-001](./FR-001-trace-attribute.md), [FR-003](./FR-003-argument-grammar.md),
  quoin FR-120 (emitter contract).
- **Downstream**: [FR-005](./FR-005-emission-run.md) consumes the registry.
