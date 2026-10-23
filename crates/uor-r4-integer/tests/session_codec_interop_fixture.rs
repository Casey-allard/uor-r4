#![cfg(feature = "serving-wasm")]

//! Cross-crate interop control for #2873's recorded follow-up (integer side).
//!
//! Pins the exact `UR4SESS1` bytes the integer codec emits for the contract state
//! against the fixture the object store's durable-consumer test consumes, so a framing change
//! here fails a test instead of drifting from the object store.

use sha2::{Digest, Sha256};
use uor_r4_integer::session_codec::{
    decode, encode, export_scope, import_scope, ScopedSessionIdent, DURABLE_SCOPE_MAGIC,
    DURABLE_SCOPE_VERSION,
};
use uor_r4_integer::stack::{
    SerializedStackLayerState, SerializedStackSession, STACK_SESSION_SCHEMA,
};

const INTEROP_FIXTURE_HEX: &str =
    include_str!("../../uor-r4-object-store/tests/fixtures/integer_ur4sess1_v1.hex");
const INTEROP_FIXTURE_SHA256: &str =
    "ef0ffeeeeb82baa7fa83186a6fc3416cfc660a5d993f2d990f8a5c2893e478b1";

fn fixture() -> Vec<u8> {
    let clean: String = INTEROP_FIXTURE_HEX
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    assert_eq!(clean.len() % 2, 0, "hex fixture has whole bytes");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex fixture"))
        .collect()
}

fn sample() -> SerializedStackSession {
    SerializedStackSession {
        schema: STACK_SESSION_SCHEMA.to_string(),
        version: 1,
        artifact_sha256: "ab".repeat(32),
        position: 7,
        cache_at: 7 * 288,
        lift_at: 7 * 6,
        copy_scale_q16: 65_536,
        logits: vec![-3, 0, 65_536, i32::MIN, i32::MAX],
        tokens: vec![0, 5, 77, 4095],
        snap_trace: Some(vec![0, 119]),
        pointer_keys: vec![1, -2, 3],
        pointer_lifts: vec![u64::MAX, 0, 1 << 62],
        mixture: vec![9, -9],
        layers: vec![
            SerializedStackLayerState::Recurrence {
                state: vec![i64::MIN, -1, 0, 1, i64::MAX],
                history: vec![4, -4, 16],
            },
            SerializedStackLayerState::Read {
                keys: vec![11, 12],
                values: vec![-11, -12],
                lifts: vec![1, 2, 3, 4],
            },
        ],
    }
}

#[test]
fn fixture_matches_the_integer_encoding_and_digest() {
    let bytes = encode(&sample());
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        INTEROP_FIXTURE_SHA256
    );
    assert_eq!(bytes, fixture(), "the codec must emit the contract bytes");
    assert_eq!(decode(&fixture()).expect("fixture decodes"), sample());
}

#[test]
fn durable_envelope_identity_and_snapshot_embedding_are_pinned() {
    assert_eq!(DURABLE_SCOPE_MAGIC, b"UORMEM01");
    assert_eq!(DURABLE_SCOPE_VERSION, 1);
    let ident = ScopedSessionIdent::new("alice", "uor-r4", "s1").expect("valid identity scope");
    let state = sample();
    let envelope = export_scope(&ident, &state).expect("integer envelope");
    assert_eq!(&envelope[..8], DURABLE_SCOPE_MAGIC);
    assert_eq!(
        u32::from_le_bytes(envelope[8..12].try_into().expect("version field")),
        DURABLE_SCOPE_VERSION
    );
    let snapshot = encode(&state);
    assert!(
        envelope
            .windows(snapshot.len())
            .any(|window| window == snapshot.as_slice()),
        "the envelope must embed the exact codec snapshot bytes"
    );
    let (got_ident, got_state) = import_scope(&envelope).expect("envelope round trip");
    assert_eq!(got_ident, ident);
    assert_eq!(got_state, state);
}
