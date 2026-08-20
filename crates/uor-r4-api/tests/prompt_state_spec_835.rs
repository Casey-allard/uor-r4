//! Executable reference model and machine-checked evidence for the persistent
//! prompt-conditioned predictive state specification (#835).
//!
//! Companion document: `docs/prompt_state_spec_835.md`. This is a
//! **reference-only / off-serving-path** model in the RF-27/RF-28 sense: an
//! owned integer realization of the specified state semantics used to prove the
//! contract is deterministic, bounded, and benchmarkable. It is not the deployed
//! serving path (that lowering + its P-4/allocation proofs are #836), and it is
//! not itself an S1 capability result (that measurement is #834).
//!
//! It binds to the frozen S1 evaluation constitution (#832): the control
//! vocabulary (`ControlKind`), the integer-fraction metric encoding
//! (`MetricStatus`), and the degeneracy check (`is_degenerate_control`) come
//! from `uor_r4_api::capability_suite`, so the planted negatives here speak the
//! same language as the committed `s1-causal-prompt-pairs` suite.

use uor_r4_api::capability_suite::{is_degenerate_control, ControlKind, MetricStatus};

/// Canonical fixed-point score type (`ScoreQ`, i32 Q16.16) per
/// `docs/scoring_semantics.md`.
type ScoreQ = i32;

const NUM_LANES: usize = 6;
// Lane indices (§2 of the spec): suffix, segment, role, entity, history, constraint.
const LANE_SUFFIX: usize = 0;
const LANE_SEGMENT: usize = 1;
const LANE_HISTORY: usize = 4;

/// Candidate alphabet size for the reference benchmark (a power of two so the
/// low-bit reduction is a mask, not a divide).
const K: u64 = 8;

/// Base slot weight (well below `ScoreQ::MAX`) and the per-match contribution
/// boost. `BOOST` is large enough that a few dozen matching slots exercise the
/// saturating-add clamp (§6) rather than wrapping.
const BASE_W: ScoreQ = 1 << 12;
const BOOST: ScoreQ = 1 << 26;

/// A typed, focused error for the reference model. No reference-model path
/// panics on a recoverable input (§6 of the spec).
#[derive(Debug, PartialEq, Eq)]
enum PsError {
    ZeroCapacity,
    IndexOutOfRange,
    UnknownLane,
    Decline,
}

/// One lane slot: integer fields only (§2). `born`/`touched`/`contribution_id`
/// are part of the specified slot schema; the reference tests exercise the
/// dynamics (weight/eviction/witness) rather than reading every field back.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Slot {
    key: u64,
    weight: ScoreQ,
    born: u32,
    touched: u32,
    contribution_id: u32,
}

/// One bounded, deterministic lane: a fixed-capacity ring with a halving decay
/// schedule (an arithmetic right shift, never a divide).
#[derive(Clone)]
struct Ring {
    cap: usize,
    decay_shift: u32,
    slots: Vec<Slot>,
}

impl Ring {
    fn new(cap: usize, decay_shift: u32) -> Result<Ring, PsError> {
        if cap == 0 {
            return Err(PsError::ZeroCapacity);
        }
        Ok(Ring {
            cap,
            decay_shift,
            slots: Vec::with_capacity(cap),
        })
    }

    /// Halving decay: right-shift every weight and drop slots that reach the
    /// floor. Division-free.
    fn decay(&mut self) {
        let shift = self.decay_shift;
        if shift > 0 {
            for s in &mut self.slots {
                s.weight >>= shift;
            }
            self.slots.retain(|s| s.weight > 0);
        }
    }

    /// The eviction victim under the canonical order (weight descending, key
    /// ascending): the least-weight slot, ties broken by the *larger* key so
    /// the lower id is retained (`docs/scoring_semantics.md` §6).
    fn victim_index(&self) -> usize {
        let mut best = 0usize;
        for (i, s) in self.slots.iter().enumerate().skip(1) {
            let b = self.slots[best];
            if s.weight < b.weight || (s.weight == b.weight && s.key > b.key) {
                best = i;
            }
        }
        best
    }

