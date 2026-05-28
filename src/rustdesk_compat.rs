//! Bereme-owned planning contract for future RustDesk compatibility.
//!
//! This module deliberately does **not** implement, mirror, or describe
//! RustDesk's wire protocol. The types here are Bereme-side metadata used to
//! plan and review a possible bridge/fork integration without importing
//! upstream schemas or generated code.

use serde::{Deserialize, Serialize};

/// Version for the Bereme-side compatibility planning contract.
///
/// This is independent of both [`crate::PROTOCOL_VERSION`] and any upstream
/// RustDesk release number.
pub const RUSTDESK_COMPAT_CONTRACT_VERSION: u16 = 1;

fn default_contract_version() -> u16 {
    RUSTDESK_COMPAT_CONTRACT_VERSION
}

/// Operational maturity for a RustDesk compatibility path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustDeskCompatibilityMode {
    /// Compatibility is intentionally unavailable.
    #[default]
    Disabled,
    /// Metadata and review gates may be recorded, but no bridge traffic flows.
    InventoryOnly,
    /// Candidate routing may be evaluated out of band without operator impact.
    Shadow,
    /// Limited, allowlisted usage after review gates pass.
    Pilot,
    /// General use is allowed after review gates pass.
    Enabled,
}

/// Planned implementation route for compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustDeskRouteKind {
    /// No compatibility route exists or should be selected.
    #[default]
    Disabled,
    /// A Bereme-native adapter/bridge owns the integration boundary.
    NativeBridge,
    /// Compatibility depends on a separately maintained fork.
    Fork,
}

/// Where RustDesk-compatible rendezvous/relay infrastructure may live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustDeskInfrastructurePolicy {
    /// No RustDesk-compatible infrastructure is allowed.
    #[default]
    Disabled,
    /// `hbbs`/`hbbr` or compatible services must run inside the operator's
    /// controlled boundary. This is the only acceptable airgap posture.
    SelfHostedOnly,
    /// Operator-managed infrastructure may be used, including a managed cloud
    /// account under the operator's control.
    OperatorManaged,
    /// Public third-party rendezvous/relay may be used after explicit review.
    PublicAllowed,
}

/// Generic capability flags for a future compatibility route.
///
/// These flags describe Bereme product capabilities, not upstream message
/// names or protocol fields. Missing flags default to `false` so older JSON
/// remains readable when the struct grows.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RustDeskFeatureFlags {
    /// Display/framebuffer streaming.
    #[serde(default)]
    pub display_stream: bool,
    /// Keyboard and pointer control.
    #[serde(default)]
    pub input_control: bool,
    /// File movement through a reviewed bridge boundary.
    #[serde(default)]
    pub file_transfer: bool,
    /// Clipboard synchronization.
    #[serde(default)]
    pub clipboard_sync: bool,
    /// Audio stream forwarding.
    #[serde(default)]
    pub audio_stream: bool,
    /// Relay or rendezvous support owned by the compatibility route.
    #[serde(default)]
    pub relay_rendezvous: bool,
    /// Unattended access flows.
    #[serde(default)]
    pub unattended_access: bool,
    /// Session recording or audit correlation across the bridge.
    #[serde(default)]
    pub audit_correlation: bool,
}

/// Version pin metadata for a reviewed compatibility target.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RustDeskVersionPinMetadata {
    /// Human-readable upstream release/version, when a release is pinned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_version: Option<String>,
    /// Upstream commit, tag, or package digest used for provenance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream_revision: Option<String>,
    /// Bereme adapter/fork version or commit that owns the boundary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bereme_revision: Option<String>,
    /// RFC 3339 timestamp for when the pin was approved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned_at: Option<String>,
    /// Free-form review note; keep secrets and upstream schemas out of here.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Review gate that must be accepted before compatibility is activated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustDeskAcceptanceGateKind {
    /// License obligations and attribution have been reviewed.
    LicenseReview,
    /// Threat model, isolation, and abuse paths have been reviewed.
    SecurityReview,
    /// The adapter/fork boundary avoids copying upstream wire schemas here.
    ProtocolBoundary,
    /// Interop tests exist outside this crate for the selected target.
    InteropFixtureCoverage,
    /// Performance and resource ceilings are understood.
    PerformanceBaseline,
    /// Bereme audit and session-correlation hooks are complete.
    AuditIntegration,
    /// Airgap/no-egress behavior has been validated for the route.
    AirgapEgressReview,
    /// Operators can disable or roll back the route quickly.
    RollbackPlan,
}

