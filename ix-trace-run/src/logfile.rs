//! Parsing libtest's `--logfile` (FR-006).
//!
//! The harness writes one line per finished test: `ok <name>`, `failed <name>`,
//! `ignored <name>`, and the reason forms `ignored: <reason> <name>` and
//! `failed: <message> <name>`. Test output cannot reach the file, so outcomes
//! never come from stdout.

use std::collections::{BTreeSet, HashMap, HashSet};

/// The result of one test.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    /// `ok`.
    Pass,
    /// `failed`.
    Fail,
    /// `ignored`.
    Skip,
}

/// What a log file says about the listed tests.
#[derive(Debug, Default)]
pub struct Log {
    /// The outcome of each listed test that has a log line.
    pub outcomes: HashMap<String, Outcome>,
    /// Listed tests with more than one log line.
    pub duplicates: BTreeSet<String>,
    /// Whether any line was `failed`.
    pub saw_failed: bool,
}

/// Parse `text` against the `listed` test names. A line is read as a record
/// only when the name it ends with is a listed name.
#[must_use]
pub fn parse(text: &str, listed: &[String]) -> Log {
    let names: HashSet<&str> = listed.iter().map(String::as_str).collect();
    let mut log = Log::default();
    let mut seen: HashSet<&str> = HashSet::new();
    let mut pending: Option<Outcome> = None;

    for line in text.lines() {
        if let Some(outcome) = pending {
            // Continuation of a reason that held a newline: it ends at a line
            // whose last word is a listed name not yet seen.
            if let Some(name) = tail_name(line, &names).filter(|name| !seen.contains(name)) {
                log.record(outcome, name, &mut seen);
                pending = None;
            }
            continue;
        }
        let Some((outcome, reasoned, rest)) = split_line(line) else {
            continue;
        };
        if reasoned {
            match tail_name(rest, &names) {
                Some(name) => log.record(outcome, name, &mut seen),
                None => pending = Some(outcome),
            }
        } else if let Some(name) = names.get(rest) {
            log.record(outcome, name, &mut seen);
        }
    }
    log
}

impl Log {
    fn record<'a>(&mut self, outcome: Outcome, name: &'a str, seen: &mut HashSet<&'a str>) {
        if outcome == Outcome::Fail {
            self.saw_failed = true;
        }
        if seen.insert(name) {
            self.outcomes.insert(name.to_owned(), outcome);
        } else {
            self.duplicates.insert(name.to_owned());
        }
    }
}

/// Split a line into its outcome, whether it is a reason form, and the rest.
fn split_line(line: &str) -> Option<(Outcome, bool, &str)> {
    for (prefix, outcome, reasoned) in [
        ("ok ", Outcome::Pass, false),
        ("failed ", Outcome::Fail, false),
        ("ignored ", Outcome::Skip, false),
        ("failed: ", Outcome::Fail, true),
        ("ignored: ", Outcome::Skip, true),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            return Some((outcome, reasoned, rest));
        }
    }
    None
}

/// The listed name a line ends with, after its last space.
fn tail_name<'a>(line: &str, names: &HashSet<&'a str>) -> Option<&'a str> {
    let (_, tail) = line.rsplit_once(' ')?;
    names.get(tail).copied()
}

#[cfg(test)]
mod tests {
    use ix_trace_rs::trace;

    use super::{parse, Outcome};

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[trace("FR-006-AC-15")]
    #[test]
    fn a_second_line_for_a_test_is_a_duplicate_and_a_reason_form_is_skipped() {
        let listed = names(&["a", "b", "c"]);
        let log = parse("ok a\nignored: not ready b\nok a\nfailed c\n", &listed);
        assert!(log.duplicates.contains("a"));
        assert_eq!(log.outcomes["b"], Outcome::Skip);
        assert_eq!(log.outcomes["c"], Outcome::Fail);
        assert!(log.saw_failed);
    }

    #[trace("FR-006-AC-17")]
    #[test]
    fn a_reason_with_a_newline_is_one_skipped_test_and_later_lines_keep_their_outcomes() {
        let listed = names(&["first", "second", "third"]);
        let log = parse(
            "ok first\nignored: line one\nline two second\nfailed third\n",
            &listed,
        );
        assert_eq!(log.outcomes["first"], Outcome::Pass);
        assert_eq!(log.outcomes["second"], Outcome::Skip);
        assert_eq!(log.outcomes["third"], Outcome::Fail);
        assert_eq!(log.outcomes.len(), 3);
    }

    #[trace("FR-006-AC-14")]
    #[test]
    fn a_line_naming_no_listed_test_is_not_read() {
        let listed = names(&["real"]);
        let log = parse("ok other\nfailed self\nok real\n", &listed);
        assert_eq!(log.outcomes.len(), 1);
        assert_eq!(log.outcomes["real"], Outcome::Pass);
        assert!(!log.saw_failed);
    }
}
