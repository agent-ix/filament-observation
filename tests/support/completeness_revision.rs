// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Focused TC-015 cases that need the owner and history fixtures in authority.rs.

use super::*;
use ix_trace_rs::trace;

#[trace("TC-015", "FR-013-AC-4")]
#[test]
fn lineage_and_selection_defects_refuse_at_their_stage() {
    let qualified = qualified();
    let owner = owner();
    let subject_selection = subject(&qualified);
    let history = History::batch(&qualified);
    let limits = Limits::owner_max();
    let selection = completeness_selection(&qualified.records()[0]);
    let prior_context = Context::new(history, &owner, &subject_selection, 1, None);
    let prior_document = authority::completeness::derive(prior_context, &selection, limits)
        .expect("derive retained completeness assertion");
    let prior =
        authority::completeness::read(prior_document.bytes(), prior_context, &selection, limits)
            .expect("read retained completeness assertion");

    let no_predecessor = Context::new(history, &owner, &subject_selection, 2, None);
    let error = authority::completeness::revise(&prior, no_predecessor, &selection, limits)
        .expect_err("revision requires its retained predecessor");
    assert_eq!(error.stage(), authority::completeness::RevisionStage::Prior);
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::PredecessorMismatch
    ));

    let same_revision = Context::new(
        history,
        &owner,
        &subject_selection,
        1,
        Some(&prior_document),
    );
    let error = authority::completeness::revise(&prior, same_revision, &selection, limits)
        .expect_err("revision must advance");
    assert_eq!(error.stage(), authority::completeness::RevisionStage::Prior);
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::RevisionMismatch
    ));

    let mut another_owner = owner.clone();
    another_owner.definition_digest = digest(8);
    let wrong_authority = Context::new(
        history,
        &another_owner,
        &subject_selection,
        2,
        Some(&prior_document),
    );
    let error = authority::completeness::revise(&prior, wrong_authority, &selection, limits)
        .expect_err("prior cannot be revised under another authority");
    assert_eq!(error.stage(), authority::completeness::RevisionStage::Prior);
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::PredecessorMismatch
    ));

    let other_scope = qualified_with_window("window:O2");
    let other_subject = subject(&other_scope);
    let other_history = History::batch(&other_scope);
    let other_selection = completeness_selection(&other_scope.records()[0]);
    let other_context = Context::new(
        other_history,
        &owner,
        &other_subject,
        2,
        Some(&prior_document),
    );
    let revised = authority::completeness::revise(&prior, other_context, &other_selection, limits)
        .expect("a successor assesses its own admitted history and scope");
    let revised = authority::completeness::read_revised(
        revised.bytes(),
        &prior,
        other_context,
        &other_selection,
        limits,
    )
    .expect("read revision over the successor history");
    assert_eq!(
        revised.payload().basis(),
        authority::completeness::Basis::Revised
    );

    let other_clock = TemporalBoundary::EventPosition {
        lower: 0,
        upper_inclusive: 29,
        carrier_end_exclusive: 30,
        watermark: 30,
    };
    let next_context = Context::new(
        history,
        &owner,
        &subject_selection,
        2,
        Some(&prior_document),
    );
    let other_clock_selection = authority::completeness::Selection::new(
        id("boundary:O1"),
        other_clock,
        vec![authority::completeness::Fact::new(
            id("member:O1"),
            Some(qualified.records()[0].identity.clone()),
            authority::completeness::FactStatus::Available,
        )],
    );
    let error =
        authority::completeness::revise(&prior, next_context, &other_clock_selection, limits)
            .expect_err("successor boundary must use prior's clock family");
    assert_eq!(
        error.stage(),
        authority::completeness::RevisionStage::Selection
    );
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::ExpectedMismatch
    ));

    let bad_observation = authority::completeness::Selection::new(
        id("boundary:O1"),
        boundary(OpenClosed::Closed),
        vec![authority::completeness::Fact::new(
            id("member:O1"),
            Some(id("record:foreign")),
            authority::completeness::FactStatus::Available,
        )],
    );
    let error = authority::completeness::revise(&prior, next_context, &bad_observation, limits)
        .expect_err("selection must pass derive's observation checks");
    assert_eq!(
        error.stage(),
        authority::completeness::RevisionStage::Selection
    );
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::ExpectedMismatch
    ));
}

#[trace("TC-015", "FR-013-AC-6")]
#[test]
fn revision_document_limits_and_progress_are_independent() {
    let qualified = qualified();
    let owner = owner();
    let subject_selection = subject(&qualified);
    let history = History::batch(&qualified);
    let selection = completeness_selection(&qualified.records()[0]);
    let limits = Limits::owner_max();
    let prior_context = Context::new(history, &owner, &subject_selection, 1, None);
    let prior_document = authority::completeness::derive(prior_context, &selection, limits)
        .expect("derive retained assertion");
    let prior =
        authority::completeness::read(prior_document.bytes(), prior_context, &selection, limits)
            .expect("read retained assertion");
    let context = Context::new(
        history,
        &owner,
        &subject_selection,
        2,
        Some(&prior_document),
    );

    let mut narrow_limits = limits;
    narrow_limits.max_population_entries = 0;
    let error = authority::completeness::revise(&prior, context, &selection, narrow_limits)
        .expect_err("the revised population exceeds the caller limit");
    assert_eq!(
        error.stage(),
        authority::completeness::RevisionStage::Document
    );
    assert!(matches!(
        error.cause(),
        authority::completeness::RevisionCause::Authority(error)
            if error.code() == authority::ErrorCode::ResourceIncomplete
    ));

    let progress_selection = progress_authority_selection();
    let progress_context = Context::new(history, &owner, &subject_selection, 1, None);
    let progress_document =
        authority::progress::derive(progress_context, &progress_selection, limits)
            .expect("derive the independent progress authority");
    let request = progress_admission_request_wire(&qualified, &progress_document, true);
    let bytes = serde_json::to_vec(&request).expect("request is JSON");
    let before = authority::admission::admit_progress(&bytes, None, limits)
        .expect("admit progress before completeness revision");

    let _revised = authority::completeness::revise(&prior, context, &selection, limits)
        .expect("completeness revision does not consume or change progress authority");
    let after = authority::admission::admit_progress(&bytes, None, limits)
        .expect("same progress bytes remain independently admissible");
    assert_eq!(before, after);
}
