//! Reading the trace records `ix-trace-rs` embeds in a test binary (FR-005).
//!
//! The reader opens the binary as data. Nothing in the binary runs.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use ix_trace_rs::implements;
use object::{Object, ObjectSection, ObjectSymbol};

/// Start of a record; matches `ix-trace-rs`'s emitter.
const START: &[u8] = b"\x02IXTR1";
/// The record start without its format version.
const FAMILY: &[u8] = b"\x02IXTR";
/// Field separator.
const SEP: char = '\u{1f}';
/// End of a record.
const END: u8 = 3;

/// What compiler object symbols establish about one record's source definition.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Definition {
    /// The binary carries no supported owner proof.
    Unknown,
    /// One compiler-qualified function owns this record.
    Proven(String),
    /// Multiple compiler-qualified functions share this record's storage.
    Ambiguous,
}

/// One trace record: the ids declared on a `fn` item and where it lives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Compiler-qualified owner of the emitted static, when uniquely proved by object symbols.
    /// Absent for byte-only parsing or stripped/unsupported symbol tables.
    pub definition: Definition,
    /// The ids, in declaration order.
    pub ids: Vec<String>,
    /// The function identifier.
    pub ident: String,
    /// `module_path!()` at the function, crate name first.
    pub module_path: String,
    /// `file!()` at the function.
    pub file: String,
    /// A test attribute followed `#[trace]`.
    pub test: bool,
    /// The function carries `rstest`.
    pub parametrized: bool,
}

impl Record {
    /// The name libtest gives the function: its module path without the crate
    /// segment, joined to the identifier by `::`.
    #[must_use]
    pub fn libtest_name(&self) -> String {
        match self.module_path.split_once("::") {
            Some((_, rest)) => format!("{rest}::{}", self.ident),
            None => self.ident.clone(),
        }
    }
}

