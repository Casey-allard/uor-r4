//! `R4RouteAttentionV1` reference semantics, witness, and independent
//! replay (#604): the versioned reference specification and scalar
//! reference implementation of the first genuinely R4-native TARGET
//! attention/relation operator — certifier-side code, next to the
//! Phase-4 reference scorer (`score_runtime`) where reference
//! scoring/routing semantics live.
//!
//! The operator is DORMANT (`r4-route-attention-dormant` in
//! `model/ledger.toml`): constructible and testable here and in the
//! packed lowering, referenced by no serving path, activating nothing.
//! Its registry record is `r4-route-attention/1` in
//! `uor-r4-model-source::attention` — a separate identity from the two
//! #602 SOURCE operators, which remain the teacher's semantics; the
//! `r4_attention` switch still selects only between those two and never
//! selects this operator.
//!
//! # Reference specification (version 1)
//!
//! An operator INSTANCE is the canonical byte object defined by
//! `uor-r4-graph-format::route_attention` (`RAT1` layout): a declared
//! relation mask, `N` candidate route codes with one declared ScoreQ
//! contribution each, and a declared selection width `M`, under the hard
//! caps `1 <= N <= 64` and `1 <= M <= min(8, N)`. Route codes are
//! 288-bit (36-byte) vectors — the deployed signature width
//! (`compiler::D = 288`, HEAD `signature_bytes`, ROUT prototype/mask
//! windows), reused rather than invented. No Q/K/V weight enters this
//! operator anywhere: codes, mask, and contributions are declared
//! tables.
//!
//! One STEP over a query route code `q`:
//!
//! 1. **Relation** (masked XOR+popcount): for every candidate `j` in
//!    index order, `d_j = Σ_{b<36} popcount((q[b] XOR c_j[b]) AND
//!    mask[b])`, accumulated by integer adds with one popcount-table
//!    read per byte. Bits outside the declared mask never enter any
//!    distance.
//! 2. **Selection** (bounded top-M): the `M` smallest candidates under
//!    the strict lexicographic order `(distance, index)`. Deterministic
//!    tie rule: on equal distance the LOWEST candidate index wins.
//!    The reference performs exactly `M` ordered slot comparisons per
//!    candidate, so the comparison count is data-independent.
//! 3. **Aggregation** (integer/table): the selected contributions fold
//!    from `ScoreQ::ZERO` in selection order (ascending
//!    `(distance, index)`) with SATURATING i32 adds. Saturating
//!    addition is not associative at the rails, so the fold order is
//!    part of the semantics and is pinned here.
//!
//! Every operation is integer/bitwise/table: XOR, AND (inside the masked
//! popcount), popcount-table reads, integer add/subtract, integer
//! compares, table reads. No float, no multiply, no divide, no modulo —
//! by construction in this module and the packed lowering
//! (`uor-r4-graph-runtime::route_attention`, which is P-4-scanned), and
//! machine-checked by the source-scan tests in
//! `tests/route_attention_604.rs`.
//!
//! # State bounds
//!
//! The packed lowering runs over a borrowed instance view and a
//! caller-owned `RouteState` whose selection slots are a fixed
//! `ROUTE_MAX_TOP_M` array stamped by a step epoch (the `StepState`
//! epoch discipline of the reference scorer): steady state allocates
//! nothing, and the state capacity bound holds by construction because
//! every validated instance's `M` fits the fixed capacity. The scalar
//! reference here may allocate (certifier-side; the deployed contract
//! explicitly excludes test-only reference implementations).
//!
//! # Witness and independent replay
//!
//! [`RouteAttentionReference::run`] emits a [`RouteAttentionWitness`]:
//! the operator identity, the instance digest (blake3 of the canonical
//! instance bytes), an inputs digest binding the instance digest and the
//! query sequence, per-step selected candidates with their distances in
//! selection order, per-step aggregate, and the op census.
//! [`replay_route_witness`] verifies a witness against the instance
//! bytes and queries WITHOUT running the operator: it recomputes each
//! recorded candidate's masked distance, checks selection order,
//! completeness-by-optimality (no unselected candidate beats the worst
//! selected slot under `(distance, index)`), refolds the aggregate over
//! the recorded selection, and checks the census against its closed
//! form ([`expected_route_census`]) — every per-step count is a closed
//! form of `(N, M)`, deliberately data-independent.
//!
//! # Census (per step, closed form)
//!
//! ```text
//! adds                = 36*N + M
//! xors                = 36*N
//! popcounts           = 36*N          (mask AND is part of this op)
//! compares            = M*N
//! table_reads         = 2*N + M
//! bytes_read          = 72*N + 4*M    (instance bytes only; the
//!                                      caller-owned query is an input)
//! candidates_examined = N
//! ```
//!
//! # Artifact/witness carriage (decision, #604)
//!
//! Operator instances are SEPARATE canonical serialized objects (the
//! `RAT1` bytes; witnesses are serde records serialized by the certify
//! harnesses, e.g. via ciborium like `Certificate::to_cbor`), loaded by
//! tests/certify only. Nothing is emitted into R4G1 artifacts: while the
//! operator is dormant no artifact consumer exists, so historical
//! artifacts and every existing fixture stay byte-identical without an
//! optional-section reader in the loop. If a later activation needs
//! in-artifact carriage, the `EDGE_KIND_OPTIONAL_BIT` / optional-section
//! conventions (`records.rs`, `SectionId::OPTIONAL_BIT`) are the
//! designated mechanism, and the instance layout already matches the
//! ROUT signature substrate width for that move.

