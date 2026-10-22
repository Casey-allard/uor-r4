//! M6 intermediate-state causal ablation instrument and integer-path sorted-merge
//! GCD addressed-memory sieve (`m6-causal-ablation-instrument`, `#2863`).
//!
//! Evaluates a served [`IntegerDiscreteModel`] (`uor-r4.lut-discrete/1`) paired with an
//! exact-addressed [`IntegerAddressedMemory`] across four deterministic knockout arms:
//! - [`AblationArm::None`]: un-ablated baseline, bit-identical to the unhooked path;
//! - [`AblationArm::KnockoutStateTransport`]: projects every intermediate binder-route
//!   action to the identity element (`A_0 = I_8`) on the 16-root `E_8` orbit and zeroes
//!   level and relation transport gates;
//! - [`AblationArm::KnockoutAddressedMemory`]: suppresses exact-addressed memory reads to
//!   [`MemoryLookupOutcome::Absent`], forcing prediction to rely solely on in-window
//!   trunk state transport;
//! - [`AblationArm::KnockoutBoth`]: applies both knockouts simultaneously.

use serde::{Deserialize, Serialize};

use crate::discrete_d11::{DiscretePrefixEvaluation, IntegerDiscreteModel};
use crate::{invalid, Result};

/// Prime-factor and ordered-pair address key for the integer-path GCD sieve.
///
/// Primes are stored sorted and strictly increasing (`p >= 2`), and ordered adjacent
/// pairs (`(p_a, p_b)`) are stored sorted in lexicographic order so overlap queries
/// execute by linear two-pointer sorted merge with zero division and zero multiplication.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimeFactorKey {
    primes: Vec<u32>,
    ordered_pairs: Vec<(u32, u32)>,
}

impl PrimeFactorKey {
    /// Construct a canonical sorted, deduplicated prime-factor key.
    /// Rejects empty prime lists or any factor `< 2`.
    pub fn new(mut primes: Vec<u32>, mut ordered_pairs: Vec<(u32, u32)>) -> Result<Self> {
        if primes.is_empty() {
            return Err(invalid("PrimeFactorKey requires at least one prime factor"));
        }
        if primes.iter().any(|&p| p < 2) {
            return Err(invalid("PrimeFactorKey factors must be >= 2"));
        }
        primes.sort_unstable();
        primes.dedup();
        ordered_pairs.sort_unstable();
        ordered_pairs.dedup();
        Ok(Self {
            primes,
            ordered_pairs,
        })
    }

    pub fn primes(&self) -> &[u32] {
        &self.primes
    }

    pub fn ordered_pairs(&self) -> &[(u32, u32)] {
        &self.ordered_pairs
    }

    /// Compute `(shared_primes, shared_ordered_pairs)` by linear two-pointer sorted merge.
    /// A positive `shared_primes` count is equivalent to `gcd(q, A_c) > 1` by unique
    /// factorization, requiring zero division and zero multiplication instructions.
    pub fn sorted_merge_gcd_overlap(&self, other: &Self) -> (u32, u32) {
        let mut shared_primes = 0u32;
        let mut i = 0usize;
        let mut j = 0usize;
        while i < self.primes.len() && j < other.primes.len() {
            let a = self.primes[i];
            let b = other.primes[j];
            if a == b {
                shared_primes += 1;
                i += 1;
                j += 1;
            } else if a < b {
                i += 1;
            } else {
                j += 1;
            }
        }

        let mut shared_pairs = 0u32;
        let mut pi = 0usize;
        let mut pj = 0usize;
        while pi < self.ordered_pairs.len() && pj < other.ordered_pairs.len() {
            let a = self.ordered_pairs[pi];
            let b = other.ordered_pairs[pj];
            if a == b {
                shared_pairs += 1;
                pi += 1;
                pj += 1;
            } else if a < b {
                pi += 1;
            } else {
                pj += 1;
            }
        }

        (shared_primes, shared_pairs)
    }

    /// Returns `true` iff `gcd(self, other) > 1` (at least one shared prime factor).
    pub fn admits(&self, other: &Self) -> bool {
        self.sorted_merge_gcd_overlap(other).0 > 0
    }
}

/// Which version of a stored fact to read from [`IntegerAddressedMemory`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryVersionSelector {
    Current,
    Previous,
    Initial,
}

