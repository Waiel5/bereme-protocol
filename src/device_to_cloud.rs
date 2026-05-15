//! Frames the device emits to the cloud.

use crate::common::{CloseMessage, ErrorMessage, LoadAvg, MemStats, OtaStage, TermCloseMessage};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Top-level enum for every device-emitted frame.
///
/// The `type` discriminant is the snake_case form of the variant
/// name (e.g. `DeviceToCloud::Register` → `{"type": "register"}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeviceToCloud {
    Register(RegisterMessage),
    Heartbeat(HeartbeatMessage),
    /// Reply to `CloudToDevice::Ping`. No payload.
    Pong,
    TermData(TermDataMessage),
    TermClose(TermCloseMessage),
    FileChunk(FileChunkMessage),
    HttpResp(HttpRespMessage),
    CmdResponse(CmdResponseMessage),
    Audit(AuditMessage),
    DeviceInfo(DeviceInfoMessage),
    /// Acknowledgement of a `CloudToDevice::ConfigNudge`. Per
    /// `WIRE-PROTOCOL.md §control plane`, this is device-emitted.
    ConfigAck(ConfigAckMessage),
    /// Lifecycle transition reply to a `CloudToDevice::OtaPush`. The
    /// device emits one OtaAck per stage so the cloud's plan ledger
    /// reflects real device state.
    OtaAck(OtaAckMessage),
    Error(ErrorMessage),
    Close(CloseMessage),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,

    pub device_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firmware_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mac: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
    pub uptime_seconds: u64,
    pub load: LoadAvg,
    pub mem: MemStats,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TermDataMessage {
    pub sid: String,
    /// Base64-encoded bytes from the device's pty.
    pub data: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileChunkMessage {
    pub sid: String,
    pub seq: u64,
    /// Base64-encoded chunk payload.
    pub data: String,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpRespMessage {
    pub sid: String,
    pub status: u16,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    /// Base64-encoded chunk body. Use `last: true` on the terminal frame.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub body_chunk: String,
    pub last: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CmdResponseMessage {
    /// Correlation back to the cloud's `CmdMessage::id`.
    pub id: String,
    pub code: i32,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stdout: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stderr: String,
    #[serde(default, skip_serializing_if = "serde_json::Value::is_null")]
    pub attrs: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuditMessage {
    /// Device-emitted audit record. Carried as a generic JSON value
    /// so the protocol crate doesn't pin the device's exact audit
    /// catalog — versions drift independently and the cloud forwards
    /// the record to SIEM without re-encoding.
    pub record: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfoMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub os: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_ip: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kernel: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigAckMessage {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Outcome string. Production currently uses `"applied"` /
    /// `"rejected"`; we keep the field free-form so future
    /// dispatchers can add reasons without a schema bump.
    pub outcome: String,
}

/// Device-emitted ack for an [`crate::OtaPushMessage`].
///
/// The device emits one OtaAck per [`OtaStage`] transition; the
/// cloud's plan ledger collects them into the per-device state
/// machine. The `error` field is `Some(_)` only when `stage ==
/// OtaStage::Failed` or `OtaStage::RolledBack`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OtaAckMessage {
    pub plan_id: String,
    pub stage: OtaStage,
    pub version_code: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