use serde::{Deserialize, Serialize};

use uor_r4_graph_format::route_attention::{
    route_instance_digest, RouteAttentionView, RouteOpCensus, ROUTE_ATTENTION_OPERATOR_ID,
    ROUTE_ATTENTION_OPERATOR_VERSION, ROUTE_CODE_BYTES, ROUTE_POPCOUNT_TABLE,
};
use uor_r4_graph_format::{NotAProduct, ScoreQ};
use uor_r4_graph_runtime::route_attention::{route_attention_step, RouteState};

/// Format tag of the witness record.
pub const ROUTE_WITNESS_FORMAT: &str = "uor-r4-route-attention-witness/1";
/// Domain-separation prefix of the inputs digest.
pub const ROUTE_INPUTS_DOMAIN: &str = "uor-r4-route-attention-inputs/1";

/// One selected candidate of one step, in selection order.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteSelection {
    /// Candidate index into the instance's declared tables.
    #[serde(default)]
    pub candidate: u32,
    /// Masked XOR+popcount distance of that candidate to the query.
    #[serde(default)]
    pub distance: u32,
}

/// One step of the witness: the selected candidates (selection order —
/// ascending `(distance, candidate)`) and the aggregate they fold to.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteWitnessStep {
    /// Selected candidates with distances, selection order.
    #[serde(default)]
    pub selected: Vec<RouteSelection>,
    /// Raw ScoreQ of the selection-order saturating fold.
    #[serde(default)]
    pub aggregate_raw: i32,
}

/// The bounded, replayable record of one route-attention run: identity,
/// input binding, per-step selections and outputs, and the op census.
/// All fields serde-defaulted; a partial document parses with absent
/// fields defaulted (era discipline of the peer records).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteAttentionWitness {
    /// [`ROUTE_WITNESS_FORMAT`].
    #[serde(default)]
    pub format: String,
    /// Operator registry id (`r4-route-attention`).
    #[serde(default)]
    pub operator_id: String,
    /// Operator registry version (1).
    #[serde(default)]
    pub operator_version: u32,
    /// `blake3:<hex>` of the canonical instance bytes.
    #[serde(default)]
    pub instance_digest: String,
    /// `blake3:<hex>` binding the instance digest and the query
    /// sequence ([`route_inputs_digest`]).
    #[serde(default)]
    pub inputs_digest: String,
    /// Per-step selections and aggregates.
    #[serde(default)]
    pub steps: Vec<RouteWitnessStep>,
    /// Op census of the whole run.
    #[serde(default)]
    pub census: RouteOpCensus,
}

