---
id: SR-002
title: "Code review — QSL-953 #[string_edge] marker (PR #14)"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-trace-rs@13a8bdf71209cbad6c60990b444b60489e833eb6; src/lib.rs, tests/string_edge.rs, tests/compile_fail.rs, tests/ui/string_edge_with_args.{rs,stderr}, Cargo.toml"
review_set: subset
---

## Summary

Ticket: QSL-953. Code review, with the rust-review lane folded in, of PR #14:
a new `#[proc_macro_attribute] string_edge` in `src/lib.rs` ported from
`qsl-attrs` (agent-ix/quire-spec-language, QSL FR-064), plus a transparency
test and one trybuild compile-fail fixture.

The port is faithful. The happy path returns `item` unchanged. Any non-empty
argument stream returns `compile_error!("#[string_edge] takes no arguments")`
followed by the unchanged item, with the same message, the same call-site span
and the same item retention as `qsl-attrs`. The only difference is
`::core::compile_error!` instead of a bare `compile_error!`, which is more
hygienic and gives the same diagnostic. The PR adds no `[dependencies]`.

The consumer scanner (`qsl-string-edge` `has_attr_named`) matches on the last
path segment `string_edge`. So `#[string_edge]` after `use`,
`#[ix_trace_rs::string_edge]` and `#[::ix_trace_rs::string_edge]` are all
recognised the same way. The scanner reads only `ItemFn` and `ImplItemFn`.
All 250-odd current QSL sites are functions: free, `pub`/`pub(crate)`/
`pub(super)`, inherent and trait-impl methods (`from_str`, `try_from`,
`visit_*`), generic and lifetime-bearing functions, and `main`. An
identity-transform attribute macro serves every one of those shapes.

## Verdict

**APPROVE**: no high or medium findings. Three low findings, none blocking.

Measured, not assumed:

- `make ci` on `13a8bdf` passed with exit 0 (fmt-check, clippy `-D warnings`,
  test, deny licenses, audit-unsafe). `cargo deny` reports `licenses ok`, and
  `[dependencies]` is still empty.
- `tests/ui/string_edge_with_args.stderr` holds exactly one error. The fixture
  calls `edge("x")`, so if the item were not retained a second `E0425` would
  appear and change the stderr.
- Mutation probe (`src/lib.rs` changed, the tests re-run, then restored). All
  four mutants were killed:
  - Dropping the item on error: the fixture fails with
    `E0425 cannot find function edge`.
  - Changing the message text: the fixture stderr no longer matches.
  - Accepting arguments silently: the fixture compiles, so the compile-fail
    test fails.
  - Expanding the happy path to nothing: `tests/string_edge.rs` fails to
    compile.
- `#[string_edge()]` (empty parentheses) is accepted, because its attribute
  stream is empty. `qsl-attrs` behaves the same way, so the port is faithful
  here too.

Rust review: the PR adds no panic paths (it uses the existing `compile_error`
helper's `expect` on a static literal), no `unsafe` and no integer
conversions. The `kind.len() as u8` in the test is test-only and bounded by
its inputs.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low | The `Cargo.toml` `description` and the `CLAUDE.md` headline still describe the crate as only the `#[trace(...)]` and `#[implements(...)]` markers. `string_edge` is now public surface, and crates.io/docs metadata does not mention it | Cargo.toml:5 |
| FND-002 | low | The crate version is stale: `Cargo.toml` says `0.1.0` while tag `v0.1.1` exists (v0.1.1 was cut from a commit that still read `0.1.0`). The release after this merge is `v0.1.2`. The repo has no CHANGELOG, no release make target and no prior bump commit, so it has no release-commit convention. Bump `version = "0.1.2"` in this PR, or in one commit right before tagging, so the tag and `Cargo.toml` agree | Cargo.toml:3 |
| FND-003 | low | The transparency test covers a free function and an inherent method only. Real consumers also mark trait-impl methods (`FromStr::from_str`, `TryFrom::try_from`, `Visit::visit_*`), generic and lifetime-bearing functions, and `fn main`. These work, because the macro is an identity transform, but no test pins them | tests/string_edge.rs:7 |

## Dispositions

| FND | outcome | sha |
| --- | ------- | --- |
| FND-001 | fixed | caf3723 |
| FND-002 | fixed | caf3723 |
| FND-003 | fixed | caf3723 |
