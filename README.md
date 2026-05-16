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

The shape of every variant is exercised by `tests/round_trip.rs`'s 32 serde round-trip tests. CI gates fmt, clippy, doc, and MSRV.

## Why a separate crate

Wave-05 of the Bereme KVM hardening pass extracted the wire types from `cloud-rust/crates/cloud-protocol` so the device firmware (`kvm-rust/crates/kvm-cloud`) and the cloud control plane (`cloud-rust/crates/cloud-protocol`) could both depend on the *same* serde-tagged Rust types — eliminating the prior `serde_json::Value` channels on the device side and the silent shape drift between repos.

Both production repos consume this crate via git-dep pinned to a tag:

```toml
# kvm-rust/Cargo.toml (workspace.dependencies)
bereme-protocol = { git = "https://github.com/Waiel5/bereme-protocol.git", tag = "bereme-protocol-v0.1.1" }

# cloud-rust/Cargo.toml (workspace.dependencies) — identical
```

Bumping the tag is a coordinated three-repo operation: tag this crate, then bump the pin in both consumer repos in the same PR/release window so wire compatibility stays in lock-step.

## Wire shape (summary)

`DeviceToCloud` — 14 variants:

```
register, heartbeat, pong, device_info, audit_event, config_ack,
ota_ack, term_open, term_input, term_resize, term_close, term_data,
cmd_result, http_response, file_chunk, error
```

`CloudToDevice` — 13 variants:

```
register_ack, ping, config_nudge, ota_push, term_open, term_data,
term_resize, term_close, cmd, http_req, heartbeat_ack, file_chunk,
error
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
    pub deadline_unix:       Option<i64>,
}

pub struct OtaAckMessage {
    pub plan_id:      String,
    pub stage:        OtaStage,         // Verifying | Applying | RolledForward | Failed
    pub version_code: u64,
    pub error:        Option<String>,
}
```

## Building / testing

```sh
cargo build
cargo test                                                # 32 round-trip variant tests
cargo test --all-features
cargo doc --no-deps --all-features                        # public API surface
cargo clippy --all-targets --all-features -- -D warnings
```

MSRV is **Rust 1.85**. CI runs the MSRV gate on every push.

## Versioning policy

SemVer-compatible. The package version is the wire-compat marker:

- **0.1.x** — `PROTOCOL_VERSION = "2"` baseline. The wave-05 closure shipped 0.1.1; 0.1.0 differs only in `tests/round_trip.rs` rustfmt drift.
- A **new wire variant** that the cloud emits but the device does not understand is a `0.2.0` bump (cloud-only minor). The device-side deserializer tolerates unknown variants by warn-logging and continuing, so consumers will not crash — but the new variant's payload will be ignored on old devices.
- A **renamed or removed variant** is a `1.0.0` bump (breaking).

## Companion repos

- [`Waiel5/kvm-rust`](https://github.com/Waiel5/kvm-rust) — device firmware. `kvm-cloud` consumes this crate.
- [`Waiel5/cloud-rust`](https://github.com/Waiel5/cloud-rust) — cloud control plane. `cloud-protocol` re-exports from this crate (shim).
- [`Waiel5/bereme-e2e`](https://github.com/Waiel5/bereme-e2e) — cross-repo end-to-end tests.
- [`Waiel5/bereme-kvm-docs`](https://github.com/Waiel5/bereme-kvm-docs) — cross-cutting architecture + threat model.

## License

See LICENSE in this repo (matches the workspace policy of the consuming repos).