/// One step's outcome as returned to callers (identical content to the
/// witness step; the witness carries the run-level identity around it).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteStepRecord {
    /// Selected candidates with distances, selection order.
    pub selected: Vec<RouteSelection>,
    /// The selection-order saturating ScoreQ fold.
    pub aggregate: ScoreQ,
}

/// `blake3:<hex>` presentation of a raw digest.
pub fn digest_string(digest: &[u8; 32]) -> String {
    format!("blake3:{}", blake3::Hash::from_bytes(*digest).to_hex())
}

/// The inputs digest: blake3 over the domain tag, the instance digest,
/// the query count (u32 LE), and every query in order. Binds the
/// witness to exactly one (instance, query-sequence) pair.
pub fn route_inputs_digest(
    instance_digest: &[u8; 32],
    queries: &[[u8; ROUTE_CODE_BYTES]],
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(ROUTE_INPUTS_DOMAIN.as_bytes());
    hasher.update(b"\n");
    hasher.update(instance_digest);
    hasher.update(&(queries.len() as u32).to_le_bytes());
    for query in queries {
        hasher.update(query);
    }
    *hasher.finalize().as_bytes()
}

/// The census closed form for `steps` steps over an instance with
/// `candidate_count` candidates selecting `top_m` (module docs table).
/// Accumulated by loop adds — like both implementations, this module
/// carries no value multiplication anywhere, so the closed form is
/// literally the operation schedule replayed without the data.
pub fn expected_route_census(candidate_count: u32, top_m: u16, steps: usize) -> RouteOpCensus {
    let per_candidate_bytes = (ROUTE_CODE_BYTES as u64) + (ROUTE_CODE_BYTES as u64);
    let mut census = RouteOpCensus::default();
    let mut step = 0usize;
    while step < steps {
        let mut candidate = 0u32;
        while candidate < candidate_count {
            census.candidates_examined += 1;
            census.table_reads += 2;
            census.bytes_read += per_candidate_bytes;
            census.xors += ROUTE_CODE_BYTES as u64;
            census.popcounts += ROUTE_CODE_BYTES as u64;
            census.adds += ROUTE_CODE_BYTES as u64;
            census.compares += u64::from(top_m);
            candidate += 1;
        }
        let mut slot = 0u16;
        while slot < top_m {
            census.table_reads += 1;
            census.bytes_read += 4;
            census.adds += 1;
            slot += 1;
        }
        step += 1;
    }
    census
}

/// The scalar reference implementation: owned copies of the validated
/// instance's declared tables (fail-closed construction — invalid bytes
/// never yield a reference), plus the canonical bytes for identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteAttentionReference {
    bytes: Vec<u8>,
    mask: [u8; ROUTE_CODE_BYTES],
    codes: Vec<[u8; ROUTE_CODE_BYTES]>,
    contributions: Vec<ScoreQ>,
    top_m: usize,
}

impl RouteAttentionReference {
    /// Build the reference from canonical instance bytes. Every shape
    /// or bound violation is refused by
    /// [`RouteAttentionView::parse`] on the sanctioned surface.
    pub fn from_instance_bytes(bytes: &[u8]) -> Result<Self, NotAProduct> {
        let view = RouteAttentionView::parse(bytes)?;
        let mut mask = [0u8; ROUTE_CODE_BYTES];
        mask.copy_from_slice(view.mask());
        let mut codes = Vec::with_capacity(view.candidate_count() as usize);
        for window in view.codes().chunks_exact(ROUTE_CODE_BYTES) {
            let mut code = [0u8; ROUTE_CODE_BYTES];
            code.copy_from_slice(window);
            codes.push(code);
        }
        let mut contributions = Vec::with_capacity(codes.len());
        for window in view.contributions().chunks_exact(4) {
            contributions.push(ScoreQ::from_raw(i32::from_le_bytes([
                window[0], window[1], window[2], window[3],
            ])));
        }
        Ok(Self {
            bytes: bytes.to_vec(),
            mask,
            codes,
            contributions,
            top_m: usize::from(view.top_m()),
        })
    }

