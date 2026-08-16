//! `MsaStructuredSelectorV1` reference semantics and witness (#643): the
//! second TARGET operator, alongside `r4-route-attention/1` (#604),
//! registered for a pre-registered A/B evaluation against it. Certifier
//! -side code, next to `route_attention.rs` where the first target
//! operator's reference semantics live — same layering, deliberately
//! smaller: this operator has no per-byte relation to compute, so its
//! reference and witness are proportionally shorter.
//!
//! # Grounding (Modular Structural Arithmetic, Casey-approved publication
//! 2026-08-16)
//!
//! Two THEOREMS of the paper are reused directly as fixed tables, no
//! runtime derivation:
//!
//! - **MSA7, "The 11-Theorem":** `DP(11)` holds with role anchors
//!   `mod_11(γ) = 2` (Gen), `mod_11(μ) = 4` (Med), `mod_11(ε) = 8`
//!   (Man) — the only three residues mod 11 the paper actually assigns
//!   a role.
//! - **Theorem M4, "The 11-Cascade Theorem":** the doubling cascade
//!   `(2, 4, 8, 5, 10, 9, 7, 3, 6, 1)` has maximal period 10 = p−1 in
//!   ℤ/11ℤ (Lagrange, via Theorem M3).
//!
//! # Role extension beyond the paper (explicit, confirmed choice)
//!
//! The paper never assigns a role to the other 8 nonzero-or-zero
//! residues (0, 1, 3, 5, 6, 7, 9, 10) — that depends on the base
//! Structural Arithmetic theory's `Gen`/`Med`/`Man` predicates for
//! values other than γ/μ/ε, which is not in this paper. This module
//! extends role coverage to every residue by the anchors' own cascade
//! POSITIONS: γ=2 sits at orbit position 0, μ=4 at position 1, ε=8 at
//! position 2, so every residue's role is `orbit_position(residue) mod
//! 3`. Residue 0 is outside the multiplicative group entirely (it has
//! no orbit under doubling) and gets its own fourth "zero" class. This
//! extension was proposed and confirmed by Casey (2026-08-16, in
//! response to an explicit question) — it is this operator's design
//! choice, not a proven MSA theorem, and every doc comment that states
//! it says so.
//!
//! # Reference specification (version 1)
//!
//! An operator INSTANCE is `N` declared candidate ids (`u32`, arbitrary
//! — typically route/token ids) with one declared `ScoreQ` contribution
//! each, and a declared selection width `M` under `1 <= M <= N`. No
//! query enters the classification at all — unlike
//! `r4-route-attention/1`, this operator's per-step "query" parameter
//! exists only for interface parity (the same fixture can drive either
//! operator through the same harness for the A/B); it does not affect
//! the outcome, and every step over the same instance is identical.
//!
//! One STEP:
//!
//! 1. **Classification** (table lookup): for every candidate `i` in
//!    index order, `residue = candidate_id[i] mod 11`, then
//!    `role_rank = cascade_position(residue) mod 3` (or the sentinel
//!    zero-class `3` when `residue == 0`), `cascade_position =
//!    cascade_position(residue)` (or the sentinel `10`, one past the
//!    max valid orbit index, when `residue == 0`). Table reads only —
//!    the reference computes `mod 11` directly with an integer `%`
//!    (certifier-side; not the P-4-scanned deployed surface), but the
//!    result is exactly what a precomputed per-vocabulary table would
//!    return, which is what a future packed lowering must actually use
//!    (`permitted_operation_class` on the registry record already
//!    names this: no runtime modulo in the deployed path).
//! 2. **Selection** (bounded top-M): the `M` smallest candidates under
//!    the strict lexicographic order `(role_rank, cascade_position,
//!    index)`. Deterministic tie rule: on equal role and cascade
//!    position, the LOWEST candidate INDEX wins — the same
//!    tie-breaking shape as `r4-route-attention/1`.
//! 3. **Aggregation** (integer/table): the selected contributions fold
//!    from `ScoreQ::ZERO` in selection order with SATURATING i32
//!    adds — identical convention to `r4-route-attention/1`, so the two
//!    operators are plug-compatible for the A/B harness.
//!
//! # Non-goals of this slice
//!
//! No packed R4G1 lowering exists yet (mirrors #604's own two-stage
//! plan: reference semantics first). No A/B run has happened — the
//! exit rule is pre-registered on #643 before any run, per the #626
//! convention. No wire-format/artifact carriage exists (same dormant
//! posture as `r4-route-attention/1`).
//!
//! # Census (per step, closed form)
//!
//! ```text
//! table_reads         = N + M   (one classification read per candidate,
//!                                one contribution read per selected slot)
//! compares             = M*N
//! adds                 = M
//! candidates_examined  = N
//! ```

