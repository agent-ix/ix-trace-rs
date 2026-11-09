//! Trace record emission (FR-005), compiled only with the `emit` feature.
//!
//! `#[trace(...)]` on a `fn` item with a body and no `self` receiver gets one
//! record: a `#[used]` static of plain bytes in a linker section named per
//! object format, declared as the first item of the function body. The record
//! is inert data. The `ix-trace-run` runner reads it from the binary file.
//!
//! Record layout (text, fields separated by `U+001F`):
//!
//! ```text
//! U+0002 "IXTR1" FLAGS SEP IDS SEP IDENT SEP MODULE_PATH SEP FILE U+0003
//! ```
//!
//! `FLAGS` is two characters, `t` or `-` for the test flag and `p` or `-` for
//! the parametrized flag. `IDS` are comma-joined. The module path and file come
//! from `module_path!()` and `file!()` evaluated in the user's crate.

use proc_macro::{Delimiter, Group, TokenStream, TokenTree};

/// Start of a record. The reader scans for it, so it must match
/// `ix-trace-run`'s registry reader.
const START: &str = "\u{2}IXTR1";
/// Field separator.
const SEP: &str = "\u{1f}";
/// End of a record.
const END: &str = "\u{3}";

/// What the item looks like to the record, once it is known to qualify.
struct Target {
    ident: String,
    test: bool,
    parametrized: bool,
    /// Index of the body group, the last token.
    body: usize,
}

/// Return `item` with a record at the start of its body, or unchanged when the
/// item is not a function with a body and no receiver.
pub fn with_record(ids: &[String], item: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = item.into_iter().collect();
    let Some(target) = analyse(&tokens) else {
        return tokens.into_iter().collect();
    };
    let record = record_source(ids, &target);
    let record: TokenStream = record.parse().expect("the record is valid Rust");

    let mut out: Vec<TokenTree> = Vec::with_capacity(tokens.len());
    for (index, token) in tokens.into_iter().enumerate() {
        match token {
            TokenTree::Group(body) if index == target.body => {
                out.push(TokenTree::Group(prepend(&body, record.clone())));
            }
            other => out.push(other),
        }
    }
    out.into_iter().collect()
}

/// Rebuild `body` with `record` ahead of its statements and after any inner
/// attributes, which must stay first in a block.
fn prepend(body: &Group, record: TokenStream) -> Group {
    let inner: Vec<TokenTree> = body.stream().into_iter().collect();
    let mut keep = 0;
    while let (
        Some(TokenTree::Punct(hash)),
        Some(TokenTree::Punct(bang)),
        Some(TokenTree::Group(attr)),
    ) = (inner.get(keep), inner.get(keep + 1), inner.get(keep + 2))
    {
        if hash.as_char() != '#' || bang.as_char() != '!' || attr.delimiter() != Delimiter::Bracket
        {
            break;
        }
        keep += 3;
    }
    let mut stream: Vec<TokenTree> = inner[..keep].to_vec();
    stream.extend(record);
    stream.extend(inner[keep..].iter().cloned());
    let mut rebuilt = Group::new(Delimiter::Brace, stream.into_iter().collect());
    rebuilt.set_span(body.span());
    rebuilt
}

/// Source of the record item. Only `core` is named, and the whole item is
/// absent on every target whose object format the runner does not read.
fn record_source(ids: &[String], target: &Target) -> String {
    let head = format!(
        "{START}{}{}{SEP}{}{SEP}{}{SEP}",
        if target.test { 't' } else { '-' },
        if target.parametrized { 'p' } else { '-' },
        ids.join(","),
        target.ident,
    );
    format!(
        r#"
#[cfg(any(target_os = "linux", target_os = "macos", all(windows, target_env = "msvc")))]
#[allow(unknown_lints, unsafe_attr_outside_unsafe)]
const _: () = {{
    const TEXT: &str = ::core::concat!({head:?}, ::core::module_path!(), {SEP:?}, ::core::file!(), {END:?});
    #[used]
    #[cfg_attr(target_os = "linux", link_section = "ixtrace")]
    #[cfg_attr(target_os = "macos", link_section = "__DATA,__ixtrace")]
    #[cfg_attr(all(windows, target_env = "msvc"), link_section = ".ixtrace")]
    static RECORD: [u8; TEXT.len()] = {{
        let text = TEXT.as_bytes();
        let mut bytes = [0u8; TEXT.len()];
        let mut at = 0;
        while at < text.len() {{
            bytes[at] = text[at];
            at += 1;
        }}
        bytes
    }};
}};
"#
    )
}

