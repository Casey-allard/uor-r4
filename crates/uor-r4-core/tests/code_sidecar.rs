//! Regression suite for the per-record graded-code sidecar (#469 lever A).
//!
//! The sidecar is a cache of an existing deterministic computation. Two
//! things must therefore be true, and both are asserted here on the fixture
//! corpus:
//!
//! - what comes back is IDENTICAL to what a fresh computation produces, for
//!   every record — never approximately, never for a prefix;
//! - anything that is not exactly the right file for exactly this artifact
//!   and this corpus fails to load, returning `None` so the caller computes
//!   as before. A wrong-code load must be unreachable, not unlikely.
//!
//! Only `sidecar_round_trip_matches_fresh_codes` touches the process
//! environment (`R4_CODES_PATH`); every rejection case exercises the pure
//! parser, so the suite is parallel-safe within this test binary.

use uor_r4_core::transformerless::code_sidecar::{self, parse_sidecar, sidecar_bytes};
use uor_r4_core::transformerless::compiler::{self, Compiled, Corpus, STAGES};
use uor_r4_core::transformerless::runtime;

fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"))
}

/// Records of the fixture corpus used by the round-trip test. The fixture
/// is 500k records and one code pass over it is minutes of work; the
/// property under test (read-back equals fresh computation for EVERY
/// record) is a per-record identity, so a prefix witnesses it at a cost
/// that belongs in the default suite. `code_plain` at position `i` reads
/// only positions at or before `i` within the same story, so a prefix
/// corpus produces exactly the codes the full corpus produces there.
const ROUND_TRIP_RECORDS: usize = 12_000;

fn load_corpus() -> Corpus {
    compiler::load_corpus_from(&fixture("c_meta.bin"), &fixture("c_recs.bin"))
        .expect("corpus fixtures load")
}

/// The first `records` positions of the fixture corpus, as a corpus.
fn prefix_corpus(records: usize) -> Corpus {
    let mut c = load_corpus();
    let n = records.min(c.n);
    c.story.truncate(n);
    c.input.truncate(n);
    c.next.truncate(n);
    c.t_argmax.truncate(n);
    c.top_tokens.truncate(n);
    c.top_weights.truncate(n);
    c.span_start.truncate(n);
    c.span_end.truncate(n);
    c.byte_start.truncate(n);
    c.byte_end.truncate(n);
    c.hidden = None;
    c.stories = u64::from(c.story[n - 1]) + 1;
    c.n = n;
    c
}

fn load() -> (Compiled, Corpus) {
    let artifacts =
        compiler::load_artifacts_from(&fixture("tless_artifacts.bin")).expect("artifact fixture");
    (artifacts, prefix_corpus(ROUND_TRIP_RECORDS))
}

/// Freshly computed codes, straight through the ordinary runtime path.
fn fresh_codes(art: &Compiled, c: &Corpus) -> Vec<[u8; STAGES]> {
    let rot = compiler::derive_rotations();
    (0..c.n)
        .map(|i| runtime::code_plain(art, &rot, c, i))
        .collect()
}

/// THE regression test: a sidecar written from one run and read back in
/// another yields byte-identical codes for every record, and the store built
/// from them is byte-identical too.
#[test]
fn sidecar_round_trip_matches_fresh_codes() {
    let (art, corpus) = load();
    let path = std::env::temp_dir().join(format!("r4-codes-roundtrip-{}.bin", std::process::id()));
    std::env::set_var(code_sidecar::CODES_PATH_ENV, &path);
    let _ = std::fs::remove_file(&path);

    let expected = fresh_codes(&art, &corpus);
    assert_eq!(expected.len(), corpus.n, "fixture corpus is non-empty");

    // Cold: no sidecar exists, so the closure runs and the result is written.
    let mut computed = 0usize;
    let cold = code_sidecar::corpus_codes_cached(&art, &corpus, || {
        computed += 1;
        runtime::codes_with_threads(&art, &corpus, 2)
    });
    assert_eq!(computed, 1, "cold run must compute");
    assert_eq!(cold, expected, "computed codes match code_plain");
    assert!(path.exists(), "cold run writes the sidecar");

    // Warm: the sidecar verifies, so the closure must NOT run, and every
    // record's code must be identical to the fresh computation.
    let warm = code_sidecar::corpus_codes_cached(&art, &corpus, || {
        panic!("warm run must not recompute");
    });
    assert_eq!(warm.len(), expected.len());
    for (i, (got, want)) in warm.iter().zip(expected.iter()).enumerate() {
        assert_eq!(got, want, "record {i} code differs after read-back");
    }

    // The store built from read-back codes is byte-identical to the store
    // built by the uncached path.
    let (reference_store, reference_codes) = runtime::build_store_with_threads(&art, &corpus, 2);
    assert_eq!(reference_codes, expected);
    let (cached_store, cached_codes) = code_sidecar::build_store_cached(&art, &corpus, 2);
    assert_eq!(cached_codes, expected);
    assert_eq!(
        runtime::store_bytes(&cached_store),
        runtime::store_bytes(&reference_store),
        "store bytes must not depend on where the codes came from"
    );

    std::env::remove_var(code_sidecar::CODES_PATH_ENV);
    let _ = std::fs::remove_file(&path);
}