    /// The canonical instance bytes this reference was built from.
    pub fn instance_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Declared candidate count `N`.
    pub fn candidate_count(&self) -> u32 {
        self.codes.len() as u32
    }

    /// Declared selection width `M`.
    pub fn top_m(&self) -> u16 {
        self.top_m as u16
    }

    /// One reference step (module-docs semantics), counting into
    /// `census` with exactly the declared operation schedule.
    pub fn reference_step(
        &self,
        query: &[u8; ROUTE_CODE_BYTES],
        census: &mut RouteOpCensus,
    ) -> RouteStepRecord {
        // Selection slots, ascending (distance, candidate); sentinel
        // pairs order after every real candidate.
        let mut slots: Vec<(u32, u32)> = Vec::with_capacity(self.top_m + 1);
        slots.resize(self.top_m, (u32::MAX, u32::MAX));
        for (index, code) in self.codes.iter().enumerate() {
            let index = index as u32;
            census.candidates_examined += 1;
            census.table_reads += 2;
            census.bytes_read += (ROUTE_CODE_BYTES as u64) + (ROUTE_CODE_BYTES as u64);
            let distance = masked_distance(query, code, &self.mask, census);
            // Exactly M ordered slot comparisons; the first slot this
            // candidate beats is its insertion point. Strict
            // lexicographic (distance, index) order makes equal
            // distance resolve to the lowest index by construction.
            let mut insert_at = None;
            for (slot, &(slot_distance, slot_candidate)) in slots.iter().enumerate() {
                census.compares += 1;
                if (distance, index) < (slot_distance, slot_candidate) && insert_at.is_none() {
                    insert_at = Some(slot);
                }
            }
            if let Some(position) = insert_at {
                slots.insert(position, (distance, index));
                slots.truncate(self.top_m);
            }
        }
        // Selection-order saturating fold of the selected contributions.
        let mut aggregate = ScoreQ::ZERO;
        let mut selected = Vec::with_capacity(self.top_m);
        for &(distance, candidate) in &slots {
            let contribution = self.contributions[candidate as usize];
            census.table_reads += 1;
            census.bytes_read += 4;
            aggregate = aggregate.saturating_add(contribution);
            census.adds += 1;
            selected.push(RouteSelection {
                candidate,
                distance,
            });
        }
        RouteStepRecord {
            selected,
            aggregate,
        }
    }

    /// Run the reference over a query sequence, producing the step
    /// records and the replayable witness.
    pub fn run(
        &self,
        queries: &[[u8; ROUTE_CODE_BYTES]],
    ) -> (Vec<RouteStepRecord>, RouteAttentionWitness) {
        let mut census = RouteOpCensus::default();
        let mut records = Vec::with_capacity(queries.len());
        let mut steps = Vec::with_capacity(queries.len());
        for query in queries {
            let record = self.reference_step(query, &mut census);
            steps.push(RouteWitnessStep {
                selected: record.selected.clone(),
                aggregate_raw: record.aggregate.raw(),
            });
            records.push(record);
        }
        let instance_digest = route_instance_digest(&self.bytes);
        let witness = RouteAttentionWitness {
            format: ROUTE_WITNESS_FORMAT.to_owned(),
            operator_id: ROUTE_ATTENTION_OPERATOR_ID.to_owned(),
            operator_version: ROUTE_ATTENTION_OPERATOR_VERSION,
            instance_digest: digest_string(&instance_digest),
            inputs_digest: digest_string(&route_inputs_digest(&instance_digest, queries)),
            steps,
            census,
        };
        (records, witness)
    }
}

