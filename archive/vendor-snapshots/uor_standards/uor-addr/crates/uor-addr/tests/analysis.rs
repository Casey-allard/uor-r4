//! Analysis suite — large-sample empirical scaling tests from
//! [ANALYSIS.md](../../../ANALYSIS.md). Implements the CP class from
//! [CONFORMANCE.md](../../../CONFORMANCE.md).
//!
//! All randomness uses a deterministic xorshift PRNG seeded from a
//! const literal so failures are reproducible. Run via
//! `just analysis` (release mode); under `cargo test` (debug) these
//! tests still pass but at a smaller sample size — see the per-test
//! `N_DEBUG_*` constants.

#![allow(clippy::needless_range_loop)]
#![allow(non_snake_case)]

use prism::vocabulary::Hasher;
use uor_addr::json::{address, canonicalize};
use uor_addr::Sha256Hasher;

/// Deterministic PRNG seed — `UOR_ADDR_ANALYSIS_SEED`. Anchored so
/// CI failures are reproducible from the commit hash alone.
const SEED: u64 = 0x554F525F41444452; // "UOR_ADDR" in ASCII bytes

/// Release-mode sample-size multiplier. Debug builds use the reduced
/// `_DEBUG` consts to keep wall-clock under 5 s.
#[cfg(debug_assertions)]
const SCALE: u64 = 1;
#[cfg(not(debug_assertions))]
const SCALE: u64 = 100;

// ───────────────────────────────────────────────────────────────────────────
// xorshift PRNG (no external deps)
// ───────────────────────────────────────────────────────────────────────────

