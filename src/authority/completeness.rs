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
    cause: Box<RevisionCause>,
}

impl RevisionError {
    /// Returns the exact revision stage that refused.
    #[must_use]
    pub const fn stage(&self) -> RevisionStage {
        self.stage
    }

    /// Returns the exact owner error or monotonicity contradiction.
    #[must_use]
    pub fn cause(&self) -> &RevisionCause {
        self.cause.as_ref()
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
    let prior_state = prior.payload().state;
    let prior_upper = boundary_upper(prior_boundary);
    let successor_upper = boundary_upper(selection.boundary);
    // FR-013 check 1 runs before the selection is validated. The successor
    // state is not known yet, so the prior's own state stands in and check 2
    // cannot fire here; it runs after `derive` below.
    check_revision(
        Some(prior_upper),
        prior_state,
        prior,
        Some(successor_upper),
        prior_state,
        &[],
    )
    .map_err(|found| revision_failure(found, prior_boundary, selection.boundary, &[]))?;
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
    let mut validation_limits = limits;
    validation_limits.max_population_entries = required_members.len();
    let selected_document = derive(validation_context, &coverage_selection, validation_limits)
        .map_err(|error| revision_authority(RevisionStage::Selection, error))?;
    let selected_view = read(
        selected_document.bytes(),
        validation_context,
        &coverage_selection,
        validation_limits,
    )
    .map_err(|error| revision_authority(RevisionStage::Selection, error))?;
    // FR-013 check 2: the state `derive` gives the selection's own facts.
    check_revision(
        Some(prior_upper),
        prior_state,
        prior,
        Some(successor_upper),
        selection_state(&selection.facts),
        &[],
    )
    .map_err(|found| revision_failure(found, prior_boundary, selection.boundary, &[]))?;
    // FR-013 check 3a (population) stays here: it reads the successor's
    // required population, which `check_revision` does not see.
    if let Some(member_identity) = missing_unheld.into_iter().next() {
        return Err(revision_contradiction(Contradiction::UncoveredMember {
            member_identity,
            prior: prior_boundary,
        }));
    }
    let mut listed_facts: Vec<&Fact> = Vec::with_capacity(selection.facts.len());
    let mut listed: Vec<ListedFact<'_, str>> = Vec::with_capacity(selection.facts.len());
    let mut pending: Option<RevisionError> = None;
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
            pending = Some(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness observation is absent from qualified history",
                    Usage::default(),
                ),
            ));
            break;
        };
        let Some(placed) = anchor_coordinate(record.anchor, selection.boundary) else {
            pending = Some(revision_authority(
                RevisionStage::Selection,
                Error::new(
                    ErrorCode::ExpectedMismatch,
                    "selected completeness member uses another clock family",
                    Usage::default(),
                ),
            ));
            break;
        };
        listed_facts.push(fact);
        listed.push(ListedFact {
            key: fact.member_identity.as_str(),
            status: fact.status,
            observation_identity: Some(observation_identity.as_str()),
            placed,
        });
    }
    let outcome = check_revision(
        Some(prior_upper),
        prior_state,
        prior,
        Some(successor_upper),
        selection_state(&selection.facts),
        &listed,
    );
    if let Some(error) = pending {
        // An authority error on a later fact is reached only after every
        // earlier fact passed the uncovered-member check; check 4 follows it.
        return Err(match outcome {
            Err(found @ ListedContradiction::UncoveredMember { .. }) => {
                revision_failure(found, prior_boundary, selection.boundary, &listed_facts)
            }
            _ => error,
        });
    }
    outcome.map_err(|found| {
        revision_failure(found, prior_boundary, selection.boundary, &listed_facts)
    })?;
    let mut revised_payload = selected_view.payload().clone();
    revised_payload.basis = Basis::Revised;
    revised_payload.facts = merge_facts(&prior.payload().facts, &selection.facts, required_members);
    revised_payload.state = aggregate_state(&revised_payload.facts);
    let usage = completeness_usage(&revised_payload);
    let revision_context = context.allow_predecessor_subject_change();
    build_document(CONTRACT, revision_context, &revised_payload, usage, limits)
        .map_err(|error| revision_authority(RevisionStage::Document, error))
}

/// One fact a retained assessment holds, as [`check_revision`] reads it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HeldFact<'a> {
    /// The held fact's status.
    pub status: FactStatus,
    /// The held fact's observation identity, when it has one.
    pub observation_identity: Option<&'a str>,
}

/// A retained assessment's held facts, keyed by member.
pub trait RetainedHolding {
    /// The member key the holding is looked up by.
    type Key: ?Sized;

