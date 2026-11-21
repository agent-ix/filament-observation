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
| `check_revision(prior_upper: Option<i128>, prior_state: State, prior: &H, successor_upper: Option<i128>, successor_state: State, listed: &[ListedFact<'_, H::Key>]) -> Result<(), ListedContradiction>` for `H: RetainedHolding` | the revision rule without a history |
| `RetainedHolding` | trait with `type Key: ?Sized` and `held(&self, key: &Key) -> Option<HeldFact<'_>>`; implemented for `View` keyed by member identity |
| `HeldFact` | `{ status: FactStatus, observation_identity: Option<&str> }` |
| `ListedFact` | `{ key: &K, status: FactStatus, observation_identity: Option<&str>, placed: i128 }` |
| `ListedContradiction` | `BoundaryRegressed { prior_upper, successor_upper }`, `CompleteRegressed { prior_upper }`, `UncoveredMember { listed }`, `MemberObservationChanged { listed }`, where `listed` indexes the listed facts |

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
  successor history's required population minus the members it carries
  forward from `prior` (below).
- `limits`: the owner limits.

## Outputs

On acceptance, the revised document (`basis` `revised`). Its `boundary` is
the selection's. Its facts are the **revised population**: first one fact per
member of the successor's required population, in that population's order,
either the selection's fact with the merged status below and `origin`
`selected`, or, for a carried member, `prior`'s fact with `origin` `carried`;
then each fact `prior` holds for a member outside the successor's required
population, in `prior`'s order, with `prior`'s status and observation identity
and `origin` `carried`. Its `state` is FR-287's aggregate
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
- A **carried member** is a member of the successor's required population
  that the selection omits and `prior` holds. Its fact is `prior`'s, taken
  unchanged (status and observation identity, which need not name a record of
  the successor history), and it counts toward coverage of the successor
  population.
- The successor's selection SHALL pass every check `derive` applies to it,
  except that `revise` runs `derive`'s exact-coverage check over the
  successor's required population minus the carried members: the selection
  lists every other member, in the population's order with the carried members
  skipped. `derive` itself keeps exact coverage of the whole required
  population. The selection's boundary SHALL be of `prior`'s clock family.
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
3. **Uncovered member.** In the population's order, a member of the
   successor's required population that the selection omits and `prior` does
   not hold, wherever it is placed: it can be neither carried nor assessed.
   Then, in the selection's order, a member the selection lists `Available` or `Contradicted`, placed at or before `prior`'s boundary,
   that `prior` does not hold. Either refuses `UncoveredMember` naming the
   first such member.
   A member `prior` holds `Incomplete` and the selection supplies is accepted,
   and so is any listed member placed after `prior`'s boundary.
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

### Revision without a history

`check_revision` SHALL apply checks 1, 2, 3 (for the listed members) and 4 in that order and return the first contradiction, for a caller that retains its held members without a completeness document. `revise` SHALL apply those checks through `check_revision`, so the two cannot differ; check 3 for omitted members of the successor population stays in `revise`, which alone sees that population.

- A `None` upper bound places nothing at or before it: no listed member is uncovered by it and it takes part in no boundary comparison.
- `prior_state` and `successor_state` are read as given, so a caller may pass a retained incomplete state with no boundary. Per-fact status governs only checks 3 and 4: a listed member counts for check 3 when `Available` or `Contradicted`, and for check 4 when `Available`.
- Check 4 compares observation identities as optional values: a present identity differs from an absent one.
- A member placed at or before the upper bound that the retained assessment holds, whatever its status, is not uncovered.

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
| any refusal `derive` returns for the context and selection, its coverage check run without the carried members | `Selection` | `Authority` with that error unchanged |
| the boundary fails FR-289's mapping | `Selection` | `Authority` `InvalidSelection` |
| the boundary is of another clock family than `prior`'s | `Selection` | `Authority` `ExpectedMismatch` |
| a member is placed in another clock family than the boundary | `Selection` | `Authority` `ExpectedMismatch` |
| the selection's boundary is below `prior`'s | `Revision` | `Contradiction` `BoundaryRegressed` |
| `prior` is `Complete` and the selection's own state is `Incomplete` | `Revision` | `Contradiction` `CompleteRegressed` |
| an omitted member of the successor population that `prior` does not hold, or an `Available` or `Contradicted` member at or before `prior`'s boundary that `prior` does not hold | `Revision` | `Contradiction` `UncoveredMember` |
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
| FR-013-AC-7 | End to end: `prior` is `derive`d from `Selection::new` over members stamped 0, 30, 60 and 90, `Complete` through 100, and read back. The successor history admits members stamped 0, 30, 60, 90 and 120, and its `Selection` (through 140) lists the members at 0, 60, 90 and 120 and omits the one at 30. `derive` of that selection refuses `ExpectedMismatch` because it does not cover the population. `revise` accepts it: the revised population lists all five members in population order, the one at 30 with `prior`'s fact and `origin` `carried`, the state is `Complete`, and `read_revised` admits the bytes. The same successor omitting the member at 120, which `prior` does not hold, refuses `Revision` `UncoveredMember` naming it. | Test (TC-015) |
| FR-013-AC-8 | `check_revision` returns `BoundaryRegressed`, `CompleteRegressed`, `UncoveredMember` and `MemberObservationChanged` for the cases of checks 1 to 4 in that order; with a `None` prior upper bound a member stamped 0 is accepted after an `Incomplete` prior while a changed observation identity of a held `Available` member is still refused; a present and an absent observation identity differ; and a fact `prior` holds for a member outside the successor population is carried with `origin` `carried`, so a later revision that lists that member at or before the boundary is not uncovered. | Test (TC-015) |

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
