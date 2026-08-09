//! Normative reproducibility byte-equality harness (#167).
//!
//! Verifies that sequential and parallel execution produce identical output bytes
//! for the configured thread-count sweep.

#[cfg(not(target_arch = "wasm32"))]
use crate::executor::RayonExecutor;
use crate::executor::{CompilerExecutor, SequentialExecutor};

/// Verbatim normative invariant statement required by Issue #167 AC.
pub const NORMATIVE_REPRODUCIBILITY_INVARIANT: &str = "Parallel execution may change compilation time, but must not change the canonical graph artifact produced from the same pinned inputs, compiler version, configuration, and target-independent compilation mode.";

/// Verification report produced by `ParallelReproducibilityHarness`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReproducibilityReport {
    /// True if sequential and all parallel thread counts produced bit-identical bytes.
    pub is_byte_identical: bool,
    /// Thread count sweep tested (e.g. [1, 2, 4]).
    pub thread_counts_tested: Vec<usize>,
    /// BLAKE3 / hex digest of sequential output bytes.
    pub sequential_hash: String,
    /// Hashes produced at each tested thread count.
    pub parallel_hashes: Vec<(usize, String)>,
}

/// Harness for verifying canonical artifact byte equality under compiler parallelism.
pub struct ParallelReproducibilityHarness;

impl ParallelReproducibilityHarness {
    /// Compute simple deterministic digest for byte comparison.
    fn compute_digest(bytes: &[u8]) -> String {
        blake3::hash(bytes).to_hex().to_string()
    }

    /// Execute reproducibility verification across a thread count sweep. Total:
    /// always produces a [`ReproducibilityReport`]; `is_byte_identical` is
    /// `false` when any tested thread count's output diverges from the
    /// sequential reference (R5 — a measured byte mismatch is a report, not a
    /// raised error). The mapping closure is infallible; a worker panic
    /// propagates.
    pub fn verify_reproducibility<I, F>(inputs: &[I], map_fn: F) -> ReproducibilityReport
    where
        I: Sync + Send,
        F: Fn(&I) -> Vec<u8> + Sync + Send,
    {
        // 1. Sequential reference run
        let seq_exec = SequentialExecutor::new();
        let seq_bytes: Vec<u8> = seq_exec
            .map(inputs, &map_fn)
            .into_iter()
            .flatten()
            .collect();
        let seq_hash = Self::compute_digest(&seq_bytes);

        #[cfg(target_arch = "wasm32")]
        let thread_counts = vec![1];

        #[cfg(not(target_arch = "wasm32"))]
        let thread_counts = vec![1, 2, 4];

        let mut parallel_hashes = Vec::new();
        let mut is_byte_identical = true;

        for &threads in &thread_counts {
            let par_bytes: Vec<u8> = if threads == 1 {
                SequentialExecutor::new()
                    .map(inputs, &map_fn)
                    .into_iter()
                    .flatten()
                    .collect()
            } else {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    RayonExecutor::new(threads)
                        .map(inputs, &map_fn)
                        .into_iter()
                        .flatten()
                        .collect()
                }
                #[cfg(target_arch = "wasm32")]
                {
                    seq_bytes.clone()
                }
            };

            let par_hash = Self::compute_digest(&par_bytes);
            parallel_hashes.push((threads, par_hash.clone()));
            if par_hash != seq_hash {
                is_byte_identical = false;
                break;
            }
        }

        ReproducibilityReport {
            is_byte_identical,
            thread_counts_tested: thread_counts,
            sequential_hash: seq_hash,
            parallel_hashes,
        }
    }
}

/// κ-label of a container read from disk (`blake3:<hex>` over the raw
/// bytes) — the single spelling of the teacher/artifact container address
/// used by every CLI entry point that loads one (#450).
pub fn container_kappa(bytes: &[u8]) -> String {
    format!("blake3:{}", blake3::hash(bytes).to_hex())
}

/// κ-label of a corpus stream: meta bytes then record bytes (#450).
pub fn corpus_stream_kappa(meta_bytes: &[u8], recs_bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(meta_bytes);
    hasher.update(recs_bytes);
    format!("blake3:{}", hasher.finalize().to_hex())
}

/// Announce the resolved teacher/artifact container on stderr, before the
/// long work starts (#450).
///
/// The `--artifacts` default is a shared, mutable path, and several helper
/// scripts stage different teacher containers into it. Without this line a
/// concurrent run can silently read a different teacher and produce a
/// materially different table with nothing in the output to say so. One
/// greppable line, emitted early, makes every run self-diagnosing.
#[cfg(not(target_arch = "wasm32"))]
pub fn announce_teacher_container(path: &std::path::Path, kappa: &str) {
    eprintln!("teacher container: {} (κ {kappa})", path.display());
}

/// Announce the resolved corpus streams on stderr alongside the teacher
/// container (#450). Same motivation: pin what was actually read.
#[cfg(not(target_arch = "wasm32"))]
pub fn announce_corpus(meta: &std::path::Path, recs: &std::path::Path, kappa: &str) {
    eprintln!(
        "corpus streams: {} + {} (κ {kappa})",
        meta.display(),
        recs.display()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_kappa_matches_the_inline_spelling() {
        let bytes = b"teacher-container-bytes";
        assert_eq!(
            container_kappa(bytes),
            format!("blake3:{}", blake3::hash(bytes).to_hex())
        );
    }

    #[test]
    fn corpus_stream_kappa_hashes_meta_then_recs() {
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"meta");
        hasher.update(b"recs");
        assert_eq!(
            corpus_stream_kappa(b"meta", b"recs"),
            format!("blake3:{}", hasher.finalize().to_hex())
        );
    }

    #[test]
    fn test_normative_invariant_verbatim_statement() {
        assert_eq!(
            NORMATIVE_REPRODUCIBILITY_INVARIANT,
            "Parallel execution may change compilation time, but must not change the canonical graph artifact produced from the same pinned inputs, compiler version, configuration, and target-independent compilation mode."
        );
    }

    #[test]
    fn test_reproducibility_harness_positive_sweep() {
        let inputs = vec![10u32, 20u32, 30u32, 40u32];
        let report = ParallelReproducibilityHarness::verify_reproducibility(&inputs, |&x| {
            x.to_le_bytes().to_vec()
        });

        assert!(report.is_byte_identical);
        assert!(!report.thread_counts_tested.is_empty());
        assert_eq!(report.parallel_hashes[0].1, report.sequential_hash);
    }

    #[test]
    fn test_harness_catches_nondeterministic_reduction() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

        let inputs = vec![1u32, 2u32, 3u32];
        // Intentionally non-deterministic reduction function that depends on call order
        let result = ParallelReproducibilityHarness::verify_reproducibility(&inputs, |_| {
            let c = CALL_COUNT.fetch_add(1, Ordering::SeqCst);
            vec![c as u8]
        });

        // The harness must detect nondeterminism by reporting a byte mismatch
        // when outputs differ across runs.
        assert!(
            !result.is_byte_identical,
            "Harness failed to detect non-deterministic reduction"
        );
    }
}
