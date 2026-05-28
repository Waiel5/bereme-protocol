# bereme-protocol

Typed, versioned wire-protocol crate for Bereme KVM. The shared source of truth for every message that crosses the device ↔ cloud WebSocket.

[![ci](https://github.com/Waiel5/bereme-protocol/actions/workflows/ci.yml/badge.svg)](https://github.com/Waiel5/bereme-protocol/actions/workflows/ci.yml)
[![crate-version](https://img.shields.io/badge/crate-0.1.1-blue)](https://github.com/Waiel5/bereme-protocol/releases/tag/bereme-protocol-v0.1.1)
[![MSRV](https://img.shields.io/badge/rustc-1.85+-orange)](https://forge.rust-lang.org/release/version-stabilizations.html)

## What this crate is

A small Rust library (≤ 100 LOC of types per direction; no runtime, no async) that defines:

- `pub const PROTOCOL_VERSION: &str = "2"` — the value the device sends in the `X-Protocol-Version` WebSocket-upgrade header.
- `enum DeviceToCloud` — every frame the device can send. Tagged with `#[serde(tag = "type", rename_all = "snake_case")]` so the wire form is `{"type": "register", ...}`.
- `enum CloudToDevice` — every frame the cloud can send. Same tag pattern.
- Per-variant payload structs (`RegisterMessage`, `HeartbeatMessage`, `OtaPushMessage`, `OtaAckMessage`, etc.) with explicit field names that match the documented wire spec verbatim.
- `Envelope` — the common id/ts wrapper that frames carry for correlation.
- `RustDeskCompatibilityPlan` and related `rustdesk_compat` types — a Bereme-owned planning contract for a future RustDesk compatibility route. These are metadata/review types only; they do not implement, copy, or describe RustDesk wire protocol.

The shape of every variant is exercised by `tests/round_trip.rs`'s 32 serde round-trip tests. CI gates fmt, clippy, doc, and MSRV.

## Why a separate crate

The wire types live in this separate crate so the device firmware
(`kvm-rust/crates/kvm-cloud`) and the cloud control plane
(`cloud-rust/crates/cloud-protocol`) both depend on the same serde-tagged Rust
types. That eliminates ad hoc `serde_json::Value` channels on the device side
and prevents silent shape drift between repos.

This local checkout consumes the crate through sibling path dependencies:

```toml
# kvm-rust/Cargo.toml (workspace.dependencies)
bereme-protocol = { path = "../bereme-protocol" }

# cloud-rust/Cargo.toml (workspace.dependencies) — same sibling path
```

Release builds may pin a git tag. Bumping a release tag is a coordinated
three-repo operation: tag this crate, then bump both consumer repos in the same
release window so wire compatibility stays in lock-step.

## Wire shape (summary)

`DeviceToCloud` — 14 variants:

```
register, heartbeat, pong, term_data, term_close, file_chunk,
http_resp, cmd_response, audit, device_info, config_ack, ota_ack,
error, close
```

`CloudToDevice` — 13 variants:

```
register_ack, heartbeat_ack, ping, term_open, term_resize,
term_input, term_close, cmd, http_req, config_nudge, ota_push,
error, close
```

Both enums use `#[serde(tag = "type", rename_all = "snake_case")]`. Every frame travels as a JSON text WebSocket message. The envelope provides:

```rust
pub struct Envelope {
    pub r#type: String,          // matches the `type` tag above
    pub id:    Option<String>,   // uuid v4, for correlation
    pub ts:    Option<String>,   // RFC 3339; the device clock
}
```

OTA round trip is split into push and ack:

```rust
pub struct OtaPushMessage {
    pub manifest_url:        String,
    pub manifest_signatures: Vec<String>,
    pub version_code:        u64,
    pub plan_id:             String,
    pub deadline_unix:       i64,
}

pub struct OtaAckMessage {
    pub plan_id:      String,
    pub stage:        OtaStage,         // Verifying | Applying | RolledForward | Failed | RolledBack
    pub version_code: u64,
    pub error:        Option<String>,
}
```

## RustDesk compatibility planning

`rustdesk_compat` defines a small, serde-stable Bereme contract for planning a
future compatibility bridge or fork:

- `RustDeskCompatibilityMode` — `disabled`, `inventory_only`, `shadow`, `pilot`, `enabled`.
- `RustDeskRouteKind` — `disabled`, `native_bridge`, `fork`.
- `RustDeskInfrastructurePolicy` — `disabled`, `self_hosted_only`, `operator_managed`, `public_allowed`.
- `RustDeskFeatureFlags` — Bereme capability booleans such as display stream, input control, file transfer, clipboard sync, audio stream, relay/rendezvous, unattended access, and audit correlation.
- `RustDeskVersionPinMetadata` — optional upstream/adapter revision metadata and review notes.
- `RustDeskAcceptanceGate` — review gates for license, security, protocol-boundary, interop, performance, audit integration, airgap egress, and rollback readiness.

These types are intentionally not RustDesk protocol schemas. They are safe
planning metadata for downstream bridge work, with defaults that deserialize
older/minimal JSON to a disabled, non-activatable plan.

## Building / testing

```sh
cargo build
cargo test                                                # serde round-trip + compatibility planning tests
cargo test --all-features
cargo doc --no-deps --all-features                        # public API surface
cargo clippy --all-targets --all-features -- -D warnings
```

MSRV is **Rust 1.85**. CI runs the MSRV gate on every push.

## Versioning policy

SemVer-compatible. The package version is the wire-compat marker:

- **0.1.x** — `PROTOCOL_VERSION = "2"` baseline.
- A **new wire variant** that the cloud emits but the device does not understand is a `0.2.0` bump (cloud-only minor). The device-side deserializer tolerates unknown variants by warn-logging and continuing, so consumers will not crash — but the new variant's payload will be ignored on old devices.
- A **renamed or removed variant** is a `1.0.0` bump (breaking).

## Companion repos

- [`Waiel5/kvm-rust`](https://github.com/Waiel5/kvm-rust) — device firmware. `kvm-cloud` consumes this crate.
- [`Waiel5/cloud-rust`](https://github.com/Waiel5/cloud-rust) — cloud control plane. `cloud-protocol` re-exports this crate's types.
- [`Waiel5/bereme-e2e`](https://github.com/Waiel5/bereme-e2e) — cross-repo end-to-end tests.
- [`Waiel5/bereme-kvm-docs`](https://github.com/Waiel5/bereme-kvm-docs) — cross-cutting architecture + threat model.

## License

See LICENSE in this repo (matches the workspace policy of the consuming repos).
