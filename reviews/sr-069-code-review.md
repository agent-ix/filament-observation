---
id: SR-069
title: "Code review — filament-observation PR 61"
type: SpecReview
analysis: code-review
scope: "agent-ix/filament-observation; PR #61; Cargo.toml, Cargo.lock, schemas/*, src/authority/{admission,common,completeness,progress}.rs, tests/authority.rs; FR-013, FR-014, FR-004"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-observation/FR-013", type: references }
  - { target: "ix://agent-ix/filament-observation/FR-014", type: references }
  - { target: "ix://agent-ix/filament-observation/FR-004", type: references }
---

## Summary

Ticket: QSL-1867. Reviewed the frozen PR #61 cross-ticket diff against merged FR-013/014 and the FR-004 authority contract, including the Rust wire boundary, dependency migration, and active tests. The review found ignored acceptance-test stubs, a carry-forward loss when a successor population changes, and required checks that fail.

## Verdict

**FAIL** — the focused authority suite has three failures, Clippy with `-D warnings` fails, and multiple acceptance criteria have ignored-only tests.

## Scope

- FR-013-AC-1 through FR-013-AC-6 (examined): completeness boundary, basis/origin, monotonic revision behavior, lineage, strict reads, limits, and progress independence.
- FR-014-AC-1 through FR-014-AC-13 (examined): request/event admission, selection binding, error stages, resource bounds, runtime event/watermark handling, frontier ordering, and deterministic input.
- FR-004 completeness/progress authority readers and boundary contracts (context only).
- QSL-2374 dependency change to `filament-exact` `node_wire`, package pins, and removal of the FSL dependency (examined).

