//! Fast derived-evidence tests for the #932 teacher-parity scheduler contract.
//!
//! Claim boundary: these tests exercise the public `BatchedTeacher` surface and
//! the harness-level ordering/transcript logic with a deterministic tiny
//! teacher. Exact projection arithmetic is owned by the model-source tests
//! named in `derived_evidence_is_anchored_to_model_source_exact_bit_gates`.
//! The fixture-present bounded four-worker/all-available probe remains
//! empirical evidence.

use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread;
use uor_r4_model_source::{BatchedTeacher, TeacherExecutionConfig};

const VOCAB: usize = 6;
const TOP_K: usize = 3;
const STREAMS: usize = 7;
const HORIZON: usize = 5;

#[derive(Clone)]
struct MockWeights {
    // Three exactly tied maxima exercise the canonical token-id tie break.
    logit_lut: [f32; VOCAB],
}

#[derive(Clone)]
struct MockTeacher {
    weights: Arc<MockWeights>,
}

impl MockTeacher {
    fn new() -> Self {
        Self {
            weights: Arc::new(MockWeights {
                logit_lut: [0.0, 1.0, 1.0, -1.0, 0.5, 1.0],
            }),
        }
    }

    fn weight_address(&self) -> usize {
        Arc::as_ptr(&self.weights) as usize
    }
}

struct MockState {
    stream_id: usize,
    next_position: usize,
    logits: Vec<f32>,
    input_history: Vec<u32>,
}

impl MockState {
    fn assign_stream(&mut self, stream_id: usize) {
        assert_eq!(self.stream_id, usize::MAX, "state is assigned exactly once");
        self.stream_id = stream_id;
    }
}

impl BatchedTeacher for MockTeacher {
    type State = MockState;

    fn new_state(&self) -> Self::State {
        MockState {
            stream_id: usize::MAX,
            next_position: 0,
            logits: vec![0.0; VOCAB],
            input_history: Vec::new(),
        }
    }

    fn reset_state(&self, state: &mut Self::State) {
        state.stream_id = usize::MAX;
        state.next_position = 0;
        state.logits.fill(0.0);
        state.input_history.clear();
    }

