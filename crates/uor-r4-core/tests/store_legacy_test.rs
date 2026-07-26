use std::collections::BTreeMap;
use std::fs;
use uor_r4_core::transformerless::compiler::STAGES;
use uor_r4_core::transformerless::runtime::{
    parse_store_strict_u32, purge_legacy_store_cache, store_bytes, Store, StoreParseError,
};

#[allow(deprecated)]
fn build_legacy_u16_store_bytes() -> Vec<u8> {
    // Magic header
    let mut bytes = Vec::from(b"TLS1".as_slice());
    // Stage 0: 1 key
    let mut level_0 = Vec::new();
    // n_keys = 1
    level_0.extend_from_slice(&1u32.to_le_bytes());
    // klen = 0
    level_0.push(0u8);
    // n_entries = 1
    level_0.extend_from_slice(&1u32.to_le_bytes());
    // entry: u16 token = 42, u32 cnt = 100
    level_0.extend_from_slice(&42u16.to_le_bytes());
    level_0.extend_from_slice(&100u32.to_le_bytes());

    bytes.extend_from_slice(&level_0);
    for _ in 1..=STAGES {
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    bytes
}

#[test]
fn test_parse_store_strict_u32_rejects_legacy_u16_binary() {
    let legacy_bytes = build_legacy_u16_store_bytes();
    assert_eq!(
        parse_store_strict_u32(&legacy_bytes),
        Err(StoreParseError::LegacyStoreFormatDeprecated)
    );
}

#[test]
fn test_parse_store_strict_u32_accepts_valid_u32_store() {
    let mut store: Store = Vec::new();
    for _ in 0..=STAGES {
        store.push(BTreeMap::new());
    }
    store[0].insert(vec![], BTreeMap::from([(42u32, 100u32)]));
    let bytes = store_bytes(&store);

    let parsed = parse_store_strict_u32(&bytes).expect("valid u32 store parses cleanly");
    assert_eq!(parsed[0].get(&vec![]).unwrap().get(&42), Some(&100));
}

#[test]
fn test_purge_legacy_store_cache() {
    let tmp_dir = std::env::temp_dir().join(format!("r4_legacy_purge_test_{}", std::process::id()));
    let _ = fs::remove_dir_all(&tmp_dir);
    fs::create_dir_all(&tmp_dir).expect("create test dir");

    let legacy_file = tmp_dir.join("cache_store.u16");
    let valid_file = tmp_dir.join("cache_store.u32");

    fs::write(&legacy_file, b"legacy u16 cache content").expect("write legacy cache file");
    fs::write(&valid_file, b"valid u32 cache content").expect("write valid cache file");

    assert!(legacy_file.exists());
    assert!(valid_file.exists());

    let purged = purge_legacy_store_cache(&tmp_dir).expect("purge legacy store cache");
    assert_eq!(purged, 1);
    assert!(!legacy_file.exists());
    assert!(valid_file.exists());

    let _ = fs::remove_dir_all(&tmp_dir);
}
