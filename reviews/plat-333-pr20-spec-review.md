---
id: SR-014
title: "Spec review of PLAT-333 TC-022 flip (ix-trace-rs PR 20)"
type: SpecReview
analysis: base
scope: "agent-ix/ix-trace-rs; spec/tests.md (PR 20 diff only, head f349cfc)"
review_set: subset
---

## Summary

Ticket: PLAT-333. Checked that the flipped TC-022 and FR-006 rows agree with every other status in `spec/tests.md` and with FR-006, and that the matrix validates. The owner decision to flip on loaded-host evidence is taken as given.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Verdict

Clean. The FR-006 row (AC-1..23; CON-1..2) is now consistent: TC-020 and TC-021 were already passing and cover every AC except AC-12, TC-022 covers AC-12 and CON-1, and CON-2's verification method is Inspection (FR-006 line 132), so it needs no TC. The closing line "GREEN for the v1 surface, with two rows honestly pending" names FR-003 and StR-001-VC-1, which are exactly the two rows still pending after this PR (before it, FR-006 and TC-022 were also pending, so the PR makes that line true). Nothing else in the file mentions a quiet host or a pending TC-022. `quire validate spec/tests.md` (quire 0.39.0, engine 0.55.0) exits 0 at head and 3 on main (the column-name assertion this PR fixes).