/// Four-way outcome of an exact-addressed memory read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryLookupOutcome {
    Hit {
        value_tokens: Vec<u32>,
        version_count: u32,
        shared_primes: u32,
        shared_pairs: u32,
        turn_index: u32,
    },
    Absent,
    Evicted {
        overwritten_by_turn: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct MemorySlot {
    key: PrimeFactorKey,
    initial: Vec<u32>,
    previous: Option<Vec<u32>>,
    current: Vec<u32>,
    version_count: u32,
    last_turn: u32,
}

/// Integer-path exact-addressed memory store with sorted-merge GCD sieve admission,
/// `[initial, previous, current]` version chains, and explicit capacity eviction records.
#[derive(Clone, Debug)]
pub struct IntegerAddressedMemory {
    capacity: usize,
    slots: Vec<MemorySlot>,
    evicted: Vec<(PrimeFactorKey, u32)>,
}

impl IntegerAddressedMemory {
    pub fn new(capacity: usize) -> Result<Self> {
        if capacity == 0 {
            return Err(invalid("IntegerAddressedMemory capacity must be >= 1"));
        }
        Ok(Self {
            capacity,
            slots: Vec::with_capacity(capacity),
            evicted: Vec::new(),
        })
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn active_len(&self) -> usize {
        self.slots.len()
    }

    /// Write or update a fact under `key` at `turn_index`.
    /// Updating an existing exact `key` shifts `current -> previous`, keeps `initial`,
    /// and increments `version_count`. Inserting a new `key` when `slots.len() == capacity`
    /// evicts the oldest active slot and records its key in the eviction log.
    pub fn write(
        &mut self,
        key: PrimeFactorKey,
        value_tokens: Vec<u32>,
        turn_index: u32,
    ) -> Result<()> {
        if value_tokens.is_empty() {
            return Err(invalid(
                "IntegerAddressedMemory value_tokens must not be empty",
            ));
        }
        if let Some(slot) = self.slots.iter_mut().find(|s| s.key == key) {
            if slot.current != value_tokens {
                slot.previous = Some(slot.current.clone());
                slot.current = value_tokens;
                slot.version_count += 1;
            }
            slot.last_turn = turn_index;
            return Ok(());
        }
        if self.slots.len() == self.capacity {
            let evicted_slot = self.slots.remove(0);
            self.evicted.push((evicted_slot.key, turn_index));
        }
        self.slots.push(MemorySlot {
            key,
            initial: value_tokens.clone(),
            previous: None,
            current: value_tokens,
            version_count: 1,
            last_turn: turn_index,
        });
        Ok(())
    }

    /// Read from the store by sorted-merge GCD admission (`gcd(query, A_c) > 1`),
    /// ranking admitted slots by `(shared_primes, shared_ordered_pairs, last_turn)`.
    pub fn lookup(
        &self,
        query: &PrimeFactorKey,
        version: MemoryVersionSelector,
    ) -> MemoryLookupOutcome {
        let mut best: Option<(&MemorySlot, u32, u32)> = None;
        for slot in &self.slots {
            let (shared_primes, shared_pairs) = query.sorted_merge_gcd_overlap(&slot.key);
            if shared_primes == 0 {
                continue;
            }
            let take = match best {
                None => true,
                Some((prev_slot, prev_primes, prev_pairs)) => {
                    (shared_primes, shared_pairs, slot.last_turn)
                        > (prev_primes, prev_pairs, prev_slot.last_turn)
                }
            };
            if take {
                best = Some((slot, shared_primes, shared_pairs));
            }
        }

        if let Some((slot, shared_primes, shared_pairs)) = best {
            let tokens = match version {
                MemoryVersionSelector::Current => Some(slot.current.clone()),
                MemoryVersionSelector::Previous => slot.previous.clone(),
                MemoryVersionSelector::Initial => Some(slot.initial.clone()),
            };
            return match tokens {
                Some(value_tokens) => MemoryLookupOutcome::Hit {
                    value_tokens,
                    version_count: slot.version_count,
                    shared_primes,
                    shared_pairs,
                    turn_index: slot.last_turn,
                },
                None => MemoryLookupOutcome::Absent,
            };
        }

        for (evicted_key, overwritten_by_turn) in self.evicted.iter().rev() {
            if query.admits(evicted_key) {
                return MemoryLookupOutcome::Evicted {
                    overwritten_by_turn: *overwritten_by_turn,
                };
            }
        }
        MemoryLookupOutcome::Absent
    }
}

/// Knockout arm for the M6 intermediate-state causal ablation instrument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AblationArm {
    /// Un-ablated baseline: runs both geometric state transport and addressed memory.
    None,
    /// Projects intermediate state transport to the identity frame (`A_0 = I_8`) and
    /// zeroes level and relation gates while keeping addressed memory active.
    KnockoutStateTransport,
    /// Suppresses exact-addressed memory reads to [`MemoryLookupOutcome::Absent`] while
    /// keeping trunk state transport active.
    KnockoutAddressedMemory,
    /// Applies both state-transport and addressed-memory knockouts simultaneously.
    KnockoutBoth,
}

impl AblationArm {
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::KnockoutStateTransport,
        Self::KnockoutAddressedMemory,
        Self::KnockoutBoth,
    ];

    pub fn knocks_out_state_transport(self) -> bool {
        matches!(self, Self::KnockoutStateTransport | Self::KnockoutBoth)
    }

    pub fn knocks_out_addressed_memory(self) -> bool {
        matches!(self, Self::KnockoutAddressedMemory | Self::KnockoutBoth)
    }
}

