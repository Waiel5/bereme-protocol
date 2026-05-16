//! Deserializer fuzz target for `CloudToDevice`.
//!
//! Mirror of `device_to_cloud.rs`. The cloud-to-device path is the one
//! the *device* parses on its WebSocket; a panic here would be a
//! denial-of-service against the device.
//!
//! Run locally with:
//!
//!   cargo +nightly fuzz run cloud_to_device --release
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<bereme_protocol::CloudToDevice>(data);
});
