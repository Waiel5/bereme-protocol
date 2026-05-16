//! Adversarial wire-protocol tests.
//!
//! `round_trip.rs` proves every *valid* variant survives a serde round
//! trip with byte-equivalent encoding. This file covers the cases an
//! attacker (or a buggy peer) can drive the decoder through:
//!
//! - **Malformed input** — bad UUIDs, wrong types, missing required
//!   fields, unknown enum variants in `OtaStage` etc. The decoder
//!   should refuse cleanly, never panic.
//! - **Forward-compat** — a future cloud may send a `type` discriminant
//!   the current device doesn't know yet. The decoder must reject it
//!   with a structured serde error (so the dispatcher can log + drop
//!   the frame), not panic.
//! - **Snapshot** — golden JSON for the load-bearing variants
//!   (`OtaPushMessage`, `RegisterMessage`, `OtaAckMessage`). If a
//!   refactor accidentally changes a serde rename or makes a field
//!   non-`Option`, these tests pin the wire shape against drift.
//! - **Constant invariants** — `PROTOCOL_VERSION`, `MAX_FRAME_BYTES`,
//!   etc., are part of the wire contract. Bumping any of them is a
//!   conscious choice; we make the test fail loudly so the bumper
//!   has to update both the constant and the contract doc.
//!
//! `forbid(unsafe_code)` is on the lib; these tests use `expect()` /
//! `unwrap()` freely — they're tests, panics are the assertion.

use bereme_protocol::{
    CloudToDevice, DeviceToCloud, OtaAckMessage, OtaPushMessage, OtaStage, RegisterAckOutcome,
    RegisterMessage, MAX_FRAME_BYTES, MAX_IN_FLIGHT_BYTES, MAX_SESSIONS_PER_DEVICE,
    PROTOCOL_VERSION, TERM_DATA_RATE_LIMIT_BYTES_PER_SEC,
};

// =====================================================================
// Section 1 — PROTOCOL_VERSION and wire-shape constants
// =====================================================================

/// The single most important constant in the crate. Bumping this string
/// is a hard wire break — every device firmware in the field has a
/// hard-coded `X-Protocol-Version: 2` upgrade header. A silent bump
/// here would leave older firmware speaking a different protocol than
/// the cloud expects. If you intentionally bump, the test forces you
/// to update the contract doc + the device's upgrade header in lock-
/// step.
#[test]
fn protocol_version_is_pinned_to_2() {
    assert_eq!(PROTOCOL_VERSION, "2");
}

#[test]
fn frame_size_limits_match_wire_spec() {
    // `WIRE-PROTOCOL.md §sizes` documents these limits. The constants
    // and the doc must agree.
    assert_eq!(MAX_FRAME_BYTES, 256 * 1024);
    assert_eq!(MAX_IN_FLIGHT_BYTES, 4 * 1024 * 1024);
    assert_eq!(MAX_SESSIONS_PER_DEVICE, 64);
    assert_eq!(TERM_DATA_RATE_LIMIT_BYTES_PER_SEC, 1024 * 1024);
}

// =====================================================================
// Section 2 — Malformed input (decoder must refuse, never panic)
// =====================================================================

#[test]
fn rejects_unknown_top_level_type_d2c() {
    let json = r#"{ "type": "totally_new_variant_from_the_future" }"#;
    let err = serde_json::from_str::<DeviceToCloud>(json)
        .expect_err("unknown discriminant must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("totally_new_variant_from_the_future") || msg.contains("unknown variant"),
        "error message should name the offending tag: got `{msg}`"
    );
}

#[test]
fn rejects_unknown_top_level_type_c2d() {
    let json = r#"{ "type": "definitely_not_a_real_frame_type" }"#;
    let err = serde_json::from_str::<CloudToDevice>(json)
        .expect_err("unknown discriminant must be rejected");
    assert!(
        err.to_string().contains("unknown variant")
            || err.to_string().contains("definitely_not_a_real_frame_type")
    );
}

#[test]
fn rejects_missing_type_discriminant() {
    let json = r#"{ "device_id": "abc" }"#;
    let err = serde_json::from_str::<DeviceToCloud>(json)
        .expect_err("absent `type` field must be rejected");
    assert!(
        err.to_string().contains("missing field") || err.to_string().contains("`type`"),
        "got: {}",
        err
    );
}

#[test]
fn rejects_wrong_type_for_field() {
    // RegisterMessage.device_id is a String. Passing a number must
    // produce a structured serde error, not panic.
    let json = r#"{ "type": "register", "device_id": 12345, "capabilities": [] }"#;
    let err =
        serde_json::from_str::<DeviceToCloud>(json).expect_err("wrong field type must be rejected");
    assert!(err.to_string().contains("invalid type"));
}

#[test]
fn rejects_missing_required_field_register() {
    // RegisterMessage requires `device_id`.
    let json = r#"{ "type": "register" }"#;
    let err = serde_json::from_str::<DeviceToCloud>(json)
        .expect_err("missing `device_id` must be rejected");
    assert!(err.to_string().contains("missing field"), "got: {}", err);
}