/// Why a binary's records could not be read.
#[derive(Debug)]
pub enum RegistryError {
    /// The file could not be read.
    Io(std::io::Error),
    /// The file is not an object file the reader understands.
    Format(object::Error),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "cannot read the binary: {err}"),
            Self::Format(err) => write!(f, "not a readable object file: {err}"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// The records of a binary and how many entries could not be read.
#[derive(Debug, Default)]
pub struct Registry {
    /// The records that parsed.
    pub records: Vec<Record>,
    /// Entries that carry the record start but did not parse: a wrong field
    /// count, bad UTF-8, a missing end, or another format version.
    pub malformed: usize,
}

/// Read every record of the trace section of the binary at `path`.
///
/// `Ok(None)` means the binary has no trace section.
///
/// # Errors
///
/// Returns [`RegistryError`] when the file cannot be read or is not an object
/// file.
#[implements("FR-006-AC-1", "FR-006-AC-24")]
pub fn read_records(path: &Path) -> Result<Option<Registry>, RegistryError> {
    let data = std::fs::read(path).map_err(RegistryError::Io)?;
    let file = object::File::parse(&*data).map_err(RegistryError::Format)?;
    let mut found: Option<Registry> = None;
    for section in file.sections() {
        let Ok(name) = section.name() else { continue };
        if matches!(name, "ixtrace" | "__ixtrace" | ".ixtrace") {
            let bytes = section.data().map_err(RegistryError::Format)?;
            let (mut parsed, offsets) = parse_records_with_offsets(bytes);
            let mut owners_by_address: BTreeMap<u64, BTreeSet<String>> = BTreeMap::new();
            for offset in &offsets {
                if let Ok(offset) = u64::try_from(*offset) {
                    if let Some(address) = section.address().checked_add(offset) {
                        owners_by_address.entry(address).or_default();
                    }
                }
            }
            for symbol in file
                .symbols()
                .filter(|symbol| symbol.section_index() == Some(section.index()))
            {
                let Some(owners) = owners_by_address.get_mut(&symbol.address()) else {
                    continue;
                };
                if let Some(owner) = symbol.name().ok().and_then(compiler_owner) {
                    owners.insert(owner);
                }
            }
            for (record, offset) in parsed.records.iter_mut().zip(offsets) {
                let Ok(offset) = u64::try_from(offset) else {
                    continue;
                };
                let Some(address) = section.address().checked_add(offset) else {
                    continue;
                };
                let Some(owners) = owners_by_address.get(&address) else {
                    continue;
                };
                record.definition = match owners.len() {
                    0 => Definition::Unknown,
                    1 => match owners.first() {
                        Some(owner) => Definition::Proven(owner.clone()),
                        None => Definition::Unknown,
                    },
                    _ => Definition::Ambiguous,
                };
            }
            let registry = found.get_or_insert_with(Registry::default);
            registry.records.extend(parsed.records);
            registry.malformed += parsed.malformed;
        }
    }
    Ok(found)
}

/// Parse the records out of the bytes of a trace section. Bytes between
/// records, such as padding, are skipped. An entry that does not parse is
/// counted in [`Registry::malformed`], never dropped silently.
#[must_use]
#[implements("FR-006-AC-1")]
pub fn parse_records(bytes: &[u8]) -> Registry {
    parse_records_with_offsets(bytes).0
}

#[implements("FR-006-AC-24")]
fn parse_records_with_offsets(bytes: &[u8]) -> (Registry, Vec<usize>) {
    let mut offsets = Vec::new();
    let mut registry = Registry::default();
    let mut rest = bytes;
    while let Some(start) = find(rest, FAMILY) {
        let after = &rest[start + FAMILY.len()..];
        if !after.starts_with(&START[FAMILY.len()..]) {
            // The same family under another format version.
            registry.malformed += 1;
            rest = after;
            continue;
        }
        let body = &after[START.len() - FAMILY.len()..];
        let Some(end) = body.iter().position(|&b| b == END) else {
            registry.malformed += 1;
            break;
        };
        match parse_one(&body[..end]) {
            Some(record) => {
                offsets.push(bytes.len() - rest.len() + start);
                registry.records.push(record);
            }
            None => registry.malformed += 1,
        }
        rest = &body[end + 1..];
    }
    (registry, offsets)
}

/// Only the v0 compiler symbol shape emitted by the supported toolchain proves an owner.
/// Unknown or stripped symbol forms remain unproved rather than guessing from record fields.
#[implements("FR-006-AC-24")]
fn compiler_owner(name: &str) -> Option<String> {
    rustc_demangle::try_demangle(name).ok()?;
    let body = name
        .strip_prefix("__R")
        .or_else(|| name.strip_prefix("_R"))?;
    // The emitter defines RECORD in an anonymous const in the function body:
    // N v (N v OWNER DISAMBIG IDENT("_")) DISAMBIG IDENT("RECORD").
    // Keep the entire raw OWNER encoding and its original backreference offsets.
    body.strip_prefix("NvNv")?;
    let mut proved = None;
    for boundary in 4..body.len() {
        let suffix = body.get(boundary..)?;
        let Some((anonymous, rest)) = wrapper_identifier(suffix) else {
            continue;
        };
        let Some((record, trailing)) = wrapper_identifier(rest) else {
            continue;
        };
        if anonymous != "_" || record != "RECORD" || !trailing.is_empty() {
            continue;
        }
        // Explicit canonical disambiguators make an incorrect boundary that
        // leaves a wrapper disambiguator in OWNER fail public parser validation.
        let canonical = format!("_R{}s_1__s_6RECORD", body.get(..boundary)?);
        let Ok(parsed) = rustc_demangle::try_demangle(&canonical) else {
            continue;
        };
        // Validate that the public parser consumes the intended two wrappers.
        // Display is used only to check their literal names, never as identity.
        if !parsed.to_string().ends_with("::_::RECORD") {
            continue;
        }
        if proved.is_some() {
            return None;
        }
        proved = Some(canonical);
    }
    proved
}

/// Parse one v0 wrapper's optional base-62 disambiguator and length-prefixed
/// ASCII identifier. The complete source and canonical symbols are validated
/// by rustc-demangle; unknown wrapper structures never establish an owner.
#[implements("FR-006-AC-24")]
fn wrapper_identifier(input: &str) -> Option<(&str, &str)> {
    let input = match input.strip_prefix('s') {
        Some(disambiguated) => {
            let (digits, rest) = disambiguated.split_once('_')?;
            if !digits.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
                return None;
            }
            rest
        }
        None => input,
    };
    let digits = input.bytes().take_while(u8::is_ascii_digit).count();
    let length: usize = input.get(..digits)?.parse().ok()?;
    let input = input.get(digits..)?;
    let input = input.strip_prefix('_').unwrap_or(input);
    Some((input.get(..length)?, input.get(length..)?))
}