/// Masked XOR+popcount distance, counted per byte (relation step of the
/// reference specification).
fn masked_distance(
    query: &[u8; ROUTE_CODE_BYTES],
    code: &[u8; ROUTE_CODE_BYTES],
    mask: &[u8; ROUTE_CODE_BYTES],
    census: &mut RouteOpCensus,
) -> u32 {
    let mut distance = 0u32;
    for ((&query_byte, &code_byte), &mask_byte) in query.iter().zip(code.iter()).zip(mask.iter()) {
        let xored = query_byte ^ code_byte;
        census.xors += 1;
        let masked = xored & mask_byte;
        let ones = ROUTE_POPCOUNT_TABLE[masked as usize];
        census.popcounts += 1;
        distance += u32::from(ones);
        census.adds += 1;
    }
    distance
}

/// Drive the PACKED lowering (`uor-r4-graph-runtime::route_attention`)
/// over the same inputs the reference takes, producing step records and
/// a witness of identical shape — the differential tests require the
/// two to agree bit-for-bit on selections, aggregates, census, and the
/// whole witness. Borrowed bytes in, caller-owned bounded state, no
/// allocation inside the per-step kernel (the record assembly here is
/// harness-side).
pub fn run_packed(
    instance_bytes: &[u8],
    queries: &[[u8; ROUTE_CODE_BYTES]],
) -> Result<(Vec<RouteStepRecord>, RouteAttentionWitness), NotAProduct> {
    let view = RouteAttentionView::parse(instance_bytes)?;
    let mut state = RouteState::new();
    let mut census = RouteOpCensus::default();
    let mut records = Vec::with_capacity(queries.len());
    let mut steps = Vec::with_capacity(queries.len());
    for query in queries {
        let aggregate = route_attention_step(&view, query, &mut state, &mut census)?;
        let mut selected = Vec::with_capacity(state.selected_len());
        let mut slot = 0usize;
        while let Some((candidate, distance)) = state.selected(slot) {
            selected.push(RouteSelection {
                candidate,
                distance,
            });
            slot += 1;
        }
        steps.push(RouteWitnessStep {
            selected: selected.clone(),
            aggregate_raw: aggregate.raw(),
        });
        records.push(RouteStepRecord {
            selected,
            aggregate,
        });
    }
    let instance_digest = route_instance_digest(instance_bytes);
    let witness = RouteAttentionWitness {
        format: ROUTE_WITNESS_FORMAT.to_owned(),
        operator_id: ROUTE_ATTENTION_OPERATOR_ID.to_owned(),
        operator_version: ROUTE_ATTENTION_OPERATOR_VERSION,
        instance_digest: digest_string(&instance_digest),
        inputs_digest: digest_string(&route_inputs_digest(&instance_digest, queries)),
        steps,
        census,
    };
    Ok((records, witness))
}

/// Why a witness failed independent replay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouteReplayError {
    /// The instance bytes are not a route-attention instance.
    Instance(NotAProduct),
    /// The witness format tag is not [`ROUTE_WITNESS_FORMAT`].
    FormatTagMismatch,
    /// The witness names a different operator id/version.
    OperatorMismatch,
    /// The recorded instance digest does not match the instance bytes.
    InstanceDigestMismatch,
    /// The recorded inputs digest does not bind these queries.
    InputsDigestMismatch,
    /// Step count differs from the query count.
    StepCountMismatch {
        /// Steps recorded in the witness.
        declared: usize,
        /// Queries supplied to the replayer.
        actual: usize,
    },
    /// A step's selection is not exactly `top_m` wide.
    SelectionWidthMismatch {
        /// Offending step.
        step: usize,
    },
    /// A recorded candidate index is outside the instance's tables.
    CandidateOutOfRange {
        /// Offending step.
        step: usize,
        /// Offending selection slot.
        slot: usize,
    },
    /// A recorded distance does not recompute from the fixture.
    DistanceMismatch {
        /// Offending step.
        step: usize,
        /// Offending selection slot.
        slot: usize,
    },
    /// Recorded selections are not in strict selection order
    /// (ascending `(distance, candidate)` — this also rejects
    /// duplicates).
    SelectionOrderViolation {
        /// Offending step.
        step: usize,
        /// Offending selection slot (the later of the pair).
        slot: usize,
    },
    /// An unselected candidate beats the worst selected slot — the
    /// recorded set is not the top-M under `(distance, index)`.
    SelectionNotOptimal {
        /// Offending step.
        step: usize,
        /// The unselected candidate that should have been selected.
        candidate: u32,
    },
    /// The recorded aggregate does not refold from the recorded
    /// selection in selection order.
    AggregateMismatch {
        /// Offending step.
        step: usize,
    },
    /// The recorded census does not equal the closed form.
    CensusMismatch,
}

