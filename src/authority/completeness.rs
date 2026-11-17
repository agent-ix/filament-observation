// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! `filament.observation.completeness-assertion/v1` owner artifact.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use super::common;
use super::common::{boundary_wire, build_document, read_exact, BoundaryWire, Validated};
use super::{Context, Document, Error, ErrorCode, Result, TemporalBoundary, Usage};
use crate::Identity;

pub use super::Limits;

/// Immutable owner-contract label.
pub const CONTRACT: &str = "filament.observation.completeness-assertion/v1";
/// Pinned JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/observation-completeness-assertion-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "5e907500423366083f3b70a7761aa7e2e3d21ae347acfa8cdbd4cee6fea1a433";

/// Per-member input completeness status.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactStatus {
    /// The exact required observation is available.
    Available,
    /// The required observation is absent or incomplete.
    Incomplete,
    /// An admitted correction contradicts the retained fact.
    Contradicted,
}

/// Closed FR-287 aggregate completeness vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum State {
    /// Every exact required member is available.
    Complete,
    /// At least one required member is incomplete and none is contradicted.
    Incomplete,
    /// At least one required fact is contradicted.
    Contradicted,
}

/// Whether the assessment was derived directly or revised from a retained view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Basis {
    /// The document was derived from its selected facts.
    Derived,
    /// The document was revised from a retained completeness view.
    Revised,
}

/// Whether one fact came from the current selection or a retained prior view.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FactOrigin {
    /// The fact is present in the current selection.
    Selected,
    /// The fact was retained from the prior view because the selection omitted it.
    Carried,
}

/// One exact required-member completeness input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fact {
    member_identity: Identity,
    observation_identity: Option<Identity>,
    status: FactStatus,
}

impl Fact {
    /// Constructs one explicit member status.
    #[must_use]
    pub const fn new(
        member_identity: Identity,
        observation_identity: Option<Identity>,
        status: FactStatus,
    ) -> Self {
        Self {
            member_identity,
            observation_identity,
            status,
        }
    }
}

/// Exact boundary and complete per-member fact selection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selection {
    boundary_identity: Identity,
    boundary: TemporalBoundary,
    facts: Vec<Fact>,
}

impl Selection {
    /// Constructs a completeness selection; derivation validates exact coverage.
    #[must_use]
    pub const fn new(
        boundary_identity: Identity,
        boundary: TemporalBoundary,
        facts: Vec<Fact>,
    ) -> Self {
        Self {
            boundary_identity,
            boundary,
            facts,
        }
    }
}

/// Borrowed per-member fact exposed by a validated completeness view.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FactRef<'a> {
    /// Exact required-member identity.
    pub member_identity: &'a str,
    /// Exact admitted observation identity, when present.
    pub observation_identity: Option<&'a str>,
    /// Owner-derived per-member status.
    pub status: FactStatus,
    /// Whether this fact was selected or carried from a prior revision.
    pub origin: FactOrigin,
}

/// Read-only payload produced only inside a validated completeness view.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompletenessView {
    population_identity: String,
    boundary_identity: String,
    #[serde(
        serialize_with = "serialize_boundary",
        deserialize_with = "deserialize_boundary"
    )]
    boundary: TemporalBoundary,
    basis: Basis,
    facts: Vec<FactWire>,
    state: State,
}

impl CompletenessView {
    /// Returns the exact FR-263 population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Returns the exact selected boundary identity.
    #[must_use]
    pub fn boundary_identity(&self) -> &str {
        &self.boundary_identity
    }

    /// Returns the exact temporal boundary asserted by this view.
    #[must_use]
    pub fn boundary(&self) -> TemporalBoundary {
        self.boundary
    }

    /// Returns whether the assessment was directly derived or revised.
    #[must_use]
    pub const fn basis(&self) -> Basis {
        self.basis
    }

    /// Iterates the complete per-member fact population in canonical order.
    pub fn facts(&self) -> impl ExactSizeIterator<Item = FactRef<'_>> {
        self.facts.iter().map(|fact| FactRef {
            member_identity: &fact.member_identity,
            observation_identity: fact.observation_identity.as_deref(),
            status: fact.status,
            origin: fact.origin,
        })
    }

    /// Returns the owner-derived aggregate state.
    #[must_use]
    pub const fn state(&self) -> State {
        self.state
    }

    /// Returns the exact number of required-member facts.
    #[must_use]
    pub fn fact_count(&self) -> usize {
        self.facts.len()
    }
}

