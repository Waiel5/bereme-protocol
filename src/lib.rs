//! Shared wire-protocol types for the Bereme JSON-WS v2 protocol.
//!
//! ## Scope
//!
//! This crate is consumed by **both** sides of the WebSocket:
//!
//! - The cloud server (`cloud-rust/crates/cloud-protocol`) re-exports
//!   these types and feeds them to its per-device dispatcher.
//! - The device firmware (`kvm-rust/crates/kvm-cloud`) decodes inbound
//!   frames into [`CloudToDevice`] and encodes outbound frames from
//!   [`DeviceToCloud`].
//!
//! The crate has no async, IO, or transport dependencies — it owns
//! the *bytes-on-the-wire contract* and nothing else.
//!
//! ## Frame format
//!
//! Every frame is a single WebSocket Text frame carrying a JSON
//! object. The object has:
//!
//! - a discriminant string field `type` (snake_case);
//! - an optional `id` (UUID v4 for correlation);
//! - an optional `ts` (RFC 3339 UTC string set by the sender);
//! - the per-variant payload fields.
//!
//! Serde provides the `type` discriminant via
//! `#[serde(tag = "type", rename_all = "snake_case")]` on the two
//! top-level enums.
//!
//! See `bereme-protocol/README.md` for the complete
//! catalog and `kvm-rust/docs/hardening/evidence/wave05/`
//! for the Phase 4 extraction record.

#![forbid(unsafe_code)]

pub mod cloud_to_device;
pub mod common;
pub mod device_to_cloud;
pub mod envelope;
pub mod rustdesk_compat;

pub use cloud_to_device::{
    CloudToDevice, CmdMessage, ConfigNudgeMessage, HttpReqMessage, OtaPushMessage,
    RegisterAckMessage, RegisterAckOutcome, TermInputMessage, TermOpenMessage, TermResizeMessage,
};
pub use common::{
    CloseMessage, ErrorMessage, HeartbeatAckMessage, LoadAvg, MemStats, OtaStage, TermCloseMessage,
};
pub use device_to_cloud::{
    AuditMessage, CmdResponseMessage, ConfigAckMessage, DeviceInfoMessage, DeviceToCloud,
    FileChunkMessage, HeartbeatMessage, HttpRespMessage, OtaAckMessage, RegisterMessage,
    TermDataMessage,
};
pub use envelope::Envelope;
pub use rustdesk_compat::{
    RustDeskAcceptanceGate, RustDeskAcceptanceGateKind, RustDeskAcceptanceGateStatus,
    RustDeskCompatibilityMode, RustDeskCompatibilityPlan, RustDeskFeatureFlags,
    RustDeskInfrastructurePolicy, RustDeskRouteKind, RustDeskVersionPinMetadata,
    RUSTDESK_COMPAT_CONTRACT_VERSION, RUSTDESK_REQUIRED_ACCEPTANCE_GATES,
};

/// Wire-protocol version surfaced via the `X-Protocol-Version`
/// upgrade header. Bumping this string is a hard protocol break.
pub const PROTOCOL_VERSION: &str = "2";

/// Maximum WebSocket text frame this protocol accepts (256 KB per
/// `WIRE-PROTOCOL.md §sizes`). Larger payloads must be split — see
/// `file_chunk` / `http_resp` / `http_req` `last` flag.
pub const MAX_FRAME_BYTES: usize = 256 * 1024;

/// Maximum in-flight bytes per device WebSocket (4 MB).
pub const MAX_IN_FLIGHT_BYTES: usize = 4 * 1024 * 1024;

/// Maximum concurrent sessions per device (SSH + HTTP-proxy combined).
pub const MAX_SESSIONS_PER_DEVICE: usize = 64;

/// Rate cap for `term_data` per session (1 MB/s).
pub const TERM_DATA_RATE_LIMIT_BYTES_PER_SEC: u64 = 1024 * 1024;