    /// Returns the fact held for `key`, whatever its status, or `None`.
    fn held(&self, key: &Self::Key) -> Option<HeldFact<'_>>;
}

impl RetainedHolding for View {
    type Key = str;

    fn held(&self, key: &str) -> Option<HeldFact<'_>> {
        self.payload()
            .facts
            .iter()
            .find(|fact| fact.member_identity == key)
            .map(|fact| HeldFact {
                status: fact.status,
                observation_identity: fact.observation_identity.as_deref(),
            })
    }
}

/// One member a successor lists, with where its record is placed.
#[derive(Clone, Copy, Debug)]
pub struct ListedFact<'a, K: ?Sized> {
    /// The member key.
    pub key: &'a K,
    /// The listed status; it governs only checks 3 and 4.
    pub status: FactStatus,
    /// The listed observation identity, when it has one.
    pub observation_identity: Option<&'a str>,
    /// The member's placement on the boundary's coordinate.
    pub placed: i128,
}

/// One FR-013 contradiction found by [`check_revision`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ListedContradiction {
    /// The successor's upper bound is below the prior's.
    BoundaryRegressed {
        /// The retained upper bound.
        prior_upper: i128,
        /// The successor's upper bound.
        successor_upper: i128,
    },
    /// A complete prior became incomplete.
    CompleteRegressed {
        /// The retained upper bound, when the prior states one.
        prior_upper: Option<i128>,
    },
    /// The listed member at this index is placed at or before the prior's
    /// bound and the prior does not hold it.
    UncoveredMember {
        /// Index into the listed facts.
        listed: usize,
    },
    /// The listed member at this index is held `Available` by both with
    /// another observation identity.
    MemberObservationChanged {
        /// Index into the listed facts.
        listed: usize,
        /// The held observation identity the listed one differs from.
        prior_observation: Option<String>,
    },
}

/// Checks a successor against a retained assessment without any history.
///
/// Runs FR-013 checks 1, 2, 3 (listed members) and 4, in that order, and
/// returns the first contradiction. A `None` upper bound places nothing at or
/// before it. States are read as given; per-fact status governs only checks 3
/// and 4.
pub fn check_revision<H: RetainedHolding>(
    prior_upper: Option<i128>,
    prior_state: State,
    prior: &H,
    successor_upper: Option<i128>,
    successor_state: State,
    listed: &[ListedFact<'_, H::Key>],
) -> std::result::Result<(), ListedContradiction> {
    if let (Some(prior_upper), Some(successor_upper)) = (prior_upper, successor_upper) {
        if successor_upper < prior_upper {
            return Err(ListedContradiction::BoundaryRegressed {
                prior_upper,
                successor_upper,
            });
        }
    }
    if prior_state == State::Complete && successor_state == State::Incomplete {
        return Err(ListedContradiction::CompleteRegressed { prior_upper });
    }
    let counts = |fact: &ListedFact<'_, H::Key>| {
        matches!(
            fact.status,
            FactStatus::Available | FactStatus::Contradicted
        )
    };
    for (index, fact) in listed.iter().enumerate() {
        if counts(fact)
            && prior_upper.is_some_and(|upper| fact.placed <= upper)
            && prior.held(fact.key).is_none()
        {
            return Err(ListedContradiction::UncoveredMember { listed: index });
        }
    }
    for (index, fact) in listed.iter().enumerate() {
        if fact.status != FactStatus::Available {
            continue;
        }
        if let Some(held) = prior.held(fact.key) {
            if held.status == FactStatus::Available
                && held.observation_identity != fact.observation_identity
            {
                return Err(ListedContradiction::MemberObservationChanged {
                    listed: index,
                    prior_observation: held.observation_identity.map(str::to_owned),
                });
            }
        }
    }
    Ok(())
}