/// Task family on the M6 reasoning and coding panel (`#2034`, `#2863`, `#2866`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum M6TaskFamily {
    Compositions,
    AlteredConstraints,
    CodeTasks,
}

impl M6TaskFamily {
    pub const ALL: [Self; 3] = [
        Self::Compositions,
        Self::AlteredConstraints,
        Self::CodeTasks,
    ];
}

/// One generation/retrieval step inside an [`M6PanelRow`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M6StepQuery {
    /// Predict one token from the trunk on `prefix`.
    TrunkPrefix { prefix: Vec<u32> },
    /// Query exact-addressed memory (`query_key`, `version`); if the read is `Hit`,
    /// emit `value_tokens`, otherwise fall back to predicting one token from the trunk
    /// on `fallback_prefix`.
    AddressedMemoryOrTrunk {
        query_key: PrimeFactorKey,
        version: MemoryVersionSelector,
        fallback_prefix: Vec<u32>,
    },
}

/// Deterministic grading oracle for an [`M6PanelRow`] (CHARTER §3.10, D28).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum M6OracleKind {
    /// Exact sequence match on emitted token IDs.
    ExactTokenSequence { expected: Vec<u32> },
    /// Deterministic structural/execution check for coding tasks: requires exact prefix
    /// and suffix token spans, every token in `required_tokens` present, and zero tokens
    /// from `forbidden_tokens`.
    CodeVerificationOracle {
        expected_prefix: Vec<u32>,
        required_tokens: Vec<u32>,
        forbidden_tokens: Vec<u32>,
        expected_suffix: Vec<u32>,
    },
}

impl M6OracleKind {
    /// Deterministically grade `emitted_tokens` with zero external teachers or neural graders.
    pub fn grade(&self, emitted_tokens: &[u32]) -> bool {
        match self {
            Self::ExactTokenSequence { expected } => emitted_tokens == expected.as_slice(),
            Self::CodeVerificationOracle {
                expected_prefix,
                required_tokens,
                forbidden_tokens,
                expected_suffix,
            } => {
                if !emitted_tokens.starts_with(expected_prefix)
                    || !emitted_tokens.ends_with(expected_suffix)
                {
                    return false;
                }
                if required_tokens.iter().any(|r| !emitted_tokens.contains(r)) {
                    return false;
                }
                if forbidden_tokens.iter().any(|f| emitted_tokens.contains(f)) {
                    return false;
                }
                true
            }
        }
    }
}

/// One row of the M6 reasoning and coding evaluation panel.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6PanelRow {
    pub row_id: String,
    pub family: M6TaskFamily,
    pub steps: Vec<M6StepQuery>,
    pub oracle: M6OracleKind,
}

/// Step-level trace for one row evaluated under one [`AblationArm`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M6RowArmOutcome {
    pub row_id: String,
    pub family: M6TaskFamily,
    pub arm: AblationArm,
    pub emitted_tokens: Vec<u32>,
    pub trunk_evaluations: Vec<DiscretePrefixEvaluation>,
    pub memory_hits: u32,
    pub passed: bool,
}

/// Paired McNemar contrast between [`AblationArm::None`] and a knockout arm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6PairedMcNemarContrast {
    pub knockout_arm: AblationArm,
    /// Rows that pass under `None` and fail under `knockout_arm`.
    pub b_pass_to_fail: u32,
    /// Rows that fail under `None` and pass under `knockout_arm`.
    pub c_fail_to_pass: u32,
    /// Net drop `b - c`.
    pub net_drop: i32,
    /// Pre-registered integer threshold `max(min_drop_floor, ceil(2 * sqrt(b + c)))`.
    pub f_i_floor: u32,
    /// Whether `net_drop >= f_i_floor`.
    pub passes_causal_dependence_bar: bool,
}

/// Per-family summary across all 4 ablation arms.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct M6FamilyReport {
    pub family: M6TaskFamily,
    pub total_rows: u32,
    pub pass_none: u32,
    pub pass_knockout_state_transport: u32,
    pub pass_knockout_addressed_memory: u32,
    pub pass_knockout_both: u32,
    pub state_transport_contrast: M6PairedMcNemarContrast,
    pub addressed_memory_contrast: M6PairedMcNemarContrast,
    pub both_contrast: M6PairedMcNemarContrast,
}

