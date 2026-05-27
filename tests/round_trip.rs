//! Wave-05 Phase 4: round-trip identity + byte-equivalence coverage
//! for every variant of [`DeviceToCloud`] and [`CloudToDevice`].
//!
//! Each test:
//!   1. constructs a typed value;
//!   2. serializes it to JSON;
//!   3. deserializes the result back into the same enum;
//!   4. asserts structural equality (`v == round_trip(v)`);
//!   5. re-serializes the parsed value and asserts API-compatible
//!      equivalence with the first serialization (`encode(v) ==
//!      encode(decode(encode(v)))`).
//!
//! The byte-equivalence property is what the Wave-2 parity test
//! (`cloud-protocol/tests/kvm_cloud_compat.rs`) was approximating
//! when the device side still used `serde_json::Value`. With both
//! sides now sharing this crate, byte-equivalence is the identity
//! check the parity tests promised.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic
)]

use bereme_protocol::{
    AuditMessage, CloseMessage, CloudToDevice, CmdMessage, CmdResponseMessage, ConfigAckMessage,
    ConfigNudgeMessage, DeviceInfoMessage, DeviceToCloud, Envelope, ErrorMessage, FileChunkMessage,
    HeartbeatAckMessage, HeartbeatMessage, HttpReqMessage, HttpRespMessage, LoadAvg, MemStats,
    OtaAckMessage, OtaPushMessage, OtaStage, RegisterAckMessage, RegisterAckOutcome,
    RegisterMessage, TermCloseMessage, TermDataMessage, TermInputMessage, TermOpenMessage,
    TermResizeMessage,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

// ----------------------------------------------------------------------
// Round-trip helpers.
// ----------------------------------------------------------------------

fn assert_round_trip_d2c(orig: &DeviceToCloud) {
    let json = serde_json::to_string(orig).unwrap();
    let back: DeviceToCloud = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, &back, "structural inequality after round-trip");
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(
        json, json2,
        "byte-equivalence broke between successive encodes"
    );
}

fn assert_round_trip_c2d(orig: &CloudToDevice) {
    let json = serde_json::to_string(orig).unwrap();
    let back: CloudToDevice = serde_json::from_str(&json).unwrap();
    assert_eq!(orig, &back, "structural inequality after round-trip");
    let json2 = serde_json::to_string(&back).unwrap();
    assert_eq!(
        json, json2,
        "byte-equivalence broke between successive encodes"
    );
}

// ----------------------------------------------------------------------
// DeviceToCloud variants (one test per variant).
// ----------------------------------------------------------------------

#[test]
fn d2c_register_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Register(RegisterMessage {
        id: Some("11111111-2222-3333-4444-555555555555".into()),
        ts: Some("2026-05-13T00:00:00Z".into()),
        device_id: "lv99862".into(),
        description: Some("rack 3 / desk 2".into()),
        firmware_version: Some("0.5.9".into()),
        hardware: Some("rk3588".into()),
        local_ip: Some("10.0.5.42".into()),
        mac: Some("aa:bb:cc:dd:ee:ff".into()),
        capabilities: vec!["ssh".into(), "http_proxy".into(), "file".into()],
    }));
}

#[test]
fn d2c_heartbeat_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Heartbeat(HeartbeatMessage {
        id: Some("hb-001".into()),
        ts: Some("2026-05-13T00:00:15Z".into()),
        uptime_seconds: 1234,
        load: LoadAvg {
            one_m: 0.05,
            five_m: 0.10,
            fifteen_m: 0.15,
        },
        mem: MemStats {
            total: 4_000_000_000,
            used: 800_000_000,
        },
    }));
}

#[test]
fn d2c_pong_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Pong);
}

#[test]
fn d2c_term_data_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::TermData(TermDataMessage {
        sid: "33333333-1111-1111-1111-111111111111".into(),
        data: "aGVsbG8sIHdvcmxk".into(),
    }));
}

#[test]
fn d2c_term_close_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::TermClose(TermCloseMessage {
        sid: "sid".into(),
        code: 0,
        reason: "operator closed".into(),
    }));
}

#[test]
fn d2c_file_chunk_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::FileChunk(FileChunkMessage {
        sid: "f1".into(),
        seq: 7,
        data: "AAAA".into(),
        last: true,
    }));
}

