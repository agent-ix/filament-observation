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
  - { target: ix://agent-ix/filament-contract-runtime/interface_012, type: references }
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
the existing readers. The event input also carries the runtime monitor's own
event and watermark lines, so one stream feeds both this admission and the
monitor: every runtime event is authenticated as an observation record, and
every runtime watermark as an owner-derived successor progress assertion,
before the runtime sees it.

## Interface

The module `authority::admission` SHALL publish:

| Item | Form |
| --- | --- |
| `REQUEST_CONTRACT` | `"filament.observation.progress-admission-request/v1"` |
| `REQUEST_SCHEMA_BYTES` | the JSON Schema bytes of the request contract |
| `RECORD_CONTRACT` | `"filament.observation.record-candidate/v1"` |
| `RECORD_SCHEMA_BYTES` | the JSON Schema bytes of one event-input record |
| `EVENT_CONTRACT` | `"filament.observation.rt-event/v1"` |
| `EVENT_SCHEMA_BYTES` | the JSON Schema bytes of one runtime event line |
| `WATERMARK_CONTRACT` | `"filament.observation.rt-watermark/v1"` |
| `WATERMARK_SCHEMA_BYTES` | the JSON Schema bytes of one runtime watermark line |
| `AdmittedStep` | one admitted runtime line: `Event(AdmittedEvent)` or `Watermark(AdmittedWatermark)` |
| `AdmittedEvent`, `AdmittedWatermark` | the admitted runtime event and watermark |
| `admit_progress(request: &[u8], events: Option<&[u8]>, limits: Limits) -> Result<AdmittedProgress, AdmissionError>` | the one admission entry |
| `AdmittedProgress` | the admitted capability |
| `AuthoritySelections` | the opaque selections the view was authenticated against |
| `AdmissionError { stage: AdmissionStage, cause: AdmissionCause }` | the typed error |
| `AdmissionStage` | `Request`, `Events`, `Event { line: u64 }`, `Producer`, `Qualification`, `Progress` |
| `AdmissionCause` | `Authority(Error)`, `Producer(producer::Refusal)`, `Observation(AdmissionOutcome)` |

`AdmittedProgress` SHALL provide `view(&self) -> &progress::View`,
`selections(&self) -> &AuthoritySelections`,
`qualified(&self) -> &QualifiedObservation`,
`steps(&self) -> &[AdmittedStep]` (the admitted runtime lines in line order;
empty when the input holds none) and
`into_parts(self) -> (progress::View, AuthoritySelections, Vec<AdmittedStep>)`.
`AdmittedEvent` SHALL provide `line(&self) -> u64`,
`record(&self) -> &AdmittedRecord`, `reading(&self) -> u64` and
`inputs(&self) -> &[u8]`. `AdmittedWatermark` SHALL provide
`line(&self) -> u64`, `view(&self) -> &progress::View`,
`assertion(&self) -> &[u8]` (the exact progress document bytes it
authenticated) and `inclusive_boundary(&self) -> &Integer`.
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
LF (a final LF is optional). Lines are numbered from 1. An empty input holds
zero records. Each line is one closed JSON object whose `contract` member
selects one of three line kinds; the kinds may be mixed in any order in one
input.

| Line kind | `contract` | Members besides `contract` |
| --- | --- | --- |
| record candidate | `RECORD_CONTRACT` | every other `AdmittedRecord` field under its field name, in the same encodings as the request |
| runtime event | `EVENT_CONTRACT` | `record`: one record-candidate object exactly as the record-candidate line, its own `contract` included; `reading`: the raw counter reading as a canonical unsigned decimal string, at most 2^64 − 1; `inputs`: a JSON string whose UTF-8 bytes are the event's canonical captures value |
| runtime watermark | `WATERMARK_CONTRACT` | `revision`: the asserted progress document's revision; `boundary` and `state`: the advanced boundary and open or closed state in the FR-004 JSON forms; `inclusive_boundary`: the inclusive coverage as a canonical signed decimal string; `progress_document`: a JSON string whose UTF-8 bytes are the exact `filament.observation.progress-assertion/v1` document asserting that boundary |

These are the runtime monitor's input lines. A runtime event carries the
fields of a runtime `Event` that originate in the stream: the raw reading and
its captured inputs, together with the observation record the event is. A
runtime watermark carries the fields of a runtime `Watermark` that originate
in the stream: the authority assertion and its inclusive boundary. The
remaining runtime fields (the input context of package lock, clause, subject,
occurrence, clock binding, counter, tick unit, width, profile and order, the
progress context and the scope) are the same for every line of one monitor
and come from the request and the admitted selections, not from each line,
so a line cannot introduce an association the request did not select.
`inputs` is retained as its exact bytes; the runtime, not this library,
decodes the captures value.

### The admitted progress history

The event input maps to one ordered progress history:

- A record-candidate line is a record candidate. A runtime event line is a
  record candidate (its `record`) and also one `AdmittedEvent` step. Every
  record candidate, of either kind, joins the one record population that
  qualification admits, in line order, and counts toward `limits.max_records`.
- A runtime watermark line advances the frontier. The frontier before the
  first watermark line is the inclusive upper value of the request's
  `progress.boundary` (`upper_inclusive` for event positions and fixed
  samples, `upper_inclusive_nanos` for timestamped events). A watermark line's
  `inclusive_boundary` SHALL equal the inclusive upper value of its own
  `boundary`, and once its document is authenticated that value is the new
  frontier.
- Every record candidate of the input, before and after any watermark line,
  is a record of the one population the request selects: the request's
  `scope` lists every member with its record identity and anchor, FR-001
  binds every member to exactly one record and refuses a duplicate record
  identity, and one subject names that population for every progress
  document of the admission. The qualified history is therefore the
  qualification of every record of the input, and a record line after a
  watermark is a member's record that the stream delivers later, never a
  new member and never a second record of an earlier member.
- The request's `progress_document` is authenticated over that qualified
  history. Each watermark's `progress_document` is authenticated over the
  same qualified history, with the request's `progress` selection in which
  only `boundary` and `state` are replaced by the line's, at the line's
  `revision`, and with the previously authenticated progress document (the
  request's, then each earlier watermark's) as its direct predecessor. A
  watermark is therefore a successor progress assertion with a greater
  revision and unbroken lineage over the selected population, never a bare
  number; what a watermark's position in the line order governs is the
  frontier the records after it are checked against.

### Ordering

- A watermark never moves back: a watermark line whose `inclusive_boundary` is
  less than the current frontier is refused. An equal boundary is admitted,
  since a repeated inclusive assertion is valid runtime progress.
- A record at or below the frontier is refused: after the first watermark
  line, a record candidate of either kind whose anchor is at or below the
  current frontier in that frontier's clock family is refused, and so is a
  record whose anchor is of another clock family. Records before the first
  watermark line are not compared with a frontier. A record after a
  watermark line whose anchor is above the current frontier admits when it is
  the record of a population member that no earlier line delivered.
- Order is the line order. It is never inferred from timestamps, readings or
  arrival, and the library adds no event or watermark at end of input.

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
6. **Runtime lines.** In line order, it SHALL check each record against the
   current frontier, and SHALL authenticate each watermark line through
   `progress::read` as the admitted progress history states, checking its
   `inclusive_boundary` and the ordering rules; the first refused line ends
   admission with no capability.
7. **Result.** It SHALL return the view, the qualified observation, an
   `AuthoritySelections` holding exactly the `authority`, `subject`,
   `progress.clock` and `observation_source` that step 4 and 5 compared, with
   the progress document's identity, and one `AdmittedStep` per runtime line
   in line order. The view, and each watermark's view, is equal to the one
   `progress::read` returns for the same inputs assembled through the Rust
   API. An input of record-candidate lines only admits exactly what it admits
   without runtime lines, with no steps.

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
- **Each runtime event's record** is authenticated exactly as a record
  candidate is; its `reading` and `inputs` are retained as supplied and are
  admitted by the runtime against the selected association. **Each runtime
  watermark's progress document** is authenticated by `progress::read` as the
  admitted progress history states: the owner derives the successor document
  from the qualified history and the line's boundary, state and revision, and
  admits the supplied bytes only when they are that derivation.
  A watermark line's `inclusive_boundary` is admitted only when it equals the
  authenticated boundary.
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
  `Debug`, `Eq` and `PartialEq`. `AdmittedStep`, `AdmittedEvent` and
  `AdmittedWatermark` follow the same rules.
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
| an event line that is not UTF-8, not one JSON object (an empty line included), names a contract other than `RECORD_CONTRACT`, `EVENT_CONTRACT` or `WATERMARK_CONTRACT`, or has an unknown, duplicate, missing or malformed member | `Event { line }` | `Authority` `InvalidUtf8`, `InvalidJson`, `ContractMismatch` or `InvalidSelection` |
| a runtime event whose `record` names a contract other than `RECORD_CONTRACT`, or has an unknown, duplicate, missing or malformed member | `Event { line }` | `Authority` `ContractMismatch` or `InvalidSelection` |
| a runtime event `reading` that is not a canonical unsigned decimal or exceeds 2^64 − 1, or `inputs` that is not a JSON string | `Event { line }` | `Authority` `InvalidSelection` |
| a runtime watermark `revision` or `inclusive_boundary` that is not a canonical decimal, or a `boundary` or `state` not in its FR-004 form | `Event { line }` | `Authority` `InvalidSelection` |
| after a watermark line, a record (of either kind) at or below the current frontier, or anchored in another clock family | `Event { line }` of that record | `Authority` `ExpectedMismatch` |
| a watermark `inclusive_boundary` that does not equal its `boundary`'s inclusive upper value | `Event { line }` | `Authority` `ExpectedMismatch` |
| a watermark `inclusive_boundary` less than the current frontier | `Event { line }` | `Authority` `ExpectedMismatch` |
| a watermark `progress_document` that fails its strict read | `Event { line }` | `Authority` with the reader's error unchanged |
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
| FR-014-AC-9 | An event input of two runtime event lines followed by one runtime watermark line, whose records are AC-1's two records, admits a view equal to AC-1's view and three steps in line order: two `AdmittedEvent`s whose `line`, `record`, `reading` and exact `inputs` bytes are the lines', and one `AdmittedWatermark` whose `view` equals `progress::read` of its document over both records with the line's boundary, state and revision and AC-1's document as predecessor, whose `assertion` is the document bytes and whose `inclusive_boundary` is the line's. | Test (TC-016) |
| FR-014-AC-10 | Over a scope of three members whose records are anchored at `a < b < c`, an input of a record-candidate line for `a`, a runtime event line for `b`, a runtime watermark line with frontier `f` where `b ≤ f < c`, and a runtime event line for `c` admits: the record-candidate line contributes a record and no step, `steps()` is `[Event(2), Watermark(3), Event(4)]`, `qualified()` holds all three records, and the request's and the watermark's views each equal `progress::read` of their document over that three-record history; an input of record-candidate lines only admits a capability equal to AC-2's with `steps()` empty. | Test (TC-016) |
| FR-014-AC-11 | After a watermark line with frontier `f`: a later record of either kind anchored at `f`, one anchored below `f`, and one anchored in another clock family each return `Event { line }` of that record with `ExpectedMismatch`; a later watermark with `inclusive_boundary` below `f` returns `ExpectedMismatch` at its line, while one equal to `f` (with a greater revision) admits; a watermark whose `inclusive_boundary` differs from its `boundary`'s inclusive upper value returns `ExpectedMismatch` at its line. Each returns no capability. | Test (TC-016) |
| FR-014-AC-12 | A watermark whose `progress_document` has one byte changed, was derived at the predecessor's revision, names another predecessor, was derived from the empty history (every record dropped, so the progress-authority identity takes its empty-history form), or was derived with a `boundary` one tick above the line's returns `Event { line }` of the watermark with `cause` `Authority` equal to the error `progress::read` returns for the same bytes and context, and no capability. | Test (TC-016) |
| FR-014-AC-13 | On the third line, a runtime event whose `record` names `filament.observation.record/v2`, whose `reading` is `"18446744073709551616"` or `"01"`, or whose `inputs` is a JSON object, and a runtime watermark lacking `progress_document` or with `inclusive_boundary` `"+5"`, each return `Event { line: 3 }` with `ContractMismatch`, `InvalidSelection`, `InvalidSelection`, `InvalidSelection`, `InvalidSelection` or `InvalidSelection` respectively; a line naming `filament.observation.rt-event/v2` returns `ContractMismatch`. | Test (TC-016) |
| FR-014-AC-14 | In AC-10's input, with `limits.max_records` at least 4, appending a fifth line that repeats the runtime event for `c` (above `f`, same record and identity) returns `Qualification` with `admit`'s `Refused` `DuplicateRecord` outcome unchanged, appending instead a fifth line that is a runtime event above `f` whose record is not a member of the scope (every member keeping its record) returns `Qualification` with `admit`'s `Incomplete` outcome whose sole reason is `MissingMembership`, unchanged; dropping the last line, so the member `c` has no record, returns `Qualification` with `admit`'s `Refused` `MemberRecordMismatch` outcome unchanged. Each returns no capability. | Test (TC-016) |

## Dependencies

- [FR-001](FR-001-qualify-observation-admission.md) owns qualification and its incomplete and refused outcomes.
- [FR-004](FR-004-publish-observation-authority-artifacts.md) owns the progress-assertion contract, its strict reader, the error-code catalog and the owner limits.
- [FR-005](FR-005-adopt-authority-qualified-runtime-references.md) owns the Producer interface 1.2.0 static-bundle admission.
- FSL FR-251 and FR-761 own the consumer: `fsl_analyze::monitor_root::admit_progress_view` (fact `CF-RT-02`) calls this entry and passes the view and selections into the plan authority; the driver reads the request and event files.
- The driver's runtime-monitor generation (filament-driver FR-020) sends the
  runtime event and watermark lines on its `--events` input and passes each
  admitted step to the runtime in line order.
- RT [interface-012](ix://agent-ix/filament-contract-runtime/interface_012)
  owns the runtime `Event` and `Watermark` operations, their input context,
  progress context and the runtime's own regression refusal.