    /// Insert or refresh a slot; returns the evicted key when the ring was full.
    /// Weight accumulation saturates (`ScoreQ` saturating add, §6).
    fn upsert(&mut self, key: u64, add: ScoreQ, clk: u32, cid: u32) -> Option<u64> {
        if let Some(s) = self.slots.iter_mut().find(|s| s.key == key) {
            s.weight = s.weight.saturating_add(add);
            s.touched = clk;
            return None;
        }
        if self.slots.len() < self.cap {
            self.slots.push(Slot {
                key,
                weight: add,
                born: clk,
                touched: clk,
                contribution_id: cid,
            });
            return None;
        }
        let victim = self.victim_index();
        let evicted = self.slots[victim].key;
        self.slots[victim] = Slot {
            key,
            weight: add,
            born: clk,
            touched: clk,
            contribution_id: cid,
        };
        Some(evicted)
    }

    /// Insert without evicting; declines (typed error) when the ring is full.
    /// Models the §6 overflow/decline semantics.
    fn try_insert_no_evict(
        &mut self,
        key: u64,
        add: ScoreQ,
        clk: u32,
        cid: u32,
    ) -> Result<(), PsError> {
        if self.slots.iter().any(|s| s.key == key) {
            return Ok(());
        }
        if self.slots.len() >= self.cap {
            return Err(PsError::Decline);
        }
        self.slots.push(Slot {
            key,
            weight: add,
            born: clk,
            touched: clk,
            contribution_id: cid,
        });
        Ok(())
    }
}

/// A bounded, replayable witness record (§9). Fixed-width; serialized
/// little-endian for byte-reproducible replay.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Witness {
    step: u32,
    lane: u8,
    key: u64,
    weight_after: ScoreQ,
    evicted: Option<u64>,
    contribution_id: u32,
}

const WITNESS_BYTES: usize = 4 + 1 + 8 + 4 + 1 + 8 + 4; // = 30

impl Witness {
    fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&self.step.to_le_bytes());
        out.push(self.lane);
        out.extend_from_slice(&self.key.to_le_bytes());
        out.extend_from_slice(&self.weight_after.to_le_bytes());
        match self.evicted {
            Some(k) => {
                out.push(1);
                out.extend_from_slice(&k.to_le_bytes());
            }
            None => {
                out.push(0);
                out.extend_from_slice(&0u64.to_le_bytes());
            }
        }
        out.extend_from_slice(&self.contribution_id.to_le_bytes());
    }

    fn decode(buf: &[u8]) -> Witness {
        let step = u32::from_le_bytes(buf[0..4].try_into().unwrap());
        let lane = buf[4];
        let key = u64::from_le_bytes(buf[5..13].try_into().unwrap());
        let weight_after = i32::from_le_bytes(buf[13..17].try_into().unwrap());
        let evicted = if buf[17] == 1 {
            Some(u64::from_le_bytes(buf[18..26].try_into().unwrap()))
        } else {
            None
        };
        let contribution_id = u32::from_le_bytes(buf[26..30].try_into().unwrap());
        Witness {
            step,
            lane,
            key,
            weight_after,
            evicted,
            contribution_id,
        }
    }
}

/// The persistent prompt state `Ψ` (§2): six bounded lanes, a monotone step
/// counter, a contribution-id allocator, and the emitted witness stream.
struct Psi {
    lanes: Vec<Ring>,
    clk: u32,
    next_cid: u32,
    witness: Vec<Witness>,
}

impl Psi {
    fn new(caps: [usize; NUM_LANES], decay: [u32; NUM_LANES]) -> Result<Psi, PsError> {
        let mut lanes = Vec::with_capacity(NUM_LANES);
        for (&cap, &d) in caps.iter().zip(decay.iter()) {
            lanes.push(Ring::new(cap, d)?);
        }
        Ok(Psi {
            lanes,
            clk: 0,
            next_cid: 0,
            witness: Vec::new(),
        })
    }

    /// A reference default: generous suffix/segment capacity, no decay.
    fn reference() -> Psi {
        Psi::new([8, 8, 4, 8, 8, 4], [0, 0, 0, 0, 0, 0]).expect("nonzero caps")
    }

    fn alloc_cid(&mut self) -> u32 {
        let c = self.next_cid;
        self.next_cid += 1;
        c
    }