use serde::{Deserialize, Serialize};

use uor_r4_graph_format::ScoreQ;

/// Format tag of the witness record.
pub const MSA_SELECTOR_WITNESS_FORMAT: &str = "uor-r4-msa-selector-witness/1";
/// Domain-separation prefix of the inputs digest.
pub const MSA_SELECTOR_INPUTS_DOMAIN: &str = "uor-r4-msa-selector-inputs/1";
/// Domain-separation prefix of the instance bytes.
const MSA_SELECTOR_INSTANCE_TAG: &[u8] = b"uor-r4-msa-selector-instance/1\n";

/// Theorem M4 ("The 11-Cascade Theorem"): the doubling cascade starting
/// at 2 in `(ℤ/11ℤ)*`, in orbit order. Position 0 = γ's residue (2,
/// Gen), position 1 = μ's residue (4, Med), position 2 = ε's residue
/// (8, Man) — MSA7, "The 11-Theorem". Positions 3..9 are this module's
/// own mod-3 role extension (see module docs), not a paper theorem.
const CASCADE_ORBIT_11: [u8; 10] = [2, 4, 8, 5, 10, 9, 7, 3, 6, 1];
/// The modulus this operator is pinned to (MSA7/M4 are both stated for
/// p = 11 specifically).
const CASCADE_MODULUS: u32 = 11;
/// Residue 0 is outside `(ℤ/11ℤ)*` and has no orbit position; this
/// sentinel sorts after every real position (0..=9).
const CASCADE_SENTINEL_POSITION: u8 = 10;

/// Role rank of the γ anchor (`mod_11(γ) = 2`, cascade position 0) —
/// MSA7.
pub const ROLE_GEN: u8 = 0;
/// Role rank of the μ anchor (`mod_11(μ) = 4`, cascade position 1) —
/// MSA7.
pub const ROLE_MED: u8 = 1;
/// Role rank of the ε anchor (`mod_11(ε) = 8`, cascade position 2) —
/// MSA7.
pub const ROLE_MAN: u8 = 2;
/// Role rank of residue 0 — outside the multiplicative group, so
/// outside every role MSA7 assigns; sentinel class, sorts last.
pub const ROLE_ZERO: u8 = 3;

/// This residue's position in [`CASCADE_ORBIT_11`], or `None` for
/// residue 0 (not in the multiplicative group `(ℤ/11ℤ)*`).
fn cascade_position(residue: u8) -> Option<u8> {
    CASCADE_ORBIT_11
        .iter()
        .position(|&candidate| candidate == residue)
        .map(|position| position as u8)
}

/// This residue's role rank: the proven MSA7 anchors sit at cascade
/// positions 0/1/2 (Gen/Med/Man); every other nonzero residue's role is
/// its cascade position mod 3 (this module's extension, see module
/// docs); residue 0 is [`ROLE_ZERO`].
fn role_rank(residue: u8) -> u8 {
    match cascade_position(residue) {
        Some(position) => position % 3,
        None => ROLE_ZERO,
    }
}

/// One candidate id's classification: its role rank and cascade
/// position under modulus 11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MsaClassification {
    /// [`ROLE_GEN`]/[`ROLE_MED`]/[`ROLE_MAN`]/[`ROLE_ZERO`].
    pub role_rank: u8,
    /// Position in [`CASCADE_ORBIT_11`] (0..=9), or
    /// [`CASCADE_SENTINEL_POSITION`] for residue 0.
    pub cascade_position: u8,
}