/// A `/`-separated form of a path text. Only Windows writes `\` as a separator,
/// so elsewhere the text is kept: `\` there is part of a directory name.
#[cfg(windows)]
pub(crate) fn slashed(text: &str) -> String {
    text.replace('\\', "/")
}

/// A `/`-separated form of a path text (see the Windows form).
#[cfg(not(windows))]
pub(crate) fn slashed(text: &str) -> String {
    text.to_owned()
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[implements("FR-006-AC-1")]
fn parse_one(body: &[u8]) -> Option<Record> {
    let text = std::str::from_utf8(body).ok()?;
    let mut fields = text.split(SEP);
    let flags = fields.next()?;
    let ids = fields.next()?;
    let ident = fields.next()?;
    let module_path = fields.next()?;
    let file = fields.next()?;
    if fields.next().is_some() || flags.len() != 2 {
        return None;
    }
    Some(Record {
        definition: Definition::Unknown,
        ids: ids.split(',').map(str::to_owned).collect(),
        ident: ident.to_owned(),
        module_path: module_path.to_owned(),
        file: slashed(file),
        test: flags.starts_with('t'),
        parametrized: flags.ends_with('p'),
    })
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::parse_records;

    #[trace("FR-006-AC-1")]
    #[test]
    fn records_parse_between_padding_and_a_malformed_one_is_counted() {
        let mut bytes = b"\0\0".to_vec();
        bytes.extend(b"\x02IXTR1t-\x1fTC-1,FR-1-AC-1\x1ft\x1fc::tests\x1fsrc/lib.rs\x03");
        bytes.extend(b"\x02IXTR1garbage\x03\0");
        bytes.extend(b"\x02IXTR2t-\x1fTC-9\x1fv\x1fc\x1fsrc/lib.rs\x03");
        bytes.extend(b"\x02IXTR1-p\x1fTC-2\x1fu\x1fc\x1fsrc/lib.rs\x03");
        let parsed = parse_records(&bytes);
        let records = parsed.records;
        assert_eq!(records.len(), 2);
        assert_eq!(parsed.malformed, 2);
        assert_eq!(records[0].ids, ["TC-1", "FR-1-AC-1"]);
        assert!(records[0].test && !records[0].parametrized);
        assert_eq!(records[0].libtest_name(), "tests::t");
        assert!(!records[1].test && records[1].parametrized);
        assert_eq!(records[1].libtest_name(), "u");
    }

    #[trace("FR-006-AC-1")]
    #[test]
    fn a_backslash_is_a_separator_only_on_windows() {
        let bytes = b"\x02IXTR1t-\x1fTC-1\x1ft\x1fc\x1fcrates/my\\pkg/src/lib.rs\x03";
        let file = &parse_records(bytes).records[0].file;
        let expected = if cfg!(windows) {
            "crates/my/pkg/src/lib.rs"
        } else {
            "crates/my\\pkg/src/lib.rs"
        };
        assert_eq!(file, expected);
    }
    #[trace("FR-006-AC-24")]
    #[test]
    fn raw_owner_keys_normalize_only_the_record_wrappers() {
        let first = "_RNvNvNvCs36XdazKT9rZ_7stackeds_7stacked1__6RECORD";
        let second = "_RNvNvNvCs36XdazKT9rZ_7stackeds_7stackeds_1__6RECORD";
        let a = super::compiler_owner(first);
        let b = super::compiler_owner(second);
        assert!(a.is_some(), "first owner: {a:?}; second: {b:?}");
        assert_eq!(a, b);
        let distinct_function = "_RNvNvNvCs36XdazKT9rZ_7stackeds0_7stacked1__6RECORD";
        let other = super::compiler_owner(distinct_function);
        assert!(
            other.is_some(),
            "a distinct function remains a supported owner"
        );
        assert_ne!(
            a, other,
            "function disambiguators remain part of raw identity"
        );
    }
}