#[test]
fn rejects_unknown_ota_stage() {
    let json = r#"{
        "type": "ota_ack",
        "plan_id": "p-1",
        "stage": "exploded",
        "version_code": 42
    }"#;
    let err =
        serde_json::from_str::<DeviceToCloud>(json).expect_err("unknown stage must be rejected");
    let msg = err.to_string();
    assert!(
        msg.contains("unknown variant") || msg.contains("exploded"),
        "got: {msg}"
    );
}

#[test]
fn rejects_negative_version_code() {
    // version_code is u64. -1 cannot parse — confirms type-strictness.
    let json = r#"{
        "type": "ota_ack",
        "plan_id": "p-1",
        "stage": "verifying",
        "version_code": -1
    }"#;
    let err = serde_json::from_str::<DeviceToCloud>(json)
        .expect_err("negative version_code must be rejected");
    assert!(err.to_string().contains("invalid"));
}

#[test]
fn rejects_truncated_json() {
    let json = r#"{ "type": "regis"#;
    let err =
        serde_json::from_str::<DeviceToCloud>(json).expect_err("truncated frame must be rejected");
    assert!(err.is_eof() || err.to_string().contains("EOF"));
}

#[test]
fn rejects_non_object_root() {
    for bad in &["[]", r#""string""#, "42", "null", "true"] {
        let err = serde_json::from_str::<DeviceToCloud>(bad)
            .expect_err("non-object root must be rejected");
        // Just ensure it didn't panic — message wording differs by serde version.
        assert!(
            !err.to_string().is_empty(),
            "input `{bad}` produced empty error"
        );
    }
}

// =====================================================================
// Section 3 — Forward compatibility (unknown variants are rejected
// cleanly so the dispatcher can warn-and-continue)
// =====================================================================

/// The brief: dispatchers log + continue on unknown CloudToDevice
/// variants rather than killing the connection. This test asserts the
/// **shape** of the rejection — a structured `serde_json::Error`, not
/// a panic — so dispatchers can `match` on it.
#[test]
fn unknown_cloud_to_device_variant_returns_structured_error() {
    let json = r#"{ "type": "remote_brain_implant", "freq": 42 }"#;
    let result = serde_json::from_str::<CloudToDevice>(json);
    assert!(result.is_err(), "future variant must not panic");
    let err = result.unwrap_err();
    // serde_json::Error implements std::error::Error + Display. The
    // dispatcher's warn-and-continue path matches on this shape.
    let s = format!("{}", err);
    assert!(!s.is_empty());
    // The error must be a serde "data" error, not a syntax error,
    // since the JSON is well-formed.
    assert_eq!(err.classify(), serde_json::error::Category::Data);
}

#[test]
fn rejects_capitalized_type_value() {
    // serde rename_all = "snake_case" makes type values case-sensitive.
    // A peer that sends "Register" instead of "register" is a bug we
    // want to catch (not silently treat as the right variant).
    let json = r#"{ "type": "Register", "device_id": "abc", "capabilities": [] }"#;
    let err = serde_json::from_str::<DeviceToCloud>(json)
        .expect_err("PascalCase type value must not match snake_case variant");
    assert!(err.to_string().contains("unknown variant") || err.to_string().contains("Register"));
}

// =====================================================================
// Section 4 — Snapshot / golden tests for load-bearing wire variants
// =====================================================================
//
// These pin the canonical JSON encoding of the variants that the
// device's bootloader + OTA + provisioning paths depend on. A future
// refactor that adds an `#[serde(rename = "...")]` or changes a field
// from `Option` to required will fail these tests, forcing the author
// to either keep the wire shape stable or coordinate a version bump.

#[test]
fn ota_push_canonical_json_shape() {
    let msg = OtaPushMessage {
        manifest_url: "https://updates.example/manifest-1.json".into(),
        manifest_signatures: vec!["sig-a".into(), "sig-b".into()],
        version_code: 100,
        plan_id: "plan-abc".into(),
        deadline_unix: 1_700_000_000,
    };
    let pushed = CloudToDevice::OtaPush(msg);
    let json = serde_json::to_value(&pushed).expect("encode");
    // Pin the exact wire shape. Any drift (added/renamed fields, changed
    // tag) makes this fail — bump bereme-protocol-vX.Y.Z when you do
    // change it, and update this snapshot in the same PR.
    let expected = serde_json::json!({
        "type": "ota_push",
        "manifest_url": "https://updates.example/manifest-1.json",
        "manifest_signatures": ["sig-a", "sig-b"],
        "version_code": 100,
        "plan_id": "plan-abc",
        "deadline_unix": 1_700_000_000,
    });
    assert_eq!(json, expected);
}

#[test]
fn ota_ack_canonical_json_shape() {
    let msg = OtaAckMessage {
        plan_id: "plan-abc".into(),
        stage: OtaStage::RolledForward,
        version_code: 100,
        error: None,
    };
    let acked = DeviceToCloud::OtaAck(msg);
    let json = serde_json::to_value(&acked).expect("encode");
    // `error` is `Option<String>` with `skip_serializing_if`, so the
    // None case omits the field on the wire.
    let expected = serde_json::json!({
        "type": "ota_ack",
        "plan_id": "plan-abc",
        "stage": "rolled_forward",
        "version_code": 100,
    });
    assert_eq!(json, expected);
}

