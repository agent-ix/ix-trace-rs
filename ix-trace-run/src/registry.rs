//! Reading the trace records `ix-trace-rs` embeds in a test binary (FR-005).
//!
//! The reader opens the binary as data. Nothing in the binary runs.

use std::path::Path;

use object::{Object, ObjectSection};

/// Start of a record; matches `ix-trace-rs`'s emitter.
const START: &[u8] = b"\x02IXTR1";
/// The record start without its format version.
const FAMILY: &[u8] = b"\x02IXTR";
/// Field separator.
const SEP: char = '\u{1f}';
/// End of a record.
const END: u8 = 3;

/// One trace record: the ids declared on a `fn` item and where it lives.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
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
pub fn read_records(path: &Path) -> Result<Option<Registry>, RegistryError> {
    let data = std::fs::read(path).map_err(RegistryError::Io)?;
    let file = object::File::parse(&*data).map_err(RegistryError::Format)?;
    let mut found: Option<Registry> = None;
    for section in file.sections() {
        let Ok(name) = section.name() else { continue };
        if matches!(name, "ixtrace" | "__ixtrace" | ".ixtrace") {
            let bytes = section.data().map_err(RegistryError::Format)?;
            let parsed = parse_records(bytes);
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
pub fn parse_records(bytes: &[u8]) -> Registry {
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
            Some(record) => registry.records.push(record),
            None => registry.malformed += 1,
        }
        rest = &body[end + 1..];
    }
    registry
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

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
        ids: ids.split(',').map(str::to_owned).collect(),
        ident: ident.to_owned(),
        module_path: module_path.to_owned(),
        file: file.to_owned(),
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
}