impl std::fmt::Display for RouteReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteReplayError::Instance(e) => write!(f, "instance validation failed: {e}"),
            RouteReplayError::FormatTagMismatch => write!(f, "witness format tag mismatch"),
            RouteReplayError::OperatorMismatch => write!(f, "operator id or version mismatch"),
            RouteReplayError::InstanceDigestMismatch => write!(f, "instance digest mismatch"),
            RouteReplayError::InputsDigestMismatch => write!(f, "inputs digest mismatch"),
            RouteReplayError::StepCountMismatch { declared, actual } => {
                write!(
                    f,
                    "step count mismatch: witness {declared}, queries {actual}"
                )
            }
            RouteReplayError::SelectionWidthMismatch { step } => {
                write!(f, "step {step}: selection is not top_m wide")
            }
            RouteReplayError::CandidateOutOfRange { step, slot } => {
                write!(f, "step {step} slot {slot}: candidate out of range")
            }
            RouteReplayError::DistanceMismatch { step, slot } => {
                write!(f, "step {step} slot {slot}: distance does not recompute")
            }
            RouteReplayError::SelectionOrderViolation { step, slot } => {
                write!(f, "step {step} slot {slot}: selection order violated")
            }
            RouteReplayError::SelectionNotOptimal { step, candidate } => {
                write!(
                    f,
                    "step {step}: unselected candidate {candidate} beats the selection"
                )
            }
            RouteReplayError::AggregateMismatch { step } => {
                write!(f, "step {step}: aggregate does not refold")
            }
            RouteReplayError::CensusMismatch => write!(f, "census does not equal its closed form"),
        }
    }
}

impl std::error::Error for RouteReplayError {}