/// Classify a candidate id: `residue = candidate_id mod 11`, then table
/// lookup for role and cascade position. Total — every `u32` classifies
/// to something, by construction (`role_rank` and `cascade_position`
/// each have an explicit branch for residue 0).
pub fn classify(candidate_id: u32) -> MsaClassification {
    let residue = (candidate_id % CASCADE_MODULUS) as u8;
    MsaClassification {
        role_rank: role_rank(residue),
        cascade_position: cascade_position(residue).unwrap_or(CASCADE_SENTINEL_POSITION),
    }
}

/// The op census (witness): every field a closed form of `(N, M,
/// steps)` (module docs table), deliberately data-independent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsaSelectorOpCensus {
    /// Table reads: one classification per candidate, one contribution
    /// per selected slot.
    #[serde(default)]
    pub table_reads: u64,
    /// Ordered slot comparisons during top-M insertion.
    #[serde(default)]
    pub compares: u64,
    /// Saturating adds during the aggregate fold.
    #[serde(default)]
    pub adds: u64,
    /// Candidates classified.
    #[serde(default)]
    pub candidates_examined: u64,
}

/// The census closed form for `steps` steps over an instance with
/// `candidate_count` candidates selecting `top_m` (module docs table).
pub fn expected_msa_selector_census(
    candidate_count: u32,
    top_m: u16,
    steps: usize,
) -> MsaSelectorOpCensus {
    let mut census = MsaSelectorOpCensus::default();
    let mut step = 0usize;
    while step < steps {
        let mut candidate = 0u32;
        while candidate < candidate_count {
            census.candidates_examined += 1;
            census.table_reads += 1;
            census.compares += u64::from(top_m);
            candidate += 1;
        }
        let mut slot = 0u16;
        while slot < top_m {
            census.table_reads += 1;
            census.adds += 1;
            slot += 1;
        }
        step += 1;
    }
    census
}

/// One selected candidate of one step, in selection order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsaSelection {
    /// Candidate INDEX into the instance's declared id/contribution
    /// tables (the same "index, not id" convention `RouteSelection`
    /// uses) — the tie-break winner on equal classification.
    #[serde(default)]
    pub candidate: u32,
    /// The declared candidate id at that index (`instance
    /// candidate_ids[candidate]`), carried for readability.
    #[serde(default)]
    pub candidate_id: u32,
    /// [`MsaClassification::role_rank`] of that candidate.
    #[serde(default)]
    pub role_rank: u8,
    /// [`MsaClassification::cascade_position`] of that candidate.
    #[serde(default)]
    pub cascade_position: u8,
}

/// One step of the witness: the selected candidates (selection order)
/// and the aggregate they fold to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsaSelectorWitnessStep {
    /// Selected candidates, selection order.
    #[serde(default)]
    pub selected: Vec<MsaSelection>,
    /// Raw ScoreQ of the selection-order saturating fold.
    #[serde(default)]
    pub aggregate_raw: i32,
}

/// The bounded, replayable record of one run: identity, input binding,
/// per-step selections, and the op census. All fields serde-defaulted,
/// same era discipline as [`uor_r4_graph_format`]'s peer records.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsaSelectorWitness {
    /// [`MSA_SELECTOR_WITNESS_FORMAT`].
    #[serde(default)]
    pub format: String,
    /// Operator registry id (`msa-structured-selector`).
    #[serde(default)]
    pub operator_id: String,
    /// Operator registry version (1).
    #[serde(default)]
    pub operator_version: u32,
    /// `blake3:<hex>` of the canonical instance bytes.
    #[serde(default)]
    pub instance_digest: String,
    /// `blake3:<hex>` binding the instance digest and the step count.
    #[serde(default)]
    pub inputs_digest: String,
    /// Per-step selections and aggregates.
    #[serde(default)]
    pub steps: Vec<MsaSelectorWitnessStep>,
    /// Op census of the whole run.
    #[serde(default)]
    pub census: MsaSelectorOpCensus,
}