/// Full M6 intermediate-state causal ablation report across all families and arms.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct M6AblationReport {
    pub model_artifact_sha256: String,
    pub total_rows: u32,
    pub pass_none: u32,
    pub pass_knockout_state_transport: u32,
    pub pass_knockout_addressed_memory: u32,
    pub pass_knockout_both: u32,
    pub overall_state_transport_contrast: M6PairedMcNemarContrast,
    pub overall_addressed_memory_contrast: M6PairedMcNemarContrast,
    pub overall_both_contrast: M6PairedMcNemarContrast,
    pub families: Vec<M6FamilyReport>,
    pub row_outcomes: Vec<[M6RowArmOutcome; 4]>,
}

/// Exact integer ceiling `ceil(2 * sqrt(n)) = min { k >= 0 : k^2 >= 4 * n }`
/// computed by summing odd integers `1 + 3 + 5 + ... + (2k - 1) = k^2` with zero
/// multiplication, zero division, and zero floating-point operations.
pub fn ceil_two_sqrt_int(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let target = u64::from(n) << 2;
    let mut k = 0u32;
    let mut k_sq = 0u64;
    let mut odd = 1u64;
    while k_sq < target {
        k_sq += odd;
        odd += 2;
        k += 1;
    }
    k
}

fn compute_mcnemar_contrast(
    knockout_arm: AblationArm,
    pairs: &[(bool, bool)],
    min_drop_floor: u32,
) -> M6PairedMcNemarContrast {
    let mut b = 0u32;
    let mut c = 0u32;
    for &(pass_none, pass_ko) in pairs {
        if pass_none && !pass_ko {
            b += 1;
        } else if !pass_none && pass_ko {
            c += 1;
        }
    }
    let net_drop = (b as i32) - (c as i32);
    let two_sqrt = ceil_two_sqrt_int(b + c);
    let f_i_floor = min_drop_floor.max(two_sqrt);
    let passes_causal_dependence_bar = net_drop >= (f_i_floor as i32);
    M6PairedMcNemarContrast {
        knockout_arm,
        b_pass_to_fail: b,
        c_fail_to_pass: c,
        net_drop,
        f_i_floor,
        passes_causal_dependence_bar,
    }
}

/// Evaluate one [`M6PanelRow`] under a specific [`AblationArm`].
pub fn evaluate_m6_row_under_arm(
    base_model: &IntegerDiscreteModel,
    ablated_model: &IntegerDiscreteModel,
    memory: &IntegerAddressedMemory,
    row: &M6PanelRow,
    arm: AblationArm,
) -> Result<M6RowArmOutcome> {
    if row.steps.is_empty() {
        return Err(invalid("M6PanelRow must contain at least one step"));
    }
    let active_model = if arm.knocks_out_state_transport() {
        ablated_model
    } else {
        base_model
    };
    let mut emitted_tokens = Vec::new();
    let mut trunk_evaluations = Vec::new();
    let mut memory_hits = 0u32;

    for step in &row.steps {
        match step {
            M6StepQuery::TrunkPrefix { prefix } => {
                let eval = active_model.evaluate_prefix(prefix)?;
                emitted_tokens.push(eval.prediction);
                trunk_evaluations.push(eval);
            }
            M6StepQuery::AddressedMemoryOrTrunk {
                query_key,
                version,
                fallback_prefix,
            } => {
                let outcome = if arm.knocks_out_addressed_memory() {
                    MemoryLookupOutcome::Absent
                } else {
                    memory.lookup(query_key, *version)
                };
                match outcome {
                    MemoryLookupOutcome::Hit { value_tokens, .. } => {
                        memory_hits += 1;
                        emitted_tokens.extend_from_slice(&value_tokens);
                    }
                    MemoryLookupOutcome::Absent | MemoryLookupOutcome::Evicted { .. } => {
                        let eval = active_model.evaluate_prefix(fallback_prefix)?;
                        emitted_tokens.push(eval.prediction);
                        trunk_evaluations.push(eval);
                    }
                }
            }
        }
    }

    let passed = row.oracle.grade(&emitted_tokens);
    Ok(M6RowArmOutcome {
        row_id: row.row_id.clone(),
        family: row.family,
        arm,
        emitted_tokens,
        trunk_evaluations,
        memory_hits,
        passed,
    })
}

