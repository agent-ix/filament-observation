// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Focused TC-016 cases that share the owner fixtures in authority.rs.

use super::*;
use ix_trace_rs::trace;

fn progress_fixture(
    include_records: bool,
) -> (
    Box<QualifiedObservation>,
    authority::Document,
    serde_json::Value,
) {
    let qualified = qualified_with_two_records();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let selection = progress_authority_selection();
    let context = Context::new(history, &owner, &subject, 1, None);
    let document = authority::progress::derive(context, &selection, Limits::owner_max())
        .expect("derive request progress document");
    let request = progress_admission_request_wire(&qualified, &document, include_records);
    (qualified, document, request)
}

fn three_record_progress_fixture() -> (
    Box<QualifiedObservation>,
    authority::Document,
    serde_json::Value,
) {
    let qualified = qualified_with_three_records();
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let initial_selection = progress_selection(boundary(OpenClosed::Closed));
    let document = authority::progress::derive(
        Context::new(history, &owner, &subject, 1, None),
        &initial_selection,
        Limits::owner_max(),
    )
    .expect("derive three-member request progress document");
    let mut request = progress_admission_request_wire(&qualified, &document, false);
    request["progress"]["boundary"] = serde_json::json!({
        "clock_family": "timestamped-event",
        "lower_nanos": "0",
        "upper_inclusive_nanos": "29",
        "carrier_end_exclusive_nanos": "60",
        "watermark_nanos": "60",
    });
    (qualified, document, request)
}

fn progress_selection(boundary: TemporalBoundary) -> authority::progress::Selection {
    authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        boundary,
        OpenClosed::Closed,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    )
}

fn event_line(record: &AdmittedRecord, reading: &str) -> serde_json::Value {
    serde_json::json!({
        "contract": authority::admission::EVENT_CONTRACT,
        "record": record_candidate_wire(record),
        "reading": reading,
        "inputs": "{\"member\":0}",
    })
}

fn watermark_line(
    document: &authority::Document,
    revision: &str,
    boundary: TemporalBoundary,
) -> serde_json::Value {
    let TemporalBoundary::TimestampedEvent {
        lower_nanos,
        upper_inclusive_nanos,
        carrier_end_exclusive_nanos,
        watermark_nanos,
    } = boundary
    else {
        panic!("the test watermark uses timestamped event time");
    };
    serde_json::json!({
        "contract": authority::admission::WATERMARK_CONTRACT,
        "revision": revision,
        "boundary": {
            "clock_family": "timestamped-event",
            "lower_nanos": lower_nanos.to_string(),
            "upper_inclusive_nanos": upper_inclusive_nanos.to_string(),
            "carrier_end_exclusive_nanos": carrier_end_exclusive_nanos.to_string(),
            "watermark_nanos": watermark_nanos.to_string(),
        },
        "state": "closed",
        "inclusive_boundary": upper_inclusive_nanos.to_string(),
        "progress_document": String::from_utf8(document.bytes().to_vec()).expect("UTF-8 document"),
    })
}

fn successor_document(
    qualified: &QualifiedObservation,
    prior_document: &authority::Document,
    revision: u64,
    boundary: TemporalBoundary,
) -> authority::Document {
    let owner = owner();
    let subject = subject(qualified);
    let history = History::batch(qualified);
    let selection = authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        boundary,
        OpenClosed::Closed,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    );
    authority::progress::derive(
        Context::new(history, &owner, &subject, revision, Some(prior_document)),
        &selection,
        Limits::owner_max(),
    )
    .expect("derive successor watermark document")
}

fn admit_lines(
    request: &serde_json::Value,
    lines: &[serde_json::Value],
) -> Result<authority::admission::AdmittedProgress, authority::admission::AdmissionError> {
    let event_bytes = lines
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    authority::admission::admit_progress(
        &serde_json::to_vec(request).expect("request is JSON"),
        Some(event_bytes.as_bytes()),
        Limits::owner_max(),
    )
}

