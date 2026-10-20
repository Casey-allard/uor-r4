//! Product/Coproduct Completion Amendment §2.3i / plan §1d validation:
//! `PartitionHandle` identity semantics and resolver-backed data access.
//!
//! The handle carries only a content fingerprint; partition record data
//! is recovered by pairing it with a `PartitionResolver`. These tests
//! cover:
//!
//! - Identity-level: `PartitionHandle` equality, hashing, and round-trip
//!   of the fingerprint without a resolver (the common content-addressing
//!   case).
//! - Data-access level: a mock `PartitionResolver` keyed by fingerprint
//!   returns the expected `PartitionRecord` via `resolve_with`, and
//!   missing lookups return `None` without breaking the handle.

use std::collections::HashMap;

use uor_foundation::enforcement::MAX_BETTI_DIMENSION;
use uor_foundation::{
    ContentFingerprint, DefaultHostTypes, PartitionHandle, PartitionRecord, PartitionResolver,
};

fn fp(byte: u8) -> ContentFingerprint {
    // 32 = `<DefaultHostBounds as HostBounds>::FINGERPRINT_MAX_BYTES` (the
    // default const-generic), 16 = active width carrying a 128-bit
    // fingerprint per `<DefaultHostBounds as HostBounds>::FINGERPRINT_MIN_BYTES`.
    let mut buf = [0u8; 32];
    buf[0] = byte;
    ContentFingerprint::from_buffer(buf, 16u8)
}

fn sample_record(site_budget: u16, euler: i32) -> PartitionRecord<DefaultHostTypes> {
    PartitionRecord::new(site_budget, euler, [1, 0, 0, 0, 0, 0, 0, 0], 0_u64)
}

// --- Identity-level: no resolver required -----------------------------------

#[test]
fn handle_fingerprint_roundtrips() {
    let handle = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0xAA));
    assert_eq!(handle.fingerprint(), fp(0xAA));
}

#[test]
fn two_handles_with_same_fingerprint_compare_equal() {
    let h1 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0xAA));
    let h2 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0xAA));
    assert_eq!(h1, h2, "handles compare by fingerprint");
    // Copy semantics: the handle is Copy so assignment does not move.
    let h3 = h1;
    assert_eq!(h1.fingerprint(), h3.fingerprint());
}

#[test]
fn distinct_fingerprints_yield_distinct_handles() {
    let h1 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0xAA));
    let h2 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0xBB));
    assert_ne!(h1, h2);
}

#[test]
fn handle_is_hashable_for_content_addressed_indexing() {
    // Content-addressed indices key on fingerprint — handle must be a
    // suitable HashMap key without any resolver.
    let mut index: HashMap<PartitionHandle<DefaultHostTypes>, &'static str> = HashMap::new();
    let h1 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x11));
    let h2 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x22));
    index.insert(h1, "apple");
    index.insert(h2, "banana");
    assert_eq!(index.get(&h1), Some(&"apple"));
    assert_eq!(index.get(&h2), Some(&"banana"));
    // Distinct-byte handle misses.
    let h3 = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x33));
    assert_eq!(index.get(&h3), None);
}

// --- Data-access level: resolver-backed -------------------------------------

struct HashMapResolver {
    records: HashMap<ContentFingerprint, PartitionRecord<DefaultHostTypes>>,
}

impl HashMapResolver {
    fn new() -> Self {
        Self {
            records: HashMap::new(),
        }
    }

    fn insert(
        &mut self,
        fingerprint: ContentFingerprint,
        record: PartitionRecord<DefaultHostTypes>,
    ) {
        self.records.insert(fingerprint, record);
    }
}

impl PartitionResolver<DefaultHostTypes> for HashMapResolver {
    fn resolve(&self, fp: ContentFingerprint) -> Option<PartitionRecord<DefaultHostTypes>> {
        self.records.get(&fp).copied()
    }
}

#[test]
fn resolve_with_returns_registered_record() {
    let mut resolver = HashMapResolver::new();
    let handle_fp = fp(0x77);
    resolver.insert(handle_fp, sample_record(5, 2));

    let handle = PartitionHandle::<DefaultHostTypes>::from_fingerprint(handle_fp);
    let record = handle
        .resolve_with(&resolver)
        .expect("registered fingerprint must resolve");
    assert_eq!(record.site_budget, 5);
    assert_eq!(record.euler, 2);
    assert_eq!(record.betti, [1, 0, 0, 0, 0, 0, 0, 0]);
    assert_eq!(record.entropy_nats_bits, 0_u64);
}

#[test]
fn resolve_with_returns_none_for_unknown_fingerprint() {
    let resolver = HashMapResolver::new();
    // Empty resolver — any fingerprint misses.
    let handle = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x99));
    assert_eq!(handle.resolve_with(&resolver), None);
}

#[test]
fn handle_remains_valid_identity_token_after_failed_resolve() {
    let resolver = HashMapResolver::new();
    let handle = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x88));
    // Failed resolve doesn't mutate the handle.
    assert_eq!(handle.resolve_with(&resolver), None);
    assert_eq!(
        handle.fingerprint(),
        fp(0x88),
        "handle survives failed resolution as an identity token"
    );
    // Handle is still equality-comparable.
    let twin = PartitionHandle::<DefaultHostTypes>::from_fingerprint(fp(0x88));
    assert_eq!(handle, twin);
}

#[test]
fn record_betti_array_is_padded_to_max_dimension() {
    let record = sample_record(3, 1);
    assert_eq!(record.betti.len(), MAX_BETTI_DIMENSION);
}
