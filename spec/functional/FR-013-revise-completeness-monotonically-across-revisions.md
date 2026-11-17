---
id: FR-013
title: "Revise a completeness assertion monotonically across revisions"
type: FR
relationships:
  - { target: ix://agent-ix/filament-observation/US-001, type: implements }
  - { target: ix://agent-ix/filament-observation/FR-004, type: depends_on }
  - { target: ix://agent-ix/filament-observation/FR-008, type: depends_on }
  - { target: ix://agent-ix/filament-observation/FR-014, type: references }
  - { target: ix://agent-ix/filament-spec-language/FR-502, type: references }
  - { target: ix://agent-ix/filament-spec-language/FR-944, type: references }
---
# FR-013: Revise a completeness assertion monotonically across revisions

## Description

When a caller supplies a retained completeness view and a successor
selection, the library SHALL return exactly one successor completeness-assertion document whose
predecessor is the retained one, or exactly one typed revision error naming
its stage and its contradicted boundary or member. This
requirement is the normative source of the completeness revision rule: the
completeness boundary never moves back, a complete assessment never becomes
incomplete, a member at or before the retained boundary that the retained
assessment did not hold is never admitted later, and a member the retained
assessment held stays held.

## Rationale

`derive` (FR-004) assesses one admitted history on its own. A monitor that
receives completeness assertions as observation arrives also needs to know
whether a new assertion is consistent with the one it already retained:
"complete through 100" followed by "complete through 60" from a lagging
replica is a contradiction, not a second independent fact. The rule belongs to
the observation owner, so every consumer (the FSL evaluator's progress ledger
among them) gets one answer for one sequence of assertions instead of keeping
its own copy. The rule is monotone in what the owner has asserted: extent and
held evidence only grow, so a successor never retracts a member a consumer may
already have relied on, and a refused successor changes nothing.

## Interface

The module `authority::completeness` SHALL publish, in addition to FR-004's
`derive` and `read`:

| Item | Form |
| --- | --- |
| `Selection::new(boundary_identity: Identity, boundary: TemporalBoundary, facts: Vec<Fact>)` | the selection, now carrying its FR-289 boundary |
| `revise(prior: &View, context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document, RevisionError>` | the revision rule |
| `read_revised(bytes: &[u8], prior: &View, context: Context<'_>, selection: &Selection, limits: Limits) -> Result<View, RevisionError>` | the strict reader of a revised document |
| `Basis` | `Derived` (a `derive` document) or `Revised` (a `revise` document) |
| `FactOrigin` | `Selected` (the fact is the selection's) or `Carried` (the fact is carried from `prior`) |
| `RevisionError { stage: RevisionStage, cause: RevisionCause }` | the typed error |
| `RevisionStage` | `Prior`, `Selection`, `Revision`, `Document` |
| `RevisionCause` | `Authority(Error)` or `Contradiction(Contradiction)` |
| `Contradiction` | `BoundaryRegressed { prior: TemporalBoundary, successor: TemporalBoundary }`, `CompleteRegressed { prior: TemporalBoundary }`, `UncoveredMember { member_identity: Identity, prior: TemporalBoundary }`, `MemberObservationChanged { member_identity: Identity, prior_observation: Identity, successor_observation: Identity }` |

`Contradiction::code(&self) -> &'static str` SHALL return the stable labels
`QOBS-REV-BOUNDARY-REGRESSED`, `QOBS-REV-COMPLETE-REGRESSED`,
`QOBS-REV-UNCOVERED-MEMBER` and `QOBS-REV-MEMBER-OBSERVATION-CHANGED`.

`CompletenessView` SHALL add `boundary(&self) -> TemporalBoundary` and
`basis(&self) -> Basis`, and `FactRef` SHALL add `origin: FactOrigin`.

## Contract

The `filament.observation.completeness-assertion/v1` payload SHALL carry, in
addition to its population identity, boundary identity, facts and state:

- `boundary`: the selection's `TemporalBoundary` in its FR-004 wire form;
- `basis`: `derived` or `revised`;
- on each fact, `origin`: `selected` or `carried`.

`derive` SHALL emit `basis` `derived` with every fact `selected`, and is
otherwise unchanged. A completeness boundary SHALL satisfy FR-289's exact
mapping without the closed-scope watermark condition: completeness asserts
which members are held through a boundary and closes no scope (FR-008).

## Inputs

- `prior`: a validated completeness view, from `read` or `read_revised`. Only
  this crate's strict readers produce a `View`, so `prior` is always bytes the
  owner derived.
- `context`: the successor's `Context` over its own admitted history, naming
  `prior`'s document as predecessor, with a revision greater than `prior`'s.
- `selection`: the successor's boundary and per-member facts over the
  successor history's required population, as `derive` takes them.
- `limits`: the owner limits.

## Outputs

On acceptance, the revised document (`basis` `revised`). Its `boundary` is
the selection's. Its facts are the **revised population**: first the
selection's facts in the selection's order, each with the merged status below
and `origin` `selected`; then each fact `prior` holds for a member the
selection does not list, in `prior`'s order, with `prior`'s status and
observation identity and `origin` `carried`. Its `state` is FR-287's aggregate
over the revised population. On refusal, one `RevisionError` and no document.

## Behavior

### Ownership and authority

- A member is **held** by an assessment when the assessment has a fact for
  that member, whatever its status. A member is **placed** by its admitted
  record's anchor in the successor history: its event position, sample index
  or nanosecond instant.
- `revise` SHALL derive the successor only as a revision of `prior`: the
  context's predecessor SHALL be the document whose identity is `prior`'s,
  of the completeness contract and `prior`'s authority, and the context's
  revision SHALL be greater than `prior`'s.
- The successor's subject MAY differ from `prior`'s. Each revision is
  assessed over the admitted history its own request carries, so its scope
  and population identity are its own; its subject SHALL still match its own
  qualified history, as for every owner document.
- The successor's selection SHALL pass every check `derive` applies to it, and
  its boundary SHALL be of `prior`'s clock family.
- The revised document's authority is recomputation by this owner. No caller
  flag, signature or transport attribute marks a document revised. A revised
  document SHALL be authenticated only by `read_revised`, which recomputes
  `revise` from the same `prior`, context and selection and admits the bytes
  only when they are that result byte for byte. `read` SHALL refuse a revised
  document with `ExpectedMismatch`, because the document it recomputes has
  `basis` `derived`; `read_revised` SHALL refuse a derived document the same
  way.

### Monotonicity and ordering

Boundaries are ordered by their inclusive upper value (`upper_inclusive` or
`upper_inclusive_nanos`) within one clock family. A completeness boundary's
watermark is carried and not compared; watermark monotonicity is the progress
owner's rule.

`revise` SHALL check, in this order, and refuse at the first that holds:

1. **Boundary.** The selection's boundary is below `prior`'s:
   `BoundaryRegressed`. A boundary equal to or above `prior`'s is accepted.
2. **Complete to incomplete.** `prior`'s state is `Complete` and the state
   `derive` gives the selection's own facts is `Incomplete`:
   `CompleteRegressed`. An `Incomplete` successor of an `Incomplete` prior,
   and a `Complete` or `Contradicted` successor of any prior, pass this check.
3. **Uncovered member.** In the selection's order, a member the selection
   lists `Available` or `Contradicted`, placed at or before `prior`'s boundary,
   that `prior` does not hold: `UncoveredMember` naming the first such member.
   A member `prior` holds `Incomplete` and the selection supplies is accepted,
   and so is any member placed after `prior`'s boundary.
4. **Changed observation.** In the selection's order, a member both `prior`
   and the selection hold `Available` with different observation identities:
   `MemberObservationChanged` naming the first such member. A different record
   for one held member is a changed delivery, not a revision; an admitted
   correction of it is reported as `Contradicted`, which check 4 does not
   refuse.

On acceptance, each selected member's merged status SHALL be the greater of
its status in `prior` (when held) and in the selection, under the order
`Incomplete` < `Available` < `Contradicted`, with the observation identity of
the side whose status it takes (the selection's on a tie). A member `prior`
held that the selection omits SHALL be carried with `prior`'s fact. So an
available member is not lost by a later omission or a later `Incomplete`
listing, and a reported contradiction is never erased.

### Refusal leaves the prior unchanged

`prior` is borrowed immutably and is never altered. On any refusal the
retained document stays the caller's current head, and re-running the same
`revise` on the same inputs SHALL return the same error. On acceptance the
same inputs SHALL return the same bytes.

### Progress admission and the other axes

- `revise` SHALL read no progress, closure or activation authority and SHALL
  derive none (FR-008). A completeness boundary is not a progress frontier:
  revising completeness neither advances nor requires a progress watermark,
  and admitting progress through FR-014's `admit_progress`, including its
  runtime watermark lines, neither revises completeness nor is refused by a
  completeness revision. A caller that keeps both axes applies each owner rule
  to its own documents.
- Both entries authenticate the same way: the owner recomputes the document
  from the admitted history and the explicit selections and compares bytes.
  A revised completeness view is exactly the document this owner revises from
  these inputs, as an admitted progress view is exactly the document it
  derives from its inputs.

## Error Conditions

Every error is one `RevisionError` and no document. Owner errors are carried
unchanged.

| Condition | `stage` | `cause` |
| --- | --- | --- |
| the context names no predecessor, a predecessor whose identity is not `prior`'s, of another contract or of another authority | `Prior` | `Authority` `PredecessorMismatch` |
| the context's revision is not greater than `prior`'s | `Prior` | `Authority` `RevisionMismatch` |
| any refusal `derive` returns for the context and selection | `Selection` | `Authority` with that error unchanged |
| the boundary fails FR-289's mapping | `Selection` | `Authority` `InvalidSelection` |
| the boundary is of another clock family than `prior`'s | `Selection` | `Authority` `ExpectedMismatch` |
| a member is placed in another clock family than the boundary | `Selection` | `Authority` `ExpectedMismatch` |
| the selection's boundary is below `prior`'s | `Revision` | `Contradiction` `BoundaryRegressed` |
| `prior` is `Complete` and the selection's own state is `Incomplete` | `Revision` | `Contradiction` `CompleteRegressed` |
| an `Available` or `Contradicted` member at or before `prior`'s boundary that `prior` does not hold | `Revision` | `Contradiction` `UncoveredMember` |
| a member held `Available` by both with different observation identities | `Revision` | `Contradiction` `MemberObservationChanged` |
| the revised population, or the encoded document, exceeds an effective limit | `Document` | `Authority` `ResourceIncomplete` |
| `read_revised` bytes that are not the recomputed document (one byte changed, non-canonical, another basis, another prior or selection) | `Document` | `Authority` with the strict reader's error unchanged |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-013-AC-1 | `derive` over a selection with a `TimestampedEvent` boundary emits `boundary` and `basis` `derived` with every fact `origin` `selected`, its view returns them through `boundary()`, `basis()` and `FactRef.origin`, and `read` admits it; a boundary violating FR-289's mapping refuses `InvalidSelection`. | Test (TC-015) |
| FR-013-AC-2 | With `prior` `Complete` through 100 over members stamped 0, 30, 60 and 90: a successor `Complete` through 100 and one through 140 over the same members plus one stamped 120 are accepted with the context's predecessor `prior`; through 60 refuses `Revision` `BoundaryRegressed` naming both boundaries; an `Incomplete` successor refuses `Revision` `CompleteRegressed`; a successor holding a member stamped 80 that `prior` does not hold refuses `Revision` `UncoveredMember` naming it; after each refusal `prior`'s bytes are unchanged. | Test (TC-015) |
| FR-013-AC-3 | A successor omitting the member stamped 30 carries it with `prior`'s observation and `origin` `carried`, and the revised state stays `Complete`; after an `Incomplete` `prior`, an `Incomplete` successor is accepted and a member `prior` held `Incomplete` that the successor supplies is accepted `Available`; a member `prior` held `Contradicted` stays `Contradicted` when the successor lists it `Available`; a member both hold `Available` under different observation identities refuses `Revision` `MemberObservationChanged`. | Test (TC-015) |
| FR-013-AC-4 | A context with no predecessor, with another document as predecessor, or with `prior`'s document under another authority refuses `Prior` `PredecessorMismatch`; a revision equal to `prior`'s refuses `Prior` `RevisionMismatch`; a successor over another admitted history (other scope and population identity) is accepted; a successor boundary of another clock family refuses `Selection` `ExpectedMismatch`; a successor selection that fails a `derive` check refuses `Selection` with `derive`'s error. | Test (TC-015) |
| FR-013-AC-5 | `read_revised` of `revise`'s bytes returns a view equal to the revised document; the same bytes with one byte changed, re-encoded non-canonically, or read against another `prior` refuse at `Document` with the strict reader's error; `read` of the revised bytes refuses `ExpectedMismatch` and `read_revised` of a derived document refuses at `Document`; revising twice from the same inputs gives equal bytes. | Test (TC-015) |
| FR-013-AC-6 | A revised population of one entry over `max_population_entries` refuses `Document` `ResourceIncomplete`; `revise` takes no progress view, and admitting progress through `admit_progress` over the same history before and after a revision returns equal capabilities. | Test (TC-015) |

## Dependencies

- [FR-004](FR-004-publish-observation-authority-artifacts.md) owns the
  completeness-assertion contract, `derive`, the strict readers, the error
  codes and the owner limits.
- [FR-008](FR-008-bind-activation-progress-and-completeness.md) keeps
  completeness independent of progress, closure and activation.
- [FR-014](FR-014-admit-progress-authority-from-request-input.md) admits
  progress authority by the same recomputation and is not changed by this
  rule.
- FSL FR-502 and FR-944 are the consumer: the evaluator's progress ledger maps
  each retained `Complete { through }` onto a completeness view and accepts,
  revises or refuses a new assertion through `revise`, reporting its
  `Completeness` contradiction exactly when `revise` refuses at `Revision`.
