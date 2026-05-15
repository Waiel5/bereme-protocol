//! Frames the cloud emits to the device.

use crate::common::{CloseMessage, ErrorMessage, HeartbeatAckMessage, TermCloseMessage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Top-level enum for every cloud-emitted frame.
///
/// The `type` discriminant is the snake_case form of the variant
/// name (e.g. `CloudToDevice::TermOpen` → `{"type": "term_open"}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CloudToDevice {
    RegisterAck(RegisterAckMessage),
    HeartbeatAck(HeartbeatAckMessage),
    /// Periodic liveness probe; device replies with
    /// `DeviceToCloud::Pong`. No payload.
    Ping,
    TermOpen(TermOpenMessage),
    TermResize(TermResizeMessage),
    TermInput(TermInputMessage),
    /// Cloud asks the device to terminate an open session. Shape is
    /// shared with the device-emitted variant.
    TermClose(TermCloseMessage),
    Cmd(CmdMessage),
    HttpReq(HttpReqMessage),
    ConfigNudge(ConfigNudgeMessage),
    /// Wave-05 Phase 4: fleet-OTA push. Device replies with
    /// `DeviceToCloud::OtaAck` per stage. Field names match the
    /// brief verbatim and the cloud's `OtaPlanRequest`.
    OtaPush(OtaPushMessage),
    Error(ErrorMessage),
    Close(CloseMessage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RegisterAckOutcome {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterAckMessage {
    /// Correlates back to RegisterMessage::id when set.
    pub id: String,
    pub outcome: RegisterAckOutcome,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Server-assigned canonical device id (may equal the requested one).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// Hint for next heartbeat interval (seconds).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heartbeat_interval_seconds: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermOpenMessage {
    pub sid: String,
    pub cols: u16,
    pub rows: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Auth bundle. Carried as a generic value so the dispatcher can
    /// extend it (key, token, otp) without a protocol bump.
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub auth: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermResizeMessage {
    pub sid: String,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermInputMessage {
    pub sid: String,
    /// Base64-encoded operator keystrokes.
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CmdMessage {
    /// Correlation id; CmdResponseMessage quotes it back.
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub cmd: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpReqMessage {
    pub sid: String,
    pub method: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub query: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub body_chunk: String,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConfigNudgeMessage {
    pub keys: serde_json::Value,
}

/// Wave-05 Phase 4: fleet-OTA push. Field names match the wave-05
/// Phase 4 brief verbatim — the cloud-domain ledger consumes these
/// directly when materializing per-device WS sends.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtaPushMessage {
    pub manifest_url: String,
    /// Detached manifest signatures (dual-sig: PKI + transparency
    /// witness). Encoded as base64 strings; the device validates them
    /// against `kvm_supervisor::install::validate_device_manifest`.
    pub manifest_signatures: Vec<String>,
    pub version_code: u32,
    pub plan_id: String,
    /// Unix epoch (seconds, UTC) past which the device should give up
    /// and emit `OtaStage::Failed`.
    pub deadline_unix: i64,
}
