//! The JUnit result file (quoin FR-120): one `<testcase>` per test with
//! `name` equal to `classname`, a `file` attribute and the `trace` property.

use std::fmt::Write as _;

use crate::logfile::Outcome;

/// One `<testcase>`.
#[derive(Debug, Eq, PartialEq)]
pub struct Case {
    /// Reported as both `name` and `classname`.
    pub name: String,
    /// Repo-relative `/`-separated path of the source file, when a record
    /// belongs to the case.
    pub file: Option<String>,
    /// The ids of the record that belongs to the case.
    pub trace: Option<Vec<String>>,
    /// The runner's result.
    pub outcome: Outcome,
    /// Why the case failed, when it did.
    pub failure: Option<String>,
}

/// One `<testsuite>`: the cases of one test binary.
#[derive(Debug, Eq, PartialEq)]
pub struct Suite {
    /// The target name.
    pub name: String,
    /// The cases in listed order.
    pub cases: Vec<Case>,
}

/// Render `suites` as one JUnit document.
#[must_use]
pub fn render(suites: &[Suite]) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<testsuites>\n");
    for suite in suites {
        let failures = count(suite, Outcome::Fail);
        let skipped = count(suite, Outcome::Skip);
        let _ = writeln!(
            out,
            "  <testsuite name=\"{}\" tests=\"{}\" failures=\"{failures}\" skipped=\"{skipped}\">",
            escape(&suite.name),
            suite.cases.len()
        );
        for case in &suite.cases {
            write_case(&mut out, case);
        }
        out.push_str("  </testsuite>\n");
    }
    out.push_str("</testsuites>\n");
    out
}

fn count(suite: &Suite, outcome: Outcome) -> usize {
    suite
        .cases
        .iter()
        .filter(|case| case.outcome == outcome)
        .count()
}

fn write_case(out: &mut String, case: &Case) {
    let name = escape(&case.name);
    let _ = write!(out, "    <testcase classname=\"{name}\" name=\"{name}\"");
    if let Some(file) = &case.file {
        let _ = write!(out, " file=\"{}\"", escape(file));
    }
    let children = case.trace.is_some() || case.outcome != Outcome::Pass;
    if !children {
        out.push_str("/>\n");
        return;
    }
    out.push_str(">\n");
    if let Some(ids) = &case.trace {
        let _ = writeln!(
            out,
            "      <property name=\"trace\" value=\"{}\"/>",
            escape(&ids.join(","))
        );
    }
    match case.outcome {
        Outcome::Pass => {}
        Outcome::Skip => out.push_str("      <skipped/>\n"),
        Outcome::Fail => {
            let message = case.failure.as_deref().unwrap_or("test failed");
            let _ = writeln!(out, "      <failure message=\"{}\"/>", escape(message));
        }
    }
    out.push_str("    </testcase>\n");
}

/// Escape text for an attribute value, dropping characters XML 1.0 forbids.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\n' | '\r' | '\t' => {
                let _ = write!(out, "&#{};", u32::from(c));
            }
            c if u32::from(c) < 0x20 => {}
            c => out.push(c),
        }
    }
    out
}
