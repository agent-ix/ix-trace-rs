//! Compile-checked `#[trace(...)]` and `#[implements(...)]` attributes, so a
//! symbol can name the spec ids it answers for in a form that is **statically
//! parseable** and that the compiler refuses to let you misspell.
//!
//! ```ignore
//! use ix_trace_rs::trace;
//!
//! #[trace("FR-047-AC-1", "TC-707")]
//! #[test]
//! fn tc707_shape_classification() { /* … */ }
//! ```
//!
//! # Import it; do not path-qualify it
//!
//! Write `use ix_trace_rs::trace;` and then the bare `#[trace(...)]`. The
//! path-qualified form compiles and does exactly the same thing at build
//! time — and binds **nothing**:
//!
//! ```ignore
//! #[ix_trace_rs::trace("TC-009")]   // compiles, and the coverage tool cannot see it
//! ```
//!
//! The module's canonical marker pattern is `#\[trace\(…\)\]`, anchored on the
//! literal attribute name, so a leading path does not match. Nothing warns: the
//! test passes, the id looks present in the source, and the row it should back
//! reads as unbacked. This crate hit it in its own suite — `TC-009` was a status
//! lie until the qualified form was replaced.
//!
//! Both attributes expand to the annotated item **unchanged**. They exist so
//! the marker is a real construct the compiler accepts and a coverage tool can
//! read out of the source without running anything — quire-rs `FR-051` parses
//! these markers statically to mint `verifies` relations, and its plan tracks
//! this crate as external deliverable EXT-4b.
//!
//! Nothing here inspects, rewrites, or executes the annotated item, and the
//! crate has no dependencies: the `proc_macro` API alone is enough to validate
//! the argument list and hand the item back.
//!
//! # Why two attributes rather than one with a flag
//!
//! `verifies` is **evidence** and may back an acceptance criterion;
//! `implements` is **scope** and never may. The module contract binds them to
//! complementary symbol kinds, so a marker declared as the wrong one binds
//! *nothing* rather than binding the wrong thing. A single attribute with a
//! discriminator argument would put one typo between evidence and scope; two
//! attributes keep that separation at the type level.
//!
//! # What is checked, and what deliberately is not
//!
//! Arguments are checked for **shape** — `KIND-NUMBER` with optional
//! `-SUBKIND-NUMBER` tails — and nothing more. The macro does **not** know
//! which kinds exist, and does not check that an id appears in any matrix.
//!
//! That boundary is deliberate. The kind vocabulary is declared by the module
//! (`spec-artifacts-process`), and a second copy compiled into this crate would
//! drift from it. That is not hypothetical: the same repository already carries
//! a `Status` column declared twice — a regex admitting a marker its own
//! vocabulary gives no class — and the test meant to keep the two honest
//! checked only one direction, so 261 rows across 31 repositories slipped
//! through (agent-ix/spec-artifacts-process#52). Membership checking belongs to
//! the stage that reads `spec/tests.md` and has the real vocabulary in front of
//! it, not here.

use proc_macro::{Delimiter, Group, Span, TokenStream, TokenTree};

/// Attach one or more spec trace ids to an item, declaring that it **verifies**
/// them.
///
/// Arguments must be **string literals** in id shape, comma-separated:
/// `#[trace("FR-047-AC-1")]`, `#[trace("FR-047-AC-1", "TC-707")]`. Anything
/// else is a compile error, because a marker a tool cannot read is worse than
/// no marker — it looks like coverage while providing none.
///
/// The expansion is the input item, unchanged.
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand("trace", attr, item)
}

/// Attach one or more spec ids to an item, declaring that it **implements**
/// them.
///
/// Same argument grammar as [`macro@trace`], and the same pass-through
/// expansion. Kept separate rather than folded into `trace` behind a flag: see
/// the module docs — `implements` is scope, never evidence, and the two must
/// not be one typo apart.
#[proc_macro_attribute]
pub fn implements(attr: TokenStream, item: TokenStream) -> TokenStream {
    expand("implements", attr, item)
}

/// Validate `attr` and return `item` untouched, prepending a spanned
/// `compile_error!` when the argument list is not well formed.
fn expand(name: &str, attr: TokenStream, item: TokenStream) -> TokenStream {
    match validate(name, attr) {
        Ok(()) => item,
        Err(Rejection { message, span }) => {
            let mut out = compile_error(&message, span);
            out.extend(item);
            out
        }
    }
}

/// A rejection carries the span of the token that caused it, so the diagnostic
/// underlines the offending literal rather than the whole attribute.
struct Rejection {
    message: String,
    span: Span,
}