/// Constructor-private validated completeness view.
pub type View = Validated<CompletenessView>;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FactWire {
    member_identity: String,
    observation_identity: Option<String>,
    status: FactStatus,
    origin: FactOrigin,
}

fn serialize_boundary<S>(
    boundary: &TemporalBoundary,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    boundary_wire(*boundary).serialize(serializer)
}

fn deserialize_boundary<'de, D>(deserializer: D) -> std::result::Result<TemporalBoundary, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let wire = BoundaryWire::deserialize(deserializer)?;
    let invalid = || serde::de::Error::custom("completeness boundary integer is not canonical");
    match wire {
        BoundaryWire::EventPosition {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => {
            let text = [lower, upper_inclusive, carrier_end_exclusive, watermark];
            let mut values = [0_u64; 4];
            for (value, text) in values.iter_mut().zip(text) {
                *value = text.parse().map_err(|_| invalid())?;
                if value.to_string() != text {
                    return Err(invalid());
                }
            }
            Ok(TemporalBoundary::EventPosition {
                lower: values[0],
                upper_inclusive: values[1],
                carrier_end_exclusive: values[2],
                watermark: values[3],
            })
        }
        BoundaryWire::FixedSample {
            lower,
            upper_inclusive,
            carrier_end_exclusive,
            watermark,
        } => {
            let text = [lower, upper_inclusive, carrier_end_exclusive, watermark];
            let mut values = [0_u64; 4];
            for (value, text) in values.iter_mut().zip(text) {
                *value = text.parse().map_err(|_| invalid())?;
                if value.to_string() != text {
                    return Err(invalid());
                }
            }
            Ok(TemporalBoundary::FixedSample {
                lower: values[0],
                upper_inclusive: values[1],
                carrier_end_exclusive: values[2],
                watermark: values[3],
            })
        }
        BoundaryWire::TimestampedEvent {
            lower_nanos,
            upper_inclusive_nanos,
            carrier_end_exclusive_nanos,
            watermark_nanos,
        } => {
            let text = [
                lower_nanos,
                upper_inclusive_nanos,
                carrier_end_exclusive_nanos,
                watermark_nanos,
            ];
            let mut values = [0_i128; 4];
            for (value, text) in values.iter_mut().zip(text) {
                *value = text.parse().map_err(|_| invalid())?;
                if value.to_string() != text {
                    return Err(invalid());
                }
            }
            Ok(TemporalBoundary::TimestampedEvent {
                lower_nanos: values[0],
                upper_inclusive_nanos: values[1],
                carrier_end_exclusive_nanos: values[2],
                watermark_nanos: values[3],
            })
        }
    }
}

