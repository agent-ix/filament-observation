---
id: SR-071
title: "Code review — optional observation ingestion time"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-observation; Cargo.toml, src/authority/, schemas/, tests/"
review_set: subset
relationships:
  - { target: ix://agent-ix/filament-observation/FR-004, type: references }
---

## Summary

Reviewed QSL-2524 PR #66 at `c6283d1beb78182c490aa3bad00a0a1166e5d8dc`. The optional ingestion representation, required nullable wire member, strict-read behavior, partial-fact position, and exact dependency revisions are consistent; one high repository-policy finding remains because the changed bundle module retains schema digest pins.

## Verdict

FAIL. The code updates and continues asserting a schema digest in a file whose repository guidance explicitly requires removing digest pins when that file is touched.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-001 | high | Bundle schema digest constants and pin assertions remain in the changed module despite the repository rule to remove them. | src/authority/bundle.rs:25,1905-1913 | implementation-bug-despite-evidence |

## Scope and evidence

- Examined the changed public APIs, record identity preimage, admission wire parsing, bundle and owner schemas, focused tests, Cargo dependency graph, and FR-004 wording.
- `cargo fmt --all -- --check` passed.
- `cargo clippy --locked -p filament-observation --lib -- -D warnings` passed through `locked-build.sh`.
- Focused tests passed through `locked-build.sh`: `embedded_record_v2_schema_allows_absent_ingestion_and_lateness`, `tc004_absent_ingestion_time_is_null_and_is_not_classified`, `progress_admission::record_candidate_line_preserves_null_ingestion_time`, `tc007_partial_fact_retains_absent_ingestion_position_as_null`, `tc007_ingestion_position_and_midpoint_are_not_semantic_order_inputs`, `tc001_record_without_ingestion_instant_is_admitted_without_defaulting`, and `tc004_every_required_field_is_missing_duplicate_and_order_strict`.
- `cargo tree --locked` confirmed `filament-exact` at `b26a8e4c`, `filament-semantic-value` at `1df71c27`, and `filament-contract-ir` at `a461f0a1`.
- `quire validate --scope . "spec/**/*.md" --summary` passed with two existing FR-008 grammar warnings; no warning concerned the changed FR-004 text.
- No aggregate gate was run.

## Examined units

- `src/authority/observation.rs`: optional ingestion and lateness flow through identity derivation, serialization, derivation, and strict reading.
- `src/authority/admission.rs`: required-option deserialization preserves member presence while accepting JSON null.
- `src/authority/partial.rs`: optional ingestion position is retained but not used in interval ordering.
- `schemas/observation-record-v2.schema.json`, `schemas/observation-partial-fact-v1.schema.json`, `schemas/observation-authority-v1.schema.json`, and `schemas/record-candidate-v1.schema.json`: nullable values remain required fields.
- `tests/authority.rs`, `tests/admission.rs`, `tests/partial_interval.rs`, and `tests/support/progress_admission.rs`: focused behavior evidence.
- `spec/functional/FR-004-publish-observation-authority-artifacts.md`: pre-release in-place contract rule and post-release immutability wording.

## Finding detail

### FND-001 — Bundle schema digest remains pinned

`AGENTS.md` says the repository's hash/digest/pin pattern is slated for removal and directs contributors to remove it when touching code or specs containing one. This PR modifies `src/authority/bundle.rs`, updates `SCHEMA_SHA256` for the embedded bundle schema, and retains `schema_digest_is_pinned` checks for the bundle, v2 bundle, and lineage schema. The schema bytes are already compiled into `SCHEMA_BYTES`; the constant and digest assertions are not needed to derive or strict-read the contract. Remove the digest constants and pin assertions in this touched module while keeping the schema-closure validation.

## New findings (disposition pass 1)

| ID | Severity | Summary | Refs | Escape Cause |
| --- | --- | --- | --- | --- |
| FND-002 | medium | The follow-up removes the lineage schema from the closure-validation test along with the digest pin, reducing schema-contract coverage. | src/authority/bundle.rs:1896-1900 | implementation-bug-despite-evidence |

### FND-002 — Lineage schema closure check removed

The digest-pin removal correctly replaces digest assertions with closure checks, but the follow-up commit also drops `LINEAGE_SCHEMA_BYTES` from `schemas_are_closed`. That schema remains a published contract and the prior test checked its closed envelope. Keep the lineage closure assertion while removing digest pins.

## Dispositions

| FND | outcome | reason |
| --- | --- | --- |
| FND-001 | fixed | The digest constants and `schema_digest_is_pinned` assertions are removed; the bundle and v2 schema closure checks remain. |
| FND-002 | fixed | The lineage schema is again checked for closed properties, its contract label, and the expected seven required fields. |