/// Accept `"lit"` (`,` `"lit"`)* with an optional trailing comma, where every
/// literal is in id shape, and nothing else.
fn validate(name: &str, attr: TokenStream) -> Result<(), Rejection> {
    let mut expect_literal = true;
    let mut seen_any = false;

    for token in attr {
        match token {
            TokenTree::Literal(literal) if expect_literal => {
                let span = literal.span();
                let text = literal.to_string();
                let Some(id) = text
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
                else {
                    return Err(Rejection {
                        message: format!(
                            "`{name}` arguments must be string literals; found `{text}`"
                        ),
                        span,
                    });
                };
                if id.is_empty() {
                    return Err(Rejection {
                        message: format!("`{name}` arguments must not be empty strings"),
                        span,
                    });
                }
                if !is_id_shaped(id) {
                    return Err(Rejection {
                        message: format!(
                            "`{id}` is not a spec id: expected `KIND-NUMBER`, optionally \
                             followed by `-SUBKIND-NUMBER` (for example `TC-707` or \
                             `FR-047-AC-1`). `{name}` does not check which kinds exist — \
                             only that the argument is an id and not prose."
                        ),
                        span,
                    });
                }
                expect_literal = false;
                seen_any = true;
            }
            TokenTree::Punct(punct) if !expect_literal && punct.as_char() == ',' => {
                expect_literal = true;
            }
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                // An invisible group wraps a macro-expanded fragment; look inside
                // rather than reject a legitimate `#[trace($id)]` expansion.
                validate(name, group.stream())?;
                expect_literal = false;
                seen_any = true;
            }
            other => {
                return Err(Rejection {
                    message: format!(
                        "`{name}` takes comma-separated string literals; found `{other}`"
                    ),
                    span: other.span(),
                });
            }
        }
    }

    if !seen_any {
        return Err(Rejection {
            message: format!(
                "`{name}` needs at least one trace id, e.g. #[{name}(\"FR-001-AC-1\")]"
            ),
            span: Span::call_site(),
        });
    }
    Ok(())
}

/// A kind that begins with an ASCII letter, then any mix of letters, digits,
/// and underscores, then hyphen-separated alphanumeric segments, at least one
/// of which contains a digit.
///
/// Shape only. `TC-707`, `FR-047-AC-1`, `StR-004-AC-2`, `TC-001a`,
/// `FR-003-CON-1`, `TC-CB-01` and `interface_004-AC-1` pass; `hello world`,
/// `TC`, `non-canonical`, `-707`, `004-AC-1` and `4interface-AC-1` do not.
/// Which kinds are real is the module's business, not this crate's — see the
/// module docs.
///
/// A digit somewhere in a tail segment — not necessarily its first
/// character — is the whole discriminator there. It is what separates an id
/// from a hyphenated word: `vague-response` and `FR-backed` carry no number,
/// so they are prose. `FR-S003` is accepted on exactly this basis: the digit
/// is inside the tail segment, not leading it.
///
/// The rule is deliberately the loosest one that still rejects prose, because
/// every notch tighter is a real id somewhere in the corpus. Three were found
/// by measurement rather than reasoning, and all would have failed the build
/// in repositories that adopted the marker:
///
/// * requiring the number to be the **second** segment rejected `TC-CB-01`,
///   `IT-EDGE-008`, `TC-EC-01` — 308 distinct ids, and a shape the module's own
///   `TestMatrix` id_pattern explicitly admits
///   (`^(TC|IT)(-[A-Za-z0-9]+)*-\d+[A-Za-z0-9]*(-[A-Za-z0-9]+)*$`).
/// * requiring the digit-bearing segment to **begin** with its digit rejected
///   `FR-S003` (golden-path security) and `FR-M6` (ix-cli).
/// * requiring the KIND segment to be letters-only rejected `interface_004`:
///   this program's object ids use underscores, not hyphens, so the kind
///   segment itself carries the object's numeral
///   (agent-ix/ix-trace-rs#7).
///
/// Being stricter than the declared grammar is the same drift as being looser.
/// It just fails louder.
fn is_id_shaped(id: &str) -> bool {
    let mut segments = id.split('-');

    // KIND: begins with an ASCII letter, then any mix of ASCII letters,
    // digits, and underscores — matching quire-rs's own `is_object_id`
    // (src/semantic/target.rs) exactly, so an object id like `interface_004`
    // is itself a valid kind segment, but `_interface_004` and `4interface`
    // are not: a leading underscore or digit is not a letter.
    let Some(kind) = segments.next() else {
        return false;
    };
    if !kind.starts_with(|c: char| c.is_ascii_alphabetic())
        || !kind.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return false;
    }

    // Every remaining segment is non-empty and alphanumeric, and at least one
    // of them contains a digit (not necessarily leading — see `FR-S003` in
    // the doc comment above).
    let mut saw_number = false;
    let mut saw_any = false;
    for segment in segments {
        saw_any = true;
        if segment.is_empty() || !segment.bytes().all(|b| b.is_ascii_alphanumeric()) {
            return false;
        }
        if segment.bytes().any(|b| b.is_ascii_digit()) {
            saw_number = true;
        }
    }

    saw_any && saw_number
}

