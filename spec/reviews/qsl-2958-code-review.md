---
id: SR-078
title: "Code review of QSL-2958 semantic-value pin in filament-observation"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-observation; Cargo.toml, Cargo.lock"
review_set: all
---
## Summary

This review examined the pin-only QSL-2958 change in `Cargo.toml` and
`Cargo.lock`. The manifest and lockfile select the same merged semantic-value
revision, and Cargo resolves one semantic-value package for this repository.
Formatting, Clippy, the targeted check, the filtered admission test, and docs
passed. The repository's `cargo deny check` reports the same newly pinned
semantic-value dependency as disallowed by existing license and git-source
rules; the base revision already depended on that same crate/source.

## Verdict

**PASS** — no defect was found in this pin-only diff. The existing deny policy
failure is not introduced by the revision change.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Gate Results

- `cargo fmt --check` — passed.
- `cargo clippy --locked --all-targets -j 4 -- -D warnings` — passed.
- `cargo check --locked --lib --test admission -j 4` — passed.
- `cargo test --locked --test admission tc005_relationship_extent_selections_are_unique -j 4` — passed: 1 passed, 20 filtered out.
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps -j 4` — passed.
- `cargo deny check` — failed: the existing policy rejects `AGPL-3.0-or-later` for `filament-semantic-value` and does not allow its Git source. The base lockfile already contains semantic-value 0.1.5 from the same Git URL at revision `4274b4e`; this PR changes only its revision.
- GitHub checks — CLA and gate passed; CLA check skipped.