#[test]
fn d2c_http_resp_round_trips() {
    let mut headers = BTreeMap::new();
    headers.insert("content-type".into(), "text/plain".into());
    assert_round_trip_d2c(&DeviceToCloud::HttpResp(HttpRespMessage {
        sid: "h1".into(),
        status: 200,
        headers,
        body_chunk: "T0s=".into(),
        last: true,
    }));
}

#[test]
fn d2c_cmd_response_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::CmdResponse(CmdResponseMessage {
        id: "cccccc".into(),
        code: 0,
        stdout: "ok\n".into(),
        stderr: String::new(),
        attrs: json!({ "exec_ms": 12 }),
    }));
}

#[test]
fn d2c_audit_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Audit(AuditMessage {
        record: json!({"event": "auth_success", "actor": "admin"}),
    }));
}

#[test]
fn d2c_device_info_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::DeviceInfo(DeviceInfoMessage {
        client: Some("kvm-rust".into()),
        os: Some("linux".into()),
        hostname: Some("kvm-1".into()),
        local_ip: Some("10.0.0.4".into()),
        kernel: Some("6.6".into()),
        model: Some("RK3588".into()),
    }));
}

#[test]
fn d2c_config_ack_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::ConfigAck(ConfigAckMessage {
        id: Some("nudge-1".into()),
        outcome: "applied".into(),
    }));
}

#[test]
fn d2c_ota_ack_round_trips_all_stages() {
    for stage in [
        OtaStage::Verifying,
        OtaStage::Applying,
        OtaStage::RolledForward,
        OtaStage::Failed,
        OtaStage::RolledBack,
    ] {
        let err = matches!(stage, OtaStage::Failed | OtaStage::RolledBack)
            .then(|| "signature mismatch".to_string());
        assert_round_trip_d2c(&DeviceToCloud::OtaAck(OtaAckMessage {
            plan_id: "plan-001".into(),
            stage,
            version_code: 42,
            error: err,
        }));
    }
}

#[test]
fn d2c_error_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Error(ErrorMessage {
        id: Some("e1".into()),
        code: "E_AUTH".into(),
        message: "bad token".into(),
        terminal: true,
    }));
}

#[test]
fn d2c_close_round_trips() {
    assert_round_trip_d2c(&DeviceToCloud::Close(CloseMessage {
        code: 1000,
        reason: "normal".into(),
    }));
}

// ----------------------------------------------------------------------
// CloudToDevice variants (one test per variant).
// ----------------------------------------------------------------------

#[test]
fn c2d_register_ack_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::RegisterAck(RegisterAckMessage {
        id: "11111111-2222-3333-4444-555555555555".into(),
        outcome: RegisterAckOutcome::Accepted,
        reason: None,
        device_id: Some("lv99862".into()),
        heartbeat_interval_seconds: Some(15),
    }));
}

#[test]
fn c2d_register_ack_rejected_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::RegisterAck(RegisterAckMessage {
        id: "abcd".into(),
        outcome: RegisterAckOutcome::Rejected,
        reason: Some("device blocked".into()),
        device_id: None,
        heartbeat_interval_seconds: None,
    }));
}

#[test]
fn c2d_heartbeat_ack_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::HeartbeatAck(HeartbeatAckMessage {
        id: Some("h".into()),
        extra: Value::Null,
    }));
}

#[test]
fn c2d_ping_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::Ping);
}

#[test]
fn c2d_term_open_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::TermOpen(TermOpenMessage {
        sid: "sid".into(),
        cols: 80,
        rows: 24,
        user: Some("root".into()),
        auth: json!({ "method": "password" }),
    }));
}

#[test]
fn c2d_term_resize_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::TermResize(TermResizeMessage {
        sid: "sid".into(),
        cols: 120,
        rows: 30,
    }));
}

#[test]
fn c2d_term_input_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::TermInput(TermInputMessage {
        sid: "sid".into(),
        data: "AA==".into(),
    }));
}

#[test]
fn c2d_term_close_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::TermClose(TermCloseMessage {
        sid: "sid".into(),
        code: 0,
        reason: String::new(),
    }));
}

#[test]
fn c2d_cmd_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::Cmd(CmdMessage {
        id: "x".into(),
        username: Some("admin".into()),
        cmd: "uptime".into(),
        params: vec![],
        timeout_ms: Some(2000),
    }));
}