/// One step's outcome as returned to callers.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MsaSelectorStepRecord {
    /// Selected candidates, selection order.
    pub selected: Vec<MsaSelection>,
    /// The selection-order saturating ScoreQ fold.
    pub aggregate: ScoreQ,
}

/// `blake3:<hex>` presentation of a raw digest.
pub fn digest_string(digest: &[u8; 32]) -> String {
    format!("blake3:{}", blake3::Hash::from_bytes(*digest).to_hex())
}

/// Canonical instance bytes: format tag, candidate count, `top_m`, then
/// every `(candidate_id, contribution)` pair in declared order. Fixed
/// line/field layout, not derived from any serializer, so the digest
/// over these bytes is reproducible everywhere — same convention as
/// `AttentionOperatorSpec::canonical_bytes`.
pub fn msa_instance_bytes(candidate_ids: &[u32], contributions: &[ScoreQ], top_m: u16) -> Vec<u8> {
    let mut bytes =
        Vec::with_capacity(MSA_SELECTOR_INSTANCE_TAG.len() + 8 + candidate_ids.len() * 8);
    bytes.extend_from_slice(MSA_SELECTOR_INSTANCE_TAG);
    bytes.extend_from_slice(&(candidate_ids.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&top_m.to_le_bytes());
    for (&candidate_id, &contribution) in candidate_ids.iter().zip(contributions.iter()) {
        bytes.extend_from_slice(&candidate_id.to_le_bytes());
        bytes.extend_from_slice(&contribution.raw().to_le_bytes());
    }
    bytes
}

/// The inputs digest: blake3 over the domain tag, the instance digest,
/// and the step count (u32 LE). Binds the witness to exactly one
/// (instance, step-count) pair — there is no query sequence to bind
/// (query-independent classification, module docs).
pub fn msa_inputs_digest(instance_digest: &[u8; 32], steps: usize) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(MSA_SELECTOR_INPUTS_DOMAIN.as_bytes());
    hasher.update(b"\n");
    hasher.update(instance_digest);
    hasher.update(&(steps as u32).to_le_bytes());
    *hasher.finalize().as_bytes()
}

/// The scalar reference implementation: owned declared candidate ids
/// and contributions, plus the canonical instance bytes for identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsaSelectorReference {
    candidate_ids: Vec<u32>,
    contributions: Vec<ScoreQ>,
    top_m: usize,
    instance_bytes: Vec<u8>,
}

impl MsaSelectorReference {
    /// Build the reference from declared candidate ids and their
    /// contributions. `None` when the instance is malformed: empty,
    /// mismatched lengths, or `top_m` outside `1..=candidate_ids.len()`
    /// — the same fail-closed posture as
    /// `RouteAttentionReference::from_instance_bytes`, expressed as
    /// `Option` here since there is no wire format to name a
    /// `NotAProduct` violation against yet (module docs: no artifact
    /// carriage in this slice).
    pub fn new(candidate_ids: Vec<u32>, contributions: Vec<ScoreQ>, top_m: usize) -> Option<Self> {
        if candidate_ids.is_empty() || candidate_ids.len() != contributions.len() {
            return None;
        }
        if top_m == 0 || top_m > candidate_ids.len() {
            return None;
        }
        let instance_bytes = msa_instance_bytes(&candidate_ids, &contributions, top_m as u16);
        Some(Self {
            candidate_ids,
            contributions,
            top_m,
            instance_bytes,
        })
    }

    /// The canonical instance bytes this reference was built from.
    pub fn instance_bytes(&self) -> &[u8] {
        &self.instance_bytes
    }

    /// Declared candidate count `N`.
    pub fn candidate_count(&self) -> u32 {
        self.candidate_ids.len() as u32
    }

    /// Declared selection width `M`.
    pub fn top_m(&self) -> u16 {
        self.top_m as u16
    }