fn admit_raw_lines(
    request: &serde_json::Value,
    lines: &[&str],
) -> Result<authority::admission::AdmittedProgress, authority::admission::AdmissionError> {
    let event_bytes = lines.join("\n");
    authority::admission::admit_progress(
        &serde_json::to_vec(request).expect("request is JSON"),
        Some(event_bytes.as_bytes()),
        Limits::owner_max(),
    )
}

#[trace("TC-016", "FR-014-AC-1")]
#[test]
fn request_records_admit_the_view_and_borrowed_selections() {
    let (qualified, document, request) = progress_fixture(true);
    let admitted = authority::admission::admit_progress(
        &serde_json::to_vec(&request).expect("request JSON"),
        None,
        Limits::owner_max(),
    )
    .expect("request records admit");
    assert_eq!(admitted.qualified(), qualified.as_ref());
    assert!(admitted.selections().binds(admitted.view()));
    assert_eq!(admitted.view().identity(), document.identity());
    assert_eq!(admitted.steps(), &[]);
    assert_eq!(admitted.selections().authority(), &owner());
}

#[trace("TC-016", "FR-014-AC-2")]
#[test]
fn record_candidate_lines_admit_without_runtime_steps() {
    let (qualified, _, request) = progress_fixture(false);
    let lines = qualified
        .records()
        .iter()
        .map(record_candidate_wire)
        .collect::<Vec<_>>();
    let without_final_lf = admit_lines(&request, &lines).expect("record lines admit");
    let mut text = lines
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    text.push('\n');
    let with_final_lf = authority::admission::admit_progress(
        &serde_json::to_vec(&request).expect("request JSON"),
        Some(text.as_bytes()),
        Limits::owner_max(),
    )
    .expect("record lines with final LF admit");
    assert_eq!(without_final_lf, with_final_lf);
    assert_eq!(without_final_lf.qualified().records(), qualified.records());
    assert!(without_final_lf.steps().is_empty());
}

#[trace("TC-016", "FR-014-AC-3")]
#[test]
fn invalid_request_refuses_at_request_stage() {
    let error = authority::admission::admit_progress(b"{", None, Limits::owner_max())
        .expect_err("invalid request JSON refuses");
    assert_eq!(error.stage(), authority::admission::AdmissionStage::Request);
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Authority(cause)
            if cause.code() == authority::ErrorCode::InvalidJson
    ));
}

#[trace("TC-016", "FR-014-AC-4")]
#[test]
fn malformed_event_lines_refuse_at_their_line() {
    let (qualified, _, request) = progress_fixture(false);
    let records = qualified
        .records()
        .iter()
        .map(record_candidate_wire)
        .collect::<Vec<_>>();
    let record_lines = records
        .iter()
        .map(serde_json::Value::to_string)
        .collect::<Vec<_>>();
    let mut invalid_json = record_lines.clone();
    invalid_json.push("not-json".to_owned());
    let raw = invalid_json.iter().map(String::as_str).collect::<Vec<_>>();
    let error = admit_raw_lines(&request, &raw).expect_err("malformed third line refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::InvalidJson)
    );
    let mut empty_third = record_lines.clone();
    empty_third.extend([String::new(), String::from("unused fourth line")]);
    let raw = empty_third.iter().map(String::as_str).collect::<Vec<_>>();
    let error = admit_raw_lines(&request, &raw).expect_err("empty third line refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::InvalidJson)
    );
    let mut wrong_contract = records.clone();
    wrong_contract.push(serde_json::json!({"contract":"unknown"}));
    let error = admit_lines(&request, &wrong_contract).expect_err("wrong line contract refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::ContractMismatch)
    );
    let mut malformed = record_candidate_wire(&qualified.records()[0]);
    malformed
        .as_object_mut()
        .expect("record is object")
        .remove("event_time_nanos");
    let mut lines = records.clone();
    lines.push(malformed);
    let error = admit_lines(&request, &lines).expect_err("missing event time refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::InvalidSelection)
    );
}