/// Decide whether `tokens` is a `fn` item with a body and no `self` receiver,
/// and read its identifier and test attributes.
fn analyse(tokens: &[TokenTree]) -> Option<Target> {
    let mut at = 0;
    let mut test = false;
    let mut parametrized = false;

    while let (Some(TokenTree::Punct(hash)), Some(TokenTree::Group(attr))) =
        (tokens.get(at), tokens.get(at + 1))
    {
        if hash.as_char() != '#' || attr.delimiter() != Delimiter::Bracket {
            break;
        }
        match last_path_segment(attr.stream()).as_deref() {
            Some("test") => test = true,
            Some("rstest") => {
                test = true;
                parametrized = true;
            }
            _ => {}
        }
        at += 2;
    }

    // Visibility and qualifiers up to `fn`.
    loop {
        match tokens.get(at)? {
            TokenTree::Ident(word) if word.to_string() == "fn" => break,
            TokenTree::Ident(word)
                if matches!(
                    word.to_string().as_str(),
                    "pub" | "const" | "async" | "unsafe" | "extern" | "default" | "safe"
                ) => {}
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {}
            TokenTree::Literal(_) => {}
            _ => return None,
        }
        at += 1;
    }

    let TokenTree::Ident(name) = tokens.get(at + 1)? else {
        return None;
    };
    let ident = name.to_string();
    let ident = ident.strip_prefix("r#").unwrap_or(&ident).to_owned();

    let params = parameter_list(&tokens[at + 2..])?;
    if has_receiver(params) {
        return None;
    }

    let body = tokens.len().checked_sub(1)?;
    match tokens.get(body)? {
        TokenTree::Group(group) if group.delimiter() == Delimiter::Brace => {}
        _ => return None,
    }
    Some(Target {
        ident,
        test,
        parametrized,
        body,
    })
}

/// The last identifier of the attribute's path: `test` for `tokio::test`.
fn last_path_segment(attr: TokenStream) -> Option<String> {
    let mut last = None;
    for token in attr {
        match token {
            TokenTree::Ident(word) => last = Some(word.to_string()),
            TokenTree::Punct(colon) if colon.as_char() == ':' => {}
            _ => break,
        }
    }
    last
}

/// The first parenthesized group outside generics: the parameter list.
fn parameter_list(tokens: &[TokenTree]) -> Option<&Group> {
    let mut depth = 0usize;
    let mut previous_dash = false;
    for token in tokens {
        match token {
            TokenTree::Punct(punct) => {
                match punct.as_char() {
                    '<' => depth += 1,
                    '>' if !previous_dash => depth = depth.saturating_sub(1),
                    _ => {}
                }
                previous_dash = punct.as_char() == '-';
            }
            TokenTree::Group(group)
                if depth == 0 && group.delimiter() == Delimiter::Parenthesis =>
            {
                return Some(group);
            }
            _ => previous_dash = false,
        }
    }
    None
}

/// Whether the first parameter is `self`, `mut self`, `&self`, `&'a mut self`
/// or `self: Type`.
fn has_receiver(params: &Group) -> bool {
    let mut tokens = params.stream().into_iter().peekable();
    // Attributes on the parameter.
    while matches!(tokens.peek(), Some(TokenTree::Punct(p)) if p.as_char() == '#') {
        tokens.next();
        tokens.next();
    }
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Punct(p) if p.as_char() == '&' => {}
            TokenTree::Punct(p) if p.as_char() == '\'' => {
                // The lifetime name follows the quote.
                tokens.next();
            }
            TokenTree::Ident(word) if word.to_string() == "mut" => {}
            TokenTree::Ident(word) => return word.to_string() == "self",
            _ => return false,
        }
    }
    false
}