/// Run the full 4-arm M6 intermediate-state causal ablation across `rows` and compute
/// per-family and overall paired McNemar contrasts against `min_drop_floor`.
pub fn evaluate_m6_ablation_panel(
    model: &IntegerDiscreteModel,
    memory: &IntegerAddressedMemory,
    rows: &[M6PanelRow],
    min_drop_floor: u32,
) -> Result<M6AblationReport> {
    if rows.is_empty() {
        return Err(invalid("M6 ablation panel must contain at least one row"));
    }
    let ablated_model = model.with_state_transport_knockout();
    let mut row_outcomes = Vec::with_capacity(rows.len());
    for row in rows {
        let out_none =
            evaluate_m6_row_under_arm(model, &ablated_model, memory, row, AblationArm::None)?;
        let out_st = evaluate_m6_row_under_arm(
            model,
            &ablated_model,
            memory,
            row,
            AblationArm::KnockoutStateTransport,
        )?;
        let out_mem = evaluate_m6_row_under_arm(
            model,
            &ablated_model,
            memory,
            row,
            AblationArm::KnockoutAddressedMemory,
        )?;
        let out_both = evaluate_m6_row_under_arm(
            model,
            &ablated_model,
            memory,
            row,
            AblationArm::KnockoutBoth,
        )?;
        row_outcomes.push([out_none, out_st, out_mem, out_both]);
    }

    let build_contrasts = |slice: &[[M6RowArmOutcome; 4]]| {
        let pairs_st: Vec<(bool, bool)> =
            slice.iter().map(|r| (r[0].passed, r[1].passed)).collect();
        let pairs_mem: Vec<(bool, bool)> =
            slice.iter().map(|r| (r[0].passed, r[2].passed)).collect();
        let pairs_both: Vec<(bool, bool)> =
            slice.iter().map(|r| (r[0].passed, r[3].passed)).collect();
        (
            compute_mcnemar_contrast(
                AblationArm::KnockoutStateTransport,
                &pairs_st,
                min_drop_floor,
            ),
            compute_mcnemar_contrast(
                AblationArm::KnockoutAddressedMemory,
                &pairs_mem,
                min_drop_floor,
            ),
            compute_mcnemar_contrast(AblationArm::KnockoutBoth, &pairs_both, min_drop_floor),
        )
    };

    let mut families = Vec::with_capacity(M6TaskFamily::ALL.len());
    for family in M6TaskFamily::ALL {
        let fam_rows: Vec<[M6RowArmOutcome; 4]> = row_outcomes
            .iter()
            .filter(|r| r[0].family == family)
            .cloned()
            .collect();
        let (state_transport_contrast, addressed_memory_contrast, both_contrast) =
            build_contrasts(&fam_rows);
        families.push(M6FamilyReport {
            family,
            total_rows: fam_rows.len() as u32,
            pass_none: fam_rows.iter().filter(|r| r[0].passed).count() as u32,
            pass_knockout_state_transport: fam_rows.iter().filter(|r| r[1].passed).count() as u32,
            pass_knockout_addressed_memory: fam_rows.iter().filter(|r| r[2].passed).count() as u32,
            pass_knockout_both: fam_rows.iter().filter(|r| r[3].passed).count() as u32,
            state_transport_contrast,
            addressed_memory_contrast,
            both_contrast,
        });
    }

    let (
        overall_state_transport_contrast,
        overall_addressed_memory_contrast,
        overall_both_contrast,
    ) = build_contrasts(&row_outcomes);

    Ok(M6AblationReport {
        model_artifact_sha256: model.artifact_sha256().to_owned(),
        total_rows: row_outcomes.len() as u32,
        pass_none: row_outcomes.iter().filter(|r| r[0].passed).count() as u32,
        pass_knockout_state_transport: row_outcomes.iter().filter(|r| r[1].passed).count() as u32,
        pass_knockout_addressed_memory: row_outcomes.iter().filter(|r| r[2].passed).count() as u32,
        pass_knockout_both: row_outcomes.iter().filter(|r| r[3].passed).count() as u32,
        overall_state_transport_contrast,
        overall_addressed_memory_contrast,
        overall_both_contrast,
        families,
        row_outcomes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discrete_d11::DiscreteLutShape;

    const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0]; // root 0
    const B: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0]; // root 2
    const C: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0]; // root 3
    const D: [i32; 8] = [0, 0, -2, 0, 2, 0, 0, 0]; // root 4

    fn root_idx(block: &[i32; 8]) -> u32 {
        match *block {
            A => 0,
            B => 2,
            C => 3,
            D => 4,
            _ => panic!("unexpected test root"),
        }
    }

    fn build_relation_c2_bytes() -> Vec<u8> {
        // 5-token, 2-block Relation model with SequentialSuccessor active at level 0
        // (`level_gates[1][0] = 1`, `relation_gates[0][2] = 1`).
        // On prefix `[0, 1, 2, 3, 4, 1]`, the un-ablated Relation read follows the
        // sequential successor of the earlier token `1` (which is token `2` at `[A, B]`)
        // and predicts token `2` (`[A, B]`). When state transport is knocked out, the
        // read vector stays at token `1`'s local block `[A, D]` and predicts token `1`.
        let blocks_per_token: [[[i32; 8]; 2]; 5] = [
            [B, A], // token 0
            [A, D], // token 1
            [A, B], // token 2 (successor of token 1)
            [C, A], // token 3
            [D, A], // token 4
        ];
        let mut table = Vec::new();
        let mut cells = Vec::new();
        for tb in &blocks_per_token {
            for b in tb {
                table.extend_from_slice(b);
            }
            cells.push((root_idx(&tb[0]) << 4) + root_idx(&tb[1]));
        }
        let routes = vec![0u32; 4]; // 2 blocks * 2 stages
        let mut gates = vec![0i32; 4 + 5 * 2]; // 2*2 base + 5 EDGE_TYPES * 2 stages
        gates[2] = 1; // block 1, stage 0 level gate = +1
        gates[6] = 1; // stage 0, edge 2 (SequentialSuccessor) = +1

        let encode_i32 =
            |xs: &[i32]| -> Vec<u8> { xs.iter().flat_map(|x| x.to_le_bytes()).collect() };
        let encode_u32 = |xs: &[u32]| -> Vec<u32> { xs.to_vec() };
        let encode_u32_bytes = |xs: &[u32]| -> Vec<u8> {
            encode_u32(xs)
                .iter()
                .flat_map(|x| x.to_le_bytes())
                .collect()
        };

        let sections: [(&str, &str, usize, Vec<u8>); 4] = [
            ("discrete.table", "i32", table.len(), encode_i32(&table)),
            (
                "discrete.cells",
                "u32",
                cells.len(),
                encode_u32_bytes(&cells),
            ),
            (
                "discrete.routes",
                "u32",
                routes.len(),
                encode_u32_bytes(&routes),
            ),
            ("discrete.gates", "i32", gates.len(), encode_i32(&gates)),
        ];
        let mut payload = Vec::new();
        let mut specs = Vec::new();
        for (name, kind, len, raw) in sections {
            let pad = (64 - (payload.len() % 64)) % 64;
            payload.extend(std::iter::repeat_n(0u8, pad));
            let offset = payload.len() as u64;
            let bytes = raw.len() as u64;
            payload.extend_from_slice(&raw);
            let tail = (64 - (payload.len() % 64)) % 64;
            payload.extend(std::iter::repeat_n(0u8, tail));
            specs.push(serde_json::json!({
                "name": name,
                "kind": kind,
                "len": len,
                "span": { "offset": offset, "bytes": bytes }
            }));
        }
        let shape = DiscreteLutShape {
            vocabulary: 5,
            blocks: 2,
            context: 6,
            kept_cells: 2,
            stages: 2,
            read_mode: "relation".into(),
            read_kept: 1,
            read_sweeps: 1,
            objective_policy: "nearest_address_margin_v1".into(),
            read_initialization: "active_leaf_v1".into(),
            seed: 7,
            epoch: 1,
        };
        let header = serde_json::json!({
            "schema": "uor-r4.lut-discrete/1",
            "shape": shape,
            "matrices": [],
            "tables": specs,
            "source": {
                "snapshot_schema": "uor-r4.discrete-spiralcore-fbs/2",
                "snapshot_sha256": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                "operator_kappa": "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
                "composition_kappa": "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
                "kernel_sha256": "1111111111111111111111111111111111111111111111111111111111111111",
                "source_sha256": "2222222222222222222222222222222222222222222222222222222222222222",
                "geometry_sha256": "3333333333333333333333333333333333333333333333333333333333333333"
            }
        });
        let header_bytes = serde_json::to_vec(&header).unwrap();
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header_bytes.len() as u64).to_le_bytes());
        out.extend_from_slice(&header_bytes);
        let pad = (64 - (out.len() % 64)) % 64;
        out.resize(out.len() + pad, 0);
        out.extend_from_slice(&payload);
        out
    }

    #[test]
    fn sorted_merge_gcd_sieve_and_eviction_versioning() {
        let mut mem = IntegerAddressedMemory::new(2).unwrap();
        let k1 = PrimeFactorKey::new(vec![11, 3, 5, 3], vec![(3, 5), (5, 11)]).unwrap();
        assert_eq!(k1.primes(), &[3, 5, 11]);

        let k2 = PrimeFactorKey::new(vec![7, 13], vec![(7, 13)]).unwrap();
        let k3 = PrimeFactorKey::new(vec![5, 11, 17], vec![(11, 5)]).unwrap();

        mem.write(k1.clone(), vec![2, 4], 1).unwrap();
        mem.write(k1.clone(), vec![2, 3], 2).unwrap();
        mem.write(k2.clone(), vec![4], 3).unwrap();

        // Query sharing primes (5, 11) and ordered pair (5, 11) selects k1 over k3
        let q = PrimeFactorKey::new(vec![5, 11], vec![(5, 11)]).unwrap();
        assert_eq!(
            mem.lookup(&q, MemoryVersionSelector::Current),
            MemoryLookupOutcome::Hit {
                value_tokens: vec![2, 3],
                version_count: 2,
                shared_primes: 2,
                shared_pairs: 1,
                turn_index: 2,
            }
        );
        assert_eq!(
            mem.lookup(&q, MemoryVersionSelector::Previous),
            MemoryLookupOutcome::Hit {
                value_tokens: vec![2, 4],
                version_count: 2,
                shared_primes: 2,
                shared_pairs: 1,
                turn_index: 2,
            }
        );
        assert_eq!(
            mem.lookup(&q, MemoryVersionSelector::Initial),
            MemoryLookupOutcome::Hit {
                value_tokens: vec![2, 4],
                version_count: 2,
                shared_primes: 2,
                shared_pairs: 1,
                turn_index: 2,
            }
        );

        // Writing a 3rd distinct key at capacity=2 evicts k1 (oldest slot)
        mem.write(k3.clone(), vec![1], 4).unwrap();
        let q_k1_only = PrimeFactorKey::new(vec![3], vec![]).unwrap();
        assert_eq!(
            mem.lookup(&q_k1_only, MemoryVersionSelector::Current),
            MemoryLookupOutcome::Evicted {
                overwritten_by_turn: 4
            }
        );

        // Unwritten prime returns Absent
        let q_absent = PrimeFactorKey::new(vec![29], vec![]).unwrap();
        assert_eq!(
            mem.lookup(&q_absent, MemoryVersionSelector::Current),
            MemoryLookupOutcome::Absent
        );
    }

    #[test]
    fn no_knockout_arm_is_bit_identical_to_unhooked_path() {
        let model = IntegerDiscreteModel::from_bytes(&build_relation_c2_bytes()).unwrap();
        let ablated = model.with_state_transport_knockout();
        let mut mem = IntegerAddressedMemory::new(4).unwrap();
        let key = PrimeFactorKey::new(vec![2, 3], vec![(2, 3)]).unwrap();
        mem.write(key.clone(), vec![4, 2], 1).unwrap();

        let prefix = vec![0, 1, 2, 3, 4, 1];
        let direct_eval = model.evaluate_prefix(&prefix).unwrap();
        let direct_mem = mem.lookup(&key, MemoryVersionSelector::Current);

        let row = M6PanelRow {
            row_id: "control-bit-identical".into(),
            family: M6TaskFamily::Compositions,
            steps: vec![
                M6StepQuery::TrunkPrefix {
                    prefix: prefix.clone(),
                },
                M6StepQuery::AddressedMemoryOrTrunk {
                    query_key: key,
                    version: MemoryVersionSelector::Current,
                    fallback_prefix: vec![0, 1],
                },
            ],
            oracle: M6OracleKind::ExactTokenSequence {
                expected: vec![2, 4, 2],
            },
        };

        let hooked =
            evaluate_m6_row_under_arm(&model, &ablated, &mem, &row, AblationArm::None).unwrap();
        assert!(hooked.passed);
        assert_eq!(hooked.trunk_evaluations.len(), 1);
        assert_eq!(hooked.trunk_evaluations[0], direct_eval);
        if let MemoryLookupOutcome::Hit { value_tokens, .. } = direct_mem {
            assert_eq!(
                hooked.emitted_tokens,
                vec![direct_eval.prediction, value_tokens[0], value_tokens[1]]
            );
        } else {
            panic!("expected memory hit");
        }
    }

    #[test]
    fn four_arm_causal_ablation_separates_state_transport_and_addressed_memory() {
        let model = IntegerDiscreteModel::from_bytes(&build_relation_c2_bytes()).unwrap();
        let mut mem = IntegerAddressedMemory::new(8).unwrap();

        let k_comp = PrimeFactorKey::new(vec![2, 5], vec![(2, 5)]).unwrap();
        let k_alt = PrimeFactorKey::new(vec![3, 7], vec![(3, 7)]).unwrap();
        let k_code = PrimeFactorKey::new(vec![11, 13], vec![(11, 13)]).unwrap();
        mem.write(k_comp.clone(), vec![4], 1).unwrap();
        mem.write(k_alt.clone(), vec![3], 2).unwrap();
        mem.write(k_code.clone(), vec![2, 4, 0], 3).unwrap();

        // Prefix [0, 1, 2, 3, 4, 1]:
        // - Un-ablated state transport predicts 2 (copy-successor of token 1).
        // - Knocked-out state transport predicts 1 (token 1's own un-transported block).
        let transport_prefix = vec![0, 1, 2, 3, 4, 1];
        // Fallback prefix [0, 1]: predicts 1 on the trunk, whereas memory returns 4 / 3 / [2, 4, 0].
        let fallback_prefix = vec![0, 1];

        let mut rows = Vec::new();
        for idx in 0..2 {
            // Pure state-transport rows (2 per family = 6 total)
            rows.push(M6PanelRow {
                row_id: format!("comp-transport-{idx}"),
                family: M6TaskFamily::Compositions,
                steps: vec![M6StepQuery::TrunkPrefix {
                    prefix: transport_prefix.clone(),
                }],
                oracle: M6OracleKind::ExactTokenSequence { expected: vec![2] },
            });
            rows.push(M6PanelRow {
                row_id: format!("alt-transport-{idx}"),
                family: M6TaskFamily::AlteredConstraints,
                steps: vec![M6StepQuery::TrunkPrefix {
                    prefix: transport_prefix.clone(),
                }],
                oracle: M6OracleKind::ExactTokenSequence { expected: vec![2] },
            });
            rows.push(M6PanelRow {
                row_id: format!("code-transport-{idx}"),
                family: M6TaskFamily::CodeTasks,
                steps: vec![M6StepQuery::TrunkPrefix {
                    prefix: transport_prefix.clone(),
                }],
                oracle: M6OracleKind::CodeVerificationOracle {
                    expected_prefix: vec![2],
                    required_tokens: vec![2],
                    forbidden_tokens: vec![1],
                    expected_suffix: vec![2],
                },
            });

            // Pure addressed-memory rows (2 per family = 6 total)
            rows.push(M6PanelRow {
                row_id: format!("comp-memory-{idx}"),
                family: M6TaskFamily::Compositions,
                steps: vec![M6StepQuery::AddressedMemoryOrTrunk {
                    query_key: k_comp.clone(),
                    version: MemoryVersionSelector::Current,
                    fallback_prefix: fallback_prefix.clone(),
                }],
                oracle: M6OracleKind::ExactTokenSequence { expected: vec![4] },
            });
            rows.push(M6PanelRow {
                row_id: format!("alt-memory-{idx}"),
                family: M6TaskFamily::AlteredConstraints,
                steps: vec![M6StepQuery::AddressedMemoryOrTrunk {
                    query_key: k_alt.clone(),
                    version: MemoryVersionSelector::Current,
                    fallback_prefix: fallback_prefix.clone(),
                }],
                oracle: M6OracleKind::ExactTokenSequence { expected: vec![3] },
            });
            rows.push(M6PanelRow {
                row_id: format!("code-memory-{idx}"),
                family: M6TaskFamily::CodeTasks,
                steps: vec![M6StepQuery::AddressedMemoryOrTrunk {
                    query_key: k_code.clone(),
                    version: MemoryVersionSelector::Current,
                    fallback_prefix: fallback_prefix.clone(),
                }],
                oracle: M6OracleKind::CodeVerificationOracle {
                    expected_prefix: vec![2],
                    required_tokens: vec![4],
                    forbidden_tokens: vec![1],
                    expected_suffix: vec![0],
                },
            });
        }

        let report = evaluate_m6_ablation_panel(&model, &mem, &rows, 4).unwrap();
        assert_eq!(report.total_rows, 12);
        assert_eq!(report.pass_none, 12);
        assert_eq!(report.pass_knockout_state_transport, 6);
        assert_eq!(report.pass_knockout_addressed_memory, 6);
        assert_eq!(report.pass_knockout_both, 0);

        // State-transport knockout drops the 6 transport rows (b=6, c=0, ceil(2*sqrt(6))=5, passes floor=5)
        assert_eq!(report.overall_state_transport_contrast.b_pass_to_fail, 6);
        assert_eq!(report.overall_state_transport_contrast.c_fail_to_pass, 0);
        assert_eq!(report.overall_state_transport_contrast.net_drop, 6);
        assert_eq!(report.overall_state_transport_contrast.f_i_floor, 5);
        assert!(
            report
                .overall_state_transport_contrast
                .passes_causal_dependence_bar
        );

        // Addressed-memory knockout drops the 6 memory rows (b=6, c=0, ceil(2*sqrt(6))=5, passes floor=5)
        assert_eq!(report.overall_addressed_memory_contrast.b_pass_to_fail, 6);
        assert_eq!(report.overall_addressed_memory_contrast.c_fail_to_pass, 0);
        assert_eq!(report.overall_addressed_memory_contrast.net_drop, 6);
        assert_eq!(report.overall_addressed_memory_contrast.f_i_floor, 5);
        assert!(
            report
                .overall_addressed_memory_contrast
                .passes_causal_dependence_bar
        );

        // Both knockout drops all 12 rows (b=12, c=0, ceil(2*sqrt(12))=7, passes floor=7)
        assert_eq!(report.overall_both_contrast.b_pass_to_fail, 12);
        assert_eq!(report.overall_both_contrast.c_fail_to_pass, 0);
        assert_eq!(report.overall_both_contrast.net_drop, 12);
        assert_eq!(report.overall_both_contrast.f_i_floor, 7);
        assert!(report.overall_both_contrast.passes_causal_dependence_bar);

        for fam in &report.families {
            assert_eq!(fam.total_rows, 4);
            assert_eq!(fam.pass_none, 4);
            assert_eq!(fam.pass_knockout_state_transport, 2);
            assert_eq!(fam.pass_knockout_addressed_memory, 2);
            assert_eq!(fam.pass_knockout_both, 0);
        }
    }
}
