---
id: SR-001
title: "Code review — ix-trace-rs v1 compile-checked markers"
type: SpecReview
analysis: code-review
scope: "src/, tests/, spec/, licensing"
review_set: subset
---

## Summary

Review of `29ca439` on `feat/v1-compile-checked-markers`: id-shape validation,
spanned diagnostics, a second `#[implements(...)]` attribute, a compile-fail
harness, a spec authored from nothing, and a relicense. One **high**-severity
correctness defect was found in the shape rule — it rejected 311 real corpus
ids and would have failed the build in every repository that adopted the
marker. Fixed in branch and re-verified against the whole corpus.

## Verdict

**CONDITIONAL** — one high finding, fixed in branch before this review was
written, plus two low findings recorded not fixed. The shape rule is now
calibrated against 6931 real id-shaped tokens rather than reasoned from a
pattern.

## Findings

| ID      | Severity | Summary                                                                    | Refs                  |
| ------- | -------- | -------------------------------------------------------------------------- | --------------------- |
| FND-001 | high     | **FIXED IN BRANCH.** `is_id_shaped` required the number to be the second segment, rejecting `TC-CB-01`, `IT-EDGE-008`, `TC-EC-01` — 308 distinct corpus ids, a shape the module's own `TestMatrix` id_pattern explicitly admits. Adoption in any repo using that convention would not compile | src/lib.rs:216        |
| FND-002 | medium   | **FIXED IN BRANCH.** After the first fix, requiring the digit-bearing segment to *begin* with its digit still rejected `FR-S003` (golden-path security), `FR-M6` and `FR-SP001..4` (ix-cli). Loosened to "contains a digit", the least-strict rule that still rejects prose | src/lib.rs:229        |
| FND-003 | medium   | **FIXED IN BRANCH.** `FR-003-AC-2` and TC-007 asserted `FR-AC-1` is rejected. Under the corrected rule it is accepted, and correctly so — it is `KIND-ALPHA-NUMBER`, the same shape as `TC-CB-01`. The criterion was describing the defect | spec/functional/FR-003-argument-grammar.md:96 |
| FND-004 | low      | Roughly ninety digit-free test ids exist in the corpus (`TC-CAT-APPS-FILTER`, `TC-REG-LAST-WRITE-WINS`, `TC-DET-COPY`). They are rejected, correctly — they do not match the module's `TestMatrix` id_pattern either, which mandates `-\d+`. But the repositories using that convention cannot adopt the marker until their ids conform. Recorded, not fixed: it is a corpus problem | —                     |
| FND-005 | low      | The three unit tests in `src/lib.rs` carry legacy comment tags rather than the crate's own attribute, because a proc-macro crate cannot invoke its own attribute inside its own `src/`. Stated in the Test Matrix Overview so it does not read as an inconsistency | src/lib.rs:253        |

## Priority 1 — `is_id_shaped` against the real corpus

This is where the review earned its keep, and reasoning about the pattern was
not enough to find it.

The first probe fed 1270 distinct ids extracted with the manifest's own
`rust-comment-id` pattern through the macro. All compiled — but that is close to
circular: it confirms the rule accepts the language of the pattern it was
written from.

The **`TestMatrix` id_pattern is broader**:

```
^(TC|IT)(-[A-Za-z0-9]+)*-\d+[A-Za-z0-9]*(-[A-Za-z0-9]+)*$
```

`(-[A-Za-z0-9]+)*` before `-\d+` means segments may precede the number.
`is_id_shaped` required the number to be the *second* segment, so:

```
error: `TC-CB-01` is not a spec id: expected `KIND-NUMBER` …
```

308 distinct ids across the corpus have that shape. Every repository using it
would have failed to compile on adopting the marker.

Fixing that surfaced a second, narrower case: `FR-S003` and `FR-M6` bear their
digits *inside* a segment rather than leading it. The rule is now "at least one
segment after the kind contains a digit" — the loosest rule that still rejects
prose, since a hyphenated word (`vague-response`, `FR-backed`, `non-canonical`)
carries no digit anywhere.

Re-verified by generating one `#[trace(...)]` containing every id-shaped token
found across `~/dev` and compiling it: **6804 of 6931 accepted**, zero false
rejections among ids that match the module's declared grammar. The generated
probe was removed rather than committed — it hardcodes ids from sibling
repositories and would rot.

The irony is worth recording. `FR-003-CON-1` warns that a second copy of the
vocabulary compiled into this crate would drift from the module's. The first
implementation did exactly that, in the stricter direction. Stricter drift is
the same defect as looser drift; it just fails loudly instead of quietly, which
is the only reason it was findable at all.

## Priority 2 — do the criteria match what the tests assert?

Checked criterion by criterion against test bodies.

`FR-001-AC-1..5`, `FR-002-AC-1..3`, `NFR-001-AC-1..2` — accurate. `FR-001`'s
criteria were rewritten during authoring after `quire validate` flagged
`ac:vacuous-outcome` on "behaves exactly as if the attribute were absent"; they
now name the observable result, which is what the tests assert.

`FR-003-AC-2` was **wrong** (FND-003): it listed `FR-AC-1` among the rejected
forms. That was true of the defective rule and false of the correct one.
Corrected along with TC-007.

`FR-003-AC-7` is the one criterion deliberately marked `Demonstration` rather
than `Test`, with its matrix row `🚧`. The non-vacuity of the trybuild suite was
confirmed by hand — short-circuiting `is_id_shaped` fails three of six fixtures
— and no automated trace exists. Claiming `Test` would have minted a status lie
in the same commit that specifies the anti-lie machinery.

## Priority 3 — `respan` and the rejection expansion

Both correct.

`respan` recurses into `Group` streams and rebuilds each group with
`Group::new(delimiter, respan(inner))` before stamping the span, so nested
groups are covered rather than only the top level. `::core::compile_error!("…");`
has one nesting level in practice; the recursion is right regardless.

The expansion emits `compile_error!` **and** the annotated item. Dropping the
item on rejection would produce a cascade of unresolved-name errors at every
call site, burying the real diagnostic. Verified by the fixtures: each records
exactly one error.

## Priority 4 — licensing

The branch as reviewed relicensed the crate to MIT OR Apache-2.0, on the
reasoning that the macro emits none of its own code into a consumer and that an
AGPL marker would fail the sibling repos' `cargo deny check licenses` — they
allow AGPL for their own crate only.

**That was overruled: the crate stays AGPL-3.0-or-later**, matching the rest of
the Agent IX Rust projects. The reasoning above described a real adoption cost,
not a licensing conclusion, and the cost is accepted rather than avoided.

The consequence is one line per adopting repository:

```toml
exceptions = [
    { allow = ["AGPL-3.0-or-later"], crate = "ix-trace-rs" },
]
```

That is now stated in the README and in this crate's own `deny.toml` comment, so
the next adopter meets it as documentation rather than as a failing gate.

## Gates

Re-run, not assumed. `make ci` exit 0 — `cargo fmt --check`,
`cargo clippy --all-targets -D warnings`, `cargo test`,
`cargo deny check licenses`, unsafe audit. 10 tests: 3 unit, 6 compile-fail
fixtures behind one runner, 5 transparency, 1 dependency posture.
`quire validate --scope . "spec/**/*.md"` exit 0.

Clippy required one structural change during authoring: `items_after_test_module`
moved the `#[cfg(test)] mod tests` block to the end of `src/lib.rs`.
