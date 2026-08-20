---
id: US-001
title: "Tag a test with every id it verifies"
type: US
relationships:
  - target: "ix://agent-ix/ix-trace-rs/StR-001"
    type: "traces_to"
---

# US-001: Tag a test with every id it verifies

## Story

**As a** developer writing a test in a traced Rust repository
**I want** to name every specification id the test verifies, in one place
**So that** the coverage report credits all of them instead of silently keeping
the first and discarding the rest.

## Context

The habit this replaces is a comment. Authors write the test id first and put
the criteria it satisfies in a parenthesis after it, because that reads well:

```rust
// IT-033 (FR-011-AC-1, US-005-AC-2): `lookup --heading --level 1` returns …
```

Only `IT-033` binds. There is no warning, no dead-tag report, no difference in
the output — the criteria simply never appear as covered, and the reader of the
report has no way to tell that from a criterion nobody tested.

Developers hit this in a specific order. They write a test, tag it, watch the
matrix row go green, and reasonably conclude the tagging worked. The row *did*
go green: rows bind on their own id, which is the one that survives. The
criteria are what got lost, and criterion coverage is not what anybody looks at
day to day. In `quire-cli` this ran for the life of the repository before
anyone split the rollup by target kind and saw 2/119.

The comma form the pattern does admit works today:

```rust
// IT-033, FR-011-AC-1, US-005-AC-2: prose
```

It is not a solution so much as a rule to remember, and the delimiter after the
last id matters by one character. Nothing enforces it and nothing teaches it,
so the next author writes the parenthesis again.

## Acceptance Examples (Illustrative)

A developer converts the comment above to:

```rust
#[trace("IT-033", "FR-011-AC-1", "US-005-AC-2")]
#[test]
fn it_033_lookup_returns_the_first_heading() { … }
```

All three ids bind. If one is mistyped as `"FR-011-AC"` or written as prose, the
build fails and points at that argument.

## Constraints (Contextual)

The attribute has to attach to an item, so this story covers `#[test] fn` and
ordinary items. It does not cover a traced symbol that declares no item —
`fuzz_target!` is the case in hand — where the comment forms remain the only
option.

## Dependencies (Contextual)

Depends on the crate being consumable as a dev-dependency by the repositories
that need it, and on the coverage engine continuing to prefer the canonical
marker over the legacy comment forms when both are present on one symbol.

## Notes (Informative)

Worth separating from this story: a marker still only proves that a test is
annotated, never that it ran. That is a different capability and a later stage.