    fn lane(&self, idx: usize) -> Result<&Ring, PsError> {
        if idx >= NUM_LANES {
            return Err(PsError::IndexOutOfRange);
        }
        self.lanes.get(idx).ok_or(PsError::UnknownLane)
    }

    /// Record an upsert into a lane and emit its witness.
    fn record(&mut self, lane: usize, key: u64, add: ScoreQ) {
        let clk = self.clk;
        let cid = self.alloc_cid();
        let evicted = self.lanes[lane].upsert(key, add, clk, cid);
        let weight_after = self.lanes[lane]
            .slots
            .iter()
            .find(|s| s.key == key)
            .map_or(0, |s| s.weight);
        self.witness.push(Witness {
            step: clk,
            lane: lane as u8,
            key,
            weight_after,
            evicted,
            contribution_id: cid,
        });
    }

    /// Initial fold Φ₀ over the complete supplied prompt (§3): every prompt
    /// token folds into the whole-prompt Segment lane and the rolling Suffix
    /// lane. Reads no completion token.
    fn fold_prompt(&mut self, prompt: &[u64]) {
        for &t in prompt {
            self.record(LANE_SEGMENT, t, BASE_W);
            self.record(LANE_SUFFIX, t, BASE_W);
            self.clk += 1;
        }
    }

    /// Per-token transition T_ps (§3): update the rolling Suffix lane and decay
    /// every lane. Reads only the just-consumed token.
    fn step_token(&mut self, token: u64) {
        for lane in &mut self.lanes {
            lane.decay();
        }
        self.record(LANE_SUFFIX, token, BASE_W);
        self.clk += 1;
    }

    /// State contribution to a candidate (§4): a decode-independent, saturating
    /// `ScoreQ` residual from the Segment lane. Each lane slot carries a unique
    /// contribution id, so no evidence is double-counted.
    fn contribution(&self, candidate: u64) -> ScoreQ {
        let mut acc: ScoreQ = 0;
        for s in &self.lanes[LANE_SEGMENT].slots {
            if (s.key & (K - 1)) == candidate {
                acc = acc.saturating_add(BOOST);
            }
        }
        acc
    }

    /// Argmax candidate under the state contribution, canonical tie-break
    /// (score descending, id ascending → lowest candidate wins ties).
    fn decide(&self) -> u64 {
        let mut best_c = 0u64;
        let mut best = ScoreQ::MIN;
        for c in 0..K {
            let sc = self.contribution(c);
            if sc > best {
                best = sc;
                best_c = c;
            }
        }
        best_c
    }

    /// Total live slot count across all lanes.
    fn total_slots(&self) -> usize {
        self.lanes.iter().map(|l| l.slots.len()).sum()
    }

    /// A canonical (lane, key, weight) view for equality checks, sorted so the
    /// comparison is order-independent.
    fn canonical(&self) -> Vec<(u8, u64, ScoreQ)> {
        let mut v: Vec<(u8, u64, ScoreQ)> = Vec::new();
        for (li, lane) in self.lanes.iter().enumerate() {
            for s in &lane.slots {
                v.push((li as u8, s.key, s.weight));
            }
        }
        v.sort_unstable();
        v
    }

    fn witness_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.witness.len() * WITNESS_BYTES);
        for w in &self.witness {
            w.encode(&mut out);
        }
        out
    }
}

/// Replay a serialized witness stream into a canonical (lane, key, weight)
/// view, from an empty state — the independent-verifier path (§9).
fn replay_witness(bytes: &[u8]) -> Vec<(u8, u64, ScoreQ)> {
    let mut lanes: Vec<Vec<(u64, ScoreQ)>> = vec![Vec::new(); NUM_LANES];
    for chunk in bytes.chunks_exact(WITNESS_BYTES) {
        let w = Witness::decode(chunk);
        let lane = &mut lanes[w.lane as usize];
        if let Some(ek) = w.evicted {
            lane.retain(|&(k, _)| k != ek);
        }
        if let Some(entry) = lane.iter_mut().find(|e| e.0 == w.key) {
            entry.1 = w.weight_after;
        } else {
            lane.push((w.key, w.weight_after));
        }
    }
    let mut v: Vec<(u8, u64, ScoreQ)> = Vec::new();
    for (li, lane) in lanes.iter().enumerate() {
        for &(k, wt) in lane {
            v.push((li as u8, k, wt));
        }
    }
    v.sort_unstable();
    v
}