- `FR-013-AC-1` (examined): `derive` over a selection with a `TimestampedEvent` boundary emits `boundary` and `basis` `derived` with every fact `origin` `selected`, its view returns them through `boundary()`, `basis()` and `FactRef.origin`, and `read` admits it; a boundary violating FR-289's mapping refuses `InvalidSelection`.
- `FR-013-AC-2` (examined): With `prior` `Complete` through 100 over members stamped 0, 30, 60 and 90: a successor `Complete` through 100 and one through 140 over the same members plus one stamped 120 are accepted with the context's predecessor `prior`; through 60 refuses `Revision` `BoundaryRegressed` naming both boundaries; an `Incomplete` successor refuses `Revision` `CompleteRegressed`; a successor holding a member stamped 80 that `prior` does not hold refuses `Revision` `UncoveredMember` naming it; after each refusal `prior`'s bytes are unchanged.
- `FR-013-AC-3` (examined): A successor omitting the member stamped 30 carries it with `prior`'s observation and `origin` `carried`, and the revised state stays `Complete`; after an `Incomplete` `prior`, an `Incomplete` successor is accepted and a member `prior` held `Incomplete` that the successor supplies is accepted `Available`; a member `prior` held `Contradicted` stays `Contradicted` when the successor lists it `Available`; a member both hold `Available` under different observation identities refuses `Revision` `MemberObservationChanged`.
- `FR-013-AC-4` (examined): A context with no predecessor, with another document as predecessor, or with `prior`'s document under another authority refuses `Prior` `PredecessorMismatch`; a revision equal to `prior`'s refuses `Prior` `RevisionMismatch`; a successor over another admitted history (other scope and population identity) is accepted; a successor boundary of another clock family refuses `Selection` `ExpectedMismatch`; a successor selection that fails a `derive` check refuses `Selection` with `derive`'s error.
- `FR-013-AC-5` (examined): `read_revised` of `revise`'s bytes returns a view equal to the revised document; the same bytes with one byte changed, re-encoded non-canonically, or read against another `prior` refuse at `Document` with the strict reader's error; `read` of the revised bytes refuses `ExpectedMismatch` and `read_revised` of a derived document refuses at `Document`; revising twice from the same inputs gives equal bytes.
- `FR-013-AC-6` (examined): A revised population of one entry over `max_population_entries` refuses `Document` `ResourceIncomplete`; `revise` takes no progress view, and admitting progress through `admit_progress` over the same history before and after a revision returns equal capabilities.
- `FR-014-AC-1` (examined): A request carrying two candidate records in `records` and no event input admits; its view equals `progress::read` over the same qualified history and selections assembled through the Rust API; `selections()` returns the request's authority, subject, clock and observation source; `binds(view)` is true and `progress_identity()` is the document's identity.
- `FR-014-AC-2` (examined): The same request without `records`, with an event input holding the same two records as two lines (with and without a final LF), admits a view equal to AC-1's; a request with association rows admits a view whose `association_canonical_bytes()` equal those of AC-1's Rust-API view; a request with no record source admits the empty-binding-history view.
- `FR-014-AC-3` (examined): Each request defect of the error table, one at a time (one byte over `max_input_bytes`, invalid UTF-8, trailing data, another contract, an unknown, a duplicate and a missing member, `revision` `"01"`, an uppercase digest, `records` together with an event input) returns `stage` `Request` with exactly the listed code and no capability; a request of exactly `max_input_bytes` is not refused for its size.
- `FR-014-AC-4` (examined): An event input whose third line is empty, not JSON, names `filament.observation.record/v2`, or lacks `event_time_nanos` returns `Event { line: 3 }` with `InvalidJson`, `InvalidJson`, `ContractMismatch` or `InvalidSelection`; one record past `max_records` returns `ResourceIncomplete` at that record's line before it is retained.
- `FR-014-AC-5` (examined): A producer bundle of interface 1.1.0 returns `stage` `Producer`; a record with a foreign signal returns `Qualification` with `admit`'s `Refused` outcome unchanged, and a missing required valuation `Qualification` with its `Incomplete` outcome unchanged.
- `FR-014-AC-6` (examined): A progress document with one byte changed, re-encoded non-canonically, derived at another revision, derived for another scope, or derived from the history with one record dropped returns `stage` `Progress` with `cause` `Authority` equal to the error `progress::read` returns for the same bytes and selections, and no capability; an `observation_source` outside the required sources returns `Progress` `ExpectedMismatch`.
- `FR-014-AC-7` (examined): `AdmittedProgress` and `AuthoritySelections` have no public constructor, field, `Default`, `Serialize` or `Deserialize` (a crate outside this one cannot build either); `binds` of admission A's selections over admission B's view of a different document is false; every accessor returns a borrow without allocating.
- `FR-014-AC-8` (examined): The entry reads no file, standard input, environment or clock: admitting the same request and event bytes twice, in different working directories and environments, returns equal capabilities.
- `FR-014-AC-9` (examined): An event input of two runtime event lines followed by one runtime watermark line, whose records are AC-1's two records, admits a view equal to AC-1's view and three steps in line order: two `AdmittedEvent`s whose `line`, `record`, `reading` and exact `inputs` bytes are the lines', and one `AdmittedWatermark` whose `view` equals `progress::read` of its document over both records with the line's boundary, state and revision and AC-1's document as predecessor, whose `assertion` is the document bytes and whose `inclusive_boundary` is the line's.
- `FR-014-AC-10` (examined): An input mixing a record-candidate line, a runtime event line, a runtime watermark line and a later runtime event above the new frontier admits, with the record-candidate line contributing a record and no step; an input of record-candidate lines only admits a capability equal to AC-2's with `steps()` empty.
- `FR-014-AC-11` (examined): After a watermark line with frontier `f`: a later record of either kind anchored at `f`, one anchored below `f`, and one anchored in another clock family each return `Event { line }` of that record with `ExpectedMismatch`; a later watermark with `inclusive_boundary` below `f` returns `ExpectedMismatch` at its line, while one equal to `f` (with a greater revision) admits; a watermark whose `inclusive_boundary` differs from its `boundary`'s inclusive upper value returns `ExpectedMismatch` at its line. Each returns no capability.
- `FR-014-AC-12` (examined): A watermark whose `progress_document` has one byte changed, was derived at the predecessor's revision, names another predecessor, or was derived from the history without the preceding event's record returns `Event { line }` of the watermark with `cause` `Authority` equal to the error `progress::read` returns for the same bytes and context, and no capability.
- `FR-014-AC-13` (examined): On the third line, a runtime event whose `record` names `filament.observation.record/v2`, whose `reading` is `"18446744073709551616"` or `"01"`, or whose `inputs` is a JSON object, and a runtime watermark lacking `progress_document` or with `inclusive_boundary` `"+5"`, each return `Event { line: 3 }` with `ContractMismatch`, `InvalidSelection`, `InvalidSelection`, `InvalidSelection`, `InvalidSelection` or `InvalidSelection` respectively; a line naming `filament.observation.rt-event/v2` returns `ContractMismatch`.
## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | FR-013-AC-4 and FR-013-AC-6, and FR-014-AC-1, -2, -4, -5, -6, -7, -8, -10 and -12 are backed only by ignored tests whose bodies panic with `MISSING`. The newly implemented public behavior therefore has no active behavioral test for these criteria. Replace the stubs with executable cases and remove their ignores. | tests/completeness_revision.rs:16-79; tests/progress_admission.rs:14-154 |
| FND-002 | medium | `merge_facts` carries a prior fact only when its member remains in the successor population. If the successor history has a different population that omits a member held by `prior`, that retained fact is dropped, contrary to the rule that every prior-held member omitted by the selection is carried forward. Carry the omitted prior fact independently of successor membership. | src/authority/completeness.rs:832-842 |
| FND-003 | medium | The repository-required Clippy gate fails on the new admission/revision APIs: `result_large_err` is denied for the 136-byte `AdmissionError` and 176-byte `RevisionError`; `let producer = producer` also triggers `redundant_locals`. Resolve the diagnostics so `cargo clippy --locked --all-targets -- -D warnings` passes. | src/authority/admission.rs:266,341; src/authority/completeness.rs:541,771 |
| FND-004 | medium | `cargo deny check` rejects the new `filament-exact` tag dependency: its AGPL license has no exception and the Git source is not allowed. | Cargo.toml:25; deny.toml |
| FND-005 | medium | `cargo fmt --check` reports non-rustfmt import ordering in an edited test file. | tests/activation_authority.rs:10-14 |

## Verification

- `cargo test --locked --test authority --test progress_admission`: 68 passed, 3 failed in `authority`; the process stopped before `progress_admission`.
- `cargo test --locked --test progress_admission`: 1 passed, 12 ignored.
- The three `authority` failures were `tc009_bundle_schema_accepts_emitted_typed_wire_and_rejects_role_mismatches`, `tc009_v2_round_trips_only_the_exact_structurally_empty_profile`, and `tc011_each_query_limit_admits_exact_and_refuses_one_over`.
- `cargo fmt --check`: failed with import ordering changes required in `tests/activation_authority.rs`.
- `cargo clippy --locked --all-targets -- -D warnings`: failed with 15 errors, including the diagnostics above.
- `RUSTDOCFLAGS="-D warnings" cargo doc --locked --no-deps`: passed.
- `cargo deny check`: failed because the new `filament-exact` Git source is not allowed and its AGPL license has no exception in `deny.toml`.
