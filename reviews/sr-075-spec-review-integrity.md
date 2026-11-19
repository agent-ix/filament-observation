---
id: SR-075
title: "Integrity review — optional observation ingestion time"
type: SpecReview
analysis: integrity
scope: "spec/functional/FR-004-publish-observation-authority-artifacts.md and related FR-001/FR-007 contract references"
review_set: subset
relationships:
  - { target: ix://agent-ix/filament-observation/FR-004, type: references }
---

## Summary

The edited release-boundary language is consistent with the pre-release null representation and post-release immutability rule. The nullable fields remain required, so absence is represented explicitly rather than by an omitted wire member.

## Verdict

PASS for the changed requirement structure and consistency.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

- Reviewed FR-004's existing v1/v2 contract boundaries, new pre-release/post-release rule, and FR-004-AC-1 against the changed schemas and focused tests.
- No requirement identifier, relationship, section, or validation-link structure changed. The unrelated FR-007 input wording says ingestion position is separate and does not require it to be always present.