#[test]
fn c2d_http_req_round_trips() {
    let mut headers = BTreeMap::new();
    headers.insert("accept".into(), "*/*".into());
    assert_round_trip_c2d(&CloudToDevice::HttpReq(HttpReqMessage {
        sid: "h".into(),
        method: "GET".into(),
        path: "/api".into(),
        query: "page=1".into(),
        headers,
        body_chunk: String::new(),
        last: true,
    }));
}

#[test]
fn c2d_config_nudge_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::ConfigNudge(ConfigNudgeMessage {
        keys: json!({"heartbeat_interval_seconds": 30}),
    }));
}

#[test]
fn c2d_ota_push_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::OtaPush(OtaPushMessage {
        manifest_url: "https://updates.bereme.com/manifest-42.json".into(),
        manifest_signatures: vec!["base64-pki-sig".into(), "base64-witness-sig".into()],
        version_code: 42,
        plan_id: "plan-001".into(),
        deadline_unix: 1_780_000_000,
    }));
}

#[test]
fn c2d_error_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::Error(ErrorMessage {
        id: Some("e2".into()),
        code: "E_POLICY".into(),
        message: "rate limit".into(),
        terminal: false,
    }));
}

#[test]
fn c2d_close_round_trips() {
    assert_round_trip_c2d(&CloudToDevice::Close(CloseMessage {
        code: 1011,
        reason: "server error".into(),
    }));
}

// ----------------------------------------------------------------------
// Wire-shape spot checks.
// ----------------------------------------------------------------------

/// The OtaPush wire shape must match the field names listed in the
/// wave-05 Phase 4 brief verbatim. This guards against a serde
/// rename slip silently breaking the cloud-domain ledger consumer.
#[test]
fn ota_push_wire_shape_matches_brief() {
    let msg = CloudToDevice::OtaPush(OtaPushMessage {
        manifest_url: "https://example/m.json".into(),
        manifest_signatures: vec!["a".into(), "b".into()],
        version_code: 7,
        plan_id: "plan-7".into(),
        deadline_unix: 1_700_000_000,
    });
    let v: Value = serde_json::to_value(&msg).unwrap();
    let expected = json!({
        "type": "ota_push",
        "manifest_url": "https://example/m.json",
        "manifest_signatures": ["a", "b"],
        "version_code": 7,
        "plan_id": "plan-7",
        "deadline_unix": 1_700_000_000_i64,
    });
    assert_eq!(v, expected);
}

/// Same shape-check for OtaAck. Field names + OtaStage discriminant
/// strings are load-bearing for the cloud's plan ledger update.
#[test]
fn ota_ack_wire_shape_matches_brief() {
    let msg = DeviceToCloud::OtaAck(OtaAckMessage {
        plan_id: "plan-7".into(),
        stage: OtaStage::RolledForward,
        version_code: 7,
        error: None,
    });
    let v: Value = serde_json::to_value(&msg).unwrap();
    let expected = json!({
        "type": "ota_ack",
        "plan_id": "plan-7",
        "stage": "rolled_forward",
        "version_code": 7,
    });
    assert_eq!(v, expected);
}

/// `Envelope` parses the discriminant + id without decoding the full
/// payload. Used by dispatch + audit hooks that only care about the
/// frame type.
#[test]
fn envelope_extracts_type_id_ts() {
    let raw = r#"{"type":"register","id":"deadbeef","ts":"2026-05-13T00:00:00Z","device_id":"x"}"#;
    let env: Envelope = serde_json::from_str(raw).unwrap();
    assert_eq!(env.r#type, "register");
    assert_eq!(env.id.as_deref(), Some("deadbeef"));
    assert_eq!(env.ts.as_deref(), Some("2026-05-13T00:00:00Z"));
}

/// Unknown discriminants fail closed (typed parse error). The
/// kvm-cloud-side `Envelope` path can still extract `type` from the
/// raw bytes for logging.
#[test]
fn unknown_discriminant_fails_typed_decode() {
    let raw = r#"{"type":"future_variant","x":1}"#;
    let d2c_err = serde_json::from_str::<DeviceToCloud>(raw).unwrap_err();
    let c2d_err = serde_json::from_str::<CloudToDevice>(raw).unwrap_err();
    assert!(d2c_err.to_string().contains("unknown variant"));
    assert!(c2d_err.to_string().contains("unknown variant"));
}