/// Independent witness replay: verify `witness` against the fixture
/// (`instance_bytes`, `queries`) WITHOUT running the operator. The
/// checks recompute only what the witness claims — recorded distances,
/// order, optimality of the recorded set, the aggregate refold over the
/// recorded selection, the input binding, and the closed-form census —
/// never the operator's own selection walk.
///
/// Total verdict in the `verify_witness_replay` shape (score_runtime
/// precedent): `None` means the witness verified; `Some(error)` names
/// the first observed mismatch.
pub fn replay_route_witness(
    instance_bytes: &[u8],
    queries: &[[u8; ROUTE_CODE_BYTES]],
    witness: &RouteAttentionWitness,
) -> Option<RouteReplayError> {
    if witness.format != ROUTE_WITNESS_FORMAT {
        return Some(RouteReplayError::FormatTagMismatch);
    }
    if witness.operator_id != ROUTE_ATTENTION_OPERATOR_ID
        || witness.operator_version != ROUTE_ATTENTION_OPERATOR_VERSION
    {
        return Some(RouteReplayError::OperatorMismatch);
    }
    let view = match RouteAttentionView::parse(instance_bytes) {
        Ok(view) => view,
        Err(error) => return Some(RouteReplayError::Instance(error)),
    };
    let instance_digest = route_instance_digest(instance_bytes);
    if witness.instance_digest != digest_string(&instance_digest) {
        return Some(RouteReplayError::InstanceDigestMismatch);
    }
    if witness.inputs_digest != digest_string(&route_inputs_digest(&instance_digest, queries)) {
        return Some(RouteReplayError::InputsDigestMismatch);
    }
    if witness.steps.len() != queries.len() {
        return Some(RouteReplayError::StepCountMismatch {
            declared: witness.steps.len(),
            actual: queries.len(),
        });
    }

    let candidate_count = view.candidate_count();
    let top_m = usize::from(view.top_m());
    let mut mask = [0u8; ROUTE_CODE_BYTES];
    mask.copy_from_slice(view.mask());
    let mut verify_census = RouteOpCensus::default();

    for (step, (record, query)) in witness.steps.iter().zip(queries.iter()).enumerate() {
        if record.selected.len() != top_m {
            return Some(RouteReplayError::SelectionWidthMismatch { step });
        }
        // Recorded distances are truthful and the order is strict.
        let mut previous: Option<(u32, u32)> = None;
        for (slot, selection) in record.selected.iter().enumerate() {
            if selection.candidate >= candidate_count {
                return Some(RouteReplayError::CandidateOutOfRange { step, slot });
            }
            let Some(code) = view.candidate_code(selection.candidate) else {
                return Some(RouteReplayError::CandidateOutOfRange { step, slot });
            };
            let mut code_bytes = [0u8; ROUTE_CODE_BYTES];
            code_bytes.copy_from_slice(code);
            let recomputed = masked_distance(query, &code_bytes, &mask, &mut verify_census);
            if recomputed != selection.distance {
                return Some(RouteReplayError::DistanceMismatch { step, slot });
            }
            let pair = (selection.distance, selection.candidate);
            if let Some(previous_pair) = previous {
                if pair <= previous_pair {
                    return Some(RouteReplayError::SelectionOrderViolation { step, slot });
                }
            }
            previous = Some(pair);
        }
        // Completeness by optimality: no unselected candidate may beat
        // the worst selected slot under (distance, index). Strict order
        // above made the recorded candidates distinct, so a set of
        // top_m truthful, ordered, unbeaten entries IS the top-M.
        let Some(worst) = record
            .selected
            .last()
            .map(|selection| (selection.distance, selection.candidate))
        else {
            return Some(RouteReplayError::SelectionWidthMismatch { step });
        };
        for (candidate, code) in (0_u32..).zip(view.codes().chunks_exact(ROUTE_CODE_BYTES)) {
            let is_selected = record
                .selected
                .iter()
                .any(|selection| selection.candidate == candidate);
            if !is_selected {
                let mut code_bytes = [0u8; ROUTE_CODE_BYTES];
                code_bytes.copy_from_slice(code);
                let distance = masked_distance(query, &code_bytes, &mask, &mut verify_census);
                if (distance, candidate) < worst {
                    return Some(RouteReplayError::SelectionNotOptimal { step, candidate });
                }
            }
        }
        // Aggregate refold over the RECORDED selection, selection order.
        let mut aggregate = ScoreQ::ZERO;
        for selection in &record.selected {
            let Some(contribution) = view.contribution(selection.candidate) else {
                return Some(RouteReplayError::CandidateOutOfRange { step, slot: 0 });
            };
            aggregate = aggregate.saturating_add(contribution);
        }
        if aggregate.raw() != record.aggregate_raw {
            return Some(RouteReplayError::AggregateMismatch { step });
        }
    }

    // The census is a closed form of (N, M, steps): verified without
    // any operator run.
    if witness.census != expected_route_census(candidate_count, view.top_m(), queries.len()) {
        return Some(RouteReplayError::CensusMismatch);
    }
    None
}

// The deterministic synthetic fixture used by the #604 differential,
// witness, and property tests lives in `tests/route_attention_604.rs`
// (its ramp arithmetic must stay out of this file: the source-scan test
// there asserts this module carries no value `*` `/` `%` and no float
// type, the same by-construction discipline as the packed lowering).
