---
id: SR-006
title: "EARS analysis — PLAT-333 emitter spec (PR #15)"
type: SpecReview
analysis: ears-conformance
scope: "agent-ix/ix-trace-rs@7836a1f7dccfab7a27fed5845f561a5b539041a4; spec/functional/FR-004-emission-mode.md, spec/functional/FR-005-emission-run.md, spec/functional/FR-001-trace-attribute.md"
review_set: subset
---

## Summary

Ticket: PLAT-333. Checked the new FR-004 and FR-005 statements, their constraints and the edited
FR-001-CON-1 for EARS form. FR-001-CON-1's new `While ... not enabled` prefix is correct
state-driven form; FR-005-CON-1 is state-driven and well formed; FR-004-CON-1..3 and
FR-005-CON-2 are ubiquitous and single-clause.

Units examined:

- `FR-004` (examined): spec/functional/FR-004-emission-mode.md
- `FR-005` (examined): spec/functional/FR-005-emission-run.md
- `FR-001-CON-1` (examined): spec/functional/FR-001-trace-attribute.md
- `FR-004-CON-1` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-CON-2` (examined): spec/functional/FR-004-emission-mode.md
- `FR-004-CON-3` (examined): spec/functional/FR-004-emission-mode.md
- `FR-005-CON-1` (examined): spec/functional/FR-005-emission-run.md
- `FR-005-CON-2` (examined): spec/functional/FR-005-emission-run.md

## Verdict

**APPROVE with two low findings.** Neither changes behaviour; both are wording.

## Findings

| ID      | Severity | Summary | Refs |
| ------- | -------- | ------- | ---- |
| FND-001 | low | FR-004 uses an event trigger ('When the cargo feature emit is enabled') for a state. A cargo feature being enabled is a build state, not an event. EARS state-driven form is 'While the `emit` feature is enabled, ...', matching the edited FR-001-CON-1 ('While the `emit` feature is not enabled'). | spec/functional/FR-004-emission-mode.md:19-24 |
| FND-002 | low | FR-005 statement is compound: it provides a runner that executes the binaries and also writes the JUnit file. The statement binds two obligations (execute the test binaries under libtest; write the final JUnit file with trace properties). The Behavior bullets already split them; the statement could name the one outcome ('SHALL write the run's JUnit result file, in which ...') and leave execution to the Behavior. | spec/functional/FR-005-emission-run.md:19-24 |
