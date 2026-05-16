//! Deserializer fuzz target for `DeviceToCloud`.
//!
//! Feeds arbitrary bytes into `serde_json::from_slice`. The decoder
//! must never panic, never allocate unboundedly, never loop. Either
//! parse a valid frame or return Err.
//!
//! Run locally with:
//!
//!   cargo +nightly fuzz run device_to_cloud --release
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = serde_json::from_slice::<bereme_protocol::DeviceToCloud>(data);
});
