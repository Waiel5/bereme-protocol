//! Serde shape and defaulting coverage for the Bereme-owned RustDesk
//! compatibility planning contract.

#![allow(clippy::unwrap_used)]

use bereme_protocol::{
    RustDeskAcceptanceGate, RustDeskAcceptanceGateKind, RustDeskAcceptanceGateStatus,
    RustDeskCompatibilityMode, RustDeskCompatibilityPlan, RustDeskFeatureFlags,
    RustDeskInfrastructurePolicy, RustDeskRouteKind, RustDeskVersionPinMetadata,
    RUSTDESK_COMPAT_CONTRACT_VERSION, RUSTDESK_REQUIRED_ACCEPTANCE_GATES,
};
use serde_json::json;

fn passed_required_gates() -> Vec<RustDeskAcceptanceGate> {
    RUSTDESK_REQUIRED_ACCEPTANCE_GATES
        .iter()
        .copied()
        .map(|kind| RustDeskAcceptanceGate {
            kind,
            status: RustDeskAcceptanceGateStatus::Passed,
            evidence_uri: None,
            reviewer: None,
            reviewed_at: None,
        })
        .collect()
}

#[test]
fn empty_plan_defaults_to_disabled_and_safe() {
    let plan: RustDeskCompatibilityPlan = serde_json::from_value(json!({})).unwrap();

    assert_eq!(plan.contract_version, RUSTDESK_COMPAT_CONTRACT_VERSION);
    assert_eq!(plan.mode, RustDeskCompatibilityMode::Disabled);
    assert_eq!(plan.route_kind, RustDeskRouteKind::Disabled);
    assert_eq!(
        plan.infrastructure_policy,
        RustDeskInfrastructurePolicy::Disabled
    );
    assert_eq!(plan.feature_flags, RustDeskFeatureFlags::default());
    assert_eq!(plan.version_pin, RustDeskVersionPinMetadata::default());
    assert!(plan.acceptance_gates.is_empty());
    assert!(!plan.acceptance_gates_passed());
    assert!(!plan.can_activate_route());
}

#[test]
fn partial_feature_flags_default_missing_flags_to_false() {
    let flags: RustDeskFeatureFlags = serde_json::from_value(json!({
        "display_stream": true,
        "input_control": true
    }))
    .unwrap();

    assert!(flags.display_stream);
    assert!(flags.input_control);
    assert!(!flags.file_transfer);
    assert!(!flags.clipboard_sync);
    assert!(!flags.audio_stream);
    assert!(!flags.relay_rendezvous);
    assert!(!flags.unattended_access);
    assert!(!flags.audit_correlation);
}

#[test]
fn plan_serializes_with_stable_snake_case_names() {
    let plan = RustDeskCompatibilityPlan {
        contract_version: RUSTDESK_COMPAT_CONTRACT_VERSION,
        mode: RustDeskCompatibilityMode::Pilot,
        route_kind: RustDeskRouteKind::NativeBridge,
        infrastructure_policy: RustDeskInfrastructurePolicy::SelfHostedOnly,
        feature_flags: RustDeskFeatureFlags {
            display_stream: true,
            input_control: true,
            file_transfer: false,
            clipboard_sync: true,
            audio_stream: false,
            relay_rendezvous: true,
            unattended_access: false,
            audit_correlation: true,
        },
        version_pin: RustDeskVersionPinMetadata {
            upstream_version: Some("example-upstream-release".into()),
            upstream_revision: Some("example-upstream-revision".into()),
            bereme_revision: Some("bereme-adapter-1".into()),
            pinned_at: Some("2026-05-28T00:00:00Z".into()),
            note: Some("planning metadata only".into()),
        },
        acceptance_gates: vec![RustDeskAcceptanceGate {
            kind: RustDeskAcceptanceGateKind::LicenseReview,
            status: RustDeskAcceptanceGateStatus::Passed,
            evidence_uri: Some("bereme://reviews/license/rustdesk-compat".into()),
            reviewer: Some("security".into()),
            reviewed_at: Some("2026-05-28T00:00:00Z".into()),
        }],
    };

    let encoded = serde_json::to_value(&plan).unwrap();

    assert_eq!(encoded["contract_version"], json!(1));
    assert_eq!(encoded["mode"], json!("pilot"));
    assert_eq!(encoded["route_kind"], json!("native_bridge"));
    assert_eq!(encoded["infrastructure_policy"], json!("self_hosted_only"));
    assert_eq!(
        encoded["acceptance_gates"][0]["kind"],
        json!("license_review")
    );
    assert_eq!(encoded["acceptance_gates"][0]["status"], json!("passed"));

    let decoded: RustDeskCompatibilityPlan = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, plan);
}

#[test]
fn route_activation_requires_live_route_pilot_or_enabled_and_passed_gates() {
    let mut plan = RustDeskCompatibilityPlan {
        mode: RustDeskCompatibilityMode::Pilot,
        route_kind: RustDeskRouteKind::Fork,
        infrastructure_policy: RustDeskInfrastructurePolicy::OperatorManaged,
        acceptance_gates: passed_required_gates(),
        ..RustDeskCompatibilityPlan::default()
    };

    assert!(plan.acceptance_gates_passed());
    assert!(plan.required_acceptance_gates_passed());
    assert!(plan.can_activate_route());

    plan.acceptance_gates[1].status = RustDeskAcceptanceGateStatus::Pending;
    assert!(!plan.acceptance_gates_passed());
    assert!(!plan.required_acceptance_gates_passed());
    assert!(!plan.can_activate_route());

    plan.acceptance_gates[1].status = RustDeskAcceptanceGateStatus::Passed;
    assert!(plan.required_acceptance_gates_passed());

    plan.acceptance_gates.pop();
    assert!(plan.acceptance_gates_passed());
    assert!(!plan.required_acceptance_gates_passed());
    assert!(!plan.can_activate_route());

    plan.acceptance_gates = passed_required_gates();
    plan.mode = RustDeskCompatibilityMode::Shadow;
    assert!(!plan.can_activate_route());

    plan.mode = RustDeskCompatibilityMode::Enabled;
    plan.route_kind = RustDeskRouteKind::Disabled;
    assert!(!plan.can_activate_route());

    plan.route_kind = RustDeskRouteKind::NativeBridge;
    plan.infrastructure_policy = RustDeskInfrastructurePolicy::Disabled;
    assert!(!plan.can_activate_route());
}