#[trace("TC-016", "FR-014-AC-5")]
#[test]
fn producer_and_qualification_errors_keep_their_stages_and_outcomes() {
    let (qualified, _, request) = progress_fixture(true);
    let mut wrong_producer = request.clone();
    wrong_producer["producer"]["interfaceVersion"] = "1.1.0".into();
    let error = authority::admission::admit_progress(
        &serde_json::to_vec(&wrong_producer).unwrap(),
        None,
        Limits::owner_max(),
    )
    .expect_err("unsupported producer version refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Producer
    );

    let mut foreign_signal = request.clone();
    foreign_signal["records"][0]["signal_identity"] = "signal:foreign".into();
    let error = authority::admission::admit_progress(
        &serde_json::to_vec(&foreign_signal).unwrap(),
        None,
        Limits::owner_max(),
    )
    .expect_err("foreign signal refuses qualification");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Qualification
    );
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Observation(AdmissionOutcome::Refused { .. })
    ));

    let mut missing_value = request;
    missing_value["records"][0]["value"] = serde_json::json!({"state":"missing"});
    let error = authority::admission::admit_progress(
        &serde_json::to_vec(&missing_value).unwrap(),
        None,
        Limits::owner_max(),
    )
    .expect_err("required missing valuation is incomplete");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Qualification
    );
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Observation(AdmissionOutcome::Incomplete { .. })
    ));
    assert_eq!(qualified.records().len(), 2);
}

#[trace("TC-016", "FR-014-AC-6")]
#[test]
fn progress_document_mutations_refuse_at_progress_stage() {
    let (_, _, request) = progress_fixture(true);
    for mutation in ["byte", "noncanonical", "source"] {
        let mut changed = request.clone();
        match mutation {
            "byte" => {
                let mut bytes = changed["progress_document"]
                    .as_str()
                    .unwrap()
                    .as_bytes()
                    .to_vec();
                bytes[0] ^= 1;
                changed["progress_document"] = String::from_utf8(bytes).unwrap().into();
            }
            "noncanonical" => {
                changed["progress_document"] =
                    format!("{} ", changed["progress_document"].as_str().unwrap()).into();
            }
            _ => changed["observation_source"] = "source:foreign".into(),
        }
        let error = authority::admission::admit_progress(
            &serde_json::to_vec(&changed).unwrap(),
            None,
            Limits::owner_max(),
        )
        .expect_err("progress mismatch refuses");
        assert_eq!(
            error.stage(),
            authority::admission::AdmissionStage::Progress
        );
    }
}

#[trace("TC-016", "FR-014-AC-7")]
#[test]
fn capability_selections_bind_only_their_admitted_view() {
    let (qualified, document, request) = progress_fixture(true);
    let admitted = authority::admission::admit_progress(
        &serde_json::to_vec(&request).unwrap(),
        None,
        Limits::owner_max(),
    )
    .expect("request admits");
    let owner = owner();
    let subject = subject(&qualified);
    let history = History::batch(&qualified);
    let selection = authority::progress::Selection::new(
        authority::clock::Selection::new(id("clock:event-time"), id("1")),
        vec![id("source:payments")],
        TemporalBoundary::TimestampedEvent {
            lower_nanos: 0,
            upper_inclusive_nanos: 29,
            carrier_end_exclusive_nanos: 30,
            watermark_nanos: 20,
        },
        OpenClosed::Open,
        id("trigger:refund-request"),
        cutoff(40),
        id("restoration:O1"),
    );
    let other = authority::progress::derive(
        Context::new(history, &owner, &subject, 1, None),
        &selection,
        Limits::owner_max(),
    )
    .expect("different progress selection derives");
    let other_view = authority::progress::read(
        other.bytes(),
        Context::new(history, &owner, &subject, 1, None),
        &selection,
        Limits::owner_max(),
    )
    .expect("read different progress selection");
    assert!(admitted.selections().binds(admitted.view()));
    assert!(!admitted.selections().binds(&other_view));
    assert_eq!(admitted.view().identity(), document.identity());
}

