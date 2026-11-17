// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-016: FR-014's admission of the authenticated progress view and opaque
//! authority selections from request and event bytes.
//!
//! `authority::admission` does not exist yet, so each stub states the decided
//! assertion and fails until the code ticket adds it.

use ix_trace_rs::trace;

const MISSING: &str = "authority::admission::admit_progress does not exist";

#[trace("TC-016", "FR-014-AC-1")]
#[test]
#[ignore = "QSL-1867: admit_progress from request records not implemented yet"]
fn a_request_with_records_admits_the_rust_api_view_and_its_selections() {
    // admit_progress(request_with_two_records, None, Limits::owner_max()):
    // view == progress::read(..) over the same inputs; selections() return
    // the request's authority, subject, clock and observation_source;
    // binds(view) and progress_identity() name the document.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-2")]
#[test]
#[ignore = "QSL-1867: admit_progress from event input not implemented yet"]
fn event_input_records_admit_the_same_view() {
    // The request without `records` plus two LF-separated record-candidate
    // lines (with and without a final LF) admits AC-1's view; association
    // rows give equal association_canonical_bytes(); no record source admits
    // the empty-binding-history view.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-3")]
#[test]
#[ignore = "QSL-1867: request refusals of admit_progress not implemented yet"]
fn each_request_defect_refuses_at_the_request_stage() {
    // One over max_input_bytes -> ResourceIncomplete; invalid UTF-8 ->
    // InvalidUtf8; trailing data -> InvalidJson; other contract ->
    // ContractMismatch; unknown/duplicate/missing member, revision "01",
    // uppercase digest, records plus event input -> InvalidSelection; all
    // stage Request, no capability; exactly max_input_bytes is not refused.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-4")]
#[test]
#[ignore = "QSL-1867: event-line refusals of admit_progress not implemented yet"]
fn each_event_line_defect_refuses_at_its_line() {
    // Line 3 empty or not JSON -> Event{line: 3} InvalidJson; contract
    // filament.observation.record/v2 -> ContractMismatch; missing
    // event_time_nanos -> InvalidSelection; one record past max_records ->
    // ResourceIncomplete at that line before retention.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-5")]
#[test]
#[ignore = "QSL-1867: producer and qualification errors of admit_progress not implemented yet"]
fn producer_and_qualification_outcomes_are_carried_unchanged() {
    // Interface 1.1.0 bundle -> stage Producer; foreign signal ->
    // Qualification with admit's Refused outcome; missing required valuation
    // -> Qualification with admit's Incomplete outcome.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-6")]
#[test]
#[ignore = "QSL-1867: progress-document authentication in admit_progress not implemented yet"]
fn a_progress_document_not_derived_from_the_inputs_refuses() {
    // One changed byte, non-canonical re-encoding, other revision, other
    // scope, one record dropped: stage Progress with the error progress::read
    // returns; observation_source outside required sources -> ExpectedMismatch.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-7")]
#[test]
#[ignore = "QSL-1867: opaque AdmittedProgress and AuthoritySelections not implemented yet"]
fn the_capability_is_opaque_and_bound_to_its_document() {
    // No public constructor, field, Default, Serialize or Deserialize;
    // admission A's selections do not bind admission B's view; accessors
    // borrow without allocating.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-8")]
#[test]
#[ignore = "QSL-1867: ambient-input-free admit_progress not implemented yet"]
fn admission_reads_only_the_supplied_bytes() {
    // The same bytes admitted in two working directories and environments
    // give equal capabilities.
    panic!("{MISSING}");
}
