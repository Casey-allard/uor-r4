//! Deterministic routing-program synthesis (issue #172).
//!
//! Candidate evaluation is parallelizable, but winner selection is fully
//! deterministic:
//! - candidates are generated with canonical keys and globally sorted;
//! - tie-breaking uses a documented total order over
//!   `(quality desc, cost asc, canonical key asc)`;
//! - reduction compares candidate scores directly and never depends on
//!   completion order.
//!
//! This module is explicitly CPU-only:
//! no CUDA/ROCm/Metal/OpenCL/Vulkan/WebGPU/DirectML/SYCL/tensor-runtime path.

use crate::executor::{CompilerExecutor, SequentialExecutor};
use crate::induction::{Cover, Observation};
use crate::memory_budget::CompilerMemoryBudget;
use std::cmp::Ordering;
use uor_r4_core::transformerless::compiler::SIG_WORDS;
use uor_r4_graph_format::{OP_HALT, OP_TEST_POPCOUNT_LE};

#[cfg(not(target_arch = "wasm32"))]
use crate::executor::RayonExecutor;

/// Explicit non-goal statement carried in code for auditability.
pub const CPU_ONLY_SEARCH_STATEMENT: &str = "CPU-native candidate search only: no GPU kernels, \
no shader compilation, no tensor-runtime dependency, and no accelerator-specific execution path.";