#[trace("TC-016", "FR-014-AC-8")]
#[test]
fn admission_depends_only_on_supplied_request_bytes() {
    let (_, _, request) = progress_fixture(true);
    let bytes = serde_json::to_vec(&request).expect("request is JSON");
    let first = authority::admission::admit_progress(&bytes, None, Limits::owner_max())
        .expect("first admission");
    let second = authority::admission::admit_progress(&bytes, None, Limits::owner_max())
        .expect("second admission");
    assert_eq!(first, second);
}

#[trace("TC-016", "FR-014-AC-10")]
#[test]
fn mixed_runtime_and_record_lines_admit_and_record_only_keeps_empty_steps() {
    let (qualified, initial_document, request) = three_record_progress_fixture();
    let owner = owner();
    let subject = subject(&qualified);
    let records = qualified.records();
    let frontier = boundary(OpenClosed::Closed);
    let successor = successor_document(&qualified, &initial_document, 2, frontier);
    let watermark = watermark_line(&successor, "2", frontier);
    let lines = vec![
        record_candidate_wire(&records[0]),
        event_line(&records[1], "1"),
        watermark,
        event_line(&records[2], "2"),
    ];
    let admitted = admit_lines(&request, &lines).expect("mixed records and events admit");
    assert_eq!(admitted.qualified(), qualified.as_ref());
    assert_eq!(admitted.steps().len(), 3);
    assert!(matches!(
        admitted.steps()[0],
        authority::admission::AdmittedStep::Event(_)
    ));
    let authority_selection = AuthoritySelection {
        definition_identity: owner.definition_identity.clone(),
        definition_revision: owner.definition_revision.clone(),
        definition_digest: owner.definition_digest,
    };
    let initial_expected = authority::progress::read(
        initial_document.bytes(),
        Context::new(History::batch(&qualified), &owner, &subject, 1, None),
        &progress_selection(frontier),
        Limits::owner_max(),
    )
    .expect("read initial progress document over all records");
    assert_eq!(admitted.view(), &initial_expected);
    assert_eq!(admitted.selections().authority(), &authority_selection);
    let authority::admission::AdmittedStep::Watermark(admitted_watermark) = &admitted.steps()[1]
    else {
        panic!("line 3 is not an admitted watermark");
    };
    let successor_selection = progress_selection(frontier);
    let successor_expected = authority::progress::read(
        successor.bytes(),
        Context::new(
            History::batch(&qualified),
            &owner,
            &subject,
            2,
            Some(&initial_document),
        ),
        &successor_selection,
        Limits::owner_max(),
    )
    .expect("read successor progress document over all records");
    assert_eq!(admitted_watermark.view(), &successor_expected);
    assert_eq!(admitted_watermark.line(), 3);
    assert!(matches!(
        &admitted.steps()[2],
        authority::admission::AdmittedStep::Event(event)
            if event.line() == 4 && event.record() == &records[2]
    ));

    let only_records = records
        .iter()
        .map(record_candidate_wire)
        .collect::<Vec<_>>();
    let record_only =
        admit_lines(&request, &only_records).expect("record-candidate-only lines admit");
    assert!(record_only.steps().is_empty());
    assert_eq!(record_only.qualified(), qualified.as_ref());
}