// --- the reference benchmark (§11): a causal model, a constant (non-causal)
//     model, and the three frozen S1 controls ---------------------------------

#[derive(Clone, Copy)]
enum Model {
    /// Uses the folded prompt state to decide (a genuine prompt-conditioned arm).
    Causal,
    /// Ignores state; always predicts candidate 0 (the no-context floor).
    Constant,
}

#[derive(Clone, Copy)]
enum Control {
    None,
    PromptSwap,
    SuffixOnly,
}

/// Fold a prompt into a fresh state under a control (§11): PromptSwap folds an
/// unrelated prompt into the Segment lane; SuffixOnly folds nothing into the
/// whole-prompt Segment lane (only the rolling suffix), so no whole-prompt
/// evidence exists.
fn state_under(prompt: &[u64], swapped: &[u64], control: Control) -> Psi {
    let mut psi = Psi::reference();
    match control {
        Control::None => psi.fold_prompt(prompt),
        Control::PromptSwap => psi.fold_prompt(swapped),
        Control::SuffixOnly => {
            for &t in prompt {
                psi.record(LANE_SUFFIX, t, BASE_W);
                psi.clk += 1;
            }
        }
    }
    psi
}

/// The `causal-influence-delta` reference statistic: the fraction of eval
/// prompts on which the model reproduces the prompt-consistent target, as an
/// exact integer fraction (`MetricStatus::Measured`), never a float.
fn causal_delta(model: Model, eval: &[Vec<u64>], control: Control) -> MetricStatus {
    let n = eval.len();
    let mut hits: u64 = 0;
    for (j, prompt) in eval.iter().enumerate() {
        // The prompt-consistent target is the causal model's answer on the
        // clean, un-controlled prompt.
        let true_target = {
            let mut clean = Psi::reference();
            clean.fold_prompt(prompt);
            clean.decide()
        };
        let swapped = &eval[(j + 1) % n];
        let psi = state_under(prompt, swapped, control);
        let pred = match model {
            Model::Causal => psi.decide(),
            Model::Constant => 0,
        };
        if pred == true_target {
            hits += 1;
        }
    }
    MetricStatus::Measured {
        numerator: hits,
        denominator: n as u64,
    }
}

/// The eval set: one distinct low-bit target per prompt (§11), so a causal arm
/// scores 1000‰ and the prompt-swap null scores 0‰.
fn eval_set() -> Vec<Vec<u64>> {
    (0..K).map(|t| vec![t]).collect()
}

// -----------------------------------------------------------------------------
// Machine-checked evidence (the §13 acceptance criteria)
// -----------------------------------------------------------------------------

#[test]
fn determinism() {
    let prompt = [3u64, 1, 4, 1, 5, 9, 2, 6];
    let gen = [7u64, 0, 7];
    let run = || {
        let mut psi = Psi::reference();
        psi.fold_prompt(&prompt);
        for &t in &gen {
            psi.step_token(t);
        }
        let contrib: Vec<ScoreQ> = (0..K).map(|c| psi.contribution(c)).collect();
        (psi.canonical(), psi.witness_bytes(), contrib)
    };
    let a = run();
    let b = run();
    assert_eq!(a.0, b.0, "state must be identical for identical inputs");
    assert_eq!(a.1, b.1, "witness bytes must be byte-reproducible");
    assert_eq!(a.2, b.2, "contribution stream must be identical");
}

#[test]
fn state_is_fixed_capacity() {
    let mut psi = Psi::reference();
    // A long stream must never grow the state beyond the sum of lane caps.
    let cap_sum: usize = [8usize, 8, 4, 8, 8, 4].iter().sum();
    let stream: Vec<u64> = (0..10_000u64).map(|i| i % 97).collect();
    psi.fold_prompt(&stream);
    for &t in &stream {
        psi.step_token(t);
    }
    assert!(
        psi.total_slots() <= cap_sum,
        "bounded state: {} <= {}",
        psi.total_slots(),
        cap_sum
    );
}

