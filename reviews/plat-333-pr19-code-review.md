---
id: SR-011
title: "Code review — PLAT-333 CI matrix, object 0.40, record file separators (PR #19)"
type: SpecReview
analysis: code-review
scope: "agent-ix/ix-trace-rs@fdf0a9bc25299ca834f47fce186d0faa2be0635b; .github/workflows/ci.yml, Cargo.lock, ix-trace-run/Cargo.toml, ix-trace-run/src/registry.rs, ix-trace-run/tests/emit_records.rs, spec/functional/FR-006-emission-run.md, spec/tests.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Scope is `git diff origin/main...HEAD` at head fdf0a9b (PR #19).
Methods: code review with the Rust review lane folded in. The workflow edit was
cleared by the owner and is reviewed for correctness only.

Workflow (`.github/workflows/ci.yml`): the edit is minimal. The three
`dtolnay/rust-toolchain@stable` lines become `@1.98.1`, which agrees with
`rust-toolchain.toml` (channel 1.98.1). The `Test` step leaves the `rust` job,
and a new `test` job runs `cargo test` on a matrix of `ubuntu-latest`,
`macos-latest` and `windows-latest` with `fail-fast: false`. The matrix syntax is
valid. The `rust` and `licenses` jobs are otherwise unchanged. The edit adds no
secrets, `permissions` or triggers. The workflow has no path filters.

CI evidence: run 42646173333 on b5a95a7 (the last code head) is green on all
five jobs. Its log shows rustc 1.98.1 on all three OS legs and
`tc019_every_tagged_test_keeps_its_record_through_linking ... ok` on each. Run
42645597156 on 718a7e3 failed deterministically, before the fixes:

- macOS: 7 of 10 `emit_records` tests failed with `Format(Error("Invalid Mach-O number of sections"))`.
- Windows: one test failed with `left: "src\\foo\\bar.rs"` and `right: "src/foo/bar.rs"`.

fdf0a9b has no CI run, although the workflow has no path filters. The cause is
that the PR is `CONFLICTING` with main (2c89d9c, PR #18, edits the same
known-limits paragraph of `spec/tests.md`). GitHub does not start
`pull_request` workflows when it cannot build the test merge commit. Only the
`pull_request_target` CLA workflow ran. fdf0a9b itself changes only a comment
and the spec, so the missing run hides nothing in fdf0a9b. The rebase it needs
is a different matter (FND-003).

`object` 0.37.3 to 0.40.0: the bump fixes the macOS failure for a real cause. It
is not a CI flake.

- The 718a7e3 failure was deterministic: 7 of 7 binary-reading tests failed the same way.
- In 0.37.3, `Segment::sections` reads the section-header slice with
  `read_slice_at`, which calls `pod::slice_from_bytes`. That function returns
  `Err` when the pointer is not aligned to `align_of::<T>()`, and the error is
  surfaced as "Invalid Mach-O number of sections".
- Without the `unaligned` feature, 0.37's `endian::U64<E>` is an aligned
  `u64` (`object-0.37.3/src/endian.rs:454`), so `Section64` needs 8-byte
  alignment. This crate enables `read_core` and not `read`, and `read` is what
  implies `unaligned` (`object-0.37.3/Cargo.toml`, the `read` feature).
- In 0.40.0 every endian type is a byte array, `U64<E, T>([u8; 8], ..)`
  (`object-0.40.0/src/endian.rs:398`), so the same `sections()` code
  (byte-identical in both versions) cannot fail on alignment.
- Mach-O section headers in a linked macOS binary are therefore not 8-aligned
  relative to the allocation `std::fs::read` returns.
- Adding `unaligned` to the 0.37 feature list would have fixed it too.

The 0.40 breaking API changes (newtype flag fields, `SectionKind::Elf` removal,
`imports`/`exports` iterators, `macho::Section::data` offset parameter) do not
reach this crate. It uses only `File::parse`, `Object::sections`,
`ObjectSection::name` and `ObjectSection::data` (`registry.rs:86-95`,
`emit_records.rs:159-161`). Clippy (`-D warnings`) and tests pass on all three
legs. `Cargo.lock` changes only the `object` version and checksum, and its
dependency set stays `memchr`. object 0.40.0 needs rust 1.85 and edition 2024,
both below the 1.98.1 pin.

Local check, change-scoped, via `locked-build.sh` with debug=0 and 4 jobs:
`cargo test -p ix-trace-run --lib --test emit_records` passed (9 + 10). The
target dir was deleted afterwards.