    fn logits_mut<'a>(&self, state: &'a mut Self::State) -> &'a mut [f32] {
        &mut state.logits
    }

    fn seq_len(&self) -> usize {
        HORIZON
    }

    fn vocab(&self) -> usize {
        VOCAB
    }

    fn forward_batch_into(
        &self,
        states: &mut [Self::State],
        tokens: &[usize],
        positions: &[usize],
    ) {
        assert_eq!(states.len(), tokens.len());
        assert_eq!(states.len(), positions.len());
        for ((state, &token), &position) in states.iter_mut().zip(tokens).zip(positions) {
            assert_eq!(
                position, state.next_position,
                "a stream remains autoregressively ordered"
            );
            assert!(token < VOCAB);
            let rotation = (state.stream_id + token + position) % VOCAB;
            for (token_id, logit) in state.logits.iter_mut().enumerate() {
                *logit = self.weights.logit_lut[(token_id + rotation) % VOCAB];
            }
            state
                .input_history
                .push(u32::try_from(token).expect("tiny token fits u32"));
            state.next_position += 1;
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct TranscriptRow {
    stream_id: usize,
    position: usize,
    input_token: u32,
    logit_bits: Vec<u32>,
    top_k: Vec<u32>,
    greedy_token: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StreamMetric {
    stream_id: usize,
    rows: usize,
    greedy_token_sum: u64,
    tied_top_rows: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct OrderedReduction {
    row_count: usize,
    rolling: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DerivedEvidence {
    rows: Vec<TranscriptRow>,
    transcript: Vec<u8>,
    transcript_cid: String,
    metrics: Vec<StreamMetric>,
    reduction: OrderedReduction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ScheduleStats {
    requested_workers: usize,
    batches: usize,
    maximum_parallel_batches: usize,
    batch_widths: Vec<usize>,
    weight_addresses: Vec<usize>,
    state_addresses: Vec<usize>,
}

struct BatchOutcome {
    batch_index: usize,
    rows: Vec<TranscriptRow>,
    metrics: Vec<StreamMetric>,
    weight_address: usize,
    // Keep every state allocation alive until all batches have completed so
    // pointer identity can witness private storage without allocator reuse.
    states: Vec<MockState>,
}

fn canonical_top_k(logits: &[f32]) -> Vec<u32> {
    let mut tokens: Vec<usize> = (0..logits.len()).collect();
    tokens.sort_by(|&left, &right| {
        logits[right]
            .total_cmp(&logits[left])
            .then_with(|| left.cmp(&right))
    });
    tokens
        .into_iter()
        .take(TOP_K)
        .map(|token| u32::try_from(token).expect("tiny token fits u32"))
        .collect()
}

fn run_batch(
    teacher: Arc<MockTeacher>,
    batch_index: usize,
    stream_ids: Vec<usize>,
) -> BatchOutcome {
    let mut states: Vec<MockState> = stream_ids
        .iter()
        .map(|&stream_id| {
            let mut state = teacher.new_state();
            state.assign_stream(stream_id);
            state
        })
        .collect();
    let mut tokens: Vec<usize> = stream_ids.iter().map(|id| id % VOCAB).collect();
    let mut rows = Vec::with_capacity(stream_ids.len() * HORIZON);
    let mut greedy_sums = vec![0u64; stream_ids.len()];
    let mut tie_counts = vec![0usize; stream_ids.len()];

    for position in 0..HORIZON {
        let positions = vec![position; states.len()];
        teacher.forward_batch_into(&mut states, &tokens, &positions);
        let mut next_tokens = Vec::with_capacity(states.len());
        for (batch_slot, state) in states.iter_mut().enumerate() {
            let stream_id = state.stream_id;
            let logits = teacher.logits_mut(state);
            let top_k = canonical_top_k(logits);
            let greedy_token = top_k[0];
            let top_value = logits[usize::try_from(greedy_token).expect("token fits usize")];
            let tied = top_k
                .iter()
                .filter(|&&token| {
                    logits[usize::try_from(token).expect("token fits usize")].to_bits()
                        == top_value.to_bits()
                })
                .count();
            if tied > 1 {
                tie_counts[batch_slot] += 1;
            }
            greedy_sums[batch_slot] += u64::from(greedy_token);
            rows.push(TranscriptRow {
                stream_id,
                position,
                input_token: u32::try_from(tokens[batch_slot]).expect("tiny token fits u32"),
                logit_bits: logits.iter().map(|value| value.to_bits()).collect(),
                top_k,
                greedy_token,
            });
            next_tokens.push(usize::try_from(greedy_token).expect("token fits usize"));
        }
        tokens = next_tokens;
    }

    let metrics = stream_ids
        .into_iter()
        .enumerate()
        .map(|(slot, stream_id)| StreamMetric {
            stream_id,
            rows: HORIZON,
            greedy_token_sum: greedy_sums[slot],
            tied_top_rows: tie_counts[slot],
        })
        .collect();

    BatchOutcome {
        batch_index,
        rows,
        metrics,
        weight_address: teacher.weight_address(),
        states,
    }
}

fn encode_transcript(rows: &[TranscriptRow]) -> Vec<u8> {
    let mut bytes = b"uor-r4.teacher-parity-derived/1\0".to_vec();
    bytes.extend_from_slice(
        &u64::try_from(rows.len())
            .expect("row count fits u64")
            .to_le_bytes(),
    );
    for row in rows {
        for value in [row.stream_id, row.position] {
            bytes.extend_from_slice(
                &u64::try_from(value)
                    .expect("fixture index fits u64")
                    .to_le_bytes(),
            );
        }
        bytes.extend_from_slice(&row.input_token.to_le_bytes());
        bytes.extend_from_slice(&row.greedy_token.to_le_bytes());
        bytes.extend_from_slice(
            &u64::try_from(row.logit_bits.len())
                .expect("logit count fits u64")
                .to_le_bytes(),
        );
        row.logit_bits
            .iter()
            .for_each(|bits| bytes.extend_from_slice(&bits.to_le_bytes()));
        bytes.extend_from_slice(
            &u64::try_from(row.top_k.len())
                .expect("top-k count fits u64")
                .to_le_bytes(),
        );
        row.top_k
            .iter()
            .for_each(|token| bytes.extend_from_slice(&token.to_le_bytes()));
    }
    bytes
}

fn reduce_in_canonical_order(rows: &[TranscriptRow]) -> OrderedReduction {
    let rolling = rows.iter().fold(0xcbf2_9ce4_8422_2325u64, |value, row| {
        let row_word = u64::try_from(row.stream_id)
            .expect("stream id fits u64")
            .rotate_left(7)
            ^ u64::try_from(row.position)
                .expect("position fits u64")
                .rotate_left(19)
            ^ u64::from(row.greedy_token)
            ^ u64::from(row.logit_bits[0]).rotate_left(31);
        value
            .wrapping_mul(0x0000_0100_0000_01b3)
            .wrapping_add(row_word)
    });
    OrderedReduction {
        row_count: rows.len(),
        rolling,
    }
}

fn derive_evidence(mut outcomes: Vec<BatchOutcome>) -> (DerivedEvidence, ScheduleStats) {
    let mut weight_addresses: Vec<usize> = outcomes
        .iter()
        .map(|outcome| outcome.weight_address)
        .collect();
    let mut state_addresses: Vec<usize> = outcomes
        .iter()
        .flat_map(|outcome| {
            outcome
                .states
                .iter()
                .map(|state| state.logits.as_ptr() as usize)
        })
        .collect();
    weight_addresses.sort_unstable();
    state_addresses.sort_unstable();

    outcomes.sort_by_key(|outcome| outcome.batch_index);
    let mut rows: Vec<TranscriptRow> = outcomes
        .iter_mut()
        .flat_map(|outcome| std::mem::take(&mut outcome.rows))
        .collect();
    rows.sort_by_key(|row| (row.stream_id, row.position));
    let mut metrics: Vec<StreamMetric> = outcomes
        .iter_mut()
        .flat_map(|outcome| std::mem::take(&mut outcome.metrics))
        .collect();
    metrics.sort_by_key(|metric| metric.stream_id);

    let transcript = encode_transcript(&rows);
    let transcript_cid = format!("blake3:{}", blake3::hash(&transcript).to_hex());
    let reduction = reduce_in_canonical_order(&rows);
    let evidence = DerivedEvidence {
        rows,
        transcript,
        transcript_cid,
        metrics,
        reduction,
    };
    let stats = ScheduleStats {
        requested_workers: 0,
        batches: outcomes.len(),
        maximum_parallel_batches: 0,
        batch_widths: Vec::new(),
        weight_addresses,
        state_addresses,
    };
    (evidence, stats)
}

fn run_schedule(
    teacher: Arc<MockTeacher>,
    workers: usize,
    batch_width: usize,
    shuffle_completion: bool,
    stream_count: usize,
) -> (DerivedEvidence, ScheduleStats) {
    assert!(workers > 0);
    assert!(batch_width > 0);
    let batches: Vec<Vec<usize>> = (0..stream_count)
        .collect::<Vec<_>>()
        .chunks(batch_width)
        .map(<[usize]>::to_vec)
        .collect();
    let batch_widths: Vec<usize> = batches.iter().map(Vec::len).collect();
    let maximum_parallel_batches = workers.min(batches.len());
    let mut outcomes = Vec::with_capacity(batches.len());

    for wave_start in (0..batches.len()).step_by(workers) {
        let wave_end = (wave_start + workers).min(batches.len());
        let wave = &batches[wave_start..wave_end];
        let mut completed = thread::scope(|scope| {
            let handles: Vec<_> = wave
                .iter()
                .enumerate()
                .map(|(offset, stream_ids)| {
                    let teacher = Arc::clone(&teacher);
                    let stream_ids = stream_ids.clone();
                    scope.spawn(move || run_batch(teacher, wave_start + offset, stream_ids))
                })
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("derived worker must complete"))
                .collect::<Vec<_>>()
        });
        if shuffle_completion {
            completed.reverse();
        }
        outcomes.extend(completed);
    }
    if shuffle_completion {
        let rotate_by = outcomes.len() / 2;
        outcomes.rotate_left(rotate_by);
    }

    let (evidence, mut stats) = derive_evidence(outcomes);
    stats.requested_workers = workers;
    stats.maximum_parallel_batches = maximum_parallel_batches;
    stats.batch_widths = batch_widths;
    (evidence, stats)
}

#[test]
fn worker_and_batch_schedules_preserve_all_derived_evidence() {
    let teacher = Arc::new(MockTeacher::new());
    let (reference, _) = run_schedule(Arc::clone(&teacher), 1, 1, false, STREAMS);

    // Fixed fixture row checks raw bits and canonical tie order independently
    // of comparisons among scheduler configurations.
    assert_eq!(
        reference.rows[0].logit_bits,
        [
            0.0f32.to_bits(),
            1.0f32.to_bits(),
            1.0f32.to_bits(),
            (-1.0f32).to_bits(),
            0.5f32.to_bits(),
            1.0f32.to_bits(),
        ]
    );
    assert_eq!(reference.rows[0].top_k, [1, 2, 5]);
    assert_eq!(reference.rows[0].greedy_token, 1);
    assert_eq!(
        reference
            .rows
            .iter()
            .filter(|row| row.stream_id == 0)
            .map(|row| row.greedy_token)
            .collect::<Vec<_>>(),
        [1, 0, 0, 2, 1],
        "the fixed stream guards the greedy trajectory"
    );
    assert_eq!(
        reference.transcript_cid,
        format!("blake3:{}", blake3::hash(&reference.transcript).to_hex())
    );

    for workers in [1usize, 2, 4, 8] {
        // These are the supported derived-evidence batch schedules. Production
        // arithmetic is independently gated at the same worker counts below.
        let _production_config = TeacherExecutionConfig::fixed_workers(
            NonZeroUsize::new(workers).expect("supported worker count is nonzero"),
        );
        for batch_width in [1usize, 2, 4, 8] {
            let (actual, stats) =
                run_schedule(Arc::clone(&teacher), workers, batch_width, true, STREAMS);
            assert_eq!(
                actual, reference,
                "derived evidence changed at workers={workers}, batch={batch_width}"
            );
            assert_eq!(stats.requested_workers, workers);
            assert!(stats.maximum_parallel_batches <= workers);
            assert!(stats.batch_widths.iter().all(|&width| width <= batch_width));
            assert_eq!(stats.state_addresses.len(), STREAMS);
            assert_eq!(
                stats
                    .state_addresses
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .len(),
                STREAMS,
                "every sequence owns distinct mutable logit storage"
            );
            assert_eq!(
                stats
                    .weight_addresses
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    .len(),
                1,
                "every worker batch shares one immutable weight allocation"
            );
        }
    }
}

#[test]
fn workers_above_work_and_batch_remainder_are_bounded_and_exact() {
    let teacher = Arc::new(MockTeacher::new());
    let (reference, _) = run_schedule(Arc::clone(&teacher), 1, 1, false, 5);
    let (actual, stats) = run_schedule(teacher, 8, 3, true, 5);

    assert_eq!(actual, reference);
    assert_eq!(stats.requested_workers, 8);
    assert_eq!(stats.batches, 2);
    assert_eq!(stats.batch_widths, [3, 2]);
    assert_eq!(stats.maximum_parallel_batches, 2);
}

#[test]
fn cloned_teachers_share_weights_while_sequence_state_remains_private() {
    let first = MockTeacher::new();
    let second = first.clone();
    assert!(Arc::ptr_eq(&first.weights, &second.weights));

    let mut left = first.new_state();
    let mut right = first.new_state();
    left.assign_stream(0);
    right.assign_stream(1);
    first.forward_batch_into(std::slice::from_mut(&mut left), &[0], &[0]);
    assert_eq!(left.input_history, [0]);
    assert!(right.input_history.is_empty());

    second.forward_batch_into(std::slice::from_mut(&mut right), &[1], &[0]);
    assert_eq!(left.input_history, [0]);
    assert_eq!(right.input_history, [1]);
    assert_ne!(left.logits.as_ptr(), right.logits.as_ptr());
}

#[test]
fn derived_evidence_is_anchored_to_model_source_exact_bit_gates() {
    // This source-level link prevents the cheap mock scheduler test from being
    // mistaken for arithmetic proof. The named model-source tests execute the
    // real pinned exact owner; this file exercises only evidence derivation.
    let model_source = include_str!("../crates/uor-r4-model-source/src/lib.rs");
    for required_gate in [
        "exact_matmul_matches_serial_bits_for_1_2_4_8_workers",
        "exact_batched_matmul_matches_serial_bits_for_1_2_4_8_workers",
        "forward_batch_matches_serial_forward_for_1_2_4_8_workers",
    ] {
        assert!(
            model_source.contains(required_gate),
            "#932 derived evidence requires model-source gate {required_gate}"
        );
    }

    let executor_source = include_str!("../crates/uor-r4-model-source/src/exact_executor.rs");
    assert!(executor_source.contains("partitioning only complete output rows"));
    assert!(executor_source.contains("full `k`-term exact accumulator"));
}
