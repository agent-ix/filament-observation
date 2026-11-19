---
id: SR-072
title: "Gap analysis — optional observation ingestion time"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-observation; repository matrix and changed admission, observation, partial-fact, schema, and test paths"
review_set: subset
relationships:
  - { target: ix://agent-ix/filament-observation/FR-004, type: references }
  - { target: ix://agent-ix/filament-observation/FR-007, type: references }
  - { target: ix://agent-ix/filament-observation/FR-001, type: references }
  - { target: ix://agent-ix/filament-observation/FR-014, type: references }
---

## Summary

The full computed matrix has 18 untagged criteria, all unchanged from `origin/main`; changed production paths are otherwise owned by the local requirements and focused tests. Two newly changed test bindings do not accurately connect the new absence behavior to its requirement evidence.

## Verdict

FAIL. The repository matrix retains inherited high-severity untagged criteria; the PR also adds behavior tests whose criterion bindings need correction.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Eighteen existing NFR criteria have no recognized trace-tagged test symbols in the computed matrix; this baseline is identical to origin/main. | NFR-001-AC-3, NFR-001-AC-4, NFR-001-AC-5, NFR-001-M-1 through NFR-001-M-6, NFR-002-AC-1, NFR-002-AC-2, NFR-002-M-1, NFR-002-M-2, NFR-003-M-1 through NFR-003-M-5 |
| FND-002 | medium | The new partial-fact absence test is tagged to FR-007-AC-1 but does not assert the possibility set or reason named by that criterion. | tests/partial_interval.rs::tc007_partial_fact_retains_absent_ingestion_position_as_null |
| FND-003 | medium | The new observation test for absent ingestion time is tagged only with TC-004, so the computed matrix does not bind this evidence to the changed FR-004-AC-1. | tests/authority.rs::tc004_absent_ingestion_time_is_null_and_is_not_classified |

## Coverage

- Reconciliation ran with `quoin matrix --repo . --json` (Quoin 0.36.7) and `quire validate --scope . "spec/**/*.md" --summary` (Quire 0.40.0 / engine 0.56.0).
- Matrix totals: 116 criteria; 97 tagged, 18 untagged, 1 method-without-symbol. All 116 show no run evidence in the evidence store. The same static totals and untagged IDs appear when computed from an `origin/main` archive, so FND-001 is inherited, not introduced by this PR.
- The changed admission and candidate-line tests bind FR-001-AC-1 and FR-014-AC-8. The partial-order behavior test binds FR-007-AC-5. The optional observation-record test uses only `TC-004`; the partial-fact null test claims FR-007-AC-1 without asserting its possibility/reason property.
- Reverse-gap inventory for changed public behavior: optional `AdmittedRecord` ingestion, optional record-view ingestion/lateness, and optional `partial::Selection` ingestion position are accounted for by FR-001/FR-004/FR-007/FR-014 and the owner requirement relayed for QSL-2524. No changed source stub or hollow test was found.
- No plan was supplied. Plan completion: not assessed.

## Examined units

- `FR-004-AC-1` and its new pre-release identity/schema wording.
- `FR-007-AC-1` and `FR-007-AC-5` for partial fact retention and interval ordering.
- `FR-001-AC-1` and `FR-014-AC-8` for record and candidate-line admission.
- The newly added absence tests and their computed trace bindings.

## Finding detail

### FND-001 — Existing untagged NFR criteria

The matrix reports the listed 18 NFR criteria as `untagged`. Their status is identical in `origin/main`, so this is an inherited repository assurance gap rather than a regression from QSL-2524.

### FND-002 — Partial-fact test claims an unrelated AC

`tc007_partial_fact_retains_absent_ingestion_position_as_null` constructs the `Missing` possibility set but only asserts the schema's null branch, emitted null value, and absent getter result. It does not assert the set's exact false/true possibilities or `Missing` reason required by FR-007-AC-1. The trace tag therefore claims evidence for a different behavior than the test establishes.

### FND-003 — Observation test lacks a direct AC trace

`tc004_absent_ingestion_time_is_null_and_is_not_classified` verifies that the absent instant changes identity, is emitted as null, derives null lateness, and strict-reads back as absent. Its only trace tag is `TC-004`; the matrix does not bind it to the changed FR-004-AC-1, which names the contract schema and identity behavior.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | deferred | The 18 untagged NFR criteria are unchanged from origin/main and are inherited repository assurance debt outside this PR's scope. |
| FND-002 | fixed | The test no longer claims FR-007-AC-1 coverage; its trace now names TC-007 only. |
| FND-003 | fixed | The absent-ingestion test now directly traces FR-004-AC-1. |