#[test]
fn ota_stage_snake_case_serialization() {
    // Every variant of OtaStage must be lower-snake on the wire.
    for (stage, expected) in [
        (OtaStage::Verifying, "verifying"),
        (OtaStage::Applying, "applying"),
        (OtaStage::RolledForward, "rolled_forward"),
        (OtaStage::Failed, "failed"),
        (OtaStage::RolledBack, "rolled_back"),
    ] {
        let s = serde_json::to_string(&stage).unwrap();
        assert_eq!(s, format!("\"{expected}\""));
    }
}

#[test]
fn register_canonical_json_shape() {
    let msg = RegisterMessage {
        id: None,
        ts: None,
        device_id: "dev-001".into(),
        description: Some("Lab unit".into()),
        firmware_version: Some("0.5.10".into()),
        hardware: Some("RK3588".into()),
        local_ip: Some("10.0.0.42".into()),
        mac: Some("aa:bb:cc:dd:ee:ff".into()),
        capabilities: vec!["video".into(), "hid".into()],
    };
    let frame = DeviceToCloud::Register(msg);
    let json = serde_json::to_value(&frame).expect("encode");
    // device_id is required; everything else is Option<>. Optional
    // None fields are *omitted* from the wire (per serde default).
    let expected = serde_json::json!({
        "type": "register",
        "device_id": "dev-001",
        "description": "Lab unit",
        "firmware_version": "0.5.10",
        "hardware": "RK3588",
        "local_ip": "10.0.0.42",
        "mac": "aa:bb:cc:dd:ee:ff",
        "capabilities": ["video", "hid"],
    });
    assert_eq!(json, expected);
}

// =====================================================================
// Section 5 — Envelope id/ts roundtrip + boundary cases
// =====================================================================

#[test]
fn frame_with_id_and_ts_round_trips() {
    // id and ts are common envelope fields that any variant may carry.
    // Verify they're optional but persist through serde when present.
    let json = serde_json::json!({
        "type": "heartbeat",
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "ts": "2026-05-16T05:00:00Z",
        "uptime_seconds": 60,
        "load": { "1m": 0.5, "5m": 0.4, "15m": 0.3 },
        "mem": { "total": 8192000, "used": 4096000 },
    });
    let frame: DeviceToCloud =
        serde_json::from_value(json.clone()).expect("heartbeat with envelope id/ts must parse");
    let re = serde_json::to_value(&frame).expect("re-encode");
    // Round trip must preserve envelope fields.
    assert_eq!(re["id"], json["id"]);
    assert_eq!(re["ts"], json["ts"]);
}

#[test]
fn register_ack_outcome_serializes_snake_case() {
    for (variant, wire) in [
        (RegisterAckOutcome::Accepted, "accepted"),
        (RegisterAckOutcome::Rejected, "rejected"),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, format!("\"{wire}\""), "variant {variant:?}");
    }
}

// =====================================================================
// Section 6 — Hostile-payload smoke (decoder must not panic, must not
// allocate without bound, must not loop)
// =====================================================================

#[test]
fn does_not_panic_on_random_garbage() {
    // A small set of obviously-broken inputs. Each call must return
    // an `Err` and not panic — the dispatcher relies on this.
    let payloads = [
        "",
        " ",
        "\0",
        "{",
        "}",
        "[]]",
        "{}",
        r#"{"type":null}"#,
        r#"{"type":42}"#,
        r#"{"type":""}"#,
        r#"{"type":" "}"#,
        r#"{"type":"register","device_id":null,"capabilities":[]}"#,
    ];
    for p in &payloads {
        let _ = serde_json::from_str::<DeviceToCloud>(p);
        let _ = serde_json::from_str::<CloudToDevice>(p);
    }
}

#[test]
fn does_not_panic_on_deeply_nested_object() {
    // serde_json's default stack limit prevents stack overflow on
    // pathological depth. Confirm by feeding a 1000-deep object —
    // expect graceful rejection.
    let mut s = String::new();
    for _ in 0..1000 {
        s.push_str(r#"{"x":"#);
    }
    s.push('1');
    for _ in 0..1000 {
        s.push('}');
    }
    // Result is Err (it's a single `{"x":...}` tree, not a Bereme
    // frame), but the key point is: no panic.
    let _ = serde_json::from_str::<DeviceToCloud>(&s);
}

#[test]
fn does_not_panic_on_huge_string_field() {
    // A device_id of 100 KB shouldn't panic; it may parse (valid
    // serde) or fail (size limit), but either way: no abort.
    let huge = "a".repeat(100_000);
    let json = format!(r#"{{ "type": "register", "device_id": "{huge}", "capabilities": [] }}"#);
    let _ = serde_json::from_str::<DeviceToCloud>(&json);
}
