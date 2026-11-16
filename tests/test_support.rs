// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (C) 2026 Agent-IX

//! TC-014: the `test-support` export of the owned static-bundle fixture.
#![cfg(feature = "test-support")]

use filament_observation::{test_support::FCD_STATIC_BUNDLE_1_2, StaticProducerBundle};
use ix_trace_rs::trace;

#[trace("TC-014", "FR-012-AC-1")]
#[test]
fn fr_012_ac_1_exported_bundle_is_the_owned_fixture_and_admits() {
    assert_eq!(
        FCD_STATIC_BUNDLE_1_2,
        include_bytes!("fixtures/fcd-static-bundle-1.2.json").as_slice()
    );
    let admitted = StaticProducerBundle::admit_json(FCD_STATIC_BUNDLE_1_2)
        .expect("exported bundle is admitted");

    let commerce = "ix://agent-ix/commerce";
    let declared: Vec<(String, String, String)> = admitted
        .relationships()
        .iter()
        .map(|declaration| {
            (
                declaration.relationship_identity.clone(),
                declaration.source.type_identity.clone(),
                declaration.target.type_identity.clone(),
            )
        })
        .collect();
    let expected: Vec<(String, String, String)> = [
        ("Order-paymentAttempt", "Order", "PaymentAttempt"),
        ("Order-refund", "Order", "Refund"),
        ("Order-shipment", "Order", "Shipment"),
        ("Order-successor", "Order", "Order"),
    ]
    .into_iter()
    .map(|(relationship, source, target)| {
        (
            format!("{commerce}/relationship/{relationship}"),
            format!("{commerce}/type/{source}"),
            format!("{commerce}/type/{target}"),
        )
    })
    .collect();
    assert_eq!(declared, expected);

    let document: serde_json::Value =
        serde_json::from_slice(FCD_STATIC_BUNDLE_1_2).expect("exported bundle is JSON");
    let owning_types: Vec<&str> = document["components"]
        .as_array()
        .expect("bundle declares components")
        .iter()
        .map(|component| {
            component["owningTypeIdentity"]
                .as_str()
                .expect("component names its owning type")
        })
        .collect();
    for type_name in ["Order", "PaymentAttempt", "Refund", "Shipment"] {
        let type_identity = format!("{commerce}/type/{type_name}");
        assert!(
            owning_types.contains(&type_identity.as_str()),
            "the component set owns {type_identity}"
        );
    }
}
