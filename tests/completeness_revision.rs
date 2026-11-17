// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-015: FR-013's monotone revision of a retained completeness assertion.
//!
//! `completeness::revise` and `completeness::read_revised` do not exist yet,
//! so each stub states the decided assertion and fails until the code ticket
//! adds them.

use ix_trace_rs::trace;

const MISSING: &str = "authority::completeness::revise does not exist";

#[trace("TC-015", "FR-013-AC-1")]
#[test]
#[ignore = "QSL-2327: boundary-carrying completeness selection and payload not implemented yet"]
fn derive_carries_the_boundary_basis_and_fact_origin() {
    // Selection::new(boundary_identity, TimestampedEvent boundary, facts):
    // derive emits `boundary`, basis `derived`, every fact origin `selected`;
    // read admits it and the view returns them; a boundary violating FR-289
    // refuses InvalidSelection.
    panic!("{MISSING}");
}

#[trace("TC-015", "FR-013-AC-2")]
#[test]
#[ignore = "QSL-2327: completeness::revise accept and refuse cases not implemented yet"]
fn complete_through_revisions_accept_and_refuse_as_fr013_states() {
    // prior Complete through 100 over members stamped 0, 30, 60, 90:
    // through 100 and through 140 (+ member at 120) accepted;
    // through 60 -> Revision BoundaryRegressed{prior, successor};
    // Incomplete -> Revision CompleteRegressed;
    // uncovered member stamped 80 -> Revision UncoveredMember naming it;
    // prior bytes unchanged after each refusal.
    panic!("{MISSING}");
}

#[trace("TC-015", "FR-013-AC-3")]
#[test]
#[ignore = "QSL-2327: carried and merged revision facts not implemented yet"]
fn held_members_stay_held_and_changed_observations_refuse() {
    // Omitted member stamped 30 is carried (prior observation, origin
    // carried), state stays Complete; Incomplete after Incomplete accepted;
    // a prior-Incomplete member supplied later is Available; a prior
    // Contradicted member listed Available stays Contradicted; both Available
    // under different observation identities -> MemberObservationChanged.
    panic!("{MISSING}");
}

#[trace("TC-015", "FR-013-AC-4")]
#[test]
#[ignore = "QSL-2327: revise lineage and selection refusals not implemented yet"]
fn lineage_and_selection_defects_refuse_at_their_stage() {
    // No predecessor / another predecessor / another authority -> Prior
    // PredecessorMismatch; equal revision -> Prior RevisionMismatch; another
    // admitted history accepted; boundary of another clock family -> Selection
    // ExpectedMismatch; a failing derive check -> Selection with derive's error.
    panic!("{MISSING}");
}

#[trace("TC-015", "FR-013-AC-5")]
#[test]
#[ignore = "QSL-2327: read_revised authentication by recomputation not implemented yet"]
fn revised_documents_authenticate_only_through_read_revised() {
    // read_revised(revise bytes) == revised view; one byte changed,
    // non-canonical, other prior -> Document with the reader's error; read of
    // revised bytes -> ExpectedMismatch; read_revised of a derived document ->
    // Document; revising twice gives equal bytes.
    panic!("{MISSING}");
}

#[trace("TC-015", "FR-013-AC-6")]
#[test]
#[ignore = "QSL-2327: revise limits and progress independence not implemented yet"]
fn revision_is_bounded_and_independent_of_progress_admission() {
    // One carried fact over max_population_entries -> Document
    // ResourceIncomplete; admit_progress over the same history before and
    // after a revision returns equal capabilities.
    panic!("{MISSING}");
}
