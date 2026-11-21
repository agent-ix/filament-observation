---
id: SR-077
title: "Code review of observation WorkScope owner pins"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-observation; Cargo.toml, Cargo.lock; QSL-2158"
review_set: subset
---

## Summary

Ticket QSL-2158, PR74. Independent native Codex review of frozen head 3456708ed17acd2acbc4a6e76155c42da155103b against 7cd5d994c01c5c22da8210a02c89d260b8298908. Only the exact and semantic-value dependency revisions change, with matching lock sources.

## Verdict

**PASS** — no findings in this mechanical dependency change. Both direct and transitive exact references resolve to the accepted owner revision; one exact 0.3.0 record and one semantic-value 0.1.5 record remain in the lock. Existing json and all other feature choices remain unchanged.

## Scope

Code-review with the Rust configuration lane and rust-style baseline applies. AGENTS.md, CLAUDE.md, README.md, Cargo.toml lints, deny.toml and rust-toolchain.toml were read as context; no repository rustfmt.toml or clippy.toml exists. No applicable AssuranceProfile was found in spec/. No Rust source, tests, public API, wire format, CI or gate configuration changes occur. No copied implementation, duplicate ledger, compatibility shim or vendored artifact is introduced. The explicitly authorized owner pin chain governs this revision update.

```yaml
{
  "clean": true,
  "scope": [
    {
      "id": "manifest-filament-exact",
      "path": "Cargo.toml",
      "role": "examined",
      "excerpt": "filament-exact = { git = \"https://github.com/agent-ix/filament-exact\", rev = \"a58474cc105a86f3307a11def3fc001e2459679f\", features = [\"json\"] }"
    },
    {
      "id": "lock-filament-exact",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "name = \"filament-exact\"\nversion = \"0.3.0\"\nsource = \"git+https://github.com/agent-ix/filament-exact?rev=a58474cc105a86f3307a11def3fc001e2459679f#a58474cc105a86f3307a11def3fc001e2459679f\""
    },
    {
      "id": "manifest-filament-semantic-value",
      "path": "Cargo.toml",
      "role": "examined",
      "excerpt": "filament-semantic-value = { git = \"https://github.com/agent-ix/filament-semantic-value\", rev = \"31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\" }"
    },
    {
      "id": "lock-filament-semantic-value",
      "path": "Cargo.lock",
      "role": "examined",
      "excerpt": "name = \"filament-semantic-value\"\nversion = \"0.1.5\"\nsource = \"git+https://github.com/agent-ix/filament-semantic-value?rev=31d807ccb8ffd5a88beb0b1a570b80acb099f5e9#31d807ccb8ffd5a88beb0b1a570b80acb099f5e9\""
    }
  ],
  "findings": [],
  "bindings": []
}
```

## Dependency coherence

The GitHub owner PR APIs independently confirm exact #191 merged as a58474cc105a86f3307a11def3fc001e2459679f and semantic-value #122 merged as 31d807ccb8ffd5a88beb0b1a570b80acb099f5e9. Reading those accepted owner commits confirms exact package 0.3.0 and semantic-value package 0.1.5; accepted semantic-value directly pins that same exact revision. Exact's accepted change has no Cargo.toml or Cargo.lock delta. Semantic-value's accepted manifest/lock delta contains only that exact revision substitution.

A parsed-TOML check and byte comparison prove the complete observation Cargo.toml and Cargo.lock are exactly their base bytes after only the two authorized revision substitutions. All dependencies, package records, dependency edges, features, versions, unrelated pins and source bytes are otherwise unchanged. There is exactly one package record for each owner. The reviewed head descends from the merged QSL-2145 completeness-check base 7cd5d994c01c5c22da8210a02c89d260b8298908; its source and tests are untouched. git diff --check passes.

## Validation evidence

The lead supplies actual Titan execution at 3456708ed17acd2acbc4a6e76155c42da155103b: node_identity_bound 3 plus partial_interval 7 passed (10 total), and authority::completeness::tests::check_revision_ 5 passed; all have 0 failures and 0 ignored. cargo check --locked --all-targets -p filament-observation exited 0 with no runtime tests. These are supplied service results, not local reviewer executions. Local Cargo is prohibited; no local Cargo, broad tests, clippy, documentation build or cargo-deny was run. Focused service checks follow the user's policy, which supersedes the repository's broad pre-review command list. The lead retains the mandatory final rebased exact-head check before merge.

## Method applicability

Gap-analysis is N/A: no production source or acceptance criteria change, no newly promised implementation behavior and no new tests. Spec-review and its sub-analyses are N/A: spec/ and plan/ are unchanged. No full matrix campaign ran. This review accepts only pin coherence, does not certify cumulative FSL WorkScope integration, and does not close QSL-2158.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Tracker publication

Private QSL-2158 marked review comment is prepared locally. No Linear or ix-board reads, writes, dedup, status or label operations were attempted: root holds publication until its explicit release after the tracker budget permits. No findings or excerpts were posted to GitHub. Publication and readback remain pending.
