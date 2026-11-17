// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! Byte-only admission of a qualified progress view and ordered runtime steps.

use super::{
    AuthoritySelection, Context, Error, ErrorCode, History, Limits, OpenClosed, SubjectSelection,
    TemporalBoundary,
};
use crate::{AdmittedRecord, Identity, QualifiedObservation};
use serde::Deserialize;
use serde_json::value::RawValue;

/// Immutable request-contract label.
pub const REQUEST_CONTRACT: &str = "filament.observation.progress-admission-request/v1";
/// Record-candidate line contract.
pub const RECORD_CONTRACT: &str = "filament.observation.record-candidate/v1";
/// Runtime event line contract.
pub const EVENT_CONTRACT: &str = "filament.observation.rt-event/v1";
/// Runtime watermark line contract.
pub const WATERMARK_CONTRACT: &str = "filament.observation.rt-watermark/v1";
/// Request JSON Schema bytes.
pub const REQUEST_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/progress-admission-request-v1.schema.json");
/// Record-candidate JSON Schema bytes.
pub const RECORD_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/record-candidate-v1.schema.json");
/// Runtime-event JSON Schema bytes.
pub const EVENT_SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/rt-event-v1.schema.json");
/// Runtime-watermark JSON Schema bytes.
pub const WATERMARK_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/rt-watermark-v1.schema.json");

/// Stage where progress admission refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AdmissionStage {
    /// Request preflight or decoding.
    Request,
    /// Event-input preflight.
    Events,
    /// One line in the event input.
    Event {
        /// One-based event-input line that failed.
        line: u64,
    },
    /// Producer-bundle admission.
    Producer,
    /// Observation qualification.
    Qualification,
    /// Progress-document authentication or source selection.
    Progress,
}

/// Typed reason for admission refusal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmissionCause {
    /// An authority decoder, limit check, or strict reader refused.
    Authority(Error),
    /// The Producer bundle was refused.
    Producer(crate::Refusal),
    /// Observation qualification returned an incomplete or refused outcome.
    Observation(crate::AdmissionOutcome),
}

/// One typed admission refusal; it never carries a partial capability.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("progress admission refused at {stage:?}: {cause:?}")]
pub struct AdmissionError {
    stage: AdmissionStage,
    cause: Box<AdmissionCause>,
}

impl AdmissionError {
    /// Returns the exact stage that refused admission.
    #[must_use]
    pub const fn stage(&self) -> AdmissionStage {
        self.stage
    }

    /// Returns the exact owner or qualification cause.
    #[must_use]
    pub fn cause(&self) -> &AdmissionCause {
        self.cause.as_ref()
    }
}

/// Opaque selections authenticated against one exact progress document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoritySelections {
    authority: super::AuthoritySelection,
    subject: SubjectSelection,
    clock: super::clock::Selection,
    source: Identity,
    progress_identity: Identity,
}

impl AuthoritySelections {
    /// Returns the owner definition selection.
    #[must_use]
    pub const fn authority(&self) -> &super::AuthoritySelection {
        &self.authority
    }

    /// Returns the assessment subject selection.
    #[must_use]
    pub const fn subject(&self) -> &SubjectSelection {
        &self.subject
    }

    /// Returns the exact selected clock.
    #[must_use]
    pub const fn clock(&self) -> &super::clock::Selection {
        &self.clock
    }

    /// Returns the selected observation source.
    #[must_use]
    pub const fn source(&self) -> &Identity {
        &self.source
    }

    /// Returns the identity of the authenticated progress envelope.
    #[must_use]
    pub const fn progress_identity(&self) -> &Identity {
        &self.progress_identity
    }

    /// Returns whether this selection was authenticated against `view`.
    #[must_use]
    pub fn binds(&self, view: &super::progress::View) -> bool {
        view.identity() == &self.progress_identity
    }
}

/// One runtime event admitted from an event-input line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedEvent {
    line: u64,
    record: AdmittedRecord,
    reading: u64,
    inputs: Vec<u8>,
}

impl AdmittedEvent {
    /// Returns the one-based event-input line number.
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Returns the exact admitted record carried by this event.
    #[must_use]
    pub const fn record(&self) -> &AdmittedRecord {
        &self.record
    }

    /// Returns the raw counter reading.
    #[must_use]
    pub const fn reading(&self) -> u64 {
        self.reading
    }

    /// Returns the exact UTF-8 bytes of the captures value.
    #[must_use]
    pub fn inputs(&self) -> &[u8] {
        &self.inputs
    }
}

/// One authenticated runtime watermark admitted from an event-input line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedWatermark {
    line: u64,
    view: super::progress::View,
    assertion: Vec<u8>,
    inclusive_boundary: i128,
}

impl AdmittedWatermark {
    /// Returns the one-based event-input line number.
    #[must_use]
    pub const fn line(&self) -> u64 {
        self.line
    }

    /// Returns the strict-read progress view authenticated by this watermark.
    #[must_use]
    pub const fn view(&self) -> &super::progress::View {
        &self.view
    }

    /// Returns the exact progress assertion bytes supplied on the line.
    #[must_use]
    pub fn assertion(&self) -> &[u8] {
        &self.assertion
    }

    /// Returns the canonical inclusive boundary value.
    #[must_use]
    pub const fn inclusive_boundary(&self) -> &i128 {
        &self.inclusive_boundary
    }
}

/// One runtime step retained in event-input line order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdmittedStep {
    /// A runtime event line.
    Event(AdmittedEvent),
    /// A progress watermark line.
    Watermark(AdmittedWatermark),
}

/// Fully authenticated progress authority and ordered runtime lines.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedProgress {
    view: super::progress::View,
    qualified: QualifiedObservation,
    selections: AuthoritySelections,
    steps: Vec<AdmittedStep>,
}

impl AdmittedProgress {
    /// Returns the authenticated progress view.
    #[must_use]
    pub const fn view(&self) -> &super::progress::View {
        &self.view
    }

    /// Returns the observation qualification supporting the view.
    #[must_use]
    pub const fn qualified(&self) -> &QualifiedObservation {
        &self.qualified
    }

    /// Returns the opaque selections authenticated against the view.
    #[must_use]
    pub const fn selections(&self) -> &AuthoritySelections {
        &self.selections
    }

    /// Returns runtime event and watermark steps in line order.
    #[must_use]
    pub fn steps(&self) -> &[AdmittedStep] {
        &self.steps
    }

    /// Splits the authenticated view, its selections, and ordered runtime steps.
    #[must_use]
    pub fn into_parts(
        self,
    ) -> (
        super::progress::View,
        AuthoritySelections,
        Vec<AdmittedStep>,
    ) {
        (self.view, self.selections, self.steps)
    }
}