/// Derives canonical bounded completeness-assertion owner bytes.
pub fn derive(context: Context<'_>, selection: &Selection, limits: Limits) -> Result<Document> {
    let effective = limits.effective();
    if !selection.boundary_identity.valid() {
        return Err(Error::new(
            ErrorCode::InvalidSelection,
            "completeness boundary identity must be explicit",
            Usage::default(),
        ));
    }
    selection.boundary.validate(super::OpenClosed::Open)?;
    if selection.facts.len() > effective.max_population_entries {
        return Err(Error::new(
            ErrorCode::ResourceIncomplete,
            "completeness population exceeds effective limit",
            Usage {
                population_entries: selection.facts.len(),
                ..Usage::default()
            },
        ));
    }
    let qualified = context.history().qualified();
    if selection.facts.len() != qualified.scope().required_member_identities.len()
        || selection
            .facts
            .iter()
            .map(|fact| &fact.member_identity)
            .ne(qualified.scope().required_member_identities.iter())
    {
        return Err(Error::new(
            ErrorCode::ExpectedMismatch,
            "completeness facts must exactly cover the required population",
            Usage::default(),
        ));
    }
    let records = qualified.records();
    for fact in &selection.facts {
        if !fact.member_identity.valid()
            || fact
                .observation_identity
                .as_ref()
                .is_some_and(|identity| !identity.valid())
        {
            return Err(Error::new(
                ErrorCode::InvalidSelection,
                "completeness fact contains an absent identity",
                Usage::default(),
            ));
        }
        let selected_record = qualified
            .scope()
            .members
            .iter()
            .find(|member| member.object_identity == fact.member_identity)
            .map(|member| &member.record_identity);
        let observed = fact.observation_identity.as_ref().is_some_and(|identity| {
            selected_record == Some(identity)
                && records.iter().any(|record| record.identity == *identity)
        });
        match fact.status {
            FactStatus::Available if !observed => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "available completeness fact lacks a qualified observation",
                    Usage::default(),
                ));
            }
            FactStatus::Incomplete if fact.observation_identity.is_some() => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "incomplete completeness fact cannot name an available observation",
                    Usage::default(),
                ));
            }
            FactStatus::Contradicted if !observed => {
                return Err(Error::new(
                    ErrorCode::ExpectedMismatch,
                    "contradiction must retain its admitted correction fact",
                    Usage::default(),
                ));
            }
            _ => {}
        }
    }
    let state = if selection
        .facts
        .iter()
        .any(|fact| fact.status == FactStatus::Contradicted)
    {
        State::Contradicted
    } else if selection
        .facts
        .iter()
        .any(|fact| fact.status == FactStatus::Incomplete)
    {
        State::Incomplete
    } else {
        State::Complete
    };
    let payload = CompletenessView {
        population_identity: qualified.scope().population_identity.as_str().to_owned(),
        boundary_identity: selection.boundary_identity.as_str().to_owned(),
        boundary: selection.boundary,
        basis: Basis::Derived,
        facts: selection
            .facts
            .iter()
            .map(|fact| FactWire {
                member_identity: fact.member_identity.as_str().to_owned(),
                observation_identity: fact
                    .observation_identity
                    .as_ref()
                    .map(|value| value.as_str().to_owned()),
                status: fact.status,
                origin: FactOrigin::Selected,
            })
            .collect(),
        state,
    };
    let usage = Usage {
        depth: 5,
        string_bytes: payload
            .facts
            .iter()
            .flat_map(|fact| {
                [
                    fact.member_identity.len(),
                    fact.observation_identity.as_ref().map_or(0, String::len),
                ]
            })
            .max()
            .unwrap_or(0)
            .max(payload.population_identity.len())
            .max(payload.boundary_identity.len()),
        population_entries: payload.facts.len(),
        visited_fields: 22usize.saturating_add(payload.facts.len().saturating_mul(4)),
        ..Usage::default()
    };
    build_document(CONTRACT, context, &payload, usage, limits)
}

/// Strict-reads canonical completeness bytes against independent selections.
pub fn read(
    bytes: &[u8],
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> Result<View> {
    let expected = derive(context, selection, limits)?;
    read_exact(CONTRACT, bytes, &expected, limits)
}

/// Stage where a completeness revision refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RevisionStage {
    /// The retained prior or its correction context is invalid.
    Prior,
    /// The successor selection is invalid.
    Selection,
    /// A valid selection contradicts monotonic completeness.
    Revision,
    /// Revised document bytes fail recomputation.
    Document,
}

/// Exact monotonicity contradiction between two completeness assertions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Contradiction {
    /// The inclusive boundary moved backward.
    BoundaryRegressed {
        /// Retained prior boundary.
        prior: TemporalBoundary,
        /// Proposed successor boundary.
        successor: TemporalBoundary,
    },
    /// A complete assessment would become incomplete.
    CompleteRegressed {
        /// Boundary of the retained complete assertion.
        prior: TemporalBoundary,
    },
    /// A member inside the retained boundary was not held before.
    UncoveredMember {
        /// Newly held member identity.
        member_identity: Identity,
        /// Boundary of the retained assertion.
        prior: TemporalBoundary,
    },
    /// A member held available by both assertions changed observation identity.
    MemberObservationChanged {
        /// Changed member identity.
        member_identity: Identity,
        /// Previously retained observation identity.
        prior_observation: Identity,
        /// Newly selected observation identity.
        successor_observation: Identity,
    },
}

impl Contradiction {
    /// Returns the stable machine-readable contradiction label.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::BoundaryRegressed { .. } => "QOBS-REV-BOUNDARY-REGRESSED",
            Self::CompleteRegressed { .. } => "QOBS-REV-COMPLETE-REGRESSED",
            Self::UncoveredMember { .. } => "QOBS-REV-UNCOVERED-MEMBER",
            Self::MemberObservationChanged { .. } => "QOBS-REV-MEMBER-OBSERVATION-CHANGED",
        }
    }
}

