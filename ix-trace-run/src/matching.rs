//! Joining records to listed tests and naming the reported symbol (FR-006).
//!
//! A record belongs to a listed test by libtest name only. Nothing is guessed
//! from the shape of a name.

use std::collections::{BTreeMap, HashMap};
use std::path::{Component, Path, PathBuf};

use crate::registry::{slashed, Record};

/// Where a test binary's sources live, as cargo reports them.
#[derive(Clone, Debug)]
pub struct Layout {
    /// The workspace root, against which a relative `file!()` is resolved.
    pub workspace_root: PathBuf,
    /// The target's root source file (cargo's `src_path`).
    pub root_file: PathBuf,
}

/// The mark a record gives a listed test.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mark {
    /// The file-relative symbol, reported as `name` and `classname`.
    pub symbol: String,
    /// The repo-relative `/`-separated source path.
    pub file: String,
    /// The declared ids.
    pub ids: Vec<String>,
}

/// The result of joining one binary's records to its tests.
#[derive(Debug, Default)]
pub struct Matching {
    /// The mark of each listed test that has one.
    pub marks: HashMap<String, Mark>,
    /// Records to report as belonging to no listed test.
    pub unmatched: Vec<Record>,
    /// Groups of records sharing one libtest name.
    pub ambiguities: Vec<Vec<Record>>,
    /// `#[rstest]`-above-`#[trace]` misuse warnings.
    pub warnings: Vec<String>,
}

/// Join `records` to the `listed` test names of one binary.
#[must_use]
pub fn match_records(records: &[Record], listed: &[String], layout: &Layout) -> Matching {
    let mut matching = Matching::default();

    let mut by_name: BTreeMap<String, Vec<&Record>> = BTreeMap::new();
    for record in records {
        by_name
            .entry(record.libtest_name())
            .or_default()
            .push(record);
    }
    let mut candidates: Vec<(String, &Record)> = Vec::new();
    for (name, group) in by_name {
        if let [only] = group.as_slice() {
            candidates.push((name, only));
        } else {
            matching
                .ambiguities
                .push(group.into_iter().cloned().collect());
        }
    }

    let mut attached = vec![false; candidates.len()];
    let mut prefix_failed = vec![false; candidates.len()];
    let marks_of: Vec<Option<Mark>> = candidates
        .iter()
        .map(|(name, record)| mark_of(name, record, layout))
        .collect();

    for test in listed {
        let exact = candidates.iter().position(|(name, _)| name == test);
        let found = exact.or_else(|| {
            candidates
                .iter()
                .enumerate()
                .filter(|(_, (name, record))| {
                    record.parametrized
                        && test.len() > name.len() + 2
                        && test.starts_with(name.as_str())
                        && test[name.len()..].starts_with("::")
                })
                .max_by_key(|(_, (name, _))| name.len())
                .map(|(index, _)| index)
        });
        let Some(index) = found else { continue };
        match &marks_of[index] {
            Some(mark) => {
                attached[index] = true;
                matching.marks.insert(test.clone(), mark.clone());
            }
            None => prefix_failed[index] = true,
        }
    }

    for (index, (_, record)) in candidates.iter().enumerate() {
        if !attached[index] && (record.test || prefix_failed[index]) {
            matching.unmatched.push((*record).clone());
        }
    }
    matching.warnings = misuse_warnings(records);
    matching
}

/// The mark a record would give its tests, or `None` when its libtest name
/// does not begin with the module prefix of its file.
fn mark_of(libtest_name: &str, record: &Record, layout: &Layout) -> Option<Mark> {
    let prefix = module_prefix(&record.file, layout)?;
    let symbol = if prefix.is_empty() {
        libtest_name
    } else {
        libtest_name.strip_prefix(&prefix)?.strip_prefix("::")?
    };
    Some(Mark {
        symbol: symbol.to_owned(),
        file: normalize_file(&record.file, &layout.workspace_root),
        ids: record.ids.clone(),
    })
}

/// The module path of a source file: its path below the directory of the
/// target's root file, `foo/bar.rs` as `foo::bar` and `foo/mod.rs` as `foo`.
/// `None` when the file lies outside that directory.
fn module_prefix(file: &str, layout: &Layout) -> Option<String> {
    let path = absolute(file, &layout.workspace_root);
    if path == layout.root_file {
        return Some(String::new());
    }
    let root_dir = layout.root_file.parent()?;
    let relative = path.strip_prefix(root_dir).ok()?;
    let mut segments: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => segments.push(part.to_str()?.to_owned()),
            _ => return None,
        }
    }
    let last = segments.pop()?;
    let stem = last.strip_suffix(".rs")?;
    if segments.is_empty() && matches!(stem, "lib" | "main") {
        return Some(String::new());
    }
    if stem != "mod" {
        segments.push(stem.to_owned());
    }
    Some(segments.join("::"))
}