Units examined:

- `.github/workflows/ci.yml` (examined)
- `ix-trace-run/Cargo.toml` and `Cargo.lock` (examined)
- `ix-trace-run/src/registry.rs` `parse_one`, `read_records` (examined)
- `ix-trace-run/src/matching.rs` `module_prefix`, `normalize_file`, `misuse_warnings` (context_only)
- `ix-trace-run/tests/emit_records.rs` TC-018 and TC-019 (examined)
- `FR-006` File attribute bullet (examined)

## Verdict

**REQUEST CHANGES.** The workflow and the `object` bump are correct and minimal.
The separator normalisation is right on Windows, but it runs on every host and
breaks matching on POSIX for a legal directory name (FND-001). The PR also
conflicts with main and needs a rebase and a green three-OS run on the rebased
head before merge (FND-003).

Rust-review lane: no `unsafe`, no panics, no new public API. `replace`
allocates one `String` per record, as `to_owned` did before. `normalize_file`'s
own `replace('\\', "/")` (`matching.rs:166`) is now redundant on records from
`parse_one` (FND-002).

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | medium | `parse_one` replaces `\` with `/` on every host. On Linux and macOS `\` is a legal file-name character. The record's `file` feeds `module_prefix` (`matching.rs:111,127-150`), which compares `workspace_root.join(file)` with cargo's absolute `src_path`. A workspace member under a directory such as `crates/my\pkg` (cargo's `file!()` is then `crates/my\pkg/src/lib.rs`) is rewritten to `crates/my/pkg/...`. That path never equals `root_file` and never strips its prefix. Every tagged test of that package becomes an unmatched record, and the runner exits non-zero (FR-006-AC-9). Before the PR this matched, because only the display string in `normalize_file` was rewritten. The runner only reads binaries built for and run on its own host (target runners are refused, FR-006-AC-23), so the host separator is the record separator. Make the replacement `cfg(windows)`, and say "on Windows" in the FR-006 File-attribute bullet | ix-trace-run/src/registry.rs:152; ix-trace-run/src/matching.rs:127-150; spec/functional/FR-006-emission-run.md:105-108 |
| FND-002 | low | After FND-001's normalisation in `parse_one`, the `.replace('\\', "/")` in `normalize_file` is a second copy of the same rule, and it still runs unconditionally on POSIX. That is only a display rewrite, but it is the same edge as FND-001. Keep the rule in one place | ix-trace-run/src/matching.rs:166 |
| FND-003 | medium | PR #19 is `CONFLICTING` with main. 2c89d9c (PR #18) rewrote the same known-limits paragraph of `spec/tests.md` that fdf0a9b rewrites, and its text still says TC-019 stays pending and CI runs on Ubuntu only. Because of the conflict, no `pull_request` CI ran on fdf0a9b. The rebased head needs a green three-OS run. That run also compiles main's new `ix-trace-run/tests/cost_quire_rs.rs` on macOS and Windows for the first time. Resolve the paragraph so it keeps both PR #18's TC-022 text and this PR's TC-019 text | spec/tests.md:89-100 |

## Dispositions

Round 1 was checked against head 4cf6f94, which is rebased onto main 2c89d9c. CI run 42648114494 on that head is green on all five jobs. The change-scoped tests also passed locally: `cargo test -p ix-trace-run --lib --test emit_records --test runner_basic`.

| FND | outcome | reason |
| --- | ------- | ------ |
| FND-001 | fixed | 4cf6f94: `slashed` in registry.rs rewrites `\` only under `#[cfg(windows)]` and returns the text unchanged elsewhere. `parse_one` uses it. Unit test `a_backslash_is_a_separator_only_on_windows` keeps `crates/my\pkg/src/lib.rs` on POSIX, and it passed on all three legs. The FR-006 bullet now says the conversion happens "on Windows" and that `\` is kept elsewhere |
| FND-002 | fixed | 4cf6f94: `normalize_file` calls the same `slashed`, so the rule lives in one place and no longer runs on POSIX |
| FND-003 | fixed | Rebased: the PR is now MERGEABLE. The known-limits paragraph keeps PR #18's TC-022 text and replaces the stale "TC-019 stays pending / Ubuntu only" text with this PR's text. CI 42648114494 ran on the rebased head, and all three Test legs passed, compiling `cost_quire_rs.rs` |