/// Build `compile_error!("…");` without pulling in `quote`, with every token
/// carrying `span` so rustc underlines the argument that was rejected rather
/// than the whole attribute.
fn compile_error(message: &str, span: Span) -> TokenStream {
    let raw: TokenStream = format!("::core::compile_error!({message:?});")
        .parse()
        .expect("compile_error! is valid Rust");
    respan(raw, span)
}

/// Recursively stamp `span` onto every token, including inside groups.
fn respan(stream: TokenStream, span: Span) -> TokenStream {
    stream
        .into_iter()
        .map(|token| {
            let mut token = match token {
                TokenTree::Group(group) => {
                    let mut regrouped = Group::new(group.delimiter(), respan(group.stream(), span));
                    regrouped.set_span(span);
                    TokenTree::Group(regrouped)
                }
                other => other,
            };
            token.set_span(span);
            token
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::is_id_shaped;

    // TC-006, FR-003-AC-1: the crate cannot annotate its own unit tests —
    // a proc-macro crate cannot invoke its own attribute — so these carry
    // the legacy comment form. That limit is real and worth stating: the
    // attribute is for consumers, and this crate is its own first consumer
    // everywhere except here.
    #[test]
    fn accepts_the_id_forms_the_corpus_actually_writes() {
        for id in [
            "TC-707",       // bare test case
            "IT-033",       // integration test
            "FR-047-AC-1",  // acceptance criterion
            "NFR-003-AC-2", // three-letter kind
            "StR-004-AC-2", // mixed-case kind
            "US-005-AC-2",
            "FR-003-CON-1", // constraint sub-id
            "TC-001a",      // alphanumeric suffix on the number
            // Alphabetic segment BEFORE the number. The module's TestMatrix
            // id_pattern admits these and 308 distinct ones exist across the
            // corpus; an earlier version of `is_id_shaped` rejected every one.
            "TC-CB-01",
            "IT-EDGE-008",
            "TC-EC-01",
            // Digit inside the segment rather than leading it. Real ids:
            // golden-path security and ix-cli respectively.
            "FR-S003",
            "FR-M6",
            "FR-SP001",
        ] {
            assert!(is_id_shaped(id), "rejected a real id: {id}");
        }
    }

    // TC-006, FR-003-AC-1: agent-ix/ix-trace-rs#7: this program's object ids
    // use underscores, not hyphens (`interface_004`), so the KIND segment
    // itself may carry underscores and digits after its first character.
    #[test]
    fn accepts_kind_segments_with_underscores_and_digits() {
        for id in [
            "interface_004-AC-1", // the id shape the issue is about
            "interface_004-AC-2",
            "spec_object_017-AC-3", // multiple underscores in the kind
            "a1_b2-AC-1",           // digits and underscores interleaved
        ] {
            assert!(is_id_shaped(id), "rejected a real id: {id}");
        }
    }

    // TC-007, FR-003-AC-2: prose must not pass as an id.
    #[test]
    fn rejects_prose_and_malformed_ids() {
        for id in [
            "hello world",    // the case v1 exists to catch
            "TC",             // kind with no number
            "TC-",            // trailing separator
            "-707",           // no kind
            "non-canonical",  // hyphenated word: no segment bears a number
            "vague-response", // a grammar check name, not an id
            "TC-707-",        // empty tail
            "TC 707",         // space instead of separator
            "TC-707-AC-",     // empty trailing segment
            "FR_047",         // underscore is not the separator, and no tail
            "004-AC-1",       // kind is all digits: no letter anywhere in it
            "_004-AC-1",      // kind is all underscore+digits: still no letter
            "__-AC-1",        // kind is underscores only
            // agent-ix/ix-trace-rs#7 review: the KIND segment must BEGIN with
            // an ASCII letter (matching quire-rs's `is_object_id` exactly),
            // not merely contain one anywhere.
            "_interface_004-AC-1", // starts with `_`, not a letter
            "4interface-AC-1",     // starts with a digit, not a letter
            "1_x-AC-1",            // starts with a digit, not a letter
        ] {
            assert!(!is_id_shaped(id), "accepted prose as an id: {id}");
        }
    }

    // TC-008, FR-003-AC-3, FR-003-CON-1: shape, never membership.
    #[test]
    fn the_kind_vocabulary_is_deliberately_not_checked() {
        // v1 validates shape, never membership: the kind vocabulary belongs to
        // the module, and a second copy compiled in here would drift from it.
        // An unknown-but-well-shaped kind must pass, or this crate has quietly
        // become a second declaration of that vocabulary.
        assert!(is_id_shaped("ZZ-001"));
        assert!(is_id_shaped("MADEUP-042-AC-9"));
    }
}
