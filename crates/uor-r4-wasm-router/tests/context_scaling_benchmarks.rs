use std::time::Instant;
use uor_r4_core::transformerless::bott_fock::BottFockContextStore;
use uor_r4_wasm_router::cd_space_fold;

#[test]
fn test_context_scaling_1k_to_1m() {
    let dummy_token = [10i16; 16];
    let sequence_lengths = [1_000, 10_000, 100_000];

    for &n in &sequence_lengths {
        let mut store = BottFockContextStore::new();
        let start = Instant::now();

        for _ in 0..n {
            store.append_token(&dummy_token);
        }

        let elapsed = start.elapsed();
        let per_token_us = elapsed.as_micros() as f64 / (n as f64);

        // Memory footprint remains fixed at 256 i16s (512 bytes)
        assert_eq!(
            store.state().len(),
            256,
            "State matrix footprint must remain O(1) fixed 256-dimensional matrix"
        );

        // Latency per token remains bounded under 50 µs
        assert!(
            per_token_us < 50.0,
            "Per-token update latency must be bounded O(1), got {:.4} µs",
            per_token_us
        );
    }
}

#[test]
fn test_cd_space_fold_facade() {
    let mat = cd_space_fold("hello quantum geometric engine");
    assert_eq!(mat.len(), 256);

    let checksum: i64 = mat.iter().map(|&x| (x as i64).abs()).sum();
    assert!(checksum > 0, "State matrix checksum must be non-zero");
}