fn absolute(file: &str, workspace_root: &Path) -> PathBuf {
    let path = Path::new(file);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

/// A repo-relative, `/`-separated form of a `file!()` value.
#[must_use]
pub fn normalize_file(file: &str, workspace_root: &Path) -> String {
    let path = absolute(file, workspace_root);
    let relative = path.strip_prefix(workspace_root).unwrap_or(Path::new(file));
    let text = slashed(&relative.to_string_lossy());
    text.strip_prefix("./").unwrap_or(&text).to_owned()
}

/// Two or more records of one file with the same ids and identifiers that
/// begin `case_`: the shape of `#[rstest]` written above `#[trace]`.
fn misuse_warnings(records: &[Record]) -> Vec<String> {
    let mut groups: BTreeMap<(&str, &[String]), Vec<&str>> = BTreeMap::new();
    for record in records.iter().filter(|r| r.ident.starts_with("case_")) {
        groups
            .entry((record.file.as_str(), record.ids.as_slice()))
            .or_default()
            .push(record.ident.as_str());
    }
    groups
        .into_iter()
        .filter(|(_, idents)| idents.len() > 1)
        .map(|((file, ids), idents)| {
            format!(
                "{file}: {} share the ids {}; `#[rstest]` above `#[trace]` copies the tag onto \
                 each case, so write `#[trace]` above `#[rstest]`",
                idents.join(", "),
                ids.join(",")
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ix_trace_rs::trace;

    use super::{match_records, Layout};
    use crate::registry::Record;

    fn layout() -> Layout {
        Layout {
            workspace_root: PathBuf::from("/w"),
            root_file: PathBuf::from("/w/src/lib.rs"),
        }
    }

    fn record(ids: &[&str], ident: &str, module: &str, file: &str) -> Record {
        Record {
            ids: ids.iter().map(|s| (*s).to_owned()).collect(),
            ident: ident.to_owned(),
            module_path: module.to_owned(),
            file: file.to_owned(),
            test: true,
            parametrized: false,
        }
    }

    fn listed(names: &[&str]) -> Vec<String> {
        names.iter().map(|s| (*s).to_owned()).collect()
    }

    #[trace("FR-006-AC-13")]
    #[test]
    fn identical_ids_on_case_functions_of_one_file_warn() {
        let records = [
            record(&["TC-1"], "case_1", "c::m::t", "src/lib.rs"),
            record(&["TC-1"], "case_2", "c::m::t", "src/lib.rs"),
        ];
        let matching = match_records(&records, &listed(&["m::t::case_1"]), &layout());
        assert_eq!(matching.warnings.len(), 1);
        assert!(matching.warnings[0].contains("case_1, case_2"));
    }

    #[trace("FR-006-AC-3")]
    #[test]
    fn the_symbol_drops_the_module_prefix_of_the_file() {
        let nested = record(&["TC-2"], "t", "c::foo::bar::tests", "src/foo/bar.rs");
        let modrs = record(&["TC-3"], "u", "c::foo::tests", "src/foo/mod.rs");
        let matching = match_records(
            &[nested, modrs],
            &listed(&["foo::bar::tests::t", "foo::tests::u"]),
            &layout(),
        );
        assert_eq!(matching.marks["foo::bar::tests::t"].symbol, "tests::t");
        assert_eq!(matching.marks["foo::bar::tests::t"].file, "src/foo/bar.rs");
        assert_eq!(matching.marks["foo::tests::u"].symbol, "tests::u");
    }

    #[trace("FR-006-AC-9")]
    #[test]
    fn a_name_outside_its_files_prefix_is_unmatched() {
        let stray = record(&["TC-4"], "t", "c::elsewhere", "src/foo/bar.rs");
        let matching = match_records(&[stray], &listed(&["elsewhere::t"]), &layout());
        assert!(matching.marks.is_empty());
        assert_eq!(matching.unmatched.len(), 1);
    }

    #[trace("FR-006-AC-19")]
    #[test]
    fn two_records_with_one_libtest_name_mark_nothing() {
        let a = record(&["X-1"], "check", "c", "src/lib.rs");
        let b = record(&["TC-5"], "check", "c", "src/lib.rs");
        let matching = match_records(&[a, b], &listed(&["check"]), &layout());
        assert!(matching.marks.is_empty());
        assert_eq!(matching.ambiguities.len(), 1);
        assert!(matching.unmatched.is_empty());
    }

    #[trace("FR-006-AC-7")]
    #[test]
    fn a_parametrized_record_marks_its_cases_and_a_plain_one_marks_no_child() {
        let mut param = record(&["TC-6"], "p", "c", "src/lib.rs");
        param.parametrized = true;
        let plain = record(&["TC-7"], "q", "c", "src/lib.rs");
        let matching = match_records(
            &[param, plain],
            &listed(&["p::case_1", "p::case_2", "q", "q::inner"]),
            &layout(),
        );
        assert_eq!(matching.marks["p::case_1"].symbol, "p");
        assert_eq!(matching.marks["p::case_2"].ids, ["TC-6"]);
        assert_eq!(matching.marks["q"].ids, ["TC-7"]);
        assert!(!matching.marks.contains_key("q::inner"));
    }
}