#[trace("TC-016", "FR-014-AC-14")]
#[test]
fn qualification_uses_the_complete_event_record_population() {
    let (qualified, document, mut request) = three_record_progress_fixture();
    let owner = owner();
    let subject = subject(&qualified);
    request["limits"]["max_records"] = "4".into();
    let records = qualified.records();
    let first_three = vec![
        record_candidate_wire(&records[0]),
        event_line(&records[1], "1"),
        watermark_line(
            &successor_document(&qualified, &document, 2, boundary(OpenClosed::Closed)),
            "2",
            boundary(OpenClosed::Closed),
        ),
    ];

    let mut repeated = first_three.clone();
    repeated.push(event_line(&records[2], "2"));
    repeated.push(event_line(&records[2], "3"));
    let error = admit_lines(&request, &repeated).expect_err("repeated c refuses at qualification");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Qualification
    );
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Observation(AdmissionOutcome::Refused {
            cause: filament_observation::RefusalCause::DuplicateRecord { .. }
        })
    ));

    let mut nonmember = records[2].clone();
    nonmember.identity = id("unsealed-record:ambient");
    nonmember.anchor = Anchor::TimestampNanos(50);
    nonmember.event_time_nanos = 50;
    nonmember.ingestion_time_nanos = 51;
    nonmember.identity = authority::observation::record_identity(&nonmember, Limits::owner_max())
        .expect("derive ambient record identity");
    let nonmember_line = event_line(&nonmember, "4");
    let mut with_nonmember = first_three.clone();
    with_nonmember.push(event_line(&records[2], "2"));
    with_nonmember.push(nonmember_line);
    let error = admit_lines(&request, &with_nonmember)
        .expect_err("appended non-member refuses at qualification");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Qualification
    );
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Observation(AdmissionOutcome::Incomplete {
            reasons
        }) if reasons == &[filament_observation::IncompleteReason::MissingMembership]
    ));

    let missing_third = vec![
        record_candidate_wire(&records[0]),
        event_line(&records[1], "1"),
        watermark_line(
            &successor_document(&qualified, &document, 2, boundary(OpenClosed::Closed)),
            "2",
            boundary(OpenClosed::Closed),
        ),
    ];
    let error =
        admit_lines(&request, &missing_third).expect_err("missing c refuses at qualification");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Qualification
    );
    assert!(matches!(
        error.cause(),
        authority::admission::AdmissionCause::Observation(AdmissionOutcome::Refused {
            cause: filament_observation::RefusalCause::MemberRecordMismatch { .. }
        })
    ));
}

#[trace("TC-016", "FR-014-AC-11")]
#[test]
fn frontier_rejects_regressions_and_records_at_or_below_it() {
    let (qualified, initial, mut request) = three_record_progress_fixture();
    request["limits"]["max_records"] = "4".into();
    let boundary = boundary(OpenClosed::Closed);
    let successor = successor_document(&qualified, &initial, 2, boundary);
    let watermark = watermark_line(&successor, "2", boundary);
    let records = qualified.records();
    let at_frontier = vec![
        record_candidate_wire(&records[0]),
        watermark.clone(),
        event_line(&records[1], "1"),
        record_candidate_wire(&records[2]),
    ];
    let error = admit_lines(&request, &at_frontier).expect_err("record at the frontier refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::ExpectedMismatch)
    );

    let regressed = TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 19,
        carrier_end_exclusive_nanos: 30,
        watermark_nanos: 30,
    };
    let regressed_document = successor_document(&qualified, &initial, 3, regressed);
    let mut regressing = records
        .iter()
        .map(record_candidate_wire)
        .collect::<Vec<_>>();
    regressing.push(watermark);
    regressing.push(watermark_line(&regressed_document, "3", regressed));
    let error = admit_lines(&request, &regressing).expect_err("regressed watermark refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 5 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::ExpectedMismatch)
    );
}