/// Canonical routing-candidate key used for stable ordering.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateKey {
    tap_set: Vec<u8>,
    polynomial_id: u16,
    mask_word: u8,
    mask_bits: u64,
    threshold: u16,
    radius: u16,
    decision_dag_id: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProgramShape {
    HaltOnly,
    GuardThenHalt {
        mask_word: u8,
        mask_bits: u64,
        threshold: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RoutingCandidate {
    key: CandidateKey,
    shape: ProgramShape,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateScore {
    quality_gain: i64,
    collision_penalty: u32,
    runtime_cost: u32,
    artifact_size_cost: u32,
    cache_cost: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandidateEvaluation {
    key: CandidateKey,
    score: CandidateScore,
    bytecode: Vec<u8>,
}

/// Small benchmark row for candidates-per-second sweeps.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateScalingRow {
    pub threads: usize,
    pub candidates_evaluated: usize,
    pub candidates_per_second: u64,
}

/// Fixed-size collision bitmap used to avoid per-candidate heap allocation in
/// the hot candidate-evaluation loop.
const COLLISION_BITMAP_WORDS: usize = 256; // 16,384 hashed slots

fn signature_word(sig: &[u8], word: u8) -> u64 {
    let mut bytes = [0u8; 8];
    let start = word as usize * 8;
    if start >= sig.len() {
        return 0;
    }
    let end = (start + 8).min(sig.len());
    let width = end - start;
    bytes[..width].copy_from_slice(&sig[start..end]);
    u64::from_le_bytes(bytes)
}

fn encode_shape(shape: &ProgramShape) -> Vec<u8> {
    match shape {
        ProgramShape::HaltOnly => vec![OP_HALT],
        ProgramShape::GuardThenHalt {
            mask_word,
            mask_bits,
            threshold,
        } => {
            let mut out = Vec::with_capacity(13);
            out.push(OP_TEST_POPCOUNT_LE);
            out.push(*mask_word);
            out.extend_from_slice(&mask_bits.to_le_bytes());
            out.extend_from_slice(&threshold.to_le_bytes());
            out.push(OP_HALT);
            out
        }
    }
}

fn compare_evaluations(a: &CandidateEvaluation, b: &CandidateEvaluation) -> Ordering {
    // Total order:
    // 1) quality_gain descending
    // 2) collision_penalty ascending
    // 3) runtime/artifact/cache costs ascending
    // 4) canonical key ascending
    b.score
        .quality_gain
        .cmp(&a.score.quality_gain)
        .then_with(|| a.score.collision_penalty.cmp(&b.score.collision_penalty))
        .then_with(|| a.score.runtime_cost.cmp(&b.score.runtime_cost))
        .then_with(|| a.score.artifact_size_cost.cmp(&b.score.artifact_size_cost))
        .then_with(|| a.score.cache_cost.cmp(&b.score.cache_cost))
        .then_with(|| a.key.cmp(&b.key))
}

fn reduce_best(
    mut items: impl Iterator<Item = CandidateEvaluation>,
) -> Option<CandidateEvaluation> {
    let first = items.next()?;
    Some(items.fold(first, |best, item| {
        if compare_evaluations(&item, &best).is_lt() {
            item
        } else {
            best
        }
    }))
}

fn evaluate_candidate(
    candidate: &RoutingCandidate,
    observations: &[Observation],
) -> CandidateEvaluation {
    let bytecode = encode_shape(&candidate.shape);
    let mut accepted = 0u32;
    let mut collisions = 0u32;
    let mut semantic_hits = 0u32;

    match candidate.shape {
        ProgramShape::HaltOnly => {
            accepted = observations.len() as u32;
            semantic_hits = observations.len() as u32;
        }
        ProgramShape::GuardThenHalt {
            mask_word,
            mask_bits,
            threshold,
        } => {
            let mut seen_prev = [0u64; COLLISION_BITMAP_WORDS];
            for observation in observations {
                let pop =
                    (signature_word(&observation.sig, mask_word) & mask_bits).count_ones() as u16;
                if pop <= threshold {
                    accepted += 1;
                    let slot = (observation.prev as usize) & ((COLLISION_BITMAP_WORDS * 64) - 1);
                    let word = slot / 64;
                    let bit = 1u64 << (slot % 64);
                    if (seen_prev[word] & bit) != 0 {
                        collisions += 1;
                    } else {
                        seen_prev[word] |= bit;
                    }
                    if (observation.next ^ u32::from(candidate.key.polynomial_id)) & 1 == 0 {
                        semantic_hits += 1;
                    }
                }
            }
        }
    }

    let shortlist_hits = accepted.saturating_sub(collisions);
    let quality_gain = i64::from(semantic_hits) + i64::from(shortlist_hits) - i64::from(collisions);
    let runtime_cost = bytecode.len() as u32;
    let artifact_size_cost = bytecode.len() as u32;
    let cache_cost =
        candidate.key.tap_set.len() as u32 * 8 + (candidate.key.mask_bits.count_ones() / 2);

    CandidateEvaluation {
        key: candidate.key.clone(),
        score: CandidateScore {
            quality_gain,
            collision_penalty: collisions,
            runtime_cost,
            artifact_size_cost,
            cache_cost,
        },
        bytecode,
    }
}

fn evaluate_batched<E: CompilerExecutor>(
    executor: &E,
    candidates: &[RoutingCandidate],
    observations: &[Observation],
    batch_size: usize,
) -> Vec<CandidateEvaluation> {
    let mut out = Vec::with_capacity(candidates.len());
    let chunk = batch_size.max(1);
    for batch in candidates.chunks(chunk) {
        let mut evaluated = executor.map(batch, |candidate| {
            evaluate_candidate(candidate, observations)
        });
        out.append(&mut evaluated);
    }
    out
}

fn derive_candidates(cover: &Cover, observations: &[Observation]) -> Vec<RoutingCandidate> {
    let mut candidates = Vec::new();
    candidates.push(RoutingCandidate {
        key: CandidateKey {
            tap_set: vec![],
            polynomial_id: 0,
            mask_word: 0,
            mask_bits: 0,
            threshold: 0,
            radius: 0,
            decision_dag_id: 0,
        },
        shape: ProgramShape::HaltOnly,
    });

    if cover.regions.is_empty() || observations.is_empty() {
        return candidates;
    }

    let max_regions = cover.regions.len().min(16);
    for region in cover.regions.iter().take(max_regions) {
        let mask_word = (region.id as usize % SIG_WORDS) as u8;
        let mut tap_set = vec![
            (region.id & 0xff) as u8,
            (region.depth % 8),
            (region.support as u8),
        ];
        tap_set.sort_unstable();
        tap_set.dedup();

        let mask_bits = signature_word(&region.sig, mask_word);
        let threshold = region.radius.min(64);
        let polynomial_id =
            (((region.depth as u16) << 8) | ((region.id as u16) & 0x00ff)) ^ threshold;
        let decision_dag_id = (region.children.len() as u16) ^ (region.id as u16);

        candidates.push(RoutingCandidate {
            key: CandidateKey {
                tap_set,
                polynomial_id,
                mask_word,
                mask_bits,
                threshold,
                radius: threshold,
                decision_dag_id,
            },
            shape: ProgramShape::GuardThenHalt {
                mask_word,
                mask_bits,
                threshold,
            },
        });
    }

    candidates.sort_by(|a, b| a.key.cmp(&b.key));
    candidates.dedup_by(|a, b| a.key == b.key);
    candidates
}

fn choose_best_for_threads(
    cover: &Cover,
    observations: &[Observation],
    threads: usize,
) -> CandidateEvaluation {
    let candidates = derive_candidates(cover, observations);
    if candidates.is_empty() {
        return CandidateEvaluation {
            key: CandidateKey {
                tap_set: vec![],
                polynomial_id: 0,
                mask_word: 0,
                mask_bits: 0,
                threshold: 0,
                radius: 0,
                decision_dag_id: 0,
            },
            score: CandidateScore {
                quality_gain: 0,
                collision_penalty: 0,
                runtime_cost: 1,
                artifact_size_cost: 1,
                cache_cost: 0,
            },
            bytecode: vec![OP_HALT],
        };
    }

    let worker_threads = threads.max(1);
    let total_budget_bytes = 512usize * 1024 * 1024;
    // A fixed 512 MiB budget over >=1 worker is always above the compiler
    // minimum, so this derivation cannot fail here.
    let budget = CompilerMemoryBudget::derive(total_budget_bytes, worker_threads)
        .expect("routing memory budget: 512 MiB over >=1 worker is above the compiler minimum");
    let batch_size = budget.max_in_flight_tasks.clamp(1, candidates.len());

    let evaluations = if worker_threads == 1 {
        let seq = SequentialExecutor::new();
        evaluate_batched(&seq, &candidates, observations, batch_size)
    } else {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let par = RayonExecutor::new(worker_threads);
            evaluate_batched(&par, &candidates, observations, batch_size)
        }
        #[cfg(target_arch = "wasm32")]
        {
            let seq = SequentialExecutor::new();
            evaluate_batched(&seq, &candidates, observations, batch_size)
        }
    };
    // `candidates` is non-empty here (the early return above handles the empty
    // case), so the reduction always yields a best candidate.
    reduce_best(evaluations.into_iter()).expect("non-empty candidate set yields a best candidate")
}

pub fn synthesize_routing_program(cover: &Cover, observations: &[Observation]) -> Vec<u8> {
    let threads = std::thread::available_parallelism()
        .map(|n| n.get().min(4))
        .unwrap_or(1);
    choose_best_for_threads(cover, observations, threads).bytecode
}

pub fn benchmark_candidate_scaling(
    cover: &Cover,
    observations: &[Observation],
    thread_counts: &[usize],
) -> Vec<CandidateScalingRow> {
    thread_counts
        .iter()
        .copied()
        .filter(|&threads| threads > 0)
        .map(|threads| {
            let start = std::time::Instant::now();
            let evaluated = derive_candidates(cover, observations).len();
            let _ = choose_best_for_threads(cover, observations, threads);
            let elapsed_nanos = start.elapsed().as_nanos().max(1);
            let cps = if evaluated == 0 {
                0
            } else {
                let raw = ((evaluated as u128) * 1_000_000_000u128) / elapsed_nanos;
                raw.max(1).min(u64::MAX as u128) as u64
            };
            CandidateScalingRow {
                threads,
                candidates_evaluated: evaluated,
                candidates_per_second: cps,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::induction::CoverRegion;
    use uor_r4_core::transformerless::compiler::{D, SIG_BYTES};

    fn synthetic_cover() -> Cover {
        let mk_region = |id: u32, depth: u8, radius: u16, sig_seed: u8| {
            let mut sig = [0u8; SIG_BYTES];
            for (i, byte) in sig.iter_mut().enumerate() {
                *byte = sig_seed.wrapping_add(i as u8);
            }
            CoverRegion {
                id,
                depth,
                parent: if depth == 1 { None } else { Some(0) },
                children: vec![],
                prototype: vec![0.0; D],
                sig,
                radius,
                support: 64 + id,
                entropy_bits: 0.0,
                split_gain_bits: 0.0,
            }
        };
        Cover {
            regions: vec![
                mk_region(0, 1, 12, 1),
                mk_region(1, 1, 18, 9),
                mk_region(2, 2, 24, 17),
                mk_region(3, 2, 31, 29),
            ],
            max_depth: 2,
            paths: vec![vec![0, 2], vec![1, 3]],
            members: vec![vec![0, 1], vec![2, 3], vec![0, 2], vec![1, 3]],
        }
    }

    fn synthetic_observations() -> Vec<Observation> {
        (0..64)
            .map(|i| {
                let mut sig = [0u8; SIG_BYTES];
                for (j, byte) in sig.iter_mut().enumerate() {
                    *byte = (i as u8).wrapping_mul(3).wrapping_add(j as u8);
                }
                Observation {
                    position: i,
                    sample: [i as u8; 32],
                    vector: vec![0.0; D],
                    sig,
                    prev: i % 7,
                    next: (i * 3) % 11,
                }
            })
            .collect()
    }

    #[test]
    fn canonical_candidate_keys_are_sorted() {
        let cover = synthetic_cover();
        let observations = synthetic_observations();
        let candidates = derive_candidates(&cover, &observations);
        assert!(!candidates.is_empty());
        for pair in candidates.windows(2) {
            assert!(pair[0].key <= pair[1].key);
        }
    }

    #[test]
    fn tie_break_prefers_canonical_key() {
        let a = CandidateEvaluation {
            key: CandidateKey {
                tap_set: vec![1],
                polynomial_id: 1,
                mask_word: 0,
                mask_bits: 1,
                threshold: 1,
                radius: 1,
                decision_dag_id: 1,
            },
            score: CandidateScore {
                quality_gain: 10,
                collision_penalty: 1,
                runtime_cost: 2,
                artifact_size_cost: 2,
                cache_cost: 2,
            },
            bytecode: vec![OP_HALT],
        };
        let mut b = a.clone();
        b.key.polynomial_id = 2;
        assert!(compare_evaluations(&a, &b).is_lt());
    }

    #[test]
    fn reduction_is_permutation_invariant() {
        let cover = synthetic_cover();
        let observations = synthetic_observations();
        let candidates = derive_candidates(&cover, &observations);
        let baseline: Vec<_> = candidates
            .iter()
            .map(|candidate| evaluate_candidate(candidate, &observations))
            .collect();
        let best = reduce_best(baseline.clone().into_iter()).expect("best");

        for shift in 0..baseline.len() {
            let mut permuted = baseline.clone();
            permuted.rotate_left(shift);
            let candidate = reduce_best(permuted.into_iter()).expect("best");
            assert_eq!(candidate.key, best.key);
        }
    }

    #[test]
    fn winner_is_identical_across_thread_counts() {
        let cover = synthetic_cover();
        let observations = synthetic_observations();
        let one = choose_best_for_threads(&cover, &observations, 1);
        let two = choose_best_for_threads(&cover, &observations, 2);
        let four = choose_best_for_threads(&cover, &observations, 4);
        assert_eq!(one.key, two.key);
        assert_eq!(two.key, four.key);
        assert_eq!(one.bytecode, four.bytecode);
    }

    #[test]
    fn benchmark_rows_report_positive_throughput() {
        let cover = synthetic_cover();
        let observations = synthetic_observations();
        let rows = benchmark_candidate_scaling(&cover, &observations, &[1, 2, 4]);
        assert_eq!(rows.len(), 3);
        for row in rows {
            assert!(row.candidates_evaluated > 0);
            assert!(row.candidates_per_second > 0);
        }
    }

    #[test]
    fn cpu_only_statement_mentions_no_gpu() {
        assert!(CPU_ONLY_SEARCH_STATEMENT.contains("no GPU"));
        assert!(CPU_ONLY_SEARCH_STATEMENT.contains("tensor-runtime"));
    }
}
