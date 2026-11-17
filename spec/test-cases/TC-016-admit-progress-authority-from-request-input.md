---
id: TC-016
title: "Admit the progress view and authority selections from request and event input"
type: TC
relationships:
  - { target: ix://agent-ix/filament-observation/FR-014, type: verifies }
---
# TC-016: Admit the progress view and authority selections from request and event input

## Description

Check FR-014's `authority::admission::admit_progress` entry: positive
admission from request records and from event input, every listed refusal,
authentication of the progress document, runtime event and watermark lines
with their frontier ordering, opacity and the absence of ambient input.

## Test Procedure

1. Build the TC-004 qualified history and progress document through the Rust
   API; encode the same inputs as a request with `records`, and as a request
   without `records` plus an event input (with and without a final LF), with
   and without association rows, and with no record source.
2. Admit each and compare the view with `progress::read`, the selections with
   the request's members, `binds` and `progress_identity`.
3. Apply each request, event, producer and qualification defect of FR-014's
   error table one at a time, including the exact and one-over size and record
   limits.
4. Mutate the progress document (one byte, non-canonical, other revision,
   other scope, one record dropped) and the observation source; compare each
   error with `progress::read`'s.
5. Check that the capability types cannot be built or serialized outside the
   crate, that `binds` rejects a foreign view, and that admission is equal
   across working directories and environments.
6. Encode AC-1's records as runtime event lines, add a runtime watermark
   line carrying the successor progress document, and admit two events and a
   watermark, then a mix of record-candidate, event and watermark lines;
   compare each step with the lines and each watermark view with
   `progress::read` over the records before it.
7. After a watermark, send records at, below and in another clock family than
   the frontier, a regressing, an equal and a mismatched `inclusive_boundary`,
   mutated watermark documents (one byte, predecessor revision, other
   predecessor, preceding record dropped) and malformed runtime lines; compare
   each refusal with FR-014's table and the reader's error.

## Expected Results

Every positive request admits a view equal to the Rust-API view. Every defect
returns exactly the listed stage and cause and no capability. Progress-stage
errors equal the strict reader's. The capability is opaque and bound to its
document, and admission reads only the supplied bytes. Runtime lines admit
as ordered steps; a watermark never moves back, a record at or below the
frontier refuses at its line, and an input of record-candidate lines only
admits as before with no steps.

## Status

Ignored stubs in `tests/progress_admission.rs`.
