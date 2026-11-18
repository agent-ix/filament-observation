---
id: SR-070
title: "Gap analysis — filament-observation PR 61"
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/filament-observation; repository-wide spec, source, and test matrix; PR #61 changes to FR-013, FR-014 and dependency ownership"
review_set: subset
relationships:
  - { target: "ix://agent-ix/filament-observation/FR-013", type: references }
  - { target: "ix://agent-ix/filament-observation/FR-014", type: references }
---

## Summary

Repository-driven gap analysis for the frozen PR #61. The computed matrix reports untagged and ignored-only criteria, and the new code has two stale trace IDs; plan completion was not assessed.

## Verdict

**FAIL** — 18 criteria are untagged and 11 are backed only by ignored tests.

- `NFR-001-M-1` (examined): Retained records after one-over admission | 0 excess records | 0 excess records | Test (TC-001).
- `NFR-001-M-2` (examined): Retained members after one-over admission | 0 excess members | 0 excess members | Test (TC-001).
- `NFR-001-M-3` (examined): Retained relationships after one-over admission | 0 excess relationships | 0 excess relationships | Test (TC-001).
- `NFR-001-M-4` (examined): Retained bytes/depth/strings after one-over strict read | 0 excess work | 0 excess work | Test (TC-004).
- `NFR-001-M-5` (examined): Retained semantic population after one-over derivation | 0 excess entries | 0 excess entries | Test (TC-004).
- `NFR-001-M-6` (examined): Partial view returned after resource refusal | 0 views | 0 views | Test (TC-004).
- `NFR-002-M-1` (examined): Repeated owner-artifact byte equality | 100% of fixed fixtures | 100% of fixed fixtures | Test (TC-002).
- `NFR-002-M-2` (examined): Validated-view authority/subject retention | 100% of strict reads | 100% of strict reads | Test (TC-003).
- `NFR-003-M-1` (examined): Equal-input canonical byte equality | 100% of generated fixtures | 100% | property-based-testing (TC-012).
- `NFR-003-M-2` (examined): Retained items after one-over bound | 0 excess items | 0 | property-based-testing (TC-012).
- `NFR-003-M-3` (examined): Partial artifact/view after refusal | 0 artifacts or views | 0 | unit-testing (TC-012).
- `NFR-003-M-4` (examined): Unaffected result identity/byte changes after repair | 0 changes | 0 changes | property-based-testing (TC-010).
- `NFR-003-M-5` (examined): Semantic outcomes changed by input permutation or arrival order | 0 changes | 0 changes | metamorphic-testing (TC-012).
- `FR-001-AC-1` (examined): A valid record retains its exact selections and visibility.
- `FR-001-AC-2` (examined): Wrong entity, signal, unit, trigger, schema, or producer selection is refused.
- `FR-001-AC-3` (examined): Missing required valuation, relationship, membership, or closure remains incomplete.
- `FR-001-AC-4` (examined): Conflicting or ambiguous relationships and one-over resource inputs are refused.
- `FR-001-AC-5` (examined): A member object identity is compared only as an exact opaque key and is never parsed, normalized, or interpreted.
- `FR-001-AC-6` (examined): Explicit-members, observation, and population identities are independently derived and stale caller-authored identities are refused.
- `FR-001-AC-7` (examined): Selected producer and closure definition digests are retained in their declared domains without interpreting definition bytes.
- `FR-002-AC-1` (examined): Batch and incremental paths derive byte-identical observation authority artifacts for the same admitted history and selections.
- `FR-002-AC-2` (examined): Checked incremental intake accepts only the exact next qualified record and refuses early finish, excess input, or out-of-order substitution without partial history.
- `FR-002-AC-3` (examined): Progress, closure, completeness, availability, lateness, and resource exhaustion remain independent typed facts or errors and never become Boolean or protocol/temporal results.
- `FR-002-AC-4` (examined): Two records sharing one declared order position report an ambiguous-order error without timestamp inference.
- `FR-003-AC-1` (examined): Every owner artifact remains independently readable through complete read-only payload accessors and no view exposes a truth, settlement or Boolean field.
- `FR-003-AC-2` (examined): A strict read preserves all required owner selections and rejects omission, duplication or scope cross-wiring.
- `FR-003-AC-3` (examined): An unsupported document or owner-contract representation returns a typed refusal and cannot promote loss to availability, completeness or success.
- `FR-004-AC-1` (examined): Each of the nine exact contracts publishes immutable schema bytes, deterministic canonical bytes/identity and a public strict reader returning only its constructor-private view.
- `FR-004-AC-2` (examined): Batch and incremental derivation produce byte-identical documents for equal qualified history and selections; every semantic mutation changes only the applicable identity.
- `FR-004-AC-3` (examined): Every required field is independently omitted, duplicated, reordered and cross-wired; unknown fields/labels/contracts, trailing/noncanonical bytes and same identity on unequal bytes refuse without partial authority.
- `FR-004-AC-4` (examined): Exact and one-over byte/depth/string/population/position/capture/source/work bounds admit or resource-refuse before excess retention.
- `FR-004-AC-5` (examined): All combinations of independent progress, closure, completeness and availability values remain representable; no combination yields a truth, settlement or Boolean value.
- `FR-004-AC-6` (examined): Corrections preserve predecessor bytes/direct lineage; late data, restoration and timestamp ordering cannot rewrite authority or manufacture origin/causality.
- `FR-004-AC-7` (examined): A progress view exposes owner asserted FSL association rows as a canonical value when present. Each row names the selected model, structural model node, typed claim clause node and occurrence, time source kind, tick operation and exact positive period when applicable, structural unit and dimension nodes, and an observation source and clock/revision admitted in the same progress context. Missing rows refuse with `MissingPremise`; foreign source/clock refuse with `ExpectedMismatch`; malformed or duplicate rows refuse with `InvalidSelection`, noncanonical structural node text with `NonCanonical`, and one-over node or value bytes with `ResourceIncomplete`; the association value is encoded once into a sink bounded by the owner output byte ceiling and never retains bytes past it. The document ceiling always covers the value and is what strict reading enforces first; the value-encoder boundary (exact ceiling admitted, one byte over refused) is verified at the encoder. Strict reading compares the exact selected rows. The observation owner asserts these links; FSL checks its own semantic selections independently against the rows.
- `FR-005-AC-1` (examined): Admission consumes and retains this library's own `AdmittedStaticBundle`; no local producer-selection shadow remains.
- `FR-005-AC-2` (examined): Independently mutating authority identity, revision namespace/value, digest label/value, kind, or object bytes changes the qualified-subject key; missing/empty runtime members refuse.
- `FR-005-AC-3` (examined): Semantic comparison across authority or kind refuses, while presentation, trace, timestamp, arrival and transport changes cannot affect a subject key.
- `FR-005-AC-4` (examined): A relationship resolves only against its exact declared relationship and authored endpoint order; unknown, reversed, wrong-kind and foreign endpoints refuse.
- `FR-005-AC-5` (examined): Exact relationship replay is idempotent; same-identity incompatible rebinding refuses; multiple distinct values for one slot are ambiguous.
- `FR-005-AC-6` (examined): Missing, conflicting and ambiguous are distinct non-success outcomes, and no heuristic input creates a relationship.
- `FR-005-AC-7` (examined): Collection permutation produces one lexicographic key order without creating causal, delivery, retry or timestamp semantics.
- `FR-005-AC-8` (examined): Exact and one-over relationship limits preserve the existing fail-closed resource behavior.
- `FR-005-AC-9` (examined): The two qualified owner schemas publish as v2 while both v1 schema files remain byte-identical to the pre-#18 baseline.
- `FR-006-AC-1` (examined): Independently valid self-loop, two-vertex cycle and disconnected relationship component are retained exactly; topology alone neither refuses nor removes them.
- `FR-006-AC-2` (examined): No graph order, replay, timestamp or arrival mutation creates an edge, reachability result, transitive closure or causal verdict.
- `FR-006-AC-3` (examined): Exact relationship ceilings admit and one-over inputs refuse before topology work; exact replay still counts once only after the offered-population check.
- `FR-006-AC-4` (examined): Timestamp `[a,b]` maps to the exact admitted `[a,e)` with `e > b`; records at `a` and `b` admit, a record at `e` refuses, and a carrier record between `b` and `e` gains no participation verdict.
- `FR-006-AC-5` (examined): Event-position and fixed-sample intervals require checked `b+1`; a reversed interval, overflow, wrong start/end or wrong clock family refuses.
- `FR-006-AC-6` (examined): Exact clock identity/revision and scope identity are retained; independently cross-wiring any one prevents progress/closure derivation.
- `FR-006-AC-7` (examined): A point interval admits with a valid greater carrier end; watermark equality cannot close it, while a strictly greater watermark can.
- `FR-006-AC-8` (examined): The prerelease progress v1 contract includes FR-004's owner-issued association because no external progress v1 consumer exists; its exact FR-289 boundary mapping remains intact, closure v1 retains its boundary mapping, and neither publishes a traversal, replay, settlement or truth API.
- `FR-006-AC-9` (examined): Association model, clause, tick-operation, unit and dimension nodes round-trip as real structural NodeKeys with canonical package/path/ordinal text; the claim retains its exact role and ordinal. Empty package/path, malformed segment or out-of-range ordinal refuses with `InvalidSelection`, an alternate spelling with `NonCanonical`, and one-under node-byte bounds with `ResourceIncomplete`; an exact byte bound admits.
- `FR-007-AC-1` (examined): Known true, known false, missing, and conflicting inputs retain their exact possibility sets and distinct reasons.
- `FR-007-AC-2` (examined): Empty or reason-inconsistent possibility sets refuse without an authority fact.
- `FR-007-AC-3` (examined): Disjoint intervals establish only the order required by their endpoints; overlapping intervals retain every admissible order.
- `FR-007-AC-4` (examined): Reversing an interval or cross-wiring its clock, revision, or unit refuses before comparison.
- `FR-007-AC-5` (examined): Mutating ingestion order or an interval midpoint cannot change the semantic order relation.
- `FR-008-AC-1` (examined): Equal captured values under distinct triggers create distinct activation identities and retain their original observation provenance.
- `FR-008-AC-2` (examined): An admitted trigger with complete strict capture authority is active; a closed complete optional binding whose bounded qualified history is empty for its exact selected trigger identity is inactive; every open or incomplete optional no-trigger scope is activation-unknown. A required empty binding, foreign binding/trigger, or claimed absence conflicting with matching admitted history refuses. All three valid classifications remain independent of verdict and settlement.
- `FR-008-AC-3` (examined): Silence covers a deadline only when the matching progress interval is definitely beyond it and covers every required source.
- `FR-008-AC-4` (examined): Overlapping progress/deadline intervals, missing sources, open closure, and incomplete or contradicted evidence remain distinct non-conclusive facts.
- `FR-008-AC-5` (examined): Definitely timely, definitely late, and uncertain-lateness outcomes follow interval endpoints and never ingestion order.
- `FR-008-AC-6` (examined): Any foreign or cross-wired scope authority refuses without changing an unaffected axis or emitting a fallback result.
- `FR-009-AC-1` (examined): Repeated derivation from the same complete inputs produces byte-identical bundle bytes and identity.
- `FR-009-AC-2` (examined): A valid later revision names its exact predecessor and replacements while prior bytes remain unchanged.
- `FR-009-AC-3` (examined): Missing, stale, cross-authority, cross-role, self-referential, or already-branched supplied lineage refuses without a bundle.
- `FR-009-AC-4` (examined): Exact replay is idempotent; reusing one authority/scope/revision key for unequal bytes is an identity contradiction.
- `FR-009-AC-5` (examined): Omitting, duplicating, cross-wiring, or exceeding a component bound refuses before a validated view exists.
- `FR-009-AC-6` (examined): A complete empty population publishes and strict-reads only as the exact seven-role v2 profile; v1 bytes and schema digest remain unchanged, while a v1 empty profile, a nonempty v2 profile, a sentinel record, an added/omitted role, or contract substitution refuses before a validated view exists.
- `FR-010-AC-1` (examined): A replacement invalidates every and only explicit dependent results whose scopes/windows can observe it.
- `FR-010-AC-2` (examined): Unaffected results retain byte-identical identities and bytes across repair.
- `FR-010-AC-3` (examined): Batch replay and incremental repair agree exactly on settled result axes, decision-support identities, and old/new lineage for identical inputs.
- `FR-010-AC-4` (examined): A decisive prefix is emitted before end-of-input, while an unresolved prefix remains non-settled until a decisive witness, decisive counterexample, or matching closure exists.
- `FR-010-AC-5` (examined): Out-of-order arrival and overlapping event-time intervals never become a total semantic order; every admissible order reaches the evaluator.
- `FR-010-AC-6` (examined): Cycles, unknown dependency identities, foreign revisions, and one-over work/state bounds return their typed outcomes without a partial repair plan or promoted result.
- `FR-010-AC-7` (examined): The public repair-plan contract constant and every emitted canonical repair plan name exactly `filament.observation.repair-plan/v1`; the label is immutable within v1.
- `FR-011-AC-1` (examined): A closed selected population includes every and only admitted member once according to its explicit duplicate policy and in deterministic exposure order.
- `FR-011-AC-2` (examined): A member at the half-open window start participates and one at the end does not; adjacent windows never double-count a boundary member.
- `FR-011-AC-3` (examined): Two related partial refunds sum exactly to the captured amount only when both effects, their relationships, and the complete population are authority-qualified.
- `FR-011-AC-4` (examined): Replayed receipts do not create effects, while distinct admitted effects and occurrence-preserving duplicate values remain distinguishable.
- `FR-011-AC-5` (examined): Open, unknown, stale, ambiguous, contradicted, incomplete, foreign, or over-bound authority produces no partial definitive aggregate.
- `FR-011-AC-6` (examined): Exact sum rejects wrong representation/unit, overflow, and an invalid intermediate prefix without reordering operands.
- `FR-011-AC-7` (examined): The public query contract constant and every emitted canonical query evaluation name exactly `filament.observation.closed-population-query/v1`; the label is immutable within v1.
- `FR-012-AC-1` (examined): With `test-support` enabled, the exported bytes equal the owned fixture file and are admitted by `StaticProducerBundle::admit_json`.
- `FR-012-AC-2` (examined): `test-support` is absent from the default feature set, so the default public API has no `test_support` module.
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
- `NFR-001-AC-1` (examined): Admission retains no record beyond its declared record bound.
- `NFR-001-AC-2` (examined): Admission retains no member beyond its declared member bound.
- `NFR-001-AC-3` (examined): Exact byte, depth, string and visited-work limits admit; one-over inputs resource-refuse before a view exists.
- `NFR-001-AC-4` (examined): Exact population, position, capture and source limits admit; one-over selections resource-refuse before a document exists.
- `NFR-001-AC-5` (examined): Incremental history cannot retain an out-of-order, excess or incomplete record sequence.
- `NFR-001-AC-6` (examined): Exact relationship and required-relationship limits admit; one-over inputs refuse before relationship matching.
- `NFR-002-AC-1` (examined): Repeated batch and incremental derivation of the same ordered input returns byte-identical documents and identities.
- `NFR-002-AC-2` (examined): Every validated view retains the exact authority, subject, revision and predecessor selections from its admitted document.
- `NFR-003-AC-1` (examined): The implementation uses no unbounded expansion of overlapping interval orders or dependency closure.
- `NFR-003-AC-2` (examined): Every one-over resource case fails before retaining a partial new bundle, repair plan, aggregate, or validated view.
## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | No test carries this criterion's trace ID. | NFR-001-AC-3 |
| FND-002 | high | No test carries this criterion's trace ID. | NFR-001-AC-4 |
| FND-003 | high | No test carries this criterion's trace ID. | NFR-001-AC-5 |
| FND-004 | high | No test carries this criterion's trace ID. | NFR-001-M-1 |
| FND-005 | high | No test carries this criterion's trace ID. | NFR-001-M-2 |
| FND-006 | high | No test carries this criterion's trace ID. | NFR-001-M-3 |
| FND-007 | high | No test carries this criterion's trace ID. | NFR-001-M-4 |
| FND-008 | high | No test carries this criterion's trace ID. | NFR-001-M-5 |
| FND-009 | high | No test carries this criterion's trace ID. | NFR-001-M-6 |
| FND-010 | high | No test carries this criterion's trace ID. | NFR-002-AC-1 |
| FND-011 | high | No test carries this criterion's trace ID. | NFR-002-AC-2 |
| FND-012 | high | No test carries this criterion's trace ID. | NFR-002-M-1 |
| FND-013 | high | No test carries this criterion's trace ID. | NFR-002-M-2 |
| FND-014 | high | No test carries this criterion's trace ID. | NFR-003-M-1 |
| FND-015 | high | No test carries this criterion's trace ID. | NFR-003-M-2 |
| FND-016 | high | No test carries this criterion's trace ID. | NFR-003-M-3 |
| FND-017 | high | No test carries this criterion's trace ID. | NFR-003-M-4 |
| FND-018 | high | No test carries this criterion's trace ID. | NFR-003-M-5 |
| FND-019 | high | Every test carrying this criterion is ignored. | FR-013-AC-4; tests/completeness_revision.rs::lineage_and_selection_defects_refuse_at_their_stage |
| FND-020 | high | Every test carrying this criterion is ignored. | FR-013-AC-6; tests/completeness_revision.rs::revision_is_bounded_and_independent_of_progress_admission |
| FND-021 | high | Every test carrying this criterion is ignored. | FR-014-AC-1; tests/progress_admission.rs::a_request_with_records_admits_the_rust_api_view_and_its_selections |
| FND-022 | high | Every test carrying this criterion is ignored. | FR-014-AC-2; tests/progress_admission.rs::event_input_records_admit_the_same_view |
| FND-023 | high | Every test carrying this criterion is ignored. | FR-014-AC-4; tests/progress_admission.rs::each_event_line_defect_refuses_at_its_line |
| FND-024 | high | Every test carrying this criterion is ignored. | FR-014-AC-5; tests/progress_admission.rs::producer_and_qualification_outcomes_are_carried_unchanged |
| FND-025 | high | Every test carrying this criterion is ignored. | FR-014-AC-6; tests/progress_admission.rs::a_progress_document_not_derived_from_the_inputs_refuses |
| FND-026 | high | Every test carrying this criterion is ignored. | FR-014-AC-7; tests/progress_admission.rs::the_capability_is_opaque_and_bound_to_its_document |
| FND-027 | high | Every test carrying this criterion is ignored. | FR-014-AC-8; tests/progress_admission.rs::admission_reads_only_the_supplied_bytes |
| FND-028 | high | Every test carrying this criterion is ignored. | FR-014-AC-10; tests/progress_admission.rs::mixed_line_kinds_admit_and_record_lines_alone_admit_as_before |
| FND-029 | high | Every test carrying this criterion is ignored. | FR-014-AC-12; tests/progress_admission.rs::a_watermark_document_not_derived_from_the_prefix_refuses_at_its_line |
| FND-030 | medium | Test functions carry a trace ID absent from the computed specification matrix. | FR-013-AC-7; tests/authority.rs::completeness_revision_carries_omitted_members_and_preserves_fact_strength; tests/authority.rs::completeness_revision_refuses_omitted_members_not_held_by_prior |
| FND-031 | medium | Test functions carry a trace ID absent from the computed specification matrix. | IT-001; tests/admission.rs::tc001_missing_required_value_is_incomplete_not_false; tests/authority.rs::tc002_duplicate_declared_position_is_ambiguous_not_unsupported; tests/authority.rs::tc004_batch_and_incremental_history_are_byte_identical; tests/authority.rs::tc004_boundary_mapping_is_exact_and_late_classification_is_owner_derived; tests/authority.rs::tc004_each_public_reader_requires_exact_independent_selections; tests/authority.rs::tc004_state_axes_remain_independent_and_non_boolean |