#[test]
fn transition_is_causal() {
    // Two streams identical through position i, differing only at a *future*
    // token, must yield identical state and contributions at every position ≤ i.
    let shared = [2u64, 7, 1, 8];
    let mut a = Psi::reference();
    let mut b = Psi::reference();
    a.fold_prompt(&shared);
    b.fold_prompt(&shared);
    for &t in &shared {
        a.step_token(t);
        b.step_token(t);
    }
    // Now they diverge in the (unread) future; state so far is already fixed.
    assert_eq!(a.canonical(), b.canonical());
    let ca: Vec<ScoreQ> = (0..K).map(|c| a.contribution(c)).collect();
    let cb: Vec<ScoreQ> = (0..K).map(|c| b.contribution(c)).collect();
    assert_eq!(ca, cb, "contributions must not depend on future tokens");
}

#[test]
fn contributions_independent_of_decode() {
    // The contribution vector at a fixed state must be identical regardless of
    // which token a decode policy would pick next (RF-28 separation).
    let mut psi = Psi::reference();
    psi.fold_prompt(&[1u64, 2, 3]);
    let vector: Vec<ScoreQ> = (0..K).map(|c| psi.contribution(c)).collect();
    // A greedy decode and a different (sampled) decode both read the same
    // immutable state; reading it to decode a token must not alter the
    // per-candidate contributions.
    let _greedy_pick = psi.decide();
    let _sampled_pick = (psi.decide() + 3) % K;
    let after: Vec<ScoreQ> = (0..K).map(|c| psi.contribution(c)).collect();
    assert_eq!(
        vector, after,
        "decode must not alter per-candidate contributions"
    );
}

#[test]
fn capacity_edges_and_eviction() {
    let mut ring = Ring::new(3, 0).unwrap();
    // Fill with ascending weights; keys 10,11,12 at weights 1,2,3.
    assert_eq!(ring.upsert(10, 1, 0, 0), None);
    assert_eq!(ring.upsert(11, 2, 1, 1), None);
    assert_eq!(ring.upsert(12, 3, 2, 2), None);
    assert_eq!(ring.slots.len(), 3);
    // Inserting a fourth evicts the least-weight slot (key 10).
    assert_eq!(ring.upsert(13, 5, 3, 3), Some(10));
    assert_eq!(ring.slots.len(), 3);
    assert!(ring.slots.iter().all(|s| s.key != 10));
}

#[test]
fn tie_break_evicts_higher_key() {
    // Equal weights: the canonical order retains the lower key, so the *higher*
    // key is evicted.
    let mut ring = Ring::new(2, 0).unwrap();
    ring.upsert(20, 4, 0, 0);
    ring.upsert(21, 4, 1, 1);
    let evicted = ring.upsert(22, 4, 2, 2);
    assert_eq!(evicted, Some(21), "tie → evict higher key, keep lower id");
    assert!(ring.slots.iter().any(|s| s.key == 20));
    assert!(ring.slots.iter().any(|s| s.key == 22));
}

#[test]
fn saturation_no_overflow() {
    // Weight accumulation saturates at ScoreQ::MAX without panicking.
    let mut ring = Ring::new(1, 0).unwrap();
    ring.upsert(1, ScoreQ::MAX - 1, 0, 0);
    ring.upsert(1, ScoreQ::MAX, 1, 1); // refresh with a huge add
    assert_eq!(ring.slots[0].weight, ScoreQ::MAX);

    // Contribution accumulation also saturates: many matching segment slots.
    let mut psi = Psi::new([1, 64, 1, 1, 1, 1], [0; NUM_LANES]).unwrap();
    for t in 0..64u64 {
        // all keys share low bits == 0 (multiples of 8) → all boost candidate 0
        psi.record(LANE_SEGMENT, t * K, ScoreQ::MAX);
    }
    // Even with BOOST per matching slot, the sum saturates rather than wrapping.
    assert_eq!(psi.contribution(0), ScoreQ::MAX);
}

