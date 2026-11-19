---
id: SR-073
title: "Base specification review — optional observation ingestion time"
type: SpecReview
analysis: base
scope: "spec/functional/FR-004-publish-observation-authority-artifacts.md; FR-004-AC-1; related FR-001, FR-007, FR-014 and TC-004/TC-007/TC-016"
review_set: subset
relationships:
  - { target: ix://agent-ix/filament-observation/FR-004, type: references }
  - { target: ix://agent-ix/filament-observation/FR-007, type: references }
---

## Summary

The changed FR-004 text preserves stable contract labels and schemas before first release, makes the nullable record fields explicit, and states the post-release immutability boundary. The schema and identity behavior is externally observable and the focused tests exercise the null path.

## Verdict

PASS. The base checklist found no defect in the changed specification text.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Checked identifier formats, local links, verification references, and the six coverage questions against the edited FR-004 clause and its related test cases.
- `quire validate --scope . "spec/**/*.md" --summary` completed successfully with two existing FR-008 EARS warnings; the edited FR-004 artifact had no validation warning.
- No applicable AssuranceProfile was found under `spec/`.
