---
id: SR-076
title: "Code review — filament-observation dependency graph for QSL-2540 step 1"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-observation; Cargo.toml, Cargo.lock, src/, tests/; QSL-2540 step 1"
review_set: subset
---

## Summary

Reviewed PR #67 at `a4fcebc716c61142fbdb75f9a653a91a2070314f` for QSL-2540 step 1. The diff removes the unused direct `filament-contract-ir` dependency and its now-unreachable lockfile graph; source and test search found no use, and the locked package tree no longer includes the crate.

## Verdict

**PASS** — no defect found in the reviewed dependency removal. The focused `cargo check -p filament-observation --all-targets` passed per the coder's report; I independently inspected the PR diff, searched source/tests, and ran `cargo tree --locked --package filament-observation --edges normal,dev,build`, which showed the resulting graph without `filament-contract-ir`. No aggregate workspace gate was run; this PR changes only dependency metadata and the assigned lane requires targeted checks.

## Review method

- Rust review lane was applied to the Cargo manifest and lockfile change. No Rust source, tests, CI workflows, specs, or plans changed.
- Dependency graph check: `cargo tree --locked --package filament-observation --edges normal,dev,build` completed successfully and contained no `filament-contract-ir` node.
- Search: repository source and test directories contain no `filament-contract-ir` references. Other matches are contextual spec/docs material and review history, not compile-time uses.
- The production-code gap-analysis method did not apply because no production source changed. The ticket's later eval/checked and FSL steps were not part of this PR; FSL remains arch-owned.
- `cargo fmt`, Clippy, crate tests, and `cargo deny` were not rerun: there are no Rust source or test changes, and the bounded lane uses targeted checks rather than an aggregate gate. The focused all-target check was reported green by the coder.

## Scope examined

- QSL-2540 step 1: “1. filament-observation: bump contract-ir to latest contract-ir main; merge, announce rev.” I checked whether observation actually consumes the direct crate before assessing the dependency change.
- `Cargo.toml`: the current `[dependencies]` list retains `filament-exact`, `filament-semantic-value`, and the other used dependencies, with no direct `filament-contract-ir` declaration.
- `Cargo.lock`: the `filament-observation` package stanza no longer lists `filament-contract-ir`; the removed package and transitive packages are absent from the locked resolved graph.
- `src/` and `tests/`: searched for source/test use of `filament-contract-ir`; no matches.
- PR diff and CI workflow scope: only `Cargo.toml` and `Cargo.lock` changed; no workflow gate was narrowed.

## Findings

| ID      | Severity | Summary                    | Refs |
| ------- | -------- | -------------------------- | ---- |
| FND-001 | low      | No findings (placeholder)  | -    |
