//! A no-op `#[trace(...)]` attribute, so a test can name the spec ids it
//! verifies in a form that is **statically parseable**.
//!
//! ```ignore
//! #[trace("FR-047-AC-1", "TC-707")]
//! #[test]
//! fn tc707_shape_classification() { /* … */ }
//! ```
//!
//! The attribute expands to the annotated item **unchanged**. It exists only so
//! the marker is a real construct the compiler accepts and a coverage tool can
//! read out of the source without running anything — quire-rs `FR-051` parses
//! these markers statically to mint `verifies` relations, and its plan tracks
//! this crate as external deliverable EXT-4b.
//!
//! Nothing here inspects, rewrites, or executes the annotated item, and the
//! crate has no dependencies: the `proc_macro` API alone is enough to validate
//! the argument list and hand the item back.

use proc_macro::{Delimiter, TokenStream, TokenTree};

/// Attach one or more spec trace ids to an item.
///
/// Arguments must be **string literals**, comma-separated:
/// `#[trace("FR-047-AC-1")]`, `#[trace("FR-047-AC-1", "TC-707")]`. Anything
/// else is a compile error, because a marker a tool cannot read is worse than
/// no marker — it looks like coverage while providing none.
///
/// The expansion is the input item, unchanged.
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    match validate(attr) {
        Ok(()) => item,
        Err(message) => {
            let mut out: TokenStream = compile_error(&message);
            out.extend(item);
            out
        }
    }
}

/// Accept `"lit"` (`,` `"lit"`)* with an optional trailing comma, and nothing
/// else. Returns the caller-facing reason on rejection.
fn validate(attr: TokenStream) -> Result<(), String> {
    let mut expect_literal = true;
    let mut seen_any = false;

    for token in attr {
        match token {
            TokenTree::Literal(literal) if expect_literal => {
                let text = literal.to_string();
                if !(text.starts_with('"') && text.ends_with('"') && text.len() >= 2) {
                    return Err(format!(
                        "`trace` arguments must be string literals; found `{text}`"
                    ));
                }
                if text.len() == 2 {
                    return Err("`trace` arguments must not be empty strings".to_string());
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
                validate(group.stream())?;
                expect_literal = false;
                seen_any = true;
            }
            other => {
                return Err(format!(
                    "`trace` takes comma-separated string literals; found `{other}`"
                ));
            }
        }
    }

    if !seen_any {
        return Err(
            "`trace` needs at least one trace id, e.g. #[trace(\"FR-001-AC-1\")]".to_string(),
        );
    }
    Ok(())
}

/// Build `compile_error!("…");` without pulling in `quote`.
fn compile_error(message: &str) -> TokenStream {
    format!("::core::compile_error!({message:?});")
        .parse()
        .expect("compile_error! is valid Rust")
}