#[trace("TC-016", "FR-014-AC-12")]
#[test]
fn watermark_documents_are_authenticated_against_the_qualified_history() {
    let (qualified, initial, request) = three_record_progress_fixture();
    let owner = owner();
    let subject = subject(&qualified);
    let boundary = boundary(OpenClosed::Closed);
    let successor = successor_document(&qualified, &initial, 2, boundary);
    let mut lines = vec![
        record_candidate_wire(&qualified.records()[0]),
        event_line(&qualified.records()[1], "1"),
    ];
    let mut changed_bytes = watermark_line(&successor, "2", boundary);
    let mut document = changed_bytes["progress_document"]
        .as_str()
        .expect("watermark document is text")
        .as_bytes()
        .to_vec();
    document[0] ^= 1;
    changed_bytes["progress_document"] = String::from_utf8(document)
        .expect("mutated document stays UTF-8")
        .into();
    let mut changed = lines.clone();
    changed.push(changed_bytes);
    changed.push(event_line(&qualified.records()[2], "2"));
    let error = admit_lines(&request, &changed)
        .expect_err("watermark bytes not derived from the qualified history refuse");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );

    let mut empty_request = AdmissionRequest {
        package: qualified.package().clone(),
        producer: qualified.producer().clone(),
        binding: qualified.binding().clone(),
        expected_subject: qualified.expected_subject().clone(),
        relationships: qualified.relationships().to_vec(),
        required_relationships: qualified.required_relationships().to_vec(),
        scope: qualified.scope().clone(),
        records: Vec::new(),
        limits: qualified.limits(),
    };
    empty_request.scope.members.clear();
    empty_request.scope.required_member_identities.clear();
    empty_request.limits.max_records = 0;
    empty_request.limits.max_members = 0;
    authority::population::assign_request_identities(&mut empty_request, Limits::owner_max())
        .expect("seal empty history membership");
    let empty = match admit(empty_request) {
        AdmissionOutcome::Available { observation } => observation,
        other => panic!("empty history fixture admission failed: {other:?}"),
    };
    let empty_document = authority::progress::derive(
        Context::new(History::batch(&empty), &owner, &subject, 2, Some(&initial)),
        &authority::progress::Selection::new(
            authority::clock::Selection::new(id("clock:event-time"), id("1")),
            vec![id("source:payments")],
            boundary,
            OpenClosed::Closed,
            id("trigger:refund-request"),
            cutoff(40),
            id("restoration:O1"),
        ),
        Limits::owner_max(),
    )
    .expect("derive successor document from empty history");
    let mut empty_lines = lines.clone();
    empty_lines.push(watermark_line(&empty_document, "2", boundary));
    empty_lines.push(event_line(&qualified.records()[2], "2"));
    let error = admit_lines(&request, &empty_lines).expect_err("empty-history watermark refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );

    let changed_boundary = TemporalBoundary::TimestampedEvent {
        lower_nanos: 0,
        upper_inclusive_nanos: 30,
        carrier_end_exclusive_nanos: 31,
        watermark_nanos: 31,
    };
    let boundary_document = successor_document(&qualified, &initial, 2, changed_boundary);
    let mut boundary_lines = lines;
    boundary_lines.push(watermark_line(&boundary_document, "2", boundary));
    boundary_lines.push(event_line(&qualified.records()[2], "2"));
    let error =
        admit_lines(&request, &boundary_lines).expect_err("boundary plus one watermark refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 3 }
    );
}

#[trace("TC-016", "FR-014-AC-13")]
#[test]
fn runtime_line_shape_errors_refuse_at_the_line() {
    let (qualified, _, request) = progress_fixture(false);
    let mut event = event_line(&qualified.records()[0], "01");
    let error = admit_lines(
        &request,
        &[
            record_candidate_wire(&qualified.records()[0]),
            event.clone(),
        ],
    )
    .expect_err("noncanonical reading refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 2 }
    );
    assert!(
        matches!(error.cause(), authority::admission::AdmissionCause::Authority(cause) if cause.code() == authority::ErrorCode::InvalidSelection)
    );

    event["reading"] = "1".into();
    event["inputs"] = serde_json::json!({"member":0});
    let error = admit_lines(
        &request,
        &[record_candidate_wire(&qualified.records()[0]), event],
    )
    .expect_err("non-string inputs refuses");
    assert_eq!(
        error.stage(),
        authority::admission::AdmissionStage::Event { line: 2 }
    );
}