## Coverage

- Reconciliation: `quoin matrix --repo . --json` (quoin 0.36.3); `quire coverage --scope . --json`.
- Plan completion: not assessed
- Criteria: tagged 84 · untagged 18 · tagged-by-ignored-test 11 · method-without-symbol 1.
- Evidence: no run evidence for all 114 criteria; no evidence-status distinction is asserted as a behavior gap.
- Untraced behaviors: 0 in the reviewed API surface; new admission and completeness behavior map to FR-014 and FR-013. Ignored test stubs: 18 test functions across `tests/progress_admission.rs` and `tests/completeness_revision.rs`.
- Semantic review: skipped.

## Dispositions

| FND | Outcome | Reason |
| --- | --- | --- |
| FND-001 | deferred | QSL-2382 owns the remaining NFR-001-AC-3 trace coverage. |
| FND-002 | deferred | QSL-2382 owns the remaining NFR-001-AC-4 trace coverage. |
| FND-003 | deferred | QSL-2382 owns the remaining NFR-001-AC-5 trace coverage. |
| FND-004 | deferred | QSL-2382 owns the remaining NFR-001-M-1 trace coverage. |
| FND-005 | deferred | QSL-2382 owns the remaining NFR-001-M-2 trace coverage. |
| FND-006 | deferred | QSL-2382 owns the remaining NFR-001-M-3 trace coverage. |
| FND-007 | deferred | QSL-2382 owns the remaining NFR-001-M-4 trace coverage. |
| FND-008 | deferred | QSL-2382 owns the remaining NFR-001-M-5 trace coverage. |
| FND-009 | deferred | QSL-2382 owns the remaining NFR-001-M-6 trace coverage. |
| FND-010 | deferred | QSL-2382 owns the remaining NFR-002-AC-1 trace coverage. |
| FND-011 | deferred | QSL-2382 owns the remaining NFR-002-AC-2 trace coverage. |
| FND-012 | deferred | QSL-2382 owns the remaining NFR-002-M-1 trace coverage. |
| FND-013 | deferred | QSL-2382 owns the remaining NFR-002-M-2 trace coverage. |
| FND-014 | deferred | QSL-2382 owns the remaining NFR-003-M-1 trace coverage. |
| FND-015 | deferred | QSL-2382 owns the remaining NFR-003-M-2 trace coverage. |
| FND-016 | deferred | QSL-2382 owns the remaining NFR-003-M-3 trace coverage. |
| FND-017 | deferred | QSL-2382 owns the remaining NFR-003-M-4 trace coverage. |
| FND-018 | deferred | QSL-2382 owns the remaining NFR-003-M-5 trace coverage. |
| FND-019 | fixed | `#[trace("TC-015", "FR-013-AC-4")]` is now on an active executable revision test. |
| FND-020 | fixed | `#[trace("TC-015", "FR-013-AC-6")]` is now on an active executable revision test. |
| FND-021 | fixed | `#[trace("TC-016", "FR-014-AC-1")]` is now on an active executable admission test. |
| FND-022 | fixed | `#[trace("TC-016", "FR-014-AC-2")]` is now on an active executable admission test. |
| FND-023 | fixed | `#[trace("TC-016", "FR-014-AC-4")]` is now on an active executable admission test. |
| FND-024 | fixed | `#[trace("TC-016", "FR-014-AC-5")]` is now on an active executable admission test. |
| FND-025 | fixed | `#[trace("TC-016", "FR-014-AC-6")]` is now on an active executable admission test. |
| FND-026 | fixed | `#[trace("TC-016", "FR-014-AC-7")]` is now on an active executable admission test. |
| FND-027 | fixed | `#[trace("TC-016", "FR-014-AC-8")]` is now on an active executable admission test. |
| FND-028 | fixed | `#[trace("TC-016", "FR-014-AC-10")]` covers the revised three-record full-history input. |
| FND-029 | fixed | `#[trace("TC-016", "FR-014-AC-12")]` is now on an active executable watermark test; missing AC-12 variants are tracked as SR-069 FND-006. |
| FND-030 | fixed | `#[trace("TC-015", "FR-013-AC-7")]` now refers to the merged FR-013-AC-7 matrix row. |
| FND-031 | deferred | QSL-2382 owns cleanup of the six stale IT-001 trace IDs. |

### Current matrix snapshot

- Recomputed at reviewed head `160ebab778eb299383189db59fccf12834bcf97c`: 96 tagged, 18 untagged, 0 tagged-by-ignored-test, 1 method-without-symbol (115 criteria total).
- FR-013-AC-7 and FR-014-AC-10, AC-12, and AC-14 are tagged by active tests. AC-10 covers the full qualified history and all three AC-14 qualification outcomes.
- QSL-2382 owns the 18 remaining untagged NFR criteria and six stale IT-001 trace IDs. Quoin's matrix reports no test bound only by ignored tests.
