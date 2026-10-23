//! Cross-crate interop control for #2873's recorded follow-up.
//!
//! The durable-consumer tests build `UR4SESS1` bytes by hand, so a framing change in
//! the integer crate would not fail the object store (the post-merge observation on #2873).
//! This target pins a committed contract fixture produced by the integer codec and proves the
//! store accepts it, round-trips it byte-identically, and returns it exactly; the integer crate
//! has the matching test against the same fixture digest, so drift on either side fails.

use sha2::{Digest, Sha256};
use uor_r4_object_store::durable_consumer::{
    DurableObjectMemoryStore, IdentityScope, Ur4Sess1HeaderView, DURABLE_CONSUMER_MAGIC,
    DURABLE_CONSUMER_VERSION,
};
use uor_r4_object_store::{Limits, Store};

/// Canonical `UR4SESS1` bytes produced by `uor_r4_integer::session_codec::encode`
/// for the fixture state (position 7, tokens [0, 5, 77, 4095], logits [-3, 0, 65536, i32::MIN, i32::MAX]).
const INTEROP_FIXTURE_HEX: &str = include_str!("fixtures/integer_ur4sess1_v1.hex");

/// The contract digest recorded in both crates' tests. A framing change in either crate fails
/// its own side of this pin instead of silently drifting.
const INTEROP_FIXTURE_SHA256: &str =
    "ef0ffeeeeb82baa7fa83186a6fc3416cfc660a5d993f2d990f8a5c2893e478b1";

fn decode_hex(text: &str) -> Vec<u8> {
    let clean: String = text.chars().filter(|c| !c.is_whitespace()).collect();
    assert_eq!(clean.len() % 2, 0, "hex fixture has whole bytes");
    (0..clean.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&clean[i..i + 2], 16).expect("valid hex fixture"))
        .collect()
}

fn fixture() -> Vec<u8> {
    decode_hex(INTEROP_FIXTURE_HEX)
}

fn fixture_digest() -> String {
    format!("{:x}", Sha256::digest(fixture()))
}

#[test]
fn contract_fixture_digest_and_header_are_pinned() {
    assert_eq!(fixture_digest(), INTEROP_FIXTURE_SHA256);
    assert_eq!(DURABLE_CONSUMER_MAGIC, b"UORMEM01");
    assert_eq!(DURABLE_CONSUMER_VERSION, 1);
    let header = Ur4Sess1HeaderView::parse(&fixture()).expect("fixture is a UR4SESS1 header");
    assert_eq!(header.position, 7);
    assert_eq!(header.tokens, vec![0, 5, 77, 4095]);
    assert_eq!(header.logits, vec![-3, 0, 65_536, i32::MIN, i32::MAX]);
}

/// The integer crate's real bytes must survive the store's export/import path byte-for-byte.
#[test]
fn integer_fixture_round_trips_through_the_durable_consumer() {
    let scope = IdentityScope::new("alice", "uor-r4", "s1").expect("valid scope");
    let mut memory = DurableObjectMemoryStore::new();
    memory
        .set_ur4sess1_snapshot(&scope, &fixture())
        .expect("integer fixture admitted as a snapshot");

    let mut store = Store::new(Limits::default());
    let manifest = memory
        .export_scope(&mut store, &scope)
        .expect("export through the store");

    let mut restored = DurableObjectMemoryStore::new();
    let restored_scope = restored
        .import_scope(&store, &manifest)
        .expect("import through the store");
    assert_eq!(restored_scope, scope);

    let restored_bytes = restored
        .ur4sess1_snapshot(&scope)
        .expect("restored snapshot present");
    assert_eq!(
        restored_bytes,
        fixture().as_slice(),
        "the store must return the exact integer-codec bytes"
    );
    assert_eq!(fixture_digest(), INTEROP_FIXTURE_SHA256);
    Ur4Sess1HeaderView::parse(restored_bytes).expect("restored bytes still parse");
}