/// Typed error returned by completeness revision and revised-document reads.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
#[error("completeness revision refused at {stage:?}: {cause:?}")]
pub struct RevisionError {
    stage: RevisionStage,
    cause: RevisionCause,
}

impl RevisionError {
    /// Returns the exact revision stage that refused.
    #[must_use]
    pub const fn stage(&self) -> RevisionStage {
        self.stage
    }

    /// Returns the exact owner error or monotonicity contradiction.
    #[must_use]
    pub const fn cause(&self) -> &RevisionCause {
        &self.cause
    }
}

/// Owner failure or a specific completeness monotonicity contradiction.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevisionCause {
    /// A common owner validation or strict reader error.
    Authority(Error),
    /// One explicit completeness-revision contradiction.
    Contradiction(Contradiction),
}

/// Derives a monotonic completeness correction without changing `prior`.
pub fn revise(
    prior: &View,
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> std::result::Result<Document, RevisionError> {
    let Some(predecessor) = context.predecessor() else {
        return Err(revision_authority(
            RevisionStage::Prior,
            Error::new(
                ErrorCode::PredecessorMismatch,
                "revision context does not name the retained prior",
                Usage::default(),
            ),
        ));
    };
    if predecessor.identity() != prior.identity()
        || predecessor.contract() != CONTRACT
        || predecessor.authority() != prior.authority()
        || context.authority() != prior.authority()
    {
        return Err(revision_authority(
            RevisionStage::Prior,
            Error::new(
                ErrorCode::PredecessorMismatch,
                "revision context does not name the retained prior authority",
                Usage::default(),
            ),
        ));
    }
    if context.revision() <= prior.revision() {
        return Err(revision_authority(
            RevisionStage::Prior,
            Error::new(
                ErrorCode::RevisionMismatch,
                "successor revision is not greater than the retained prior",
                Usage::default(),
            ),
        ));
    }
    let prior_boundary = prior.payload().boundary();
    selection
        .boundary
        .validate(super::OpenClosed::Open)
        .map_err(|error| revision_authority(RevisionStage::Selection, error))?;
    if !same_boundary_family(selection.boundary, prior_boundary) {
        return Err(revision_authority(
            RevisionStage::Selection,
            Error::new(
                ErrorCode::ExpectedMismatch,
                "successor completeness boundary uses another clock family",
                Usage::default(),
            ),
        ));
    }
    let validation_context = Context::new(
        context.history(),
        context.authority(),
        context.subject(),
        1,
        None,
    );
    if boundary_upper(selection.boundary) < boundary_upper(prior_boundary) {
        return Err(revision_contradiction(Contradiction::BoundaryRegressed {
            prior: prior_boundary,
            successor: selection.boundary,
        }));
    }
    let qualified = context.history().qualified();
    let required_members = &qualified.scope().required_member_identities;
    let mut previous_index = None;
    for fact in &selection.facts {
        let Some(index) = required_members
            .iter()
            .position(|member| member == &fact.member_identity)
        else {
            return Err(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness member is outside the successor population",
                    Usage::default(),
                ),
            ));
        };
        if previous_index.is_some_and(|previous| index <= previous) {
            return Err(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness members are not in population order",
                    Usage::default(),
                ),
            ));
        }
        previous_index = Some(index);
    }
    let mut missing_unheld = Vec::new();
    let mut selection_index = 0;
    let mut coverage_facts = Vec::with_capacity(required_members.len());
    for member in required_members {
        if selection
            .facts
            .get(selection_index)
            .is_some_and(|fact| fact.member_identity == *member)
        {
            coverage_facts.push(selection.facts[selection_index].clone());
            selection_index += 1;
        } else {
            let prior_fact = prior
                .payload()
                .facts
                .iter()
                .find(|fact| fact.member_identity == member.as_str());
            if prior_fact.is_none() {
                missing_unheld.push(member.clone());
            }
            // A placeholder lets derive validate the supplied subset against
            // the successor population. The retained fact is merged below.
            coverage_facts.push(Fact::new(member.clone(), None, FactStatus::Incomplete));
        }
    }
    let coverage_selection = Selection::new(
        selection.boundary_identity.clone(),
        selection.boundary,
        coverage_facts,
    );
    let selected_document = derive(validation_context, &coverage_selection, limits)
        .map_err(|error| revision_authority(RevisionStage::Selection, error))?;
    let selected_view = read(
        selected_document.bytes(),
        validation_context,
        &coverage_selection,
        limits,
    )
    .map_err(|error| revision_authority(RevisionStage::Selection, error))?;
    if prior.payload().state == State::Complete
        && selection_state(&selection.facts) == State::Incomplete
    {
        return Err(revision_contradiction(Contradiction::CompleteRegressed {
            prior: prior_boundary,
        }));
    }
    if let Some(member_identity) = missing_unheld.into_iter().next() {
        return Err(revision_contradiction(Contradiction::UncoveredMember {
            member_identity,
            prior: prior_boundary,
        }));
    }
    for fact in &selection.facts {
        let Some(observation_identity) = &fact.observation_identity else {
            continue;
        };
        if fact.status != FactStatus::Available && fact.status != FactStatus::Contradicted {
            continue;
        }
        let record = qualified
            .records()
            .iter()
            .find(|record| &record.identity == observation_identity);
        let Some(record) = record else {
            return Err(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness observation is absent from qualified history",
                    Usage::default(),
                ),
            ));
        };
        let Some(placed) = anchor_coordinate(record.anchor, selection.boundary) else {
            return Err(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness member uses another clock family",
                    Usage::default(),
                ),
            ));
        };
        let was_held = prior
            .payload()
            .facts
            .iter()
            .any(|prior_fact| prior_fact.member_identity == fact.member_identity.as_str());
        if !was_held && placed <= boundary_upper(prior_boundary) {
            return Err(revision_contradiction(Contradiction::UncoveredMember {
                member_identity: fact.member_identity.clone(),
                prior: prior_boundary,
            }));
        }
    }
    for fact in &selection.facts {
        let Some(prior_fact) = prior
            .payload()
            .facts
            .iter()
            .find(|prior_fact| prior_fact.member_identity == fact.member_identity.as_str())
        else {
            continue;
        };
        if prior_fact.status == FactStatus::Available && fact.status == FactStatus::Available {
            if let (Some(prior_observation), Some(successor_observation)) = (
                prior_fact.observation_identity.as_deref(),
                fact.observation_identity.as_ref(),
            ) {
                if prior_observation != successor_observation.as_str() {
                    return Err(revision_contradiction(
                        Contradiction::MemberObservationChanged {
                            member_identity: fact.member_identity.clone(),
                            prior_observation: Identity::new(prior_observation),
                            successor_observation: successor_observation.clone(),
                        },
                    ));
                }
            }
        }
    }
    let mut revised_payload = selected_view.payload().clone();
    revised_payload.basis = Basis::Revised;
    revised_payload.facts = merge_facts(&prior.payload().facts, &selection.facts, required_members);
    revised_payload.state = aggregate_state(&revised_payload.facts);
    let usage = completeness_usage(&revised_payload);
    let revision_context = context.allow_predecessor_subject_change();
    build_document(CONTRACT, revision_context, &revised_payload, usage, limits)
        .map_err(|error| revision_authority(RevisionStage::Document, error))
}