/// The serialized container is a deterministic function of its inputs.
#[test]
fn sidecar_bytes_are_deterministic() {
    let codes = vec![[1u8, 2, 3, 4], [5, 6, 7, 8]];
    let a = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    let b = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    assert_eq!(a, b);
    assert_eq!(&a[0..4], code_sidecar::MAGIC);
    assert_ne!(a, sidecar_bytes("blake3:other", "blake3:corpus", &codes));
    assert_ne!(a, sidecar_bytes("blake3:art", "blake3:other", &codes));
}

fn good_codes() -> Vec<[u8; STAGES]> {
    (0..64u8).map(|i| [i, i, i, i]).collect()
}

#[test]
fn a_mismatched_artifact_kappa_is_rejected() {
    let codes = good_codes();
    let bytes = sidecar_bytes("blake3:art-A", "blake3:corpus", &codes);
    let got = parse_sidecar(&bytes, "blake3:art-B", "blake3:corpus", codes.len());
    assert!(got.is_none(), "{got:?}");
}

#[test]
fn a_mismatched_corpus_kappa_is_rejected() {
    let codes = good_codes();
    let bytes = sidecar_bytes("blake3:art", "blake3:corpus-A", &codes);
    let got = parse_sidecar(&bytes, "blake3:art", "blake3:corpus-B", codes.len());
    assert!(got.is_none(), "{got:?}");
}

#[test]
fn a_truncated_container_is_rejected() {
    let codes = good_codes();
    let bytes = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    for cut in [0usize, 4, 12, 24, 60, bytes.len() - 1] {
        let got = parse_sidecar(&bytes[..cut], "blake3:art", "blake3:corpus", codes.len());
        assert!(got.is_none(), "prefix of {cut} bytes must not load");
    }
    // Over-long is rejected too: the header must account for every byte.
    let mut extended = bytes.clone();
    extended.push(0);
    let got = parse_sidecar(&extended, "blake3:art", "blake3:corpus", codes.len());
    assert!(got.is_none(), "{got:?}");
}

#[test]
fn a_wrong_stage_count_is_rejected() {
    let codes = good_codes();
    let mut bytes = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    let wrong = (STAGES as u32) + 1;
    bytes[8..12].copy_from_slice(&wrong.to_le_bytes());
    let got = parse_sidecar(&bytes, "blake3:art", "blake3:corpus", codes.len());
    assert!(got.is_none(), "{got:?}");
}

#[test]
fn a_wrong_record_count_is_rejected() {
    let codes = good_codes();
    let bytes = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    let got = parse_sidecar(&bytes, "blake3:art", "blake3:corpus", codes.len() + 1);
    assert!(got.is_none(), "{got:?}");
}

#[test]
fn a_wrong_magic_or_version_is_rejected() {
    let codes = good_codes();
    let bytes = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    let mut foreign = bytes.clone();
    foreign[0..4].copy_from_slice(b"TLA7");
    assert!(parse_sidecar(&foreign, "blake3:art", "blake3:corpus", codes.len()).is_none());
    let mut future = bytes;
    future[4..8].copy_from_slice(&(code_sidecar::VERSION + 1).to_le_bytes());
    assert!(parse_sidecar(&future, "blake3:art", "blake3:corpus", codes.len()).is_none());
}

#[test]
fn a_corrupted_code_block_is_rejected() {
    let codes = good_codes();
    let mut bytes = sidecar_bytes("blake3:art", "blake3:corpus", &codes);
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    let got = parse_sidecar(&bytes, "blake3:art", "blake3:corpus", codes.len());
    assert!(got.is_none(), "{got:?}");
}

/// The corpus key changes when the corpus content changes, so a sidecar
/// written for one corpus can never be served for another.
#[test]
fn corpus_content_kappa_separates_corpora() {
    let (art, corpus) = load();
    let base = code_sidecar::corpus_content_kappa(&corpus);
    assert_eq!(base, code_sidecar::corpus_content_kappa(&corpus));

    let mut mutated = prefix_corpus(ROUND_TRIP_RECORDS);
    mutated.input[0] ^= 1;
    assert_ne!(base, code_sidecar::corpus_content_kappa(&mutated));

    // And the artifact key round-trips through the container bytes, so a
    // loaded artifact addresses the same sidecar the writer addressed.
    let bytes = compiler::artifact_bytes(&art);
    let reparsed = compiler::parse_artifacts(&bytes).expect("artifact round-trip");
    assert_eq!(
        compiler::artifact_kappa(&art),
        compiler::artifact_kappa(&reparsed)
    );
}
