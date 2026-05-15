//! Framing envelope helpers.
//!
//! The v2 protocol's `type`/`id`/`ts` triplet is encoded directly on
//! each variant via `#[serde(tag = "type")]` plus per-variant
//! `id`/`ts` fields, so callers don't strictly need a separate
//! envelope struct. This module exists for the *inspection* path —
//! e.g. metrics that want to extract the frame's discriminant + id
//! without decoding the full payload.
//!
//! ## Usage
//!
//! ```ignore
//! use bereme_protocol::Envelope;
//! let env: Envelope = serde_json::from_str(raw)?;
//! tracing::info!(ty = %env.r#type, id = ?env.id, "rx frame");
//! ```

use serde::{Deserialize, Serialize};

/// Generic frame envelope. Use this for dispatch + audit hooks that
/// need the discriminant before full typed decode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// snake_case discriminant matching one of the enum variants in
    /// [`crate::DeviceToCloud`] or [`crate::CloudToDevice`].
    pub r#type: String,
    /// Correlation ID (UUID v4) on request/response pairs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Sender-set RFC 3339 UTC timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ts: Option<String>,
}