/// Recomputes and strict-reads a revised completeness document.
pub fn read_revised(
    bytes: &[u8],
    prior: &View,
    context: Context<'_>,
    selection: &Selection,
    limits: Limits,
) -> std::result::Result<View, RevisionError> {
    let expected = revise(prior, context, selection, limits)?;
    let view = read_exact(CONTRACT, bytes, &expected, limits)
        .map_err(|error| revision_authority(RevisionStage::Document, error))?;
    Ok(view)
}

fn selection_state(facts: &[Fact]) -> State {
    if facts
        .iter()
        .any(|fact| fact.status == FactStatus::Contradicted)
    {
        State::Contradicted
    } else if facts
        .iter()
        .any(|fact| fact.status == FactStatus::Incomplete)
    {
        State::Incomplete
    } else {
        State::Complete
    }
}

fn merge_facts(prior: &[FactWire], selected: &[Fact], population: &[Identity]) -> Vec<FactWire> {
    let mut merged = selected
        .iter()
        .map(|selected_fact| {
            let prior_fact = prior.iter().find(|prior_fact| {
                prior_fact.member_identity == selected_fact.member_identity.as_str()
            });
            let Some(prior_fact) = prior_fact else {
                return FactWire {
                    member_identity: selected_fact.member_identity.as_str().to_owned(),
                    observation_identity: selected_fact
                        .observation_identity
                        .as_ref()
                        .map(|identity| identity.as_str().to_owned()),
                    status: selected_fact.status,
                    origin: FactOrigin::Selected,
                };
            };
            let (status, observation_identity) =
                if fact_rank(prior_fact.status) > fact_rank(selected_fact.status) {
                    (prior_fact.status, prior_fact.observation_identity.clone())
                } else {
                    (
                        selected_fact.status,
                        selected_fact
                            .observation_identity
                            .as_ref()
                            .map(|identity| identity.as_str().to_owned()),
                    )
                };
            FactWire {
                member_identity: selected_fact.member_identity.as_str().to_owned(),
                observation_identity,
                status,
                origin: FactOrigin::Selected,
            }
        })
        .collect::<Vec<_>>();
    merged.extend(
        prior
            .iter()
            .filter(|prior_fact| {
                population
                    .iter()
                    .any(|member| member.as_str() == prior_fact.member_identity)
                    && !selected
                        .iter()
                        .any(|fact| fact.member_identity.as_str() == prior_fact.member_identity)
            })
            .map(|prior_fact| FactWire {
                origin: FactOrigin::Carried,
                ..prior_fact.clone()
            }),
    );
    merged
}