    /// One reference step (module-docs semantics; query-independent, so
    /// every call returns the same result), counting into `census` with
    /// exactly the declared operation schedule.
    pub fn reference_step(&self, census: &mut MsaSelectorOpCensus) -> MsaSelectorStepRecord {
        // Selection slots, ascending (role_rank, cascade_position,
        // index); sentinel triples order after every real candidate.
        let mut slots: Vec<(u8, u8, u32)> = Vec::with_capacity(self.top_m);
        slots.resize(self.top_m, (u8::MAX, u8::MAX, u32::MAX));
        for (index, &candidate_id) in self.candidate_ids.iter().enumerate() {
            let index = index as u32;
            census.candidates_examined += 1;
            census.table_reads += 1;
            let classification = classify(candidate_id);
            let key = (
                classification.role_rank,
                classification.cascade_position,
                index,
            );
            // Exactly M ordered slot comparisons; the first slot this
            // candidate beats is its insertion point — same shape as
            // `RouteAttentionReference::reference_step`.
            let mut insert_at = None;
            for (slot, &slot_key) in slots.iter().enumerate() {
                census.compares += 1;
                if key < slot_key && insert_at.is_none() {
                    insert_at = Some(slot);
                }
            }
            if let Some(position) = insert_at {
                slots.insert(position, key);
                slots.truncate(self.top_m);
            }
        }
        // Selection-order saturating fold of the selected contributions.
        let mut aggregate = ScoreQ::ZERO;
        let mut selected = Vec::with_capacity(self.top_m);
        for &(role_rank, cascade_position, index) in &slots {
            let contribution = self.contributions[index as usize];
            census.table_reads += 1;
            aggregate = aggregate.saturating_add(contribution);
            census.adds += 1;
            selected.push(MsaSelection {
                candidate: index,
                candidate_id: self.candidate_ids[index as usize],
                role_rank,
                cascade_position,
            });
        }
        MsaSelectorStepRecord {
            selected,
            aggregate,
        }
    }

    /// Run the reference for `steps` steps, producing the step records
    /// and the replayable witness. Every step is identical
    /// (query-independent classification, module docs) — `steps` exists
    /// for interface parity with the per-query `run` shape
    /// `r4-route-attention/1` uses, so the same A/B harness can drive
    /// either operator.
    pub fn run(&self, steps: usize) -> (Vec<MsaSelectorStepRecord>, MsaSelectorWitness) {
        let mut census = MsaSelectorOpCensus::default();
        let mut records = Vec::with_capacity(steps);
        let mut witness_steps = Vec::with_capacity(steps);
        for _ in 0..steps {
            let record = self.reference_step(&mut census);
            witness_steps.push(MsaSelectorWitnessStep {
                selected: record.selected.clone(),
                aggregate_raw: record.aggregate.raw(),
            });
            records.push(record);
        }
        let instance_digest = *blake3::hash(&self.instance_bytes).as_bytes();
        let witness = MsaSelectorWitness {
            format: MSA_SELECTOR_WITNESS_FORMAT.to_owned(),
            operator_id: uor_r4_model_source::attention::AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_ID
                .to_owned(),
            operator_version:
                uor_r4_model_source::attention::AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_VERSION,
            instance_digest: digest_string(&instance_digest),
            inputs_digest: digest_string(&msa_inputs_digest(&instance_digest, steps)),
            steps: witness_steps,
            census,
        };
        (records, witness)
    }
}

