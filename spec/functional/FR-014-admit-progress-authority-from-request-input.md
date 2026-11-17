---
id: FR-014
title: "Admit the authenticated progress view and opaque authority selections from request and event input"
type: FR
relationships:
  - { target: ix://agent-ix/filament-observation/US-001, type: implements }
  - { target: ix://agent-ix/filament-observation/FR-001, type: depends_on }
  - { target: ix://agent-ix/filament-observation/FR-004, type: depends_on }
  - { target: ix://agent-ix/filament-observation/FR-005, type: depends_on }
  - { target: ix://agent-ix/filament-spec-language/FR-251, type: references }
  - { target: ix://agent-ix/filament-spec-language/FR-761, type: references }
---
# FR-014: Admit the authenticated progress view and opaque authority selections from request and event input

## Description

When a caller supplies the bytes of a progress-admission request and,
optionally, the bytes of an event input, the library SHALL return exactly one
admitted progress capability, holding the strict-read progress view and the
opaque observation authority selections that view was authenticated against,
or exactly one typed admission error naming its stage, and no partial
capability.

## Rationale

A runtime monitor built from source is started from a request file and fed
from an event stream. The monitor's plan authority needs the observation
owner's progress view and the independently selected observation subject,
clock and source. Until now these could be built only by Rust callers that
assembled each owner type by hand, so a command-line consumer had no path
from its request and event bytes to the view. This entry is that path. It
reuses the FR-001 admission and the FR-004 strict readers, so it adds no
second set of admission rules and no way to obtain authority other than
the existing readers.

## Interface

The module `authority::admission` SHALL publish:

| Item | Form |
| --- | --- |
| `REQUEST_CONTRACT` | `"filament.observation.progress-admission-request/v1"` |
| `REQUEST_SCHEMA_BYTES` | the JSON Schema bytes of the request contract |
| `RECORD_CONTRACT` | `"filament.observation.record-candidate/v1"` |
| `RECORD_SCHEMA_BYTES` | the JSON Schema bytes of one event-input record |
| `admit_progress(request: &[u8], events: Option<&[u8]>, limits: Limits) -> Result<AdmittedProgress, AdmissionError>` | the one admission entry |
| `AdmittedProgress` | the admitted capability |
| `AuthoritySelections` | the opaque selections the view was authenticated against |
| `AdmissionError { stage: AdmissionStage, cause: AdmissionCause }` | the typed error |
| `AdmissionStage` | `Request`, `Events`, `Event { line: u64 }`, `Producer`, `Qualification`, `Progress` |
| `AdmissionCause` | `Authority(Error)`, `Producer(producer::Refusal)`, `Observation(AdmissionOutcome)` |

`AdmittedProgress` SHALL provide `view(&self) -> &progress::View`,
`selections(&self) -> &AuthoritySelections`,
`qualified(&self) -> &QualifiedObservation` and
`into_parts(self) -> (progress::View, AuthoritySelections)`.
`AuthoritySelections` SHALL provide `authority(&self) -> &AuthoritySelection`,
`subject(&self) -> &SubjectSelection`, `clock(&self) -> &clock::Selection`,
`source(&self) -> &Identity`, `progress_identity(&self) -> &Identity` (the
envelope identity of the progress document it authenticated) and
`binds(&self, view: &progress::View) -> bool`, true exactly when `view` is
that document's view. Every accessor borrows, runs in constant time and
allocates nothing.

The entry takes bytes only. It opens no file, reads no standard input,
environment or clock, and performs no network access: a command-line consumer
reads its request file and its event path or standard input and passes the
bytes.

## Inputs

### The request

The request is one closed JSON object with exactly these members, in any
order:

| Member | Content |
| --- | --- |
| `contract` | exactly `REQUEST_CONTRACT` |
| `package` | `{format, identity, revision, digest}`, the FR-001 package selection |
| `producer` | the Producer interface 1.2.0 static-bundle JSON object |
| `binding` | the FR-001 observation binding, every `ObservationBinding` field |
| `expected_subject` | the exact typed record subject |
| `relationships` | the complete supplied relationship graph (an array) |
| `required_relationships` | the required relationship premises (an array) |
| `scope` | every `ScopeSelection` field; `membership_document` is a JSON string whose UTF-8 bytes are the exact `filament.observation.explicit-members/v1` document |
| `limits` | `{max_records, max_members, max_relationships, max_required_relationships}` |
| `records` | optional: the candidate records, each in the record form below |
| `authority` | `{definition_identity, definition_revision, definition_digest}` |
| `subject` | `{scope_identity, population_identity}` |
| `revision` | the progress document's revision |
| `predecessor_document` | optional: a JSON string whose UTF-8 bytes are the exact direct predecessor progress document of a correction |
| `progress` | `{clock: {clock_identity, clock_revision}, required_sources, boundary, state, captured_trigger_identity, cutoff, restoration_basis_identity, association}`, the progress selection; `association` is optional |
| `progress_document` | a JSON string whose UTF-8 bytes are the exact `filament.observation.progress-assertion/v1` document to authenticate |
| `observation_source` | the selected observation source identity |

Identities are JSON strings. Digests are 64 lowercase hexadecimal digits.
Every integer (revision, positions, indices, instants, periods, uncertainties,
limits) is a canonical decimal string: no sign for an unsigned value, a
leading `-` only for a negative signed value, and no leading zero. Boundary,
state, cutoff rule, time source and association rows use the JSON forms of the
FR-004 owner documents. The request is caller-authored input, so its own
bytes need not be canonical; the owner documents it carries must be.

### The event input

The event input, when supplied, is a sequence of lines, each ending in one
LF (a final LF is optional). Each line is one closed JSON object with
`contract` exactly `RECORD_CONTRACT` and every other `AdmittedRecord` field
under its field name, in the same encodings as the request. Lines are numbered
from 1. An empty input holds zero records.

### Exactly one record source

The candidate records come from the request's `records` member or from the
event input, never both. When both are present the request is refused. When
neither is, the record population is empty, which FR-004 admits as an empty
binding history.

## Behavior

1. **Preflight.** Before deserializing, the library SHALL refuse a request,
   and an event input, longer than the effective `max_input_bytes` of
   `limits`, and SHALL apply the depth, string and visited-field ceilings of
   `limits` while reading. It SHALL refuse a record population larger than
   the request's `limits.max_records` before retaining the record past it.
2. **Decoding.** The library SHALL decode the request and each event line into
   the public owner types (`AdmissionRequest`, `AuthoritySelection`,
   `SubjectSelection`, `progress::Selection` and the rest) without defaulting,
   inferring or normalizing any value. It SHALL admit the `producer` object
   through `StaticProducerBundle::admit_json`.
3. **Qualification.** It SHALL call `admit` (FR-001) with the decoded request
   and SHALL continue only on `Available`.
4. **Authentication.** It SHALL strict-read the predecessor document, when
   supplied, and then the progress document through `progress::read`
   (FR-004), with a `Context` over the qualified history, the request's
   `authority` and `subject`, its `revision` and that predecessor.
5. **Source selection.** It SHALL require `observation_source` to be one of
   the view's `required_sources` and, when the view carries association rows,
   the observation source of at least one row.
6. **Result.** It SHALL return the view, the qualified observation and an
   `AuthoritySelections` holding exactly the `authority`, `subject`,
   `progress.clock` and `observation_source` that step 4 and 5 compared, with
   the progress document's identity. The view is equal to the one
   `progress::read` returns for the same inputs assembled through the Rust
   API.

## What is authenticated, and by what

- **The progress document**, the observation owner's assertion of progress
  authority, is authenticated by `progress::read`: the library derives the
  document itself from the qualified history and the request's selections,
  and admits the supplied bytes only when they are that derivation byte for
  byte. This covers the contract label, the envelope identity (lowercase
  SHA-256 over the contract label, a zero byte and the canonical bytes with
  `identity` omitted), the FR-268 progress-authority identity, revision,
  predecessor, authority, subject, scope, clock, sources, boundary, state,
  trigger, cutoff, restoration basis and association rows.
- **Each candidate record** is authenticated by `admit` against its FR-288
  identity preimage; **the membership** by the explicit-members strict read
  and its FR-264 identity; **the population** by its FR-263 identity; and
  **the producer bundle** by the FR-005 static-bundle admission.
- The authentication is recomputation by the observation owner. No signature,
  key, token, caller-set trust or qualified flag, file location or transport
  attribute authenticates anything. The selected package digest and
  producer-definition digest are retained exactly as selected and are not
  checked against artifact bytes (FR-001). An admitted view therefore
  attests that the progress document is exactly the one this owner derives
  from these inputs.

## Opacity

- `AdmittedProgress` and `AuthoritySelections` SHALL have private fields, no
  public constructor, no `Default`, no `Serialize` or `Deserialize`, and no
  mutable access; `admit_progress` is their only producer. They are `Clone`,
  `Debug`, `Eq` and `PartialEq`.
- The selections are bound to the document they authenticated: `binds` is
  false for a view of any other admission, and a consumer cannot put a
  caller-built selection in their place.
- The view keeps the FR-004 guarantees: it is constructor-private and exposes
  the association value only by borrow.

## Error Conditions

Every error is one `AdmissionError` and no capability. Each owner error is
carried unchanged.

| Condition | `stage` | `cause` |
| --- | --- | --- |
| request longer than `max_input_bytes`, or over a depth, string or visited-field ceiling | `Request` | `Authority` `ResourceIncomplete` |
| request bytes are not UTF-8 | `Request` | `Authority` `InvalidUtf8` |
| request is not one JSON value, or has trailing data | `Request` | `Authority` `InvalidJson` |
| `contract` is not `REQUEST_CONTRACT` | `Request` | `Authority` `ContractMismatch` |
| an unknown, duplicate or missing member; a wrong JSON type; a malformed decimal, digest or identity; an unknown enum label | `Request` | `Authority` `InvalidSelection` |
| `records` present and an event input supplied | `Request` | `Authority` `InvalidSelection` |
| event input longer than `max_input_bytes` | `Events` | `Authority` `ResourceIncomplete` |
| the record population passes `limits.max_records` | `Request` or `Event { line }` of the first record past it | `Authority` `ResourceIncomplete` |
| an event line that is not UTF-8, not one JSON object (an empty line included), names another contract, or has an unknown, duplicate, missing or malformed member | `Event { line }` | `Authority` `InvalidUtf8`, `InvalidJson`, `ContractMismatch` or `InvalidSelection` |
| the producer bundle is refused | `Producer` | `Producer` with its refusal |
| `admit` returns `Incomplete` or `Refused` | `Qualification` | `Observation` with that outcome unchanged |
| the predecessor or progress document fails its strict read | `Progress` | `Authority` with the reader's error unchanged |
| `observation_source` is not a required source of the view, or not the source of any association row | `Progress` | `Authority` `ExpectedMismatch` |

## Acceptance Criteria

| ID | Criteria | Verification |
| --- | --- | --- |
| FR-014-AC-1 | A request carrying two candidate records in `records` and no event input admits; its view equals `progress::read` over the same qualified history and selections assembled through the Rust API; `selections()` returns the request's authority, subject, clock and observation source; `binds(view)` is true and `progress_identity()` is the document's identity. | Test (TC-016) |
| FR-014-AC-2 | The same request without `records`, with an event input holding the same two records as two lines (with and without a final LF), admits a view equal to AC-1's; a request with association rows admits a view whose `association_canonical_bytes()` equal those of AC-1's Rust-API view; a request with no record source admits the empty-binding-history view. | Test (TC-016) |
| FR-014-AC-3 | Each request defect of the error table, one at a time (one byte over `max_input_bytes`, invalid UTF-8, trailing data, another contract, an unknown, a duplicate and a missing member, `revision` `"01"`, an uppercase digest, `records` together with an event input) returns `stage` `Request` with exactly the listed code and no capability; a request of exactly `max_input_bytes` is not refused for its size. | Test (TC-016) |
| FR-014-AC-4 | An event input whose third line is empty, not JSON, names `filament.observation.record/v2`, or lacks `event_time_nanos` returns `Event { line: 3 }` with `InvalidJson`, `InvalidJson`, `ContractMismatch` or `InvalidSelection`; one record past `max_records` returns `ResourceIncomplete` at that record's line before it is retained. | Test (TC-016) |
| FR-014-AC-5 | A producer bundle of interface 1.1.0 returns `stage` `Producer`; a record with a foreign signal returns `Qualification` with `admit`'s `Refused` outcome unchanged, and a missing required valuation `Qualification` with its `Incomplete` outcome unchanged. | Test (TC-016) |
| FR-014-AC-6 | A progress document with one byte changed, re-encoded non-canonically, derived at another revision, derived for another scope, or derived from the history with one record dropped returns `stage` `Progress` with `cause` `Authority` equal to the error `progress::read` returns for the same bytes and selections, and no capability; an `observation_source` outside the required sources returns `Progress` `ExpectedMismatch`. | Test (TC-016) |
| FR-014-AC-7 | `AdmittedProgress` and `AuthoritySelections` have no public constructor, field, `Default`, `Serialize` or `Deserialize` (a crate outside this one cannot build either); `binds` of admission A's selections over admission B's view of a different document is false; every accessor returns a borrow without allocating. | Test (TC-016) |
| FR-014-AC-8 | The entry reads no file, standard input, environment or clock: admitting the same request and event bytes twice, in different working directories and environments, returns equal capabilities. | Test (TC-016) |

## Dependencies

- [FR-001](FR-001-qualify-observation-admission.md) owns qualification and its incomplete and refused outcomes.
- [FR-004](FR-004-publish-observation-authority-artifacts.md) owns the progress-assertion contract, its strict reader, the error-code catalog and the owner limits.
- [FR-005](FR-005-adopt-authority-qualified-runtime-references.md) owns the Producer interface 1.2.0 static-bundle admission.
- FSL FR-251 and FR-761 own the consumer: `fsl_analyze::monitor_root::admit_progress_view` (fact `CF-RT-02`) calls this entry and passes the view and selections into the plan authority; the driver reads the request and event files.