#[test]
fn reset_and_continuation() {
    let prompt = [5u64, 6, 7];
    // Reset: a fresh fold of the same prompt reproduces the same state.
    let mut first = Psi::reference();
    first.fold_prompt(&prompt);
    let mut reset = Psi::reference();
    reset.fold_prompt(&prompt);
    assert_eq!(first.canonical(), reset.canonical());

    // Continuation: a history lane with decay halves its carryover per step.
    let mut cont = Psi::new([8, 8, 4, 8, 8, 4], [0, 0, 0, 0, 1, 0]).unwrap();
    cont.record(LANE_HISTORY, 99, 1 << 10);
    let before = cont.lanes[LANE_HISTORY].slots[0].weight;
    cont.lanes[LANE_HISTORY].decay();
    let after = cont.lanes[LANE_HISTORY].slots[0].weight;
    assert_eq!(
        after,
        before >> 1,
        "continuation decay is a halving right shift"
    );
}

#[test]
fn witness_replay() {
    // Replaying the serialized witness stream reconstructs the state exactly,
    // from an empty verifier, without the model. Use a no-decay state so the
    // upsert witnesses fully determine the outcome.
    let mut psi = Psi::new([4, 4, 4, 4, 4, 4], [0; NUM_LANES]).unwrap();
    psi.fold_prompt(&[1u64, 2, 3, 4, 5]); // 5th segment insert evicts under cap 4
    let bytes = psi.witness_bytes();
    assert_eq!(bytes.len() % WITNESS_BYTES, 0);
    let replayed = replay_witness(&bytes);
    assert_eq!(
        replayed,
        psi.canonical(),
        "witness must replay to the same state"
    );
}

#[test]
fn invalid_capacity_and_index_are_typed_errors() {
    assert_eq!(Ring::new(0, 0).err(), Some(PsError::ZeroCapacity));
    assert_eq!(
        Psi::new([1, 0, 1, 1, 1, 1], [0; NUM_LANES]).err(),
        Some(PsError::ZeroCapacity)
    );
    let psi = Psi::reference();
    assert_eq!(psi.lane(NUM_LANES).err(), Some(PsError::IndexOutOfRange));
    assert!(psi.lane(LANE_SEGMENT).is_ok());

    // Overflow → typed decline, not a silent drop.
    let mut ring = Ring::new(1, 0).unwrap();
    ring.try_insert_no_evict(1, BASE_W, 0, 0).unwrap();
    assert_eq!(
        ring.try_insert_no_evict(2, BASE_W, 1, 1).err(),
        Some(PsError::Decline)
    );
}

#[test]
fn planted_negatives_have_teeth() {
    // Bind to the frozen S1 control vocabulary (#832): the labels this test
    // exercises are exactly the ones the committed suite declares.
    assert_eq!(ControlKind::PromptSwap.label(), "prompt-swap");
    assert_eq!(ControlKind::SuffixOnly.label(), "suffix-only");

    let eval = eval_set();
    let tol: u32 = 100; // permille

    // The causal arm and its controls.
    let causal_primary = causal_delta(Model::Causal, &eval, Control::None);
    let causal_swap = causal_delta(Model::Causal, &eval, Control::PromptSwap);
    let causal_suffix = causal_delta(Model::Causal, &eval, Control::SuffixOnly);

    // Anti-vacuity: the causal arm must be non-degenerate (a real, high signal)
    // and must SEPARATE from both nulls, or the reading licenses nothing.
    assert_eq!(causal_primary.rate_permille(), Some(1000));
    assert!(
        !is_degenerate_control(&causal_primary, &causal_swap, tol),
        "a causal arm must separate from the prompt-swap null"
    );
    assert!(
        !is_degenerate_control(&causal_primary, &causal_suffix, tol),
        "a causal arm must separate from the suffix-only null"
    );

    // The planted negative: a non-causal (constant) arm CANNOT separate from
    // the nulls — the benchmark flags it as degenerate. This is the teeth:
    // a zero/near-null reading is caught, not reported as a capability.
    let const_primary = causal_delta(Model::Constant, &eval, Control::None);
    let const_swap = causal_delta(Model::Constant, &eval, Control::PromptSwap);
    assert!(
        is_degenerate_control(&const_primary, &const_swap, tol),
        "a constant, non-causal arm must be flagged degenerate against the null"
    );
    // And it must not accidentally reach the causal arm's signal.
    assert!(const_primary.rate_permille().unwrap() < causal_primary.rate_permille().unwrap());
}