fn revision_failure(
    found: ListedContradiction,
    prior_boundary: TemporalBoundary,
    successor_boundary: TemporalBoundary,
    listed: &[&Fact],
) -> RevisionError {
    let inconsistent = || {
        revision_authority(
            RevisionStage::Revision,
            Error::new(
                ErrorCode::ExpectedMismatch,
                "revision check named a listed fact or observation that does not exist",
                Usage::default(),
            ),
        )
    };
    let contradiction = match found {
        ListedContradiction::BoundaryRegressed { .. } => Contradiction::BoundaryRegressed {
            prior: prior_boundary,
            successor: successor_boundary,
        },
        ListedContradiction::CompleteRegressed { .. } => Contradiction::CompleteRegressed {
            prior: prior_boundary,
        },
        ListedContradiction::UncoveredMember { listed: index } => {
            let Some(fact) = listed.get(index) else {
                return inconsistent();
            };
            Contradiction::UncoveredMember {
                member_identity: fact.member_identity.clone(),
                prior: prior_boundary,
            }
        }
        ListedContradiction::MemberObservationChanged {
            listed: index,
            prior_observation,
        } => {
            let Some(fact) = listed.get(index) else {
                return inconsistent();
            };
            let (Some(prior_observation), Some(successor_observation)) =
                (prior_observation, fact.observation_identity.clone())
            else {
                return inconsistent();
            };
            Contradiction::MemberObservationChanged {
                member_identity: fact.member_identity.clone(),
                prior_observation: Identity::new(prior_observation),
                successor_observation,
            }
        }
    };
    revision_contradiction(contradiction)
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

/// Builds the revised population: one fact per member of `population`, in its
/// order (the merged selected fact, or the prior's carried), then each prior
/// fact for a member outside `population`, in the prior's order, carried.
fn merge_facts(prior: &[FactWire], selected: &[Fact], population: &[Identity]) -> Vec<FactWire> {
    let prior_of = |member: &str| prior.iter().find(|fact| fact.member_identity == member);
    let mut merged = Vec::with_capacity(prior.len().max(population.len()));
    for member in population {
        let selected_fact = selected.iter().find(|fact| fact.member_identity == *member);
        let prior_fact = prior_of(member.as_str());
        let wire = match (selected_fact, prior_fact) {
            (Some(selected_fact), prior_fact) => {
                let selected_wire = FactWire {
                    member_identity: member.as_str().to_owned(),
                    observation_identity: selected_fact
                        .observation_identity
                        .as_ref()
                        .map(|identity| identity.as_str().to_owned()),
                    status: selected_fact.status,
                    origin: FactOrigin::Selected,
                };
                match prior_fact {
                    Some(prior_fact)
                        if fact_rank(prior_fact.status) > fact_rank(selected_fact.status) =>
                    {
                        FactWire {
                            status: prior_fact.status,
                            observation_identity: prior_fact.observation_identity.clone(),
                            ..selected_wire
                        }
                    }
                    _ => selected_wire,
                }
            }
            (None, Some(prior_fact)) => FactWire {
                origin: FactOrigin::Carried,
                ..prior_fact.clone()
            },
            (None, None) => continue,
        };
        merged.push(wire);
    }
    merged.extend(
        prior
            .iter()
            .filter(|fact| {
                !population
                    .iter()
                    .any(|member| member.as_str() == fact.member_identity)
            })
            .map(|fact| FactWire {
                origin: FactOrigin::Carried,
                ..fact.clone()
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
        cause: Box::new(RevisionCause::Authority(error)),
    }
}

fn revision_contradiction(contradiction: Contradiction) -> RevisionError {
    RevisionError {
        stage: RevisionStage::Revision,
        cause: Box::new(RevisionCause::Contradiction(contradiction)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ix_trace_rs::trace;

    struct Held(Vec<(&'static str, FactStatus, Option<&'static str>)>);

    impl RetainedHolding for Held {
        type Key = str;

        fn held(&self, key: &str) -> Option<HeldFact<'_>> {
            self.0
                .iter()
                .find(|(member, ..)| *member == key)
                .map(|(_, status, observation)| HeldFact {
                    status: *status,
                    observation_identity: *observation,
                })
        }
    }

    fn listed<'a>(
        key: &'a str,
        status: FactStatus,
        observation: Option<&'a str>,
        placed: i128,
    ) -> ListedFact<'a, str> {
        ListedFact {
            key,
            status,
            observation_identity: observation,
            placed,
        }
    }

    fn available(
        member: &'static str,
        observation: &'static str,
    ) -> (&'static str, FactStatus, Option<&'static str>) {
        (member, FactStatus::Available, Some(observation))
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn check_revision_refuses_each_contradiction_in_order() {
        let prior = Held(vec![available("m0", "a")]);
        assert_eq!(
            check_revision(
                Some(100),
                State::Complete,
                &prior,
                Some(60),
                State::Incomplete,
                &[]
            ),
            Err(ListedContradiction::BoundaryRegressed {
                prior_upper: 100,
                successor_upper: 60
            })
        );
        assert_eq!(
            check_revision(
                Some(100),
                State::Complete,
                &prior,
                Some(100),
                State::Incomplete,
                &[]
            ),
            Err(ListedContradiction::CompleteRegressed {
                prior_upper: Some(100)
            })
        );
        let uncovered = [listed("m1", FactStatus::Available, Some("b"), 80)];
        assert_eq!(
            check_revision(
                Some(100),
                State::Complete,
                &prior,
                Some(100),
                State::Complete,
                &uncovered
            ),
            Err(ListedContradiction::UncoveredMember { listed: 0 })
        );
        let changed = [listed("m0", FactStatus::Available, Some("b"), 0)];
        assert_eq!(
            check_revision(
                Some(100),
                State::Complete,
                &prior,
                Some(100),
                State::Complete,
                &changed
            ),
            Err(ListedContradiction::MemberObservationChanged {
                listed: 0,
                prior_observation: Some("a".to_owned())
            })
        );
        let after = [listed("m2", FactStatus::Available, Some("c"), 101)];
        assert_eq!(
            check_revision(
                Some(100),
                State::Complete,
                &prior,
                Some(140),
                State::Complete,
                &after
            ),
            Ok(())
        );
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn check_revision_none_upper_places_nothing_at_or_before_it() {
        let prior = Held(vec![]);
        let stamped = [listed("m0", FactStatus::Available, Some("a"), i128::MIN)];
        assert_eq!(
            check_revision(
                None,
                State::Incomplete,
                &prior,
                Some(5),
                State::Complete,
                &stamped
            ),
            Ok(())
        );
        assert_eq!(
            check_revision(
                None,
                State::Incomplete,
                &prior,
                None,
                State::Incomplete,
                &stamped
            ),
            Ok(())
        );
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn check_revision_accepts_stamp_zero_after_an_incomplete_prior() {
        let prior = Held(vec![]);
        let zero = [listed("m0", FactStatus::Available, Some("a"), 0)];
        assert_eq!(
            check_revision(
                None,
                State::Incomplete,
                &prior,
                Some(10),
                State::Complete,
                &zero
            ),
            Ok(())
        );
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn check_revision_refuses_changed_observation_after_an_incomplete_prior() {
        let prior = Held(vec![available("m0", "a")]);
        let changed = [listed("m0", FactStatus::Available, Some("b"), 0)];
        assert_eq!(
            check_revision(
                None,
                State::Incomplete,
                &prior,
                Some(10),
                State::Incomplete,
                &changed
            ),
            Err(ListedContradiction::MemberObservationChanged {
                listed: 0,
                prior_observation: Some("a".to_owned())
            })
        );
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn check_revision_reads_some_and_none_observation_identity_as_different() {
        let prior = Held(vec![available("m0", "")]);
        let none = [listed("m0", FactStatus::Available, None, 0)];
        assert_eq!(
            check_revision(
                Some(10),
                State::Complete,
                &prior,
                Some(10),
                State::Complete,
                &none
            ),
            Err(ListedContradiction::MemberObservationChanged {
                listed: 0,
                prior_observation: Some(String::new())
            })
        );
        let same = [listed("m0", FactStatus::Available, Some(""), 0)];
        assert_eq!(
            check_revision(
                Some(10),
                State::Complete,
                &prior,
                Some(10),
                State::Complete,
                &same
            ),
            Ok(())
        );
    }

    #[trace("TC-015", "FR-013-AC-8")]
    #[test]
    fn merge_facts_orders_the_population_then_carries_prior_facts_outside_it() {
        let wire = |member: &str, origin| FactWire {
            member_identity: member.to_owned(),
            observation_identity: Some("a".to_owned()),
            status: FactStatus::Available,
            origin,
        };
        let prior = [
            wire("out1", FactOrigin::Selected),
            wire("b", FactOrigin::Selected),
            wire("out2", FactOrigin::Selected),
        ];
        let selected = [
            Fact::new(
                Identity::new("c"),
                Some(Identity::new("a")),
                FactStatus::Available,
            ),
            Fact::new(
                Identity::new("a"),
                Some(Identity::new("a")),
                FactStatus::Available,
            ),
        ];
        let population = ["a", "b", "c"].map(Identity::new);
        let merged = merge_facts(&prior, &selected, &population);
        let order = merged
            .iter()
            .map(|fact| (fact.member_identity.as_str(), fact.origin))
            .collect::<Vec<_>>();
        assert_eq!(
            order,
            [
                ("a", FactOrigin::Selected),
                ("b", FactOrigin::Carried),
                ("c", FactOrigin::Selected),
                ("out1", FactOrigin::Carried),
                ("out2", FactOrigin::Carried),
            ]
        );
    }

    #[test]
    fn schema_digest_is_pinned() {
        assert_eq!(common::schema_sha256(SCHEMA_BYTES), SCHEMA_SHA256);
        common::assert_closed_schema(SCHEMA_BYTES, CONTRACT);
    }
}
