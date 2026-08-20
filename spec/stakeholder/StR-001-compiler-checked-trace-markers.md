---
id: StR-001
title: "Spec authors need a trace marker the compiler checks"
type: StR
relationships:
  - target: "ix://agent-ix/ix-trace-rs/FR-001"
    type: "satisfied_by"
  - target: "ix://agent-ix/ix-trace-rs/FR-003"
    type: "satisfied_by"
---

# StR-001: Spec authors need a trace marker the compiler checks

## Stakeholder Need

Authors of traced Rust repositories shall be able to name every specification id
a symbol answers for in a form the compiler rejects when it is malformed, so
that a marker which looks like coverage cannot silently provide none.

## Rationale

Today the marker is a comment, and a comment cannot be wrong. It can only be
ignored.

The corpus writes its ids in a parenthetical:

```rust
// IT-033 (FR-011-AC-1, US-005-AC-2): prose
```

`rust-comment-id` captures from immediately after `//`, so `IT-033` binds and
the two acceptance criteria inside the parenthesis are read as prose. Nothing
warns. The test looks traced, the row goes green, and the criteria it claims to
verify are backed by nothing.

Measured in `agent-ix/quire-cli` during `#43`: matrix rows read 96/105 backed
while acceptance criteria read **2/119**. The rows were healthy because a row
binds on its own id — the first one, the one that survives. That repository
converted all 120 tags to the comma form the pattern does admit, reaching
108/119, but the fix is local. `quire-rs` writes 344 comments in the same
shape, and every future author will write the parenthetical again, because it
reads better and nothing tells them otherwise.

The deeper problem is that the form which expresses what authors mean was
already declared canonical and did not compile. `spec-artifacts-process`
declares `#[trace(...)]` as the canonical Rust marker, the engine prefers it
over the comment forms, and it emits rewrite suggestions telling authors to
adopt it — while `rustc` rejects the attribute, because no crate defines it.
Authors were being pointed at a form they could not use.

A marker the compiler checks removes the whole failure class: an argument list
is not prose, a malformed id is a build error, and there is no delimiter rule to
get wrong by one character.

## Validation Criteria

| ID | Criteria | Validation |
|----|----------|------------|
| StR-001-VC-1 | An author can annotate a test with several ids and see all of them bind, rather than only the first | Demonstration |
| StR-001-VC-2 | A malformed or prose argument fails the build, naming the argument at fault, rather than being silently ignored | Demonstration |
| StR-001-VC-3 | The canonical marker the module already declares and the engine already recommends compiles | Demonstration |

## Stakeholders

The primary stakeholders are authors and reviewers of traced Rust repositories,
who write the markers and read the coverage reports built from them. Affected
parties are the consumers of those reports — anyone deciding whether a
requirement is verified — who today cannot distinguish a criterion that is
tested from one whose marker was dropped on the floor.

## Context and Assumptions

It is assumed that the repository is compiled at least once in CI, since an
attribute cannot be checked without building. It is assumed that the legacy
comment forms continue to be read, because not every traced symbol is an item an
attribute can attach to — `fuzz_target!` declares no `fn`. This need is
therefore for an additional, stronger tier rather than a replacement.

## Stakeholder Constraints (Contextual)

Adoption spans roughly 150 repositories, so authors expect the marker to cost
effectively nothing to add: no meaningful build-time increase, and no dependency
that reaches a downstream consumer's tree.

## Dependencies

**Upstream**: the trace-tag grammar in `ix://agent-ix/quire-rs/FR-051`, which
declares which marker forms exist and which is canonical. **Downstream**: the
attributes specified in [FR-001](../functional/FR-001-trace-attribute.md) and
[FR-002](../functional/FR-002-implements-attribute.md), and the argument grammar
in [FR-003](../functional/FR-003-argument-grammar.md).

## Priority and Risk (Informative)

Business value is high: criterion-level coverage is the measure the verification
programme reports on, and it is currently understated across the corpus by a
mechanism no reader can see. Risk if unmet is that requirement coverage keeps
being reported from markers that bind less than they appear to.

## Notes (Informative)

Two capabilities are deliberately left for later analysis rather than folded
into this need: checking an id against the specification at compile time, and
recording that an annotated test actually ran. The second matters more than it
sounds — `quire-cli/tests/audit_no_network.rs` holds five tests that return
early when `strace` is absent, passing while asserting nothing, with their row
still reading as backed. No static marker can tell the difference.