/// Admits a request, optional event input, qualified observation and progress authority.
pub fn admit_progress(
    request_bytes: &[u8],
    event_bytes: Option<&[u8]>,
    limits: Limits,
) -> std::result::Result<AdmittedProgress, AdmissionError> {
    let effective = limits.effective();
    super::common::preflight(request_bytes, effective).map_err(|error| AdmissionError {
        stage: AdmissionStage::Request,
        cause: Box::new(AdmissionCause::Authority(error)),
    })?;
    if let Some(bytes) = event_bytes {
        if bytes.len() > effective.max_input_bytes {
            return Err(authority_error(
                AdmissionStage::Events,
                ErrorCode::ResourceIncomplete,
                "event input exceeds effective byte limit",
            ));
        }
    }
    let request: RequestWire = decode(request_bytes, AdmissionStage::Request)?;
    if request.contract != REQUEST_CONTRACT {
        return Err(authority_error(
            AdmissionStage::Request,
            ErrorCode::ContractMismatch,
            "request contract does not match",
        ));
    }
    if request.records.is_some() && event_bytes.is_some() {
        return Err(authority_error(
            AdmissionStage::Request,
            ErrorCode::InvalidSelection,
            "records and event input cannot both supply records",
        ));
    }
    let producer = crate::StaticProducerBundle::admit_json(request.producer.get().as_bytes())
        .map_err(|cause| AdmissionError {
            stage: AdmissionStage::Producer,
            cause: Box::new(AdmissionCause::Producer(cause)),
        })?;
    let package: PackageWire = decode(request.package.get().as_bytes(), AdmissionStage::Request)?;
    let binding: BindingWire = decode(request.binding.get().as_bytes(), AdmissionStage::Request)?;
    let expected_subject_wire: SubjectWire = decode(
        request.expected_subject.get().as_bytes(),
        AdmissionStage::Request,
    )?;
    let relationship_wires: Vec<RelationshipWire> = decode(
        request.relationships.get().as_bytes(),
        AdmissionStage::Request,
    )?;
    let required_relationship_wires: Vec<RequiredRelationshipWire> = decode(
        request.required_relationships.get().as_bytes(),
        AdmissionStage::Request,
    )?;
    let scope_wire: ScopeWire = decode(request.scope.get().as_bytes(), AdmissionStage::Request)?;
    let limits_wire: LimitsWire = decode(request.limits.get().as_bytes(), AdmissionStage::Request)?;
    let authority_wire: AuthorityWire =
        decode(request.authority.get().as_bytes(), AdmissionStage::Request)?;
    let subject_wire: SubjectSelectionWire =
        decode(request.subject.get().as_bytes(), AdmissionStage::Request)?;
    let progress_wire: ProgressWire =
        decode(request.progress.get().as_bytes(), AdmissionStage::Request)?;
    let revision = parse_decimal(&request.revision, false).ok_or_else(|| {
        invalid_error(
            AdmissionStage::Request,
            "revision is not a canonical positive integer",
        )
    })?;
    if revision == 0 {
        return Err(invalid_error(
            AdmissionStage::Request,
            "revision must be positive",
        ));
    }
    let package_digest = parse_digest(&package.digest).ok_or_else(|| {
        invalid_error(
            AdmissionStage::Request,
            "package digest is not lowercase SHA-256",
        )
    })?;
    let expected_subject =
        subject_from_wire(expected_subject_wire, &producer).map_err(|cause| AdmissionError {
            stage: AdmissionStage::Request,
            cause: Box::new(cause),
        })?;
    let relationships = relationship_wires
        .into_iter()
        .map(|wire| {
            relationship_from_wire(wire, &producer).map_err(|cause| AdmissionError {
                stage: AdmissionStage::Request,
                cause: Box::new(cause),
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let required_relationships = required_relationship_wires
        .into_iter()
        .map(|wire| {
            required_relationship_from_wire(wire, &producer).map_err(|cause| AdmissionError {
                stage: AdmissionStage::Request,
                cause: Box::new(cause),
            })
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let scope = scope_from_wire(scope_wire).map_err(|cause| AdmissionError {
        stage: AdmissionStage::Request,
        cause: Box::new(cause),
    })?;
    let binding_subject_kind =
        crate::SubjectKind::new(binding.subject_kind).map_err(|_| AdmissionError {
            stage: AdmissionStage::Request,
            cause: Box::new(invalid_cause("binding subject kind is empty")),
        })?;
    let binding = crate::ObservationBinding {
        identity: Identity::new(binding.identity),
        source_identity: Identity::new(binding.source_identity),
        schema_identity: Identity::new(binding.schema_identity),
        signal_identity: Identity::new(binding.signal_identity),
        trigger_identity: Identity::new(binding.trigger_identity),
        unit: Identity::new(binding.unit),
        subject_kind: binding_subject_kind,
        required: binding.required,
    };
    let resource_limits = resource_limits_from_wire(limits_wire)?;
    let (records, pending_lines) = if let Some(raw) = &request.records {
        let wire_records: Vec<RecordWire> = decode(raw.get().as_bytes(), AdmissionStage::Request)?;
        let records = wire_records
            .into_iter()
            .map(|wire| {
                record_from_wire(wire, &producer).map_err(|cause| AdmissionError {
                    stage: AdmissionStage::Request,
                    cause: Box::new(cause),
                })
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        if records.len() > resource_limits.max_records {
            return Err(authority_error(
                AdmissionStage::Request,
                ErrorCode::ResourceIncomplete,
                "record population exceeds request limit",
            ));
        }
        (records, Vec::new())
    } else if event_bytes.is_some() {
        let parsed = decode_event_input(
            event_bytes.unwrap_or_default(),
            &producer,
            resource_limits.max_records,
            effective,
        )?;
        (parsed.records, parsed.lines)
    } else {
        (Vec::new(), Vec::new())
    };
    let request_value = crate::AdmissionRequest {
        package: crate::PackageSelection {
            format: package.format,
            identity: Identity::new(package.identity),
            revision: Identity::new(package.revision),
            digest: package_digest,
        },
        producer,
        binding,
        expected_subject,
        relationships,
        required_relationships,
        scope,
        records,
        limits: resource_limits,
    };
    let authority = AuthoritySelection {
        definition_identity: Identity::new(authority_wire.definition_identity),
        definition_revision: Identity::new(authority_wire.definition_revision),
        definition_digest: parse_digest(&authority_wire.definition_digest).ok_or_else(|| {
            invalid_error(
                AdmissionStage::Request,
                "authority digest is not lowercase SHA-256",
            )
        })?,
    };
    let subject = SubjectSelection {
        scope_identity: Identity::new(subject_wire.scope_identity),
        population_identity: Identity::new(subject_wire.population_identity),
    };
    let progress_wire_for_watermarks = progress_wire.clone();
    let (progress_selection, clock_selection, initial_boundary) =
        progress_selection_from_wire(progress_wire).map_err(|cause| AdmissionError {
            stage: AdmissionStage::Request,
            cause: Box::new(cause),
        })?;
    let initial_observation =
        qualify_with_records(request_value.clone(), request_value.records.clone())?;
    let predecessor = request
        .predecessor_document
        .as_deref()
        .map(|bytes| {
            super::common::document_from_bytes(super::progress::CONTRACT, bytes.as_bytes(), limits)
                .map_err(|error| AdmissionError {
                    stage: AdmissionStage::Progress,
                    cause: Box::new(AdmissionCause::Authority(error)),
                })
        })
        .transpose()?;
    let context = Context::new(
        History::batch(&initial_observation),
        &authority,
        &subject,
        revision,
        predecessor.as_ref(),
    );
    let progress_view = super::progress::read(
        request.progress_document.as_bytes(),
        context,
        &progress_selection,
        limits,
    )
    .map_err(|error| AdmissionError {
        stage: AdmissionStage::Progress,
        cause: Box::new(AdmissionCause::Authority(error)),
    })?;
    if !progress_view
        .payload()
        .required_sources()
        .iter()
        .any(|source| source == &request.observation_source)
        || progress_view.association().is_ok_and(|association| {
            !association
                .rows()
                .iter()
                .any(|row| row.observation_source == request.observation_source)
        })
    {
        return Err(authority_error(
            AdmissionStage::Progress,
            ErrorCode::ExpectedMismatch,
            "observation source is not admitted by progress",
        ));
    }
    let mut steps = Vec::new();
    let mut frontier = None;
    let mut previous_watermark_view = progress_view.clone();
    for pending in pending_lines {
        match pending {
            PendingLine::Record { line, record } => {
                if let Some(boundary) = frontier {
                    require_record_after_frontier(&record, boundary, line)?;
                }
            }
            PendingLine::Event {
                line,
                record,
                reading,
                inputs,
            } => {
                if let Some(boundary) = frontier {
                    require_record_after_frontier(&record, boundary, line)?;
                }
                steps.push(AdmittedStep::Event(AdmittedEvent {
                    line,
                    record,
                    reading,
                    inputs,
                }));
            }
            PendingLine::Watermark { line, wire } => {
                let boundary_wire = wire.boundary.clone();
                let boundary =
                    boundary_from_wire(boundary_wire.clone()).map_err(|cause| AdmissionError {
                        stage: AdmissionStage::Event { line },
                        cause: Box::new(cause),
                    })?;
                let inclusive_boundary =
                    parse_decimal(&wire.inclusive_boundary, true).ok_or_else(|| {
                        invalid_error(
                            AdmissionStage::Event { line },
                            "inclusive boundary is not canonical",
                        )
                    })?;
                if inclusive_boundary != boundary_upper(boundary) {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ExpectedMismatch,
                        "inclusive boundary differs from its asserted boundary",
                    ));
                }
                if let Some(previous) = frontier {
                    if !boundary_is_at_least(boundary, previous) {
                        return Err(authority_error(
                            AdmissionStage::Event { line },
                            ErrorCode::ExpectedMismatch,
                            "watermark regresses or changes clock family",
                        ));
                    }
                } else if !boundary_is_at_least(boundary, initial_boundary) {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ExpectedMismatch,
                        "first watermark regresses the request boundary",
                    ));
                }
                let revision = parse_decimal(&wire.revision, false)
                    .filter(|revision: &u64| *revision > 0)
                    .ok_or_else(|| {
                        invalid_error(
                            AdmissionStage::Event { line },
                            "watermark revision is not canonical positive",
                        )
                    })?;
                let mut watermark_progress = progress_wire_for_watermarks.clone();
                watermark_progress.boundary = boundary_wire;
                watermark_progress.state = wire.state;
                let (selection, _, _) =
                    progress_selection_from_wire(watermark_progress).map_err(|cause| {
                        AdmissionError {
                            stage: AdmissionStage::Event { line },
                            cause: Box::new(cause),
                        }
                    })?;
                let predecessor = previous_watermark_view.document();
                let context = Context::new(
                    History::batch(&initial_observation),
                    &authority,
                    &subject,
                    revision,
                    Some(&predecessor),
                );
                let assertion = wire.progress_document.into_bytes();
                let view = super::progress::read(&assertion, context, &selection, limits).map_err(
                    |error| AdmissionError {
                        stage: AdmissionStage::Event { line },
                        cause: Box::new(AdmissionCause::Authority(error)),
                    },
                )?;
                steps.push(AdmittedStep::Watermark(AdmittedWatermark {
                    line,
                    view: view.clone(),
                    assertion,
                    inclusive_boundary,
                }));
                previous_watermark_view = view;
                frontier = Some(boundary);
            }
        }
    }
    let selections = AuthoritySelections {
        authority,
        subject,
        clock: clock_selection,
        source: Identity::new(request.observation_source),
        progress_identity: progress_view.identity().clone(),
    };
    Ok(AdmittedProgress {
        view: progress_view,
        qualified: initial_observation,
        selections,
        steps,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequestWire {
    contract: String,
    package: Box<RawValue>,
    producer: Box<RawValue>,
    binding: Box<RawValue>,
    expected_subject: Box<RawValue>,
    relationships: Box<RawValue>,
    required_relationships: Box<RawValue>,
    scope: Box<RawValue>,
    limits: Box<RawValue>,
    #[serde(default)]
    records: Option<Box<RawValue>>,
    authority: Box<RawValue>,
    subject: Box<RawValue>,
    revision: String,
    #[serde(default, deserialize_with = "present_optional")]
    predecessor_document: Option<String>,
    progress: Box<RawValue>,
    progress_document: String,
    observation_source: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageWire {
    format: String,
    identity: String,
    revision: String,
    digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    identity: String,
    source_identity: String,
    schema_identity: String,
    signal_identity: String,
    trigger_identity: String,
    unit: String,
    subject_kind: Vec<u8>,
    required: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorityWire {
    definition_identity: String,
    definition_revision: String,
    definition_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectSelectionWire {
    scope_identity: String,
    population_identity: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LimitsWire {
    max_records: String,
    max_members: String,
    max_relationships: String,
    max_required_relationships: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ProducerAuthorityWire {
    #[serde(rename = "bundleIdentity")]
    bundle_identity: String,
    #[serde(rename = "bundleRevision")]
    bundle_revision: crate::Revision,
    digest: crate::ProducerDigest,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SubjectWire {
    authority: ProducerAuthorityWire,
    kind: Vec<u8>,
    identity: Vec<u8>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RelationshipWire {
    declaration_identity: String,
    identity: Vec<u8>,
    source: SubjectWire,
    target: SubjectWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RequiredRelationshipWire {
    declaration_identity: String,
    source: SubjectWire,
    target: SubjectWire,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScopeWire {
    population_identity: String,
    membership_rule_identity: String,
    membership_digest: String,
    membership_document: String,
    required_member_identities: Vec<String>,
    observation_sources: Vec<String>,
    completeness_dependencies: Vec<String>,
    progress_dependencies: Vec<String>,
    clock_identity: String,
    clock_revision: String,
    membership_complete: bool,
    #[serde(deserialize_with = "required_option")]
    closure_identity: Option<String>,
    #[serde(deserialize_with = "required_option")]
    closure_digest: Option<String>,
    kind: ScopeKindWire,
    range: RangeWire,
    members: Vec<MemberWire>,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum ScopeKindWire {
    Snapshot { snapshot_identity: String },
    Window { window_identity: String },
}

#[derive(Deserialize)]
#[serde(tag = "clock_family", rename_all = "kebab-case", deny_unknown_fields)]
enum RangeWire {
    EventPosition {
        start: String,
        end_exclusive: String,
    },
    FixedSample {
        start: String,
        end_exclusive: String,
        epoch_nanos: String,
        period_nanos: String,
    },
    #[serde(rename = "timestamped-event")]
    Timestamp {
        start_nanos: String,
        end_nanos: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberWire {
    object_identity: String,
    record_identity: String,
    anchor: AnchorWire,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum AnchorWire {
    EventPosition {
        position: String,
    },
    FixedSample {
        index: String,
        epoch_nanos: String,
        period_nanos: String,
    },
    Timestamp {
        instant_nanos: String,
    },
}

#[derive(Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum ValueWire {
    Present {
        value_type: String,
        canonical_value: String,
    },
    Missing,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum VisibilityWire {
    Internal,
    External,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordWire {
    identity: String,
    binding_identity: String,
    source_identity: String,
    schema_identity: String,
    subject: SubjectWire,
    signal_identity: String,
    trigger_identity: String,
    unit: String,
    value: ValueWire,
    visibility: VisibilityWire,
    anchor: AnchorWire,
    event_time_nanos: String,
    ingestion_time_nanos: String,
    #[serde(deserialize_with = "required_option")]
    causal_relationship_identity: Option<Vec<u8>>,
    clock_identity: String,
    clock_revision: String,
    clock_uncertainty_nanos: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProgressWire {
    clock: ClockSelectionWire,
    required_sources: Vec<String>,
    boundary: BoundarySelectionWire,
    state: OpenClosed,
    captured_trigger_identity: String,
    cutoff: CutoffWire,
    restoration_basis_identity: String,
    #[serde(default, deserialize_with = "present_optional")]
    association: Option<Vec<super::progress::AssociationRow>>,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockSelectionWire {
    clock_identity: String,
    clock_revision: String,
}

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct CutoffWire {
    clock_identity: String,
    clock_revision: String,
    instant_nanos: String,
    rule: CutoffRuleWire,
    rule_revision: String,
}

#[derive(Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum CutoffRuleWire {
    IngestionTimeAtOrBefore,
}

#[derive(Clone, Deserialize)]
#[serde(tag = "clock_family", rename_all = "kebab-case", deny_unknown_fields)]
enum BoundarySelectionWire {
    EventPosition {
        lower: String,
        upper_inclusive: String,
        carrier_end_exclusive: String,
        watermark: String,
    },
    FixedSample {
        lower: String,
        upper_inclusive: String,
        carrier_end_exclusive: String,
        watermark: String,
    },
    TimestampedEvent {
        lower_nanos: String,
        upper_inclusive_nanos: String,
        carrier_end_exclusive_nanos: String,
        watermark_nanos: String,
    },
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeEventWire {
    contract: String,
    record: Box<RawValue>,
    reading: String,
    inputs: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuntimeWatermarkWire {
    contract: String,
    revision: String,
    boundary: BoundarySelectionWire,
    state: OpenClosed,
    inclusive_boundary: String,
    progress_document: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RecordCandidateLineWire {
    contract: String,
    identity: String,
    binding_identity: String,
    source_identity: String,
    schema_identity: String,
    subject: SubjectWire,
    signal_identity: String,
    trigger_identity: String,
    unit: String,
    value: ValueWire,
    visibility: VisibilityWire,
    anchor: AnchorWire,
    event_time_nanos: String,
    ingestion_time_nanos: String,
    #[serde(deserialize_with = "required_option")]
    causal_relationship_identity: Option<Vec<u8>>,
    clock_identity: String,
    clock_revision: String,
    clock_uncertainty_nanos: String,
}

enum PendingLine {
    Record {
        line: u64,
        record: AdmittedRecord,
    },
    Event {
        line: u64,
        record: AdmittedRecord,
        reading: u64,
        inputs: Vec<u8>,
    },
    Watermark {
        line: u64,
        wire: RuntimeWatermarkWire,
    },
}

struct ParsedEvents {
    records: Vec<AdmittedRecord>,
    lines: Vec<PendingLine>,
}

impl RecordCandidateLineWire {
    fn into_record(self) -> RecordWire {
        RecordWire {
            identity: self.identity,
            binding_identity: self.binding_identity,
            source_identity: self.source_identity,
            schema_identity: self.schema_identity,
            subject: self.subject,
            signal_identity: self.signal_identity,
            trigger_identity: self.trigger_identity,
            unit: self.unit,
            value: self.value,
            visibility: self.visibility,
            anchor: self.anchor,
            event_time_nanos: self.event_time_nanos,
            ingestion_time_nanos: self.ingestion_time_nanos,
            causal_relationship_identity: self.causal_relationship_identity,
            clock_identity: self.clock_identity,
            clock_revision: self.clock_revision,
            clock_uncertainty_nanos: self.clock_uncertainty_nanos,
        }
    }
}

fn authority_error(stage: AdmissionStage, code: ErrorCode, detail: &'static str) -> AdmissionError {
    AdmissionError {
        stage,
        cause: Box::new(AdmissionCause::Authority(Error::new(
            code,
            detail,
            super::Usage::default(),
        ))),
    }
}

fn required_option<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::deserialize(deserializer)
}

fn present_optional<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

fn decode<T: for<'de> Deserialize<'de>>(
    bytes: &[u8],
    stage: AdmissionStage,
) -> std::result::Result<T, AdmissionError> {
    serde_json::from_slice(bytes).map_err(|error| {
        let code = match error.classify() {
            serde_json::error::Category::Syntax | serde_json::error::Category::Eof => {
                ErrorCode::InvalidJson
            }
            serde_json::error::Category::Io => ErrorCode::InvalidJson,
            serde_json::error::Category::Data => ErrorCode::InvalidSelection,
        };
        authority_error(
            stage,
            code,
            "input does not match the closed admission shape",
        )
    })
}

fn parse_decimal<T: std::str::FromStr>(text: &str, signed: bool) -> Option<T> {
    if text.is_empty()
        || text.starts_with('+')
        || (!signed && text.starts_with('-'))
        || (text.len() > 1 && text.starts_with('0'))
        || (text.starts_with('-') && (text.len() == 1 || text.as_bytes().get(1) == Some(&b'0')))
    {
        return None;
    }
    text.parse().ok()
}

fn parse_digest(text: &str) -> Option<crate::Digest> {
    if text.len() != 64
        || !text
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return None;
    }
    let mut bytes = [0; 32];
    for (index, slot) in bytes.iter_mut().enumerate() {
        let start = index.checked_mul(2)?;
        *slot = u8::from_str_radix(text.get(start..start + 2)?, 16).ok()?;
    }
    Some(crate::Digest::new(bytes))
}

fn subject_from_wire(
    wire: SubjectWire,
    producer: &crate::AdmittedStaticBundle,
) -> std::result::Result<crate::QualifiedSubject, AdmissionCause> {
    let key = producer.key();
    let selected = wire.authority;
    if selected.bundle_identity != key.bundle_identity
        || selected.bundle_revision != key.bundle_revision
        || selected.digest != key.digest
    {
        return Err(AdmissionCause::Authority(Error::new(
            ErrorCode::InvalidSelection,
            "runtime subject names another producer authority",
            super::Usage::default(),
        )));
    }
    let kind = crate::SubjectKind::new(wire.kind).map_err(|_| {
        AdmissionCause::Authority(Error::new(
            ErrorCode::InvalidSelection,
            "runtime subject kind is empty",
            super::Usage::default(),
        ))
    })?;
    let identity = crate::SubjectIdentity::new(wire.identity).map_err(|_| {
        AdmissionCause::Authority(Error::new(
            ErrorCode::InvalidSelection,
            "runtime subject identity is empty",
            super::Usage::default(),
        ))
    })?;
    Ok(crate::QualifiedSubject::new(producer, kind, identity))
}

fn relationship_from_wire(
    wire: RelationshipWire,
    producer: &crate::AdmittedStaticBundle,
) -> std::result::Result<crate::Relationship, AdmissionCause> {
    let source = subject_from_wire(wire.source, producer)?;
    let target = subject_from_wire(wire.target, producer)?;
    let identity = crate::RelationshipIdentity::new(wire.identity).map_err(|_| {
        AdmissionCause::Authority(Error::new(
            ErrorCode::InvalidSelection,
            "runtime relationship identity is empty",
            super::Usage::default(),
        ))
    })?;
    crate::Relationship::new(
        producer,
        wire.declaration_identity,
        identity,
        source,
        target,
    )
    .map_err(|_| {
        AdmissionCause::Authority(Error::new(
            ErrorCode::InvalidSelection,
            "runtime relationship does not match the producer declaration",
            super::Usage::default(),
        ))
    })
}

fn required_relationship_from_wire(
    wire: RequiredRelationshipWire,
    producer: &crate::AdmittedStaticBundle,
) -> std::result::Result<crate::RequiredRelationship, AdmissionCause> {
    let source = subject_from_wire(wire.source, producer)?;
    let target = subject_from_wire(wire.target, producer)?;
    crate::RequiredRelationship::new(producer, wire.declaration_identity, source, target).map_err(
        |_| {
            AdmissionCause::Authority(Error::new(
                ErrorCode::InvalidSelection,
                "required relationship does not match the producer declaration",
                super::Usage::default(),
            ))
        },
    )
}

fn record_from_wire(
    wire: RecordWire,
    producer: &crate::AdmittedStaticBundle,
) -> std::result::Result<AdmittedRecord, AdmissionCause> {
    let value = match wire.value {
        ValueWire::Present {
            value_type,
            canonical_value,
        } => crate::ValueState::Present {
            value_type: Identity::new(value_type),
            canonical_value,
        },
        ValueWire::Missing => crate::ValueState::Missing,
    };
    let visibility = match wire.visibility {
        VisibilityWire::Internal => crate::Visibility::Internal,
        VisibilityWire::External => crate::Visibility::External,
    };
    let anchor = match wire.anchor {
        AnchorWire::EventPosition { position } => {
            let Some(position) = parse_decimal(&position, false) else {
                return Err(invalid_cause("event position is not canonical"));
            };
            crate::Anchor::EventPosition(position)
        }
        AnchorWire::FixedSample {
            index,
            epoch_nanos,
            period_nanos,
        } => {
            let (Some(index), Some(epoch_nanos), Some(period_nanos)) = (
                parse_decimal(&index, false),
                parse_decimal(&epoch_nanos, true),
                parse_decimal(&period_nanos, false),
            ) else {
                return Err(invalid_cause("fixed-sample anchor is not canonical"));
            };
            crate::Anchor::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            }
        }
        AnchorWire::Timestamp { instant_nanos } => {
            let Some(instant_nanos) = parse_decimal(&instant_nanos, true) else {
                return Err(invalid_cause("timestamp anchor is not canonical"));
            };
            crate::Anchor::TimestampNanos(instant_nanos)
        }
    };
    let (Some(event_time_nanos), Some(ingestion_time_nanos), Some(clock_uncertainty_nanos)) = (
        parse_decimal(&wire.event_time_nanos, true),
        parse_decimal(&wire.ingestion_time_nanos, true),
        parse_decimal(&wire.clock_uncertainty_nanos, false),
    ) else {
        return Err(invalid_cause("record integer is not canonical"));
    };
    let subject = subject_from_wire(wire.subject, producer)?;
    Ok(AdmittedRecord {
        identity: Identity::new(wire.identity),
        binding_identity: Identity::new(wire.binding_identity),
        source_identity: Identity::new(wire.source_identity),
        schema_identity: Identity::new(wire.schema_identity),
        subject,
        signal_identity: Identity::new(wire.signal_identity),
        trigger_identity: Identity::new(wire.trigger_identity),
        unit: Identity::new(wire.unit),
        value,
        visibility,
        anchor,
        event_time_nanos,
        ingestion_time_nanos,
        causal_relationship_identity: wire
            .causal_relationship_identity
            .map(crate::RelationshipIdentity::new)
            .transpose()
            .map_err(|_| invalid_cause("causal relationship identity is empty"))?,
        clock_identity: Identity::new(wire.clock_identity),
        clock_revision: Identity::new(wire.clock_revision),
        clock_uncertainty_nanos,
    })
}

fn invalid_cause(detail: &'static str) -> AdmissionCause {
    AdmissionCause::Authority(Error::new(
        ErrorCode::InvalidSelection,
        detail,
        super::Usage::default(),
    ))
}

fn scope_from_wire(wire: ScopeWire) -> std::result::Result<crate::ScopeSelection, AdmissionCause> {
    let kind = match wire.kind {
        ScopeKindWire::Snapshot { snapshot_identity } => crate::ScopeKind::Snapshot {
            snapshot_identity: Identity::new(snapshot_identity),
        },
        ScopeKindWire::Window { window_identity } => crate::ScopeKind::Window {
            window_identity: Identity::new(window_identity),
        },
    };
    let range = match wire.range {
        RangeWire::EventPosition {
            start,
            end_exclusive,
        } => {
            let (Some(start), Some(end_exclusive)) = (
                parse_decimal(&start, false),
                parse_decimal(&end_exclusive, false),
            ) else {
                return Err(invalid_cause("event-position range is not canonical"));
            };
            crate::ClockRange::EventPosition {
                start,
                end_exclusive,
            }
        }
        RangeWire::FixedSample {
            start,
            end_exclusive,
            epoch_nanos,
            period_nanos,
        } => {
            let (Some(start), Some(end_exclusive), Some(epoch_nanos), Some(period_nanos)) = (
                parse_decimal(&start, false),
                parse_decimal(&end_exclusive, false),
                parse_decimal(&epoch_nanos, true),
                parse_decimal(&period_nanos, false),
            ) else {
                return Err(invalid_cause("fixed-sample range is not canonical"));
            };
            crate::ClockRange::FixedSample {
                start,
                end_exclusive,
                epoch_nanos,
                period_nanos,
            }
        }
        RangeWire::Timestamp {
            start_nanos,
            end_nanos,
        } => {
            let (Some(start_nanos), Some(end_nanos)) = (
                parse_decimal(&start_nanos, true),
                parse_decimal(&end_nanos, true),
            ) else {
                return Err(invalid_cause("timestamp range is not canonical"));
            };
            crate::ClockRange::Timestamp {
                start_nanos,
                end_nanos,
            }
        }
    };
    let closure_identity = wire.closure_identity.map(Identity::new);
    let closure_digest = match wire.closure_digest.as_deref() {
        Some(text) => Some(
            parse_digest(text)
                .ok_or_else(|| invalid_cause("closure digest is not lowercase SHA-256"))?,
        ),
        None => None,
    };
    let members = wire
        .members
        .into_iter()
        .map(|member| {
            Ok(crate::Member {
                object_identity: Identity::new(member.object_identity),
                record_identity: Identity::new(member.record_identity),
                anchor: anchor_from_wire(member.anchor)?,
            })
        })
        .collect::<std::result::Result<Vec<_>, AdmissionCause>>()?;
    Ok(crate::ScopeSelection {
        population_identity: Identity::new(wire.population_identity),
        membership_rule_identity: Identity::new(wire.membership_rule_identity),
        membership_digest: parse_digest(&wire.membership_digest)
            .ok_or_else(|| invalid_cause("membership digest is not lowercase SHA-256"))?,
        membership_document: wire.membership_document.into_bytes(),
        required_member_identities: wire
            .required_member_identities
            .into_iter()
            .map(Identity::new)
            .collect(),
        observation_sources: wire
            .observation_sources
            .into_iter()
            .map(Identity::new)
            .collect(),
        completeness_dependencies: wire
            .completeness_dependencies
            .into_iter()
            .map(Identity::new)
            .collect(),
        progress_dependencies: wire
            .progress_dependencies
            .into_iter()
            .map(Identity::new)
            .collect(),
        clock_identity: Identity::new(wire.clock_identity),
        clock_revision: Identity::new(wire.clock_revision),
        membership_complete: wire.membership_complete,
        closure_identity,
        closure_digest,
        kind,
        range,
        members,
    })
}

fn anchor_from_wire(wire: AnchorWire) -> std::result::Result<crate::Anchor, AdmissionCause> {
    match wire {
        AnchorWire::EventPosition { position } => parse_decimal(&position, false)
            .map(crate::Anchor::EventPosition)
            .ok_or_else(|| invalid_cause("event position is not canonical")),
        AnchorWire::FixedSample {
            index,
            epoch_nanos,
            period_nanos,
        } => {
            let (Some(index), Some(epoch_nanos), Some(period_nanos)) = (
                parse_decimal(&index, false),
                parse_decimal(&epoch_nanos, true),
                parse_decimal(&period_nanos, false),
            ) else {
                return Err(invalid_cause("fixed-sample anchor is not canonical"));
            };
            Ok(crate::Anchor::FixedSample {
                index,
                epoch_nanos,
                period_nanos,
            })
        }
        AnchorWire::Timestamp { instant_nanos } => parse_decimal(&instant_nanos, true)
            .map(crate::Anchor::TimestampNanos)
            .ok_or_else(|| invalid_cause("timestamp anchor is not canonical")),
    }
}

fn boundary_from_wire(
    wire: BoundarySelectionWire,
) -> std::result::Result<TemporalBoundary, AdmissionCause> {
    match wire {
        BoundarySelectionWire::EventPosition {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => {
            let (Some(lower), Some(upper_inclusive), Some(carrier_end_exclusive), Some(watermark)) = (
                parse_decimal(&lower, false),
                parse_decimal(&upper_inclusive, false),
                parse_decimal(&carrier_end_exclusive, false),
                parse_decimal(&watermark, false),
            ) else {
                return Err(invalid_cause("event-position boundary is not canonical"));
            };
            Ok(TemporalBoundary::EventPosition {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            })
        }
        BoundarySelectionWire::FixedSample {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => {
            let (Some(lower), Some(upper_inclusive), Some(carrier_end_exclusive), Some(watermark)) = (
                parse_decimal(&lower, false),
                parse_decimal(&upper_inclusive, false),
                parse_decimal(&carrier_end_exclusive, false),
                parse_decimal(&watermark, false),
            ) else {
                return Err(invalid_cause("fixed-sample boundary is not canonical"));
            };
            Ok(TemporalBoundary::FixedSample {
                lower,
                upper_inclusive,
                carrier_end_exclusive,
                watermark,
            })
        }
        BoundarySelectionWire::TimestampedEvent {
            lower_nanos,
            upper_inclusive_nanos,
            carrier_end_exclusive_nanos,
            watermark_nanos,
        } => {
            let (
                Some(lower_nanos),
                Some(upper_inclusive_nanos),
                Some(carrier_end_exclusive_nanos),
                Some(watermark_nanos),
            ) = (
                parse_decimal(&lower_nanos, true),
                parse_decimal(&upper_inclusive_nanos, true),
                parse_decimal(&carrier_end_exclusive_nanos, true),
                parse_decimal(&watermark_nanos, true),
            )
            else {
                return Err(invalid_cause("timestamp boundary is not canonical"));
            };
            Ok(TemporalBoundary::TimestampedEvent {
                lower_nanos,
                upper_inclusive_nanos,
                carrier_end_exclusive_nanos,
                watermark_nanos,
            })
        }
    }
}

fn progress_selection_from_wire(
    wire: ProgressWire,
) -> std::result::Result<
    (
        super::progress::Selection,
        super::clock::Selection,
        TemporalBoundary,
    ),
    AdmissionCause,
> {
    let clock = super::clock::Selection::new(
        Identity::new(wire.clock.clock_identity),
        Identity::new(wire.clock.clock_revision),
    );
    let cutoff = match wire.cutoff.rule {
        CutoffRuleWire::IngestionTimeAtOrBefore => {
            super::observation::CutoffRule::IngestionTimeAtOrBefore
        }
    };
    let Some(instant_nanos) = parse_decimal(&wire.cutoff.instant_nanos, true) else {
        return Err(invalid_cause("cutoff instant is not canonical"));
    };
    if wire.cutoff.clock_identity != clock.clock_identity().as_str()
        || wire.cutoff.clock_revision != clock.clock_revision().as_str()
    {
        return Err(invalid_cause("cutoff clock is cross-wired"));
    }
    let cutoff = super::observation::CutoffSelection::new(
        Identity::new(wire.cutoff.clock_identity),
        Identity::new(wire.cutoff.clock_revision),
        instant_nanos,
        cutoff,
        Identity::new(wire.cutoff.rule_revision),
    );
    let boundary = boundary_from_wire(wire.boundary)?;
    let mut selection = super::progress::Selection::new(
        clock.clone(),
        wire.required_sources
            .into_iter()
            .map(Identity::new)
            .collect(),
        boundary,
        wire.state,
        Identity::new(wire.captured_trigger_identity),
        cutoff,
        Identity::new(wire.restoration_basis_identity),
    );
    if let Some(association) = wire.association {
        selection = selection.with_association(association);
    }
    Ok((selection, clock, boundary))
}

fn qualify_with_records(
    mut request: crate::AdmissionRequest,
    records: Vec<AdmittedRecord>,
) -> std::result::Result<QualifiedObservation, AdmissionError> {
    request.records = records;
    match crate::admit(request) {
        crate::AdmissionOutcome::Available { observation } => Ok(*observation),
        other => Err(AdmissionError {
            stage: AdmissionStage::Qualification,
            cause: Box::new(AdmissionCause::Observation(other)),
        }),
    }
}

fn boundary_upper(boundary: TemporalBoundary) -> i128 {
    match boundary {
        TemporalBoundary::EventPosition {
            upper_inclusive, ..
        }
        | TemporalBoundary::FixedSample {
            upper_inclusive, ..
        } => i128::from(upper_inclusive),
        TemporalBoundary::TimestampedEvent {
            upper_inclusive_nanos,
            ..
        } => upper_inclusive_nanos,
    }
}

fn boundary_family(boundary: TemporalBoundary) -> u8 {
    match boundary {
        TemporalBoundary::EventPosition { .. } => 0,
        TemporalBoundary::FixedSample { .. } => 1,
        TemporalBoundary::TimestampedEvent { .. } => 2,
    }
}

fn boundary_is_at_least(successor: TemporalBoundary, prior: TemporalBoundary) -> bool {
    boundary_family(successor) == boundary_family(prior)
        && boundary_upper(successor) >= boundary_upper(prior)
}

fn require_record_after_frontier(
    record: &AdmittedRecord,
    frontier: TemporalBoundary,
    line: u64,
) -> std::result::Result<(), AdmissionError> {
    let position = match (frontier, record.anchor) {
        (TemporalBoundary::EventPosition { .. }, crate::Anchor::EventPosition(position)) => {
            i128::from(position)
        }
        (TemporalBoundary::FixedSample { .. }, crate::Anchor::FixedSample { index, .. }) => {
            i128::from(index)
        }
        (TemporalBoundary::TimestampedEvent { .. }, crate::Anchor::TimestampNanos(instant)) => {
            instant
        }
        _ => {
            return Err(authority_error(
                AdmissionStage::Event { line },
                ErrorCode::ExpectedMismatch,
                "record clock family differs from watermark frontier",
            ));
        }
    };
    if position <= boundary_upper(frontier) {
        return Err(authority_error(
            AdmissionStage::Event { line },
            ErrorCode::ExpectedMismatch,
            "record is at or below watermark frontier",
        ));
    }
    Ok(())
}

fn invalid_error(stage: AdmissionStage, detail: &'static str) -> AdmissionError {
    authority_error(stage, ErrorCode::InvalidSelection, detail)
}

fn resource_limits_from_wire(
    wire: LimitsWire,
) -> std::result::Result<crate::ResourceLimits, AdmissionError> {
    let max_records = parse_decimal(&wire.max_records, false)
        .ok_or_else(|| invalid_error(AdmissionStage::Request, "max_records is not canonical"))?;
    let max_members = parse_decimal(&wire.max_members, false)
        .ok_or_else(|| invalid_error(AdmissionStage::Request, "max_members is not canonical"))?;
    let max_relationships = parse_decimal(&wire.max_relationships, false).ok_or_else(|| {
        invalid_error(
            AdmissionStage::Request,
            "max_relationships is not canonical",
        )
    })?;
    let max_required_relationships = parse_decimal(&wire.max_required_relationships, false)
        .ok_or_else(|| {
            invalid_error(
                AdmissionStage::Request,
                "max_required_relationships is not canonical",
            )
        })?;
    Ok(crate::ResourceLimits {
        max_records,
        max_members,
        max_relationships,
        max_required_relationships,
    })
}

fn decode_event_input(
    bytes: &[u8],
    producer: &crate::AdmittedStaticBundle,
    max_records: usize,
    limits: Limits,
) -> std::result::Result<ParsedEvents, AdmissionError> {
    let mut records = Vec::new();
    let mut lines = Vec::new();
    if bytes.is_empty() {
        return Ok(ParsedEvents { records, lines });
    }
    for (index, raw_line) in bytes.split(|byte| *byte == b'\n').enumerate() {
        if raw_line.is_empty() {
            if index + 1 == bytes.split(|byte| *byte == b'\n').count() && bytes.ends_with(b"\n") {
                continue;
            }
            return Err(authority_error(
                AdmissionStage::Event {
                    line: u64::try_from(index + 1).unwrap_or(u64::MAX),
                },
                ErrorCode::InvalidJson,
                "event input contains an empty line",
            ));
        }
        let line = u64::try_from(index + 1).unwrap_or(u64::MAX);
        super::common::preflight(raw_line, limits).map_err(|error| AdmissionError {
            stage: AdmissionStage::Event { line },
            cause: Box::new(AdmissionCause::Authority(error)),
        })?;
        let value: serde_json::Value = serde_json::from_slice(raw_line).map_err(|_| {
            authority_error(
                AdmissionStage::Event { line },
                ErrorCode::InvalidJson,
                "event line is not one JSON value",
            )
        })?;
        let contract = value.get("contract").and_then(serde_json::Value::as_str);
        match contract {
            Some(RECORD_CONTRACT) => {
                let wire: RecordCandidateLineWire =
                    decode(raw_line, AdmissionStage::Event { line })?;
                if wire.contract != RECORD_CONTRACT {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ContractMismatch,
                        "record line contract does not match",
                    ));
                }
                let record = event_record(
                    wire.into_record(),
                    producer,
                    max_records,
                    &mut records,
                    line,
                )?;
                lines.push(PendingLine::Record { line, record });
            }
            Some(EVENT_CONTRACT) => {
                let wire: RuntimeEventWire = decode(raw_line, AdmissionStage::Event { line })?;
                if wire.contract != EVENT_CONTRACT {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ContractMismatch,
                        "runtime event contract does not match",
                    ));
                }
                let record_line: RecordCandidateLineWire =
                    decode(wire.record.get().as_bytes(), AdmissionStage::Event { line })?;
                if record_line.contract != RECORD_CONTRACT {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ContractMismatch,
                        "runtime event record contract does not match",
                    ));
                }
                let reading = parse_decimal(&wire.reading, false).ok_or_else(|| {
                    invalid_error(
                        AdmissionStage::Event { line },
                        "runtime reading is not canonical",
                    )
                })?;
                let record =
                    record_from_wire(record_line.into_record(), producer).map_err(|cause| {
                        AdmissionError {
                            stage: AdmissionStage::Event { line },
                            cause: Box::new(cause),
                        }
                    })?;
                push_existing_event_record(&mut records, record.clone(), max_records, line)?;
                lines.push(PendingLine::Event {
                    line,
                    record,
                    reading,
                    inputs: wire.inputs.into_bytes(),
                });
            }
            Some(WATERMARK_CONTRACT) => {
                let wire: RuntimeWatermarkWire = decode(raw_line, AdmissionStage::Event { line })?;
                if wire.contract != WATERMARK_CONTRACT {
                    return Err(authority_error(
                        AdmissionStage::Event { line },
                        ErrorCode::ContractMismatch,
                        "watermark line contract does not match",
                    ));
                }
                lines.push(PendingLine::Watermark { line, wire });
            }
            _ => {
                return Err(authority_error(
                    AdmissionStage::Event { line },
                    ErrorCode::ContractMismatch,
                    "event line names an unsupported contract",
                ));
            }
        }
    }
    Ok(ParsedEvents { records, lines })
}

fn event_record(
    wire: RecordWire,
    producer: &crate::AdmittedStaticBundle,
    max_records: usize,
    records: &mut Vec<AdmittedRecord>,
    line: u64,
) -> std::result::Result<AdmittedRecord, AdmissionError> {
    let record = record_from_wire(wire, producer).map_err(|cause| AdmissionError {
        stage: AdmissionStage::Event { line },
        cause: Box::new(cause),
    })?;
    push_existing_event_record(records, record.clone(), max_records, line)?;
    Ok(record)
}

fn push_existing_event_record(
    records: &mut Vec<AdmittedRecord>,
    record: AdmittedRecord,
    max_records: usize,
    line: u64,
) -> std::result::Result<(), AdmissionError> {
    if records.len() >= max_records {
        return Err(authority_error(
            AdmissionStage::Event { line },
            ErrorCode::ResourceIncomplete,
            "event record population exceeds request limit",
        ));
    }
    records.push(record);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    #[trace("TC-016", "FR-014-AC-13")]
    #[test]
    fn canonical_decimal_parser_rejects_noncanonical_and_overflow_values() {
        assert_eq!(parse_decimal::<u64>("0", false), Some(0));
        assert_eq!(
            parse_decimal::<u64>("18446744073709551615", false),
            Some(u64::MAX)
        );
        assert_eq!(parse_decimal::<u64>("18446744073709551616", false), None);
        assert_eq!(parse_decimal::<u64>("01", false), None);
        assert_eq!(parse_decimal::<i128>("-1", true), Some(-1));
        assert_eq!(parse_decimal::<i128>("-0", true), None);
        assert_eq!(parse_decimal::<i128>("+1", true), None);
    }

    #[trace("TC-016", "FR-014-AC-11")]
    #[test]
    fn watermark_boundary_order_is_family_local_and_inclusive() {
        let at_ten = TemporalBoundary::EventPosition {
            lower: 0,
            upper_inclusive: 10,
            carrier_end_exclusive: 11,
            watermark: 11,
        };
        let at_eleven = TemporalBoundary::EventPosition {
            lower: 0,
            upper_inclusive: 11,
            carrier_end_exclusive: 12,
            watermark: 12,
        };
        let other_family = TemporalBoundary::FixedSample {
            lower: 0,
            upper_inclusive: 11,
            carrier_end_exclusive: 12,
            watermark: 12,
        };
        assert!(boundary_is_at_least(at_ten, at_ten));
        assert!(boundary_is_at_least(at_eleven, at_ten));
        assert!(!boundary_is_at_least(at_ten, at_eleven));
        assert!(!boundary_is_at_least(other_family, at_ten));
    }

    #[trace("TC-016", "FR-014-AC-13")]
    #[test]
    fn every_admission_wire_schema_is_closed_and_selects_its_contract() {
        for (bytes, contract) in [
            (REQUEST_SCHEMA_BYTES, REQUEST_CONTRACT),
            (RECORD_SCHEMA_BYTES, RECORD_CONTRACT),
            (EVENT_SCHEMA_BYTES, EVENT_CONTRACT),
            (WATERMARK_SCHEMA_BYTES, WATERMARK_CONTRACT),
        ] {
            let schema: serde_json::Value =
                serde_json::from_slice(bytes).expect("admission schema is JSON");
            assert_eq!(schema["additionalProperties"], false);
            assert_eq!(schema["properties"]["contract"]["const"], contract);
            let required = schema["required"]
                .as_array()
                .expect("admission schema has required fields");
            let properties = schema["properties"]
                .as_object()
                .expect("admission schema has properties");
            assert!(required.iter().all(|name| {
                name.as_str()
                    .is_some_and(|name| properties.contains_key(name))
            }));
        }
    }
}