/// Gate set Bereme currently requires before selecting a compatibility route.
pub const RUSTDESK_REQUIRED_ACCEPTANCE_GATES: [RustDeskAcceptanceGateKind; 8] = [
    RustDeskAcceptanceGateKind::LicenseReview,
    RustDeskAcceptanceGateKind::SecurityReview,
    RustDeskAcceptanceGateKind::ProtocolBoundary,
    RustDeskAcceptanceGateKind::InteropFixtureCoverage,
    RustDeskAcceptanceGateKind::PerformanceBaseline,
    RustDeskAcceptanceGateKind::AuditIntegration,
    RustDeskAcceptanceGateKind::AirgapEgressReview,
    RustDeskAcceptanceGateKind::RollbackPlan,
];

/// Current result for a compatibility acceptance gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RustDeskAcceptanceGateStatus {
    /// The gate has not been completed.
    #[default]
    Pending,
    /// The gate passed.
    Passed,
    /// The gate failed and must block activation.
    Failed,
}

/// One acceptance-gate record with optional evidence metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustDeskAcceptanceGate {
    pub kind: RustDeskAcceptanceGateKind,
    #[serde(default)]
    pub status: RustDeskAcceptanceGateStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_at: Option<String>,
}

impl RustDeskAcceptanceGate {
    /// Returns true when this gate should no longer block activation.
    #[must_use]
    pub fn is_passed(&self) -> bool {
        self.status == RustDeskAcceptanceGateStatus::Passed
    }
}

/// Top-level Bereme compatibility plan for a future RustDesk route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RustDeskCompatibilityPlan {
    /// Bereme-side planning contract version.
    #[serde(default = "default_contract_version")]
    pub contract_version: u16,
    #[serde(default)]
    pub mode: RustDeskCompatibilityMode,
    #[serde(default)]
    pub route_kind: RustDeskRouteKind,
    #[serde(default)]
    pub infrastructure_policy: RustDeskInfrastructurePolicy,
    #[serde(default)]
    pub feature_flags: RustDeskFeatureFlags,
    #[serde(default)]
    pub version_pin: RustDeskVersionPinMetadata,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_gates: Vec<RustDeskAcceptanceGate>,
}

impl Default for RustDeskCompatibilityPlan {
    fn default() -> Self {
        Self {
            contract_version: RUSTDESK_COMPAT_CONTRACT_VERSION,
            mode: RustDeskCompatibilityMode::default(),
            route_kind: RustDeskRouteKind::default(),
            infrastructure_policy: RustDeskInfrastructurePolicy::default(),
            feature_flags: RustDeskFeatureFlags::default(),
            version_pin: RustDeskVersionPinMetadata::default(),
            acceptance_gates: Vec::new(),
        }
    }
}

impl RustDeskCompatibilityPlan {
    /// Returns true when all configured gates have passed.
    ///
    /// An empty gate list is not accepted for activation; callers must make an
    /// explicit review record before enabling a route.
    #[must_use]
    pub fn acceptance_gates_passed(&self) -> bool {
        !self.acceptance_gates.is_empty()
            && self
                .acceptance_gates
                .iter()
                .all(RustDeskAcceptanceGate::is_passed)
    }

    /// Returns true when every currently required gate is present and passed.
    #[must_use]
    pub fn required_acceptance_gates_passed(&self) -> bool {
        RUSTDESK_REQUIRED_ACCEPTANCE_GATES.iter().all(|required| {
            self.acceptance_gates
                .iter()
                .any(|gate| gate.kind == *required && gate.is_passed())
        })
    }

    /// Returns true when the plan is mature enough to select a route.
    #[must_use]
    pub fn can_activate_route(&self) -> bool {
        matches!(
            self.mode,
            RustDeskCompatibilityMode::Pilot | RustDeskCompatibilityMode::Enabled
        ) && matches!(
            self.route_kind,
            RustDeskRouteKind::NativeBridge | RustDeskRouteKind::Fork
        ) && matches!(
            self.infrastructure_policy,
            RustDeskInfrastructurePolicy::SelfHostedOnly
                | RustDeskInfrastructurePolicy::OperatorManaged
                | RustDeskInfrastructurePolicy::PublicAllowed
        ) && self.acceptance_gates_passed()
            && self.required_acceptance_gates_passed()
    }
}
