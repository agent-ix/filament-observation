// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-016: FR-014's admission of the authenticated progress view and opaque
//! authority selections from request and event bytes.
//!
//! `authority::admission` tests for FR-014.

use filament_observation::authority::{self, Limits};
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
fn each_request_defect_refuses_at_the_request_stage() {
    let error = authority::admission::admit_progress(b"{", None, Limits::owner_max())
        .expect_err("invalid request JSON is refused");
    assert_eq!(error.stage(), authority::admission::AdmissionStage::Request);
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Authority(cause)
            if cause.code() == authority::ErrorCode::InvalidJson
    ));
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

#[trace("TC-016", "FR-014-AC-9")]
#[test]
#[ignore = "QSL-1867: runtime event and watermark lines in admit_progress not implemented yet"]
fn runtime_event_and_watermark_lines_admit_ordered_steps() {
    // Two filament.observation.rt-event/v1 lines carrying AC-1's records,
    // then one filament.observation.rt-watermark/v1 line: view == AC-1's;
    // steps() == [Event(line 1), Event(line 2), Watermark(line 3)] with each
    // event's record, reading and exact inputs bytes, and the watermark's view
    // == progress::read over both records with the line's boundary, state and
    // revision and AC-1's document as predecessor; assertion() is the document
    // bytes and inclusive_boundary() the line's.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-10")]
#[test]
#[ignore = "QSL-1867: mixed record-candidate and runtime lines not implemented yet"]
fn mixed_line_kinds_admit_and_record_lines_alone_admit_as_before() {
    // Scope of three members anchored a < b < c: record-candidate(a),
    // rt-event(b), rt-watermark with frontier f (b <= f < c), rt-event(c)
    // admits; steps() == [Event(2), Watermark(3), Event(4)]; qualified()
    // holds all three records; the request's and the watermark's views equal
    // progress::read of their documents over that three-record history.
    // Record-candidate lines only admit AC-2's capability with steps() empty.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-11")]
#[test]
#[ignore = "QSL-1867: frontier ordering of runtime lines not implemented yet"]
fn records_at_or_below_the_frontier_and_regressing_watermarks_refuse() {
    // After a watermark with frontier f: a record of either kind at f, below
    // f, or in another clock family -> Event{line} ExpectedMismatch; a
    // watermark below f -> ExpectedMismatch; one equal to f with a greater
    // revision admits; inclusive_boundary != boundary's inclusive upper ->
    // ExpectedMismatch. No capability on any refusal.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-12")]
#[test]
#[ignore = "QSL-1867: watermark progress-document authentication not implemented yet"]
fn a_watermark_document_not_derived_from_its_inputs_refuses_at_its_line() {
    // One changed byte, predecessor's revision, other predecessor, derived
    // from the empty history, boundary one tick above the line's:
    // Event{line} of the watermark with the error progress::read returns
    // for the same bytes and context.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-13")]
#[test]
#[ignore = "QSL-1867: runtime line decoding refusals not implemented yet"]
fn each_runtime_line_defect_refuses_at_its_line() {
    // Line 3: rt-event record naming filament.observation.record/v2 ->
    // ContractMismatch; reading "18446744073709551616" or "01", inputs as a
    // JSON object, watermark without progress_document, inclusive_boundary
    // "+5" -> InvalidSelection; contract filament.observation.rt-event/v2 ->
    // ContractMismatch. All Event{line: 3}.
    panic!("{MISSING}");
}

#[trace("TC-016", "FR-014-AC-14")]
#[test]
#[ignore = "QSL-1867: qualification of post-watermark records not implemented yet"]
fn post_watermark_records_outside_the_population_refuse_at_qualification() {
    // AC-10's input with max_records >= 4: a fifth line repeating rt-event(c)
    // -> Qualification Refused DuplicateRecord; instead a fifth line that is an
    // rt-event above f for a non-member record (c kept) -> Qualification
    // Incomplete with sole reason MissingMembership; the last line dropped -> Qualification Refused
    // MemberRecordMismatch. Outcomes carried unchanged, no capability.
    panic!("{MISSING}");
}