struct XorShift64(u64);

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let s = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Self(s)
    }
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }
    fn next_in_range(&mut self, n: u32) -> u32 {
        ((self.next_u32() as u64 * n as u64) >> 32) as u32
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Random JSON generator — produces canonical-form inputs directly so we
// bypass JCS variability and measure the κ-derivation alone.
// ───────────────────────────────────────────────────────────────────────────

/// Generate a canonical-form JSON number leaf. 8 hex chars in a string
/// field — enough entropy to drive the digest, deterministic enough to
/// avoid escape-rule edge cases.
fn random_canonical_json(rng: &mut XorShift64) -> Vec<u8> {
    let v = rng.next_u64();
    format!("{{\"x\":\"{v:016x}\"}}").into_bytes()
}

// ───────────────────────────────────────────────────────────────────────────
// CP-U01 — digest byte 0 uniformity (χ² at α=0.001, df=255)
// ───────────────────────────────────────────────────────────────────────────

/// Critical χ² value at α=0.001 for df=255 (from CDF tables).
const CHI_SQ_CRIT_DF255_A001: f64 = 339.7;

#[test]
fn cp_u01__digest_byte_uniformity_chi_squared() {
    // Debug N = 10_000; release N = 1_000_000.
    let n: u64 = 10_000 * SCALE;
    let mut rng = XorShift64::new(SEED);
    let mut bins = [0u64; 256];
    for _ in 0..n {
        let raw = random_canonical_json(&mut rng);
        let outcome = address(&raw).expect("valid JSON");
        // Decode hex digit 0 + 1 into byte 0 of the digest.
        let b0_hi = nibble(outcome.address.as_bytes()[7]);
        let b0_lo = nibble(outcome.address.as_bytes()[8]);
        bins[((b0_hi << 4) | b0_lo) as usize] += 1;
    }
    let expected = n as f64 / 256.0;
    let chi_sq: f64 = bins
        .iter()
        .map(|&c| {
            let d = c as f64 - expected;
            d * d / expected
        })
        .sum();
    assert!(
        chi_sq < CHI_SQ_CRIT_DF255_A001,
        "CP-U01: χ² = {chi_sq:.2} ≥ {CHI_SQ_CRIT_DF255_A001} (df=255, α=0.001, N={n})"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// CP-U02 — hex position uniformity (χ² at α=0.001, df=15)
// ───────────────────────────────────────────────────────────────────────────

/// Critical χ² value at α=0.001 for df=15 (from CDF tables).
const CHI_SQ_CRIT_DF15_A001: f64 = 37.7;

#[test]
fn cp_u02__hex_position_uniformity_chi_squared() {
    // Debug N = 1_000; release N = 100_000.
    let n: u64 = 1_000 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(1));
    // 64 hex positions × 16 cells per position.
    let mut bins = vec![[0u64; 16]; 64];
    for _ in 0..n {
        let raw = random_canonical_json(&mut rng);
        let outcome = address(&raw).expect("valid JSON");
        let suffix = &outcome.address.as_bytes()[7..];
        for (i, &c) in suffix.iter().enumerate() {
            bins[i][nibble(c) as usize] += 1;
        }
    }
    let expected = n as f64 / 16.0;
    // Aggregate position 0 only — see ANALYSIS.md §2 for why a joint
    // test does not multiplicatively tighten α.
    let chi_sq_pos0: f64 = bins[0]
        .iter()
        .map(|&c| {
            let d = c as f64 - expected;
            d * d / expected
        })
        .sum();
    assert!(
        chi_sq_pos0 < CHI_SQ_CRIT_DF15_A001,
        "CP-U02: χ² = {chi_sq_pos0:.2} ≥ {CHI_SQ_CRIT_DF15_A001} (df=15, α=0.001, N={n})"
    );
}

// ───────────────────────────────────────────────────────────────────────────
// CP-C01 — pairwise collision absence at N
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn cp_c01__no_collisions_at_scale() {
    // Debug N = 10_000; release N = 1_000_000.
    let n: u64 = 10_000 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(2));
    let mut seen = std::collections::HashSet::with_capacity(n as usize);
    for _ in 0..n {
        let raw = random_canonical_json(&mut rng);
        let outcome = address(&raw).expect("valid JSON");
        let prefix = &outcome.address.as_bytes()[7..23]; // First 16 hex = 8 digest bytes
                                                         // Track collisions on the full 64-hex digest, not just the
                                                         // prefix — birthday bound on 256 bits is `2^{-217}` at N=10⁶.
        if !seen.insert(outcome.address) {
            panic!("CP-C01: collision detected at N={n} on prefix {prefix:?}");
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CP-A01 — avalanche distance distribution
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn cp_a01__avalanche_distance_distribution() {
    // Debug N = 100; release N = 10_000.
    let n: u64 = 100 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(3));
    let mut sub_100_count: u64 = 0;
    for _ in 0..n {
        // Generate baseline.
        let baseline = random_canonical_json(&mut rng);
        let base_addr = address(&baseline).expect("valid").address;
        let base_digest = hex_decode(&base_addr[7..]);

        // Mutate one canonical-form byte at a position we know is safe
        // to perturb (inside the hex value, position [6..22) of the raw).
        let mut variant = baseline.clone();
        let target_pos = 6 + (rng.next_in_range(16) as usize); // inside the hex value
        let original = variant[target_pos];
        // Flip to a different hex digit.
        variant[target_pos] = match original {
            b'0'..=b'8' => original + 1,
            b'9' => b'a',
            b'a'..=b'e' => original + 1,
            b'f' => b'0',
            _ => original.wrapping_add(1),
        };
        let var_addr = address(&variant).expect("valid").address;
        let var_digest = hex_decode(&var_addr[7..]);
        let hd = hamming_distance(&base_digest, &var_digest);
        if hd < 100 {
            sub_100_count += 1;
        }
    }
    let fraction = sub_100_count as f64 / n as f64;
    // ANALYSIS.md §4: expect ≈ 2.3·10⁻⁴ under H₀; require ≤ 1%.
    assert!(
        fraction <= 0.01,
        "CP-A01: {sub_100_count}/{n} ({:.4}%) had Hamming distance < 100; threshold 1%",
        fraction * 100.0
    );
}

// ───────────────────────────────────────────────────────────────────────────
// CP-N01 — NFC idempotence at scale (exact)
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn cp_n01__nfc_idempotent_at_scale() {
    // Debug N = 1_000; release N = 100_000.
    let n: u64 = 1_000 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(4));
    for _ in 0..n {
        // Generate a Unicode string containing combining-character
        // sequences and ensure `canonicalize(canonicalize(x)) = canonicalize(x)`.
        let raw = random_unicode_json_string(&mut rng);
        let once = match canonicalize(&raw) {
            Ok(c) => c,
            // Skip inputs that aren't valid UTF-8 JSON; we're testing
            // idempotence on the canonicalisable subset.
            Err(_) => continue,
        };
        let twice = canonicalize(&once).expect("first round already canonical");
        assert_eq!(
            once, twice,
            "CP-N01: canonicalize not idempotent on input {raw:?}"
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CP-K01 — JCS+NFC fixed-point on canonical-form inputs
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn cp_k01__canonicalize_idempotent_at_scale() {
    // Debug N = 1_000; release N = 100_000.
    let n: u64 = 1_000 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(5));
    for _ in 0..n {
        let raw = random_canonical_json(&mut rng);
        let once = canonicalize(&raw).expect("valid");
        let twice = canonicalize(&once).expect("valid second round");
        assert_eq!(
            once,
            twice,
            "CP-K01: canonicalize not idempotent on canonical input {:?}",
            std::str::from_utf8(&raw).unwrap_or("(non-utf8)")
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// CP-K02 — deep key-permutation invariance
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn cp_k02__deep_key_permutation_invariance() {
    // Debug N = 100; release N = 10_000.
    let n: u64 = 100 * SCALE;
    let mut rng = XorShift64::new(SEED.wrapping_add(6));
    for _ in 0..n {
        let snapshot = rng.0;
        let a = random_nested_object(&mut rng, 3, false);
        // Re-seed to the same state so `b` is the SAME structure with
        // keys re-ordered at every depth.
        rng.0 = snapshot;
        let b = random_nested_object(&mut rng, 3, true);
        let addr_a = address(&a).expect("valid a").address;
        let addr_b = address(&b).expect("valid b").address;
        assert_eq!(
            addr_a,
            addr_b,
            "CP-K02: key permutation broke invariance:\n  a = {}\n  b = {}",
            std::str::from_utf8(&a).unwrap(),
            std::str::from_utf8(&b).unwrap()
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// κ-derivation cross-check — runs the same `prism::crypto::Sha256Hasher`
// the ψ_9 resolver uses through its `Hasher` interface, then asserts the
// output matches the hex-decoded suffix of the κ-label produced by
// `address()`. This grounds the κ-derivation identity end-to-end.
// ───────────────────────────────────────────────────────────────────────────

#[test]
fn sha256_one_shot_matches_pipeline() {
    let raw = br#"{"foo":"bar"}"#;
    let canonical = canonicalize(raw).expect("valid");
    let one_shot: [u8; 32] = Sha256Hasher::initial().fold_bytes(&canonical).finalize();
    let outcome = address(raw).expect("valid");
    let from_label = hex_decode(&outcome.address[7..]);
    assert_eq!(one_shot.as_slice(), from_label.as_slice());
}

// ───────────────────────────────────────────────────────────────────────────
// Helpers
// ───────────────────────────────────────────────────────────────────────────

fn random_unicode_json_string(rng: &mut XorShift64) -> Vec<u8> {
    let mut s = String::new();
    let len = 1 + (rng.next_in_range(8) as usize);
    for _ in 0..len {
        // BMP-only random codepoints from a safe range
        // (avoiding surrogates and JSON-control chars).
        let cp = match rng.next_in_range(4) {
            0 => 0x0061 + rng.next_in_range(26),    // 'a'..'z'
            1 => 0x0041 + rng.next_in_range(26),    // 'A'..'Z'
            2 => 0x00C0 + rng.next_in_range(0x100), // Latin-1 Supplement
            _ => 0x0301 + rng.next_in_range(8),     // combining marks
        };
        if let Some(c) = char::from_u32(cp) {
            s.push(c);
        }
    }
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("{{\"s\":\"{escaped}\"}}").into_bytes()
}

fn random_nested_object(rng: &mut XorShift64, depth: u32, reverse_keys: bool) -> Vec<u8> {
    // Branch = 2 at depth 3: ≤ 2^3 = 8 leaves, each ≤ ~24 bytes ⇒ payload ≤ ~400 bytes
    // (well under JSON_INPUT_MAX_BYTES = 3968).
    let mut entries: Vec<(String, String)> = (0..2)
        .map(|i| {
            let k = format!("k{i}_{:04x}", rng.next_u32() & 0xFFFF);
            let v = if depth == 0 {
                format!("\"{:08x}\"", rng.next_u32())
            } else {
                let nested = random_nested_object(rng, depth - 1, reverse_keys);
                String::from_utf8(nested).unwrap()
            };
            (k, v)
        })
        .collect();
    if reverse_keys {
        entries.reverse();
    }
    let body: String = entries
        .iter()
        .map(|(k, v)| format!("\"{k}\":{v}"))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{body}}}").into_bytes()
}

fn hex_decode(s: &str) -> Vec<u8> {
    let s = s.as_bytes();
    (0..s.len() / 2)
        .map(|i| (nibble(s[2 * i]) << 4) | nibble(s[2 * i + 1]))
        .collect()
}

fn nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => 10 + (c - b'a'),
        _ => panic!("non-hex byte: {c}"),
    }
}

fn hamming_distance(a: &[u8], b: &[u8]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())
        .sum()
}
