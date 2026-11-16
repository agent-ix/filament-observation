---
id: TC-014
title: "Export the owned test fixture"
type: TC
relationships:
  - { target: ix://agent-ix/filament-observation/FR-012, type: verifies }
---
# TC-014: Export the owned test fixture

## Description

Check the `test-support` fixture export for FR-012.

## Test Procedure

Build with `--features test-support`; compare `test_support::FCD_STATIC_BUNDLE_1_2`
with the fixture file and admit it. List the admitted relationship declarations
and the components' owning types. Inspect `Cargo.toml` for an empty default
feature set.

## Expected Results

The bytes are identical and admitted. The bundle declares exactly the
relationships `Order-paymentAttempt` (Order to PaymentAttempt), `Order-refund`
(Order to Refund), `Order-shipment` (Order to Shipment) and `Order-successor`
(Order to Order), and its component set owns the Order, PaymentAttempt, Refund
and Shipment types. No default feature enables the export.

## Status

Implemented in `tests/test_support.rs`.