fn fact_rank(status: FactStatus) -> u8 {
    match status {
        FactStatus::Incomplete => 0,
        FactStatus::Available => 1,
        FactStatus::Contradicted => 2,
    }
}

fn aggregate_state(facts: &[FactWire]) -> State {
    if facts
        .iter()
        .any(|fact| fact.status == FactStatus::Contradicted)
    {
        State::Contradicted
    } else if facts
        .iter()
        .any(|fact| fact.status == FactStatus::Incomplete)
    {
        State::Incomplete
    } else {
        State::Complete
    }
}

fn completeness_usage(payload: &CompletenessView) -> Usage {
    Usage {
        depth: 5,
        string_bytes: payload
            .facts
            .iter()
            .flat_map(|fact| {
                [
                    fact.member_identity.len(),
                    fact.observation_identity.as_ref().map_or(0, String::len),
                ]
            })
            .max()
            .unwrap_or(0)
            .max(payload.population_identity.len())
            .max(payload.boundary_identity.len()),
        population_entries: payload.facts.len(),
        visited_fields: 22usize.saturating_add(payload.facts.len().saturating_mul(4)),
        ..Usage::default()
    }
}

fn same_boundary_family(left: TemporalBoundary, right: TemporalBoundary) -> bool {
    matches!(
        (left, right),
        (
            TemporalBoundary::EventPosition { .. },
            TemporalBoundary::EventPosition { .. }
        ) | (
            TemporalBoundary::FixedSample { .. },
            TemporalBoundary::FixedSample { .. }
        ) | (
            TemporalBoundary::TimestampedEvent { .. },
            TemporalBoundary::TimestampedEvent { .. }
        )
    )
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

fn anchor_coordinate(anchor: crate::Anchor, boundary: TemporalBoundary) -> Option<i128> {
    match (anchor, boundary) {
        (crate::Anchor::EventPosition(position), TemporalBoundary::EventPosition { .. }) => {
            Some(i128::from(position))
        }
        (crate::Anchor::FixedSample { index, .. }, TemporalBoundary::FixedSample { .. }) => {
            Some(i128::from(index))
        }
        (crate::Anchor::TimestampNanos(instant), TemporalBoundary::TimestampedEvent { .. }) => {
            Some(instant)
        }
        _ => None,
    }
}

fn revision_authority(stage: RevisionStage, error: Error) -> RevisionError {
    RevisionError {
        stage,
        cause: RevisionCause::Authority(error),
    }
}

fn revision_contradiction(contradiction: Contradiction) -> RevisionError {
    RevisionError {
        stage: RevisionStage::Revision,
        cause: RevisionCause::Contradiction(contradiction),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
    }
}
