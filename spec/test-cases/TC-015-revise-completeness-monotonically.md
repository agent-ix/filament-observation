---
id: TC-015
title: "Revise a completeness assertion monotonically across revisions"
type: TC
relationships:
  - { target: ix://agent-ix/filament-observation/FR-013, type: verifies }
---
# TC-015: Revise a completeness assertion monotonically across revisions

## Description

Check FR-013's `completeness::revise` and `completeness::read_revised`: the
boundary-carrying selection and payload, every accepted and refused revision
of a retained assessment, carried and merged facts, lineage refusals,
authentication by recomputation, and independence from progress admission.

## Test Procedure

1. Derive a `Complete` assessment through 100 over members stamped 0, 30, 60
   and 90 under a `TimestampedEvent` boundary; read it back and check
   `boundary()`, `basis()` and each fact's `origin`; derive once with a boundary
   violating FR-289's mapping.
2. Revise it with successors `Complete` through 100, through 140 (plus a member
   stamped 120), through 60, `Incomplete`, and one holding an uncovered member
   stamped 80; compare each result with FR-013's stage and contradiction and
   compare `prior`'s bytes before and after each refusal.
3. Revise with a successor omitting one member, with `Incomplete` after
   `Incomplete`, with a member supplied after `prior` held it `Incomplete`,
   with a `Contradicted` prior member listed `Available`, and with one member
   under a different observation identity; check carried facts, merged
   statuses and the refusal.
4. Vary the context alone: no predecessor, another predecessor, another
   authority, an equal revision, another admitted history; vary the boundary's
   clock family and break one `derive` check of the selection.
5. Strict-read the revised bytes through `read_revised`, then mutate them (one
   byte, non-canonical, other prior), read them through `read`, read a derived
   document through `read_revised`, and revise twice from equal inputs.
6. Exceed `max_population_entries` by one carried fact; admit progress through
   `admit_progress` over the same history before and after a revision.
7. End to end through `Selection::new`, `derive` and `revise`: derive and
   read a `Complete` prior through 100 over members stamped 0, 30, 60 and 90;
   admit a successor history with members at 0, 30, 60, 90 and 120 and build
   a selection through 140 omitting the member at 30. Check that `derive`
   refuses it for coverage, that `revise` carries the member at 30 in
   population order and `read_revised` admits the bytes, and that omitting
   the member at 120 instead refuses `UncoveredMember`.

## Expected Results

Every accepted revision returns the document FR-013's revised population
states, with carried members in population order, and strict-reads back equal through `read_revised`. Every refusal
returns exactly the listed stage and cause, names the contradicted boundary or
member, returns no document and leaves `prior` unchanged. Progress admission
is unaffected by revision.

## Status

Ignored stubs in `tests/completeness_revision.rs`.
