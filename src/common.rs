//! Bidirectional message types and shared payload sub-structs.
//!
//! [`ErrorMessage`] and [`CloseMessage`] are emitted by either side.
//! [`TermCloseMessage`] is shared by `DeviceToCloud::TermClose` and
//! `CloudToDevice::TermClose` since the JSON shape is identical.
//! [`OtaStage`] is the lifecycle enum embedded in `OtaAck`.

use serde::{Deserialize, Serialize};

/// Either side may raise a protocol-layer error.
///
/// Carried as a variant of both [`crate::DeviceToCloud`] and
/// [`crate::CloudToDevice`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub terminal: bool,
}

/// Graceful close request before the WebSocket close frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseMessage {
    pub code: u16,
    pub reason: String,
}

/// Session-terminate frame. Sent by the device when its end of the
/// pty hangs up (`DeviceToCloud::TermClose`) or by the cloud when the
/// operator closes the browser tab (`CloudToDevice::TermClose`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermCloseMessage {
    pub sid: String,
    pub code: u32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reason: String,
}

/// One-minute / five-minute / fifteen-minute load averages.
///
/// The wire JSON uses bare numeric keys (`"1m"`, `"5m"`, `"15m"`),
/// not identifier-style names, so each field carries a `#[serde(rename)]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoadAvg {
    #[serde(rename = "1m")]
    pub one_m: f32,
    #[serde(rename = "5m")]
    pub five_m: f32,
    #[serde(rename = "15m")]
    pub fifteen_m: f32,
}

/// Memory totals in bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemStats {
    pub total: u64,
    pub used: u64,
}

/// Cloud's reply to a device heartbeat.
///
/// Lives in this module because the dispatcher historically allowed
/// piggybacked config nudges in `extra`. The opaque `serde_json::Value`
/// is intentional: the heartbeat-ack payload evolves independently of
/// the on-device decoder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatAckMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub extra: serde_json::Value,
}

/// OTA lifecycle stage carried in [`crate::OtaAckMessage::stage`].
///
/// The lifecycle is:
///
/// ```text
///   Verifying  → signatures + anti-rollback check before any disk write
///   Applying   → manifest accepted; writing the alternate slot
///   RolledForward → reboot succeeded, new slot is the active slot
///   Failed     → manifest verification or apply threw; device may retry
///   RolledBack → reboot+health-check rejected the new slot, A/B reverted
/// ```
///
/// See the wave-05 Phase 4 brief sections 5.3 (device-side state
/// machine) and 6.3 (cloud-side ledger reconciliation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtaStage {
    Verifying,
    Applying,
    RolledForward,
    Failed,
    RolledBack,
}