/// Independent witness replay: verify `witness` against the fixture
/// (`candidate_ids`, `contributions`, `top_m`, `steps`) WITHOUT running
/// the operator. Returns `None` when the witness verifies, or
/// `Some(reason)` naming the first observed mismatch — the
/// `Option<String>` shape `ReleaseBundleManifest::validate` and
/// `engine::validate_quality_report` use (R5: this shipped crate names
/// no custom error type for a check that is not itself the operator).
pub fn replay_msa_selector_witness(
    candidate_ids: &[u32],
    contributions: &[ScoreQ],
    top_m: u16,
    steps: usize,
    witness: &MsaSelectorWitness,
) -> Option<String> {
    if witness.format != MSA_SELECTOR_WITNESS_FORMAT {
        return Some(format!("witness format tag mismatch: {}", witness.format));
    }
    if witness.operator_id
        != uor_r4_model_source::attention::AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_ID
        || witness.operator_version
            != uor_r4_model_source::attention::AttentionOperatorSpec::MSA_STRUCTURED_SELECTOR_VERSION
    {
        return Some("witness names a different operator id or version".to_owned());
    }
    if candidate_ids.len() != contributions.len() {
        return Some("candidate_ids and contributions lengths differ".to_owned());
    }
    let instance_bytes = msa_instance_bytes(candidate_ids, contributions, top_m);
    let instance_digest = *blake3::hash(&instance_bytes).as_bytes();
    if witness.instance_digest != digest_string(&instance_digest) {
        return Some("recorded instance digest does not match the fixture".to_owned());
    }
    if witness.inputs_digest != digest_string(&msa_inputs_digest(&instance_digest, steps)) {
        return Some("recorded inputs digest does not bind this step count".to_owned());
    }
    if witness.steps.len() != steps {
        return Some(format!(
            "step count mismatch: witness {}, requested {steps}",
            witness.steps.len()
        ));
    }

    let candidate_count = candidate_ids.len() as u32;
    for (step_index, record) in witness.steps.iter().enumerate() {
        if record.selected.len() != top_m as usize {
            return Some(format!("step {step_index}: selection is not top_m wide"));
        }
        let mut previous: Option<(u8, u8, u32)> = None;
        for (slot, selection) in record.selected.iter().enumerate() {
            if selection.candidate >= candidate_count {
                return Some(format!(
                    "step {step_index} slot {slot}: candidate index out of range"
                ));
            }
            if selection.candidate_id != candidate_ids[selection.candidate as usize] {
                return Some(format!(
                    "step {step_index} slot {slot}: recorded candidate_id does not match the fixture"
                ));
            }
            let recomputed = classify(selection.candidate_id);
            if recomputed.role_rank != selection.role_rank
                || recomputed.cascade_position != selection.cascade_position
            {
                return Some(format!(
                    "step {step_index} slot {slot}: classification does not recompute"
                ));
            }
            let key = (
                selection.role_rank,
                selection.cascade_position,
                selection.candidate,
            );
            if let Some(previous_key) = previous {
                if key <= previous_key {
                    return Some(format!(
                        "step {step_index} slot {slot}: selection order violated"
                    ));
                }
            }
            previous = Some(key);
        }
        // Completeness by optimality: no unselected candidate may beat
        // the worst selected slot under (role_rank, cascade_position,
        // index).
        let Some(worst) = record.selected.last().map(|selection| {
            (
                selection.role_rank,
                selection.cascade_position,
                selection.candidate,
            )
        }) else {
            return Some(format!("step {step_index}: empty selection"));
        };
        for (index, &candidate_id) in candidate_ids.iter().enumerate() {
            let index = index as u32;
            let is_selected = record
                .selected
                .iter()
                .any(|selection| selection.candidate == index);
            if !is_selected {
                let classification = classify(candidate_id);
                let key = (
                    classification.role_rank,
                    classification.cascade_position,
                    index,
                );
                if key < worst {
                    return Some(format!(
                        "step {step_index}: unselected candidate index {index} beats the selection"
                    ));
                }
            }
        }
        // Aggregate refold over the RECORDED selection, selection order.
        let mut aggregate = ScoreQ::ZERO;
        for selection in &record.selected {
            aggregate = aggregate.saturating_add(contributions[selection.candidate as usize]);
        }
        if aggregate.raw() != record.aggregate_raw {
            return Some(format!("step {step_index}: aggregate does not refold"));
        }
    }

    if witness.census != expected_msa_selector_census(candidate_count, top_m, steps) {
        return Some("census does not equal its closed form".to_owned());
    }
    None
}
