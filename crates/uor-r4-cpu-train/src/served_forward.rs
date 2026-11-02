//! Exact served-forward contract and independent CPU audit.
//!
//! This module describes the supported SpiralCore/FBS execution path, compares
//! the independent CPU forward with `IntegerStackSession`, and reports
//! structural work. Candidate and byte counts are models of the artifact and
//! position, not hardware performance counters. Only the timing section is
//! measured.

use std::fmt;

use serde::Serialize;
use uor_r4_integer::stack::{
    read_hamming_uorlut01, read_memory_uorlut01, FbsMapImage, HammingImage, IntegerStackModel,
    MemoryImage, NativeStep,
};

use crate::{CpuFbsForward, CpuFbsHead, CpuFbsStep};

pub const CPU_SERVED_FORWARD_AUDIT_SCHEMA: &str = "uor-r4.cpu-served-forward-audit/1";
const PARITY_MISMATCH_DETAIL_LIMIT: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServedForwardAuditError {
    Invalid(String),
}

impl fmt::Display for ServedForwardAuditError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl std::error::Error for ServedForwardAuditError {}

type Result<T> = std::result::Result<T, ServedForwardAuditError>;

fn invalid(message: impl Into<String>) -> ServedForwardAuditError {
    ServedForwardAuditError::Invalid(message.into())
}

/// The exact execution contract for the supported integer/fixed-point FBS
/// forward. The contract is deliberately independent of any float training
/// implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExactFbsContract {
    pub schema: &'static str,
    pub substrate: &'static str,
    pub qualification: &'static str,
    pub representations: Vec<String>,
    pub operation_order: Vec<String>,
    pub rounding: Vec<String>,
    pub saturation: Vec<String>,
    pub ties: Vec<String>,
    pub reads: Vec<String>,
    pub memory: Vec<String>,
    pub readout: Vec<String>,
}

pub fn exact_fbs_contract() -> ExactFbsContract {
    ExactFbsContract {
        schema: "uor-r4.exact-fbs-contract/1",
        substrate: "integer-fixed-point-deterministic",
        qualification: "gpu-required-for-model-qualification",
        representations: vec![
            "token/writer table: i16 at Q12 (2^-12)".to_owned(),
            "residual stream x: i32 at 2^-16".to_owned(),
            "recurrence state: i64 at 2^-16; three previous drive rows: i32 at 2^-16".to_owned(),
            "normalized values: i64 at 2^-24; product activations: i16 at a dynamic exponent".to_owned(),
            "native readout query: i32 at Q12, clamped to [-32768,32767]".to_owned(),
            "route maps: one input 8-block atom, eight 3-bit signed-permutation coordinates per output block".to_owned(),
        ],
        operation_order: vec![
            "token id selects the shared token/writer row and converts Q12 to residual Q16".to_owned(),
            "for each layer: recurrence RMS norm, gain, rec_in/rec_gate gathers, bias/tap recurrence, transition action, Hamilton output gate and rec_out residual add".to_owned(),
            "same layer: MLP RMS norm, gain, mlp_gate/mlp_up gathers, Hamilton feed-forward, dynamic i16 quantize, mlp_down residual add".to_owned(),
            "a Hamming read runs after its declared layer depth; a read at the layer count runs after the final norm".to_owned(),
            "final RMS norm and gain, optional readout Hamming read, then optional exact opening/reply memory move".to_owned(),
            "convert the final residual to Q12 and run the artifact-declared nearest or native writer head".to_owned(),
            "on an opening exact-key miss with a point table, the point key may replace the head token".to_owned(),
        ],
        rounding: vec![
            "fixed shifts add half a unit before an arithmetic right shift; expansion shifts saturate i64".to_owned(),
            "the RMS mean division and reciprocal truncate, isqrt returns floor, and the normalized quotient rounds through the serving kernel's half-up shift".to_owned(),
            "Hamilton products round through the serving kernels' half-up integer shifts".to_owned(),
            "dynamic i16 activation quantization selects its exponent from the maximum magnitude and rounds each value once".to_owned(),
            "Hamming, memory and readout payload arithmetic use the integer kernels' signed half-away-from-zero shift".to_owned(),
            "Hamming head means divide the weighted payload sum by the number of nonzero-weight kept positions, then divide the per-head sums by the number of contributing heads; both integer quotients truncate toward zero".to_owned(),
        ],
        saturation: vec![
            "Q12 readout coordinates saturate at [-32768,32767]".to_owned(),
            "i16 product activations clamp at [-32767,32767] after exponent selection".to_owned(),
            "integer kernel additions/shifts clamp or wrap exactly as the serving kernel implementation specifies; the CPU oracle mirrors that order".to_owned(),
            "route gathers convert to residual exponent with the serving rounded shift; residual additions saturate i32".to_owned(),
            "memory moves round the weighted interpolation once and clamp only at the i64 boundary".to_owned(),
        ],
        ties: vec![
            "Hamming candidates order by (distance, earlier position) and keep the first K".to_owned(),
            "whole-table nearest and kept-cell token scans choose the smaller token id on an exact score tie".to_owned(),
            "native writer cells choose the smaller cell id on an exact score tie".to_owned(),
            "point-key targets choose the commonest target, then the smaller target id, on the least L1 distance".to_owned(),
            "icosian transitions and gates choose the first root minimizing H4 distance, as the shared serving selector does".to_owned(),
        ],
        reads: vec![
            "copy heads key the writer-table points and rank prior positions by popcount XOR".to_owned(),
            "type heads use the read's query rotation to name a Hamming word and its value rotation to produce the candidate payload".to_owned(),
            "copy/type mixtures and per-channel weights are added at the depth recorded in the artifact".to_owned(),
            "at zero-based current row r, Hamming candidates are 1 <= s < r: s = r-1 carries the immediately preceding payload, position 0 has no predecessor, and current row r is excluded".to_owned(),
            "no dense attention, probability normalization or exponentiation is present".to_owned(),
        ],
        memory: vec![
            "opening and reply keys are exact token vectors looked up in sorted tables".to_owned(),
            "turn ids are cut from the current key-window block after the latest BOS/EOS boundary, capped by the artifact's k or reply-turn limit".to_owned(),
            "a reply key is the retained turn ids when that prompt row is in the block, then MEMORY_PAD, reply length, MEMORY_PAD and the current token".to_owned(),
            "MEMORY_PAD is a key separator and trailing pads are not part of the exact key".to_owned(),
            "a hit moves the final normalized point toward the named writer row by the table's per-channel Q24 weight".to_owned(),
            "an opening miss may scan the point table once; equal least distances are grouped by target and counted".to_owned(),
        ],
        readout: vec![
            "native vector: score 256 root-pair cells by Q12 L1, keep K with smaller-cell ties, then scan only those cells' writer rows with smaller-token ties".to_owned(),
            "native vector nearest: scan every writer row by Q12 L1 with smaller-token ties and keep no cells".to_owned(),
        ],
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GradientTreatment {
    Exact,
    StraightThrough,
    Secant,
    Unresolved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SurrogateOperation {
    pub operation: &'static str,
    pub exact_artifact_forward: &'static str,
    pub float_training_behavior: &'static str,
    pub required_gradient: GradientTreatment,
    pub rationale: &'static str,
    pub evidence: &'static str,
}

fn surrogate_inventory() -> Vec<SurrogateOperation> {
    vec![
        SurrogateOperation {
            operation: "export rounding of trunk, Hamming tables and memory weights",
            exact_artifact_forward: "operates only on already-quantized artifact values",
            float_training_behavior: "train-as-served rounds float variables in place before each training forward and restores the float masters afterward",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "the round is piecewise constant and has zero almost everywhere; the float master needs the served forward's upstream gradient",
            evidence: "crates/uor-r4-float-train/src/in_space_long.rs: ServedValues, round_if_served",
        },
        SurrogateOperation {
            operation: "icosian snap for recurrence transitions and Hamilton gates",
            exact_artifact_forward: "selects a discrete H4 root and applies its exact signed action",
            float_training_behavior: "float forward uses the snapped root while the backward carries the local S^3 tangent gradient",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "root identity is discrete; the pre-snap transport direction remains the trainable quantity",
            evidence: "crates/uor-r4-float-train/src/hamilton_sheet.rs and spiralcore_fbs.rs",
        },
        SurrogateOperation {
            operation: "raw quaternion normalization and Hamilton products",
            exact_artifact_forward: "integer square root/division and integer Hamilton multiplication",
            float_training_behavior: "ordinary differentiable float normalization and Hamilton product followed by export rounding",
            required_gradient: GradientTreatment::Exact,
            rationale: "the float shadow can differentiate the same rational expression before finite quantized evaluation",
            evidence: "crates/uor-r4-cpu-train/src/fbs.rs: raw_rotation, hamilton_scale; float train Hamilton kernels",
        },
        SurrogateOperation {
            operation: "dynamic i16 activation quantization",
            exact_artifact_forward: "chooses an exponent from maximum magnitude, rounds once and clamps",
            float_training_behavior: "float activations remain differentiable until export rounding",
            required_gradient: GradientTreatment::Unresolved,
            rationale: "the exponent and integer residue are discrete; train-as-served does not yet route its float training forward through this activation quantizer",
            evidence: "crates/uor-r4-cpu-train/src/fbs.rs: quantize16; float geometric stack",
        },
        SurrogateOperation {
            operation: "Hamming copy/type candidate ranking",
            exact_artifact_forward: "sorts exact popcount XOR distances and reads the kept positions",
            float_training_behavior: "sign codes are produced with a straight-through sign bridge and payloads retain a differentiable path",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "hard candidate identity has no useful dense derivative; code and payload parameters need local credit",
            evidence: "crates/uor-r4-float-train/src/flock_read_hamming.rs and flock_read_register.rs",
        },
        SurrogateOperation {
            operation: "exact opening/reply table key selection",
            exact_artifact_forward: "binary-searches token keys; only a hit applies a move",
            float_training_behavior: "the float read chooses its memory rows and moves through a differentiable weighted point operation",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "the discrete key identity is not differentiated; hit weights and move weights carry gradient",
            evidence: "crates/uor-r4-float-train/src/flock_read_memory.rs",
        },
        SurrogateOperation {
            operation: "point-key fallback on an opening miss",
            exact_artifact_forward: "scans exact Q12 L1 rows and returns the commonest least-distance target",
            float_training_behavior: "the float reference forms an L1-named opening through a detached decision path",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "the named token is discrete; query-table parameters require an explicit surrogate or table update",
            evidence: "crates/uor-r4-float-train/src/in_space_long.rs: l1_named and opening-key probes",
        },
        SurrogateOperation {
            operation: "native writer cell and token selection",
            exact_artifact_forward: "hard top-K cells and hard smallest-token tie break",
            float_training_behavior: "float cross-entropy and rank-margin objectives train token/address scores rather than the discrete selection",
            required_gradient: GradientTreatment::Secant,
            rationale: "hard top-K has no dense derivative; registered rank-margin and secant descent provide downstream credit through score margins",
            evidence: "crates/uor-r4-float-train/src/native_vector_writer.rs and secant_descent.rs",
        },
        SurrogateOperation {
            operation: "integer RMS reciprocal and exponent bookkeeping",
            exact_artifact_forward: "isqrt and reciprocal round to finite integer values",
            float_training_behavior: "float RMSNorm is differentiable; export changes the finite values",
            required_gradient: GradientTreatment::StraightThrough,
            rationale: "the algebraic float normalization is differentiable, while the served isqrt/reciprocal and exponent changes are finite and need an explicit straight-through boundary",
            evidence: "crates/uor-r4-float-train/src/geometric_stack.rs; integer rms kernels",
        },
        SurrogateOperation {
            operation: "turn-memory dropout during training",
            exact_artifact_forward: "absent: serving applies every exact memory hit",
            float_training_behavior: "a deterministic PRNG drops configured training windows so the trunk is also trained without memory",
            required_gradient: GradientTreatment::Unresolved,
            rationale: "the mask is discrete and training-only; no current contract states how a dropout-conditioned float state maps back to the served memory-on state",
            evidence: "crates/uor-r4-float-train/src/in_space_long.rs: memory_dropped and turn_memory_dropout",
        },
        SurrogateOperation {
            operation: "reply Hamming row cut during training",
            exact_artifact_forward: "absent: serving exposes all causal prior positions",
            float_training_behavior: "reply-row training limits the Hamming read to positions before the current reply or omits it before the reply",
            required_gradient: GradientTreatment::Unresolved,
            rationale: "the row-selection mask is discrete and has no served counterpart; whether this train-only conditioning should be retained or eliminated needs a registered comparison",
            evidence: "crates/uor-r4-float-train/src/in_space_long.rs: reply_read_train and set_trained_cut",
        },
        SurrogateOperation {
            operation: "gradient through a full artifact-only integer hidden trajectory",
            exact_artifact_forward: "no gradient exists in serving",
            float_training_behavior: "train-as-served still executes the ordinary float forward and rounds values before each step",
            required_gradient: GradientTreatment::Unresolved,
            rationale: "no registered trainer currently backpropagates through the exact integer route, snap, quantize and selection graph",
            evidence: "crates/uor-r4-float-train/src/in_space_long.rs: run_long and ServedValues",
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ArtifactIdentity {
    pub sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct FbsShapeManifest {
    pub vocab: usize,
    pub width: usize,
    pub mlp: usize,
    pub layers: usize,
    pub heads: usize,
    pub lanes: usize,
    pub context: usize,
    pub kept_cells: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FbsLayerManifest {
    pub index: usize,
    pub serialized_table_payload_bytes: u64,
    pub decoded_resident_table_bytes: u64,
    pub rec_gain_bytes: u64,
    pub mlp_gain_bytes: u64,
    pub rec_in_map_bytes: u64,
    pub rec_gate_map_bytes: u64,
    pub rec_out_map_bytes: u64,
    pub mlp_gate_map_bytes: u64,
    pub mlp_up_map_bytes: u64,
    pub mlp_down_map_bytes: u64,
    pub conv_taps_bytes: u64,
    pub conv_bias_bytes: u64,
    pub gate_bias_bytes: u64,
    pub decay_offset_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HammingReadManifest {
    pub depth: usize,
    pub typed_heads: usize,
    pub raw_table_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HammingMechanismManifest {
    pub heads: usize,
    pub copy_heads: usize,
    pub type_heads: usize,
    pub kept_positions: usize,
    pub rotation_exp: i32,
    pub margin_exp: i32,
    pub weight_exp: i32,
    pub reads: Vec<HammingReadManifest>,
    pub derived_copy_code_bytes: u64,
    pub type_payload_capacity_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MemoryMechanismManifest {
    pub k: usize,
    pub key_window: usize,
    pub opening_rows: usize,
    pub reply_rows: usize,
    pub reply_turn: Option<usize>,
    pub weight_exp: i32,
    pub serialized_table_bytes: u64,
    pub opening_table_bytes: u64,
    pub reply_table_bytes: u64,
    pub point_key: Option<PointKeyManifest>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PointKeyManifest {
    pub rows: usize,
    pub width: usize,
    pub point_exp: i32,
    pub serialized_table_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReadoutMechanismManifest {
    pub mode: &'static str,
    pub kept_cells: usize,
    pub roots: usize,
    pub writer_table_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServedForwardManifest {
    pub shape: FbsShapeManifest,
    pub projection: &'static str,
    pub recurrence: &'static str,
    pub recurrence_pattern: String,
    pub recurrence_gates: &'static str,
    pub transition_action: &'static str,
    pub feed_forward_gate: &'static str,
    pub feed_forward_action: &'static str,
    pub layers: Vec<FbsLayerManifest>,
    pub final_gain_bytes: u64,
    pub writer_table_bytes: u64,
    pub hamming: Option<HammingMechanismManifest>,
    pub memory: Option<MemoryMechanismManifest>,
    pub readout: ReadoutMechanismManifest,
    pub serialized_table_payload_bytes: u64,
    pub decoded_resident_table_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServedForwardStageCost {
    pub stage: String,
    pub modelled_stage_table_footprint_bytes: u64,
    pub route_output_coordinates: u64,
    pub norm_isqrt: u64,
    pub integer_divisions: u64,
    pub hamilton_products: u64,
    pub hamming_candidate_comparisons: u64,
    pub hamming_popcount_words: u64,
    pub memory_key_lookups: u64,
    pub point_table_rows_on_opening_miss: u64,
    pub readout_candidate_tokens: u64,
    pub readout_coordinate_comparisons: u64,
}

impl ServedForwardStageCost {
    fn empty(stage: String) -> Self {
        Self {
            stage,
            modelled_stage_table_footprint_bytes: 0,
            route_output_coordinates: 0,
            norm_isqrt: 0,
            integer_divisions: 0,
            hamilton_products: 0,
            hamming_candidate_comparisons: 0,
            hamming_popcount_words: 0,
            memory_key_lookups: 0,
            point_table_rows_on_opening_miss: 0,
            readout_candidate_tokens: 0,
            readout_coordinate_comparisons: 0,
        }
    }

    fn add(&mut self, other: &Self) {
        self.modelled_stage_table_footprint_bytes = self
            .modelled_stage_table_footprint_bytes
            .saturating_add(other.modelled_stage_table_footprint_bytes);
        self.route_output_coordinates = self
            .route_output_coordinates
            .saturating_add(other.route_output_coordinates);
        self.norm_isqrt = self.norm_isqrt.saturating_add(other.norm_isqrt);
        self.integer_divisions = self
            .integer_divisions
            .saturating_add(other.integer_divisions);
        self.hamilton_products = self
            .hamilton_products
            .saturating_add(other.hamilton_products);
        self.hamming_candidate_comparisons = self
            .hamming_candidate_comparisons
            .saturating_add(other.hamming_candidate_comparisons);
        self.hamming_popcount_words = self
            .hamming_popcount_words
            .saturating_add(other.hamming_popcount_words);
        self.memory_key_lookups = self
            .memory_key_lookups
            .saturating_add(other.memory_key_lookups);
        self.point_table_rows_on_opening_miss = self
            .point_table_rows_on_opening_miss
            .saturating_add(other.point_table_rows_on_opening_miss);
        self.readout_candidate_tokens = self
            .readout_candidate_tokens
            .saturating_add(other.readout_candidate_tokens);
        self.readout_coordinate_comparisons = self
            .readout_coordinate_comparisons
            .saturating_add(other.readout_coordinate_comparisons);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StageTiming {
    pub stage: String,
    pub total_ns: u64,
    pub mean_ns: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ServedForwardTiming {
    pub measurement: &'static str,
    pub cpu_model: String,
    pub logical_cpus: usize,
    pub timed_positions: usize,
    pub cpu_oracle_elapsed_ns: u64,
    pub cpu_oracle_tokens_per_second: f64,
    pub serving_elapsed_ns: u64,
    pub serving_tokens_per_second: f64,
    pub stages: Vec<StageTiming>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DecisionMismatch {
    pub position: usize,
    pub component: &'static str,
    pub coordinate: Option<usize>,
    pub cpu: i64,
    pub serving: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServedForwardParity {
    pub positions_checked: usize,
    pub query_coordinates_compared: u64,
    pub query_coordinate_mismatches: u64,
    pub token_decisions_compared: u64,
    pub token_mismatches: u64,
    pub kept_cells_compared: u64,
    pub kept_cell_mismatches: u64,
    pub mismatches_recorded: usize,
    pub mismatches_omitted: u64,
    pub mismatches: Vec<DecisionMismatch>,
    pub cpu_decision_sha256: String,
    pub serving_decision_sha256: String,
    pub instrument_fault: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServedForwardAuditConfig {
    pub tokens: Vec<u32>,
    /// One-based next row at which the reply's opening key applies.
    pub reply_start: Option<usize>,
    pub measure_timing: bool,
}

impl ServedForwardAuditConfig {
    pub fn new(tokens: Vec<u32>) -> Self {
        Self {
            tokens,
            reply_start: None,
            measure_timing: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ServedForwardAuditReport {
    pub schema: &'static str,
    pub substrate: &'static str,
    pub qualification: &'static str,
    pub artifact: ArtifactIdentity,
    pub token_sequence_sha256: String,
    pub head: &'static str,
    pub reply_start: Option<usize>,
    pub contract: ExactFbsContract,
    pub manifest: ServedForwardManifest,
    pub parity: ServedForwardParity,
    #[serde(skip)]
    pub modelled_position_cost: Vec<Vec<ServedForwardStageCost>>,
    pub modelled_sequence_cost: Vec<ServedForwardStageCost>,
    pub cost_basis: &'static str,
    pub surrogate_gradient_operations: Vec<SurrogateOperation>,
    pub timing: Option<ServedForwardTiming>,
}

/// Audit one artifact and token sequence. The tokenizer is intentionally not
/// involved: callers supply the artifact's token ids.
pub fn audit_served_forward(
    artifact: &[u8],
    config: &ServedForwardAuditConfig,
) -> Result<ServedForwardAuditReport> {
    if config.tokens.is_empty() {
        return Err(invalid("the audit requires at least one token"));
    }
    let model = IntegerStackModel::parse(artifact)
        .map_err(|error| invalid(format!("artifact does not parse: {error}")))?;
    let context = model.shape().context;
    if config.tokens.len() > context {
        return Err(invalid(format!(
            "token sequence has {} positions but the artifact context is {context}",
            config.tokens.len()
        )));
    }
    if let Some(&token) = config
        .tokens
        .iter()
        .find(|&&token| token as usize >= model.shape().vocab)
    {
        return Err(invalid(format!(
            "token {token} is outside vocabulary {}",
            model.shape().vocab
        )));
    }
    if config
        .reply_start
        .is_some_and(|reply_start| reply_start == 0 || reply_start > config.tokens.len())
    {
        return Err(invalid(
            "reply_start is one-based and must name a position in the token sequence",
        ));
    }

    let mut cpu = CpuFbsForward::from_artifact(artifact)
        .map_err(|error| invalid(format!("CPU FBS forward refusal: {error}")))?;
    let hamming = read_hamming_uorlut01(artifact)
        .map_err(|error| invalid(format!("Hamming tables do not parse: {error}")))?;
    let memory = read_memory_uorlut01(artifact)
        .map_err(|error| invalid(format!("memory tables do not parse: {error}")))?;
    let manifest = manifest_from_model(&model, hamming.as_ref(), memory.as_ref())?;
    let head = if model.shape().head_nearest() {
        CpuFbsHead::Nearest
    } else {
        CpuFbsHead::Writer
    };
    let mut serving = model.session();

    let mut parity = ParityAccumulator::default();
    let mut position_cost = Vec::with_capacity(config.tokens.len());
    for row in 0..config.tokens.len() {
        position_cost.push(cost_for_position(
            &manifest,
            &model,
            row,
            config.reply_start,
        ));
    }
    let sequence_cost = sum_costs(&position_cost);

    for (position, &token) in config.tokens.iter().enumerate() {
        if config.reply_start == Some(position + 1) {
            cpu.mark_reply_start();
            serving.mark_reply_start();
        }
        let actual = cpu
            .step(token, head)
            .map_err(|error| invalid(format!("CPU step {position} refused: {error}")))?;
        let expected = serving
            .step_native(token)
            .map_err(|error| invalid(format!("serving step {position} refused: {error}")))?;
        parity.compare(position, &actual, &expected);
    }

    let parity = parity.finish(
        config.tokens.len(),
        model.shape().width,
        manifest.shape.kept_cells,
    );
    let timing = if config.measure_timing {
        let mut timed_cpu = CpuFbsForward::from_artifact(artifact)
            .map_err(|error| invalid(format!("timed CPU FBS forward refusal: {error}")))?;
        let mut cpu_oracle_elapsed_ns = 0u64;
        for (position, &token) in config.tokens.iter().enumerate() {
            if config.reply_start == Some(position + 1) {
                timed_cpu.mark_reply_start();
            }
            let started = std::time::Instant::now();
            timed_cpu
                .step(token, head)
                .map_err(|error| invalid(format!("timed CPU step {position} refused: {error}")))?;
            cpu_oracle_elapsed_ns =
                cpu_oracle_elapsed_ns.saturating_add(started.elapsed().as_nanos() as u64);
        }

        let mut timed_serving = model.session();
        let mut timing_samples = Vec::with_capacity(config.tokens.len());
        let mut ns = Vec::new();
        for (position, &token) in config.tokens.iter().enumerate() {
            if config.reply_start == Some(position + 1) {
                timed_serving.mark_reply_start();
            }
            timed_serving
                .measure_fbs_step_parts(token, &mut ns)
                .map_err(|error| {
                    invalid(format!("timed serving step {position} refused: {error}"))
                })?;
            timing_samples.push(ns.clone());
        }
        Some(timing_report(
            model.shape().layers(),
            config.tokens.len(),
            cpu_oracle_elapsed_ns,
            &timing_samples,
        )?)
    } else {
        None
    };

    Ok(ServedForwardAuditReport {
        schema: CPU_SERVED_FORWARD_AUDIT_SCHEMA,
        substrate: "cpu-development-research",
        qualification: "gpu-required",
        artifact: ArtifactIdentity {
            sha256: model.artifact_sha256().to_owned(),
            bytes: artifact.len() as u64,
        },
        token_sequence_sha256: token_sequence_sha256(&config.tokens),
        head: match head {
            CpuFbsHead::Nearest => "native_vector_nearest",
            CpuFbsHead::Writer => "native_vector",
        },
        reply_start: config.reply_start,
        contract: exact_fbs_contract(),
        manifest,
        parity,
        modelled_position_cost: position_cost,
        modelled_sequence_cost: sequence_cost,
        cost_basis: "stage table bytes are semantic decoded-table footprints (shared tables may recur); serialized and decoded-resident totals are separate; candidates come from artifact dimensions and position; readout coordinates model the serving kernel's precomputed root scores; none are hardware cache or performance counters",
        surrogate_gradient_operations: surrogate_inventory(),
        timing,
    })
}

fn token_sequence_sha256(tokens: &[u32]) -> String {
    let mut bytes = Vec::with_capacity(tokens.len() * 4);
    for token in tokens {
        bytes.extend_from_slice(&token.to_le_bytes());
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn manifest_from_model(
    model: &IntegerStackModel,
    hamming: Option<&HammingImage>,
    memory: Option<&MemoryImage>,
) -> Result<ServedForwardManifest> {
    let shape = model.shape();
    let image = model
        .fbs_trunk_image()
        .ok_or_else(|| invalid("artifact does not expose an FBS trunk"))?;
    if image.layers.len() != shape.layers() {
        return Err(invalid(
            "FBS image layer count differs from the artifact shape",
        ));
    }
    let layers = image
        .layers
        .iter()
        .enumerate()
        .map(|(index, layer)| {
            let rec_gain_bytes = bytes(layer.rec_gain.len(), 2);
            let mlp_gain_bytes = bytes(layer.mlp_gain.len(), 2);
            let rec_in_map_bytes = map_bytes(&layer.rec_in);
            let rec_gate_map_bytes = map_bytes(&layer.rec_gate);
            let rec_out_map_bytes = map_bytes(&layer.rec_out);
            let mlp_gate_map_bytes = map_bytes(&layer.mlp_gate);
            let mlp_up_map_bytes = map_bytes(&layer.mlp_up);
            let mlp_down_map_bytes = map_bytes(&layer.mlp_down);
            let conv_taps_bytes = bytes(layer.taps.len(), 2);
            let conv_bias_bytes = bytes(layer.conv_bias.len(), 4);
            let gate_bias_bytes = bytes(layer.gate_bias.len(), 4);
            let decay_offset_bytes = bytes(layer.decay_offset.len(), 4);
            let serialized_route_bytes = [
                &layer.rec_in,
                &layer.rec_gate,
                &layer.rec_out,
                &layer.mlp_gate,
                &layer.mlp_up,
                &layer.mlp_down,
            ]
            .into_iter()
            .map(serialized_map_bytes)
            .sum::<u64>();
            let decoded_resident_table_bytes = rec_gain_bytes
                .saturating_add(mlp_gain_bytes)
                .saturating_add(rec_in_map_bytes)
                .saturating_add(rec_gate_map_bytes)
                .saturating_add(rec_out_map_bytes)
                .saturating_add(mlp_gate_map_bytes)
                .saturating_add(mlp_up_map_bytes)
                .saturating_add(mlp_down_map_bytes)
                .saturating_add(conv_taps_bytes)
                .saturating_add(conv_bias_bytes)
                .saturating_add(gate_bias_bytes)
                .saturating_add(decay_offset_bytes);
            FbsLayerManifest {
                index,
                serialized_table_payload_bytes: serialized_route_bytes
                    .saturating_add(rec_gain_bytes)
                    .saturating_add(mlp_gain_bytes)
                    .saturating_add(conv_taps_bytes)
                    .saturating_add(conv_bias_bytes)
                    .saturating_add(gate_bias_bytes)
                    .saturating_add(decay_offset_bytes),
                decoded_resident_table_bytes,
                rec_gain_bytes,
                mlp_gain_bytes,
                rec_in_map_bytes,
                rec_gate_map_bytes,
                rec_out_map_bytes,
                mlp_gate_map_bytes,
                mlp_up_map_bytes,
                mlp_down_map_bytes,
                conv_taps_bytes,
                conv_bias_bytes,
                gate_bias_bytes,
                decay_offset_bytes,
            }
        })
        .collect::<Vec<_>>();
    let decoded_layer_bytes = layers.iter().fold(0u64, |sum, layer| {
        sum.saturating_add(layer.rec_gain_bytes)
            .saturating_add(layer.mlp_gain_bytes)
            .saturating_add(layer.rec_in_map_bytes)
            .saturating_add(layer.rec_gate_map_bytes)
            .saturating_add(layer.rec_out_map_bytes)
            .saturating_add(layer.mlp_gate_map_bytes)
            .saturating_add(layer.mlp_up_map_bytes)
            .saturating_add(layer.mlp_down_map_bytes)
            .saturating_add(layer.conv_taps_bytes)
            .saturating_add(layer.conv_bias_bytes)
            .saturating_add(layer.gate_bias_bytes)
            .saturating_add(layer.decay_offset_bytes)
    });
    let serialized_layer_bytes = layers
        .iter()
        .map(|layer| layer.serialized_table_payload_bytes)
        .sum::<u64>();
    let writer_table_bytes = bytes(image.writer_table.len(), 2);
    let final_gain_bytes = bytes(image.final_gain.len(), 2);

    let hamming = hamming.map(|image| {
        let copy_heads = image.record.heads / 2;
        let type_heads = image.record.heads - copy_heads;
        let reads = image
            .reads
            .iter()
            .map(|read| HammingReadManifest {
                depth: read.depth,
                typed_heads: if read
                    .weight
                    .get(shape.width..)
                    .is_some_and(|weights| weights.iter().any(|&weight| weight != 0))
                {
                    type_heads
                } else {
                    0
                },
                raw_table_bytes: bytes(
                    read.query_rotation.len() + read.key_rotation.len() + read.value_rotation.len(),
                    4,
                )
                .saturating_add(bytes(read.margin.len(), 4))
                .saturating_add(bytes(read.weight.len(), 4)),
            })
            .collect::<Vec<_>>();
        let typed_reads = reads.iter().filter(|read| read.typed_heads != 0).count();
        HammingMechanismManifest {
            heads: image.record.heads,
            copy_heads,
            type_heads,
            kept_positions: image.record.kept,
            rotation_exp: image.record.rotation_exp,
            margin_exp: image.record.margin_exp,
            weight_exp: image.record.weight_exp,
            reads,
            derived_copy_code_bytes: bytes(
                shape
                    .vocab
                    .saturating_mul(copy_heads)
                    .saturating_mul(image.reads.len()),
                8,
            ),
            type_payload_capacity_bytes: bytes(
                typed_reads
                    .saturating_mul(shape.context)
                    .saturating_mul(shape.width),
                8,
            ),
        }
    });
    let memory = memory.map(|image| {
        let opening_bytes = table_image_bytes(&image.opening);
        let reply_bytes = image.reply.as_ref().map_or(0, table_image_bytes);
        let point_key = image.point.as_ref().map(|point| PointKeyManifest {
            rows: point.targets.len(),
            width: point.width,
            point_exp: -12,
            serialized_table_bytes: bytes(point.points.len(), 2)
                .saturating_add(bytes(point.targets.len(), 4)),
        });
        MemoryMechanismManifest {
            k: image.record.k,
            key_window: image.record.key_window,
            opening_rows: image.record.opening,
            reply_rows: image.record.reply.unwrap_or(0),
            reply_turn: image.record.reply_turn,
            weight_exp: image.record.weight_exp,
            opening_table_bytes: opening_bytes,
            reply_table_bytes: reply_bytes,
            serialized_table_bytes: opening_bytes.saturating_add(reply_bytes).saturating_add(
                point_key
                    .as_ref()
                    .map_or(0, |point| point.serialized_table_bytes),
            ),
            point_key,
        }
    });
    let readout = ReadoutMechanismManifest {
        mode: if shape.head_nearest() {
            "whole_table_nearest_l1"
        } else {
            "native_vector_cells_l1"
        },
        kept_cells: image.kept_cells,
        roots: image.roots_q12.len(),
        writer_table_bytes,
    };
    let serialized_hamming_bytes = hamming.as_ref().map_or(0, |hamming| {
        hamming
            .reads
            .iter()
            .map(|read| read.raw_table_bytes)
            .sum::<u64>()
    });
    let serialized_memory_bytes = memory
        .as_ref()
        .map_or(0, |memory| memory.serialized_table_bytes);
    let serialized_table_payload_bytes = serialized_layer_bytes
        .saturating_add(final_gain_bytes)
        .saturating_add(writer_table_bytes)
        .saturating_add(serialized_hamming_bytes)
        .saturating_add(serialized_memory_bytes);
    let decoded_resident_table_bytes = decoded_layer_bytes
        .saturating_add(final_gain_bytes)
        .saturating_add(writer_table_bytes)
        .saturating_add(serialized_hamming_bytes)
        .saturating_add(
            hamming
                .as_ref()
                .map_or(0, |hamming| hamming.derived_copy_code_bytes),
        )
        .saturating_add(serialized_memory_bytes);
    Ok(ServedForwardManifest {
        shape: FbsShapeManifest {
            vocab: shape.vocab,
            width: shape.width,
            mlp: shape.mlp,
            layers: shape.layers(),
            heads: shape.heads,
            lanes: shape.lanes(),
            context: shape.context,
            kept_cells: image.kept_cells,
        },
        projection: "spiralcore_fbs_signed_axis_permutation",
        recurrence: "recurrence_only_geometric_decay_rate",
        recurrence_pattern: shape.pattern.clone(),
        recurrence_gates: "geometric_hamilton_output_with_dyadic_rate",
        transition_action: if shape.raw_transition {
            "raw_unit_quaternion"
        } else {
            "icosian_snapped_h4"
        },
        feed_forward_gate: if shape.raw_gates {
            "raw_unit_quaternion_hamilton"
        } else {
            "icosian_snapped_hamilton"
        },
        feed_forward_action: "hamilton_gated_signed_axis_route",
        layers,
        final_gain_bytes,
        writer_table_bytes,
        hamming,
        memory,
        readout,
        serialized_table_payload_bytes,
        decoded_resident_table_bytes,
    })
}

fn bytes(count: usize, width: usize) -> u64 {
    (count as u64).saturating_mul(width as u64)
}

fn map_bytes(map: &FbsMapImage<'_>) -> u64 {
    bytes(map.atoms.len(), 2).saturating_add(bytes(map.codes.len(), 8))
}

fn serialized_map_bytes(map: &FbsMapImage<'_>) -> u64 {
    bytes(map.atoms.len(), 4)
}

fn table_image_bytes(table: &uor_r4_integer::stack::MemoryTableImage) -> u64 {
    let keys = table
        .keys
        .iter()
        .map(|key| bytes(key.len(), 4))
        .sum::<u64>();
    keys.saturating_add(bytes(table.values.len(), 4))
        .saturating_add(bytes(table.weight.len(), 4))
}

fn cost_for_position(
    manifest: &ServedForwardManifest,
    model: &IntegerStackModel,
    row: usize,
    reply_start: Option<usize>,
) -> Vec<ServedForwardStageCost> {
    let shape = manifest.shape;
    let gate_rows = shape.lanes + shape.width;
    let mut stages = Vec::new();
    let mut embedding = ServedForwardStageCost::empty("embedding".to_owned());
    embedding.modelled_stage_table_footprint_bytes = manifest.writer_table_bytes;
    stages.push(embedding);

    for layer in &manifest.layers {
        let mut norm =
            ServedForwardStageCost::empty(format!("layer_{}_recurrence_norm_gain", layer.index));
        norm.modelled_stage_table_footprint_bytes = layer.rec_gain_bytes;
        norm.norm_isqrt = 1;
        stages.push(norm);

        let mut rec_gathers =
            ServedForwardStageCost::empty(format!("layer_{}_recurrence_gathers", layer.index));
        rec_gathers.modelled_stage_table_footprint_bytes = layer
            .rec_in_map_bytes
            .saturating_add(layer.rec_gate_map_bytes);
        rec_gathers.route_output_coordinates = (2 * shape.width + gate_rows) as u64;
        stages.push(rec_gathers);

        let mut core =
            ServedForwardStageCost::empty(format!("layer_{}_recurrence_core", layer.index));
        core.modelled_stage_table_footprint_bytes = layer
            .conv_taps_bytes
            .saturating_add(layer.conv_bias_bytes)
            .saturating_add(layer.gate_bias_bytes)
            .saturating_add(layer.decay_offset_bytes);
        core.hamilton_products = (2 * shape.lanes) as u64;
        core.integer_divisions = if manifest.transition_action == "raw_unit_quaternion" {
            shape.lanes as u64
        } else {
            0
        };
        // The recurrence's output gate is raw or snapped by raw_gates. Raw gates divide per lane;
        // the artifact also decides the transition independently.
        if model.shape().raw_gates {
            core.integer_divisions = core.integer_divisions.saturating_add(shape.lanes as u64);
        }
        stages.push(core);

        let mut rec_out =
            ServedForwardStageCost::empty(format!("layer_{}_rec_out_residual", layer.index));
        rec_out.modelled_stage_table_footprint_bytes = layer.rec_out_map_bytes;
        rec_out.route_output_coordinates = shape.width as u64;
        stages.push(rec_out);

        let mut norm =
            ServedForwardStageCost::empty(format!("layer_{}_mlp_norm_gain", layer.index));
        norm.modelled_stage_table_footprint_bytes = layer.mlp_gain_bytes;
        norm.norm_isqrt = 1;
        stages.push(norm);

        let mut gathers =
            ServedForwardStageCost::empty(format!("layer_{}_mlp_gathers", layer.index));
        gathers.modelled_stage_table_footprint_bytes = layer
            .mlp_gate_map_bytes
            .saturating_add(layer.mlp_up_map_bytes);
        gathers.route_output_coordinates = (2 * shape.mlp) as u64;
        stages.push(gathers);

        let mut product =
            ServedForwardStageCost::empty(format!("layer_{}_hamilton_product", layer.index));
        product.hamilton_products = (shape.mlp / 4) as u64;
        if model.shape().raw_gates {
            product.integer_divisions = (shape.mlp / 4) as u64;
        }
        stages.push(product);

        stages.push(ServedForwardStageCost::empty(format!(
            "layer_{}_activation_quantize",
            layer.index
        )));

        let mut down =
            ServedForwardStageCost::empty(format!("layer_{}_mlp_down_residual", layer.index));
        down.modelled_stage_table_footprint_bytes = layer.mlp_down_map_bytes;
        down.route_output_coordinates = shape.width as u64;
        stages.push(down);

        if layer.index + 1 < shape.layers {
            if let Some(hamming) = &manifest.hamming {
                if let Some(read) = hamming
                    .reads
                    .iter()
                    .find(|read| read.depth == layer.index + 1)
                {
                    stages.push(hamming_cost(
                        format!("layer_{}_hamming_stream", layer.index),
                        manifest,
                        read,
                        row,
                    ));
                }
            }
        }
    }

    let mut final_norm = ServedForwardStageCost::empty("final_norm_query".to_owned());
    final_norm.modelled_stage_table_footprint_bytes = manifest.final_gain_bytes;
    final_norm.norm_isqrt = 1;
    stages.push(final_norm);

    if let Some(hamming) = &manifest.hamming {
        if let Some(read) = hamming.reads.iter().find(|read| read.depth == shape.layers) {
            stages.push(hamming_cost(
                "hamming_readout".to_owned(),
                manifest,
                read,
                row,
            ));
        }
    }

    if let Some(memory) = &manifest.memory {
        let mut lookup = ServedForwardStageCost::empty("turn_memory".to_owned());
        if let Some(reply_start) = reply_start {
            let step = row + 1;
            if step == reply_start {
                lookup.memory_key_lookups = 1;
                lookup.modelled_stage_table_footprint_bytes =
                    memory.opening_table_bytes.saturating_add(
                        memory
                            .point_key
                            .as_ref()
                            .map_or(0, |key| key.serialized_table_bytes),
                    );
                lookup.point_table_rows_on_opening_miss =
                    memory.point_key.as_ref().map_or(0, |key| key.rows as u64);
            } else if step > reply_start && memory.reply_rows != 0 {
                lookup.memory_key_lookups = 1;
                lookup.modelled_stage_table_footprint_bytes = memory.reply_table_bytes;
            }
        }
        stages.push(lookup);
    }

    stages.push(ServedForwardStageCost::empty("query_q12".to_owned()));
    let mut readout = ServedForwardStageCost::empty("readout".to_owned());
    readout.modelled_stage_table_footprint_bytes = manifest.writer_table_bytes;
    match manifest.readout.mode {
        "whole_table_nearest_l1" => {
            readout.readout_candidate_tokens = shape.vocab as u64;
            readout.readout_coordinate_comparisons = bytes(shape.vocab, shape.width);
        }
        _ => {
            readout.readout_coordinate_comparisons = (manifest.readout.roots * 16) as u64;
            let mut candidates = 0u64;
            for cell in 0..manifest.readout.kept_cells {
                let rows = if shape.vocab > cell {
                    ((shape.vocab - 1 - cell) / 256 + 1) as u64
                } else {
                    0
                };
                candidates = candidates.saturating_add(rows);
            }
            readout.readout_candidate_tokens = candidates;
            readout.readout_coordinate_comparisons = readout
                .readout_coordinate_comparisons
                .saturating_add(candidates.saturating_mul(shape.width as u64));
        }
    }
    stages.push(readout);
    stages
}

fn hamming_cost(
    stage: String,
    manifest: &ServedForwardManifest,
    read: &HammingReadManifest,
    row: usize,
) -> ServedForwardStageCost {
    let hamming = manifest
        .hamming
        .as_ref()
        .expect("a Hamming read has its manifest");
    let candidates = (hamming
        .copy_heads
        .saturating_add(read.typed_heads)
        .saturating_mul(row.saturating_sub(1))) as u64;
    let mut cost = ServedForwardStageCost::empty(stage);
    cost.modelled_stage_table_footprint_bytes = read.raw_table_bytes;
    cost.hamming_candidate_comparisons = candidates;
    cost.hamming_popcount_words = candidates;
    cost.hamilton_products = if read.typed_heads == 0 {
        0
    } else {
        ((manifest.shape.width / hamming.heads / 4) * read.typed_heads + manifest.shape.width / 4)
            as u64
    };
    cost
}

fn sum_costs(rows: &[Vec<ServedForwardStageCost>]) -> Vec<ServedForwardStageCost> {
    let mut totals: Vec<ServedForwardStageCost> = Vec::new();
    for row in rows {
        for cost in row {
            if let Some(total) = totals.iter_mut().find(|total| total.stage == cost.stage) {
                total.add(cost);
            } else {
                totals.push(cost.clone());
            }
        }
    }
    totals
}

#[derive(Default)]
struct ParityAccumulator {
    query_coordinates_compared: u64,
    query_coordinate_mismatches: u64,
    token_decisions_compared: u64,
    token_mismatches: u64,
    kept_cells_compared: u64,
    kept_cell_mismatches: u64,
    mismatches: Vec<DecisionMismatch>,
    omitted: u64,
    cpu_hasher: blake3::Hasher,
    serving_hasher: blake3::Hasher,
}

impl ParityAccumulator {
    fn record_mismatch(
        &mut self,
        position: usize,
        component: &'static str,
        coordinate: Option<usize>,
        cpu: i64,
        serving: i64,
    ) {
        if self.mismatches.len() < PARITY_MISMATCH_DETAIL_LIMIT {
            self.mismatches.push(DecisionMismatch {
                position,
                component,
                coordinate,
                cpu,
                serving,
            });
        } else {
            self.omitted = self.omitted.saturating_add(1);
        }
    }

    fn compare(&mut self, position: usize, cpu: &CpuFbsStep, serving: &NativeStep) {
        self.cpu_hasher
            .update(&(cpu.query_q12.len() as u64).to_le_bytes());
        self.serving_hasher
            .update(&(serving.query_q12.len() as u64).to_le_bytes());
        for coordinate in 0..cpu.query_q12.len().max(serving.query_q12.len()) {
            let cpu_value = cpu.query_q12.get(coordinate).copied().unwrap_or(i32::MIN);
            let serving_value = serving
                .query_q12
                .get(coordinate)
                .copied()
                .unwrap_or(i32::MIN);
            self.cpu_hasher.update(&cpu_value.to_le_bytes());
            self.serving_hasher.update(&serving_value.to_le_bytes());
            if cpu.query_q12.get(coordinate) != serving.query_q12.get(coordinate) {
                self.query_coordinate_mismatches += 1;
                self.record_mismatch(
                    position,
                    "query_q12",
                    Some(coordinate),
                    i64::from(cpu_value),
                    i64::from(serving_value),
                );
            }
        }
        self.query_coordinates_compared += cpu.query_q12.len() as u64;

        self.token_decisions_compared += 1;
        self.cpu_hasher.update(&cpu.token.to_le_bytes());
        self.serving_hasher.update(&serving.token.to_le_bytes());
        if cpu.token != serving.token {
            self.token_mismatches += 1;
            self.record_mismatch(
                position,
                "token",
                None,
                i64::from(cpu.token),
                i64::from(serving.token),
            );
        }

        for cell in 0..cpu.kept_cells.len().max(serving.kept_cells.len()) {
            let cpu_value = cpu.kept_cells.get(cell).copied().unwrap_or(u32::MAX);
            let serving_value = serving.kept_cells.get(cell).copied().unwrap_or(u32::MAX);
            self.cpu_hasher.update(&cpu_value.to_le_bytes());
            self.serving_hasher.update(&serving_value.to_le_bytes());
            if cpu_value != serving_value {
                self.kept_cell_mismatches += 1;
                self.record_mismatch(
                    position,
                    "kept_cell",
                    Some(cell),
                    i64::from(cpu_value),
                    i64::from(serving_value),
                );
            }
        }
        self.kept_cells_compared += cpu.kept_cells.len() as u64;
    }

    fn finish(self, positions: usize, width: usize, kept_cells: usize) -> ServedForwardParity {
        let instrument_fault = self.query_coordinate_mismatches != 0
            || self.token_mismatches != 0
            || self.kept_cell_mismatches != 0;
        debug_assert_eq!(self.query_coordinates_compared, (positions * width) as u64);
        debug_assert_eq!(self.kept_cells_compared, (positions * kept_cells) as u64);
        ServedForwardParity {
            positions_checked: positions,
            query_coordinates_compared: self.query_coordinates_compared,
            query_coordinate_mismatches: self.query_coordinate_mismatches,
            token_decisions_compared: self.token_decisions_compared,
            token_mismatches: self.token_mismatches,
            kept_cells_compared: self.kept_cells_compared,
            kept_cell_mismatches: self.kept_cell_mismatches,
            mismatches_recorded: self.mismatches.len(),
            mismatches_omitted: self.omitted,
            mismatches: self.mismatches,
            cpu_decision_sha256: self.cpu_hasher.finalize().to_hex().to_string(),
            serving_decision_sha256: self.serving_hasher.finalize().to_hex().to_string(),
            instrument_fault,
        }
    }
}

fn timing_report(
    layers: usize,
    positions: usize,
    cpu_oracle_elapsed_ns: u64,
    samples: &[Vec<u64>],
) -> Result<ServedForwardTiming> {
    let names = timing_stage_names(layers);
    let expected = names.len() + 2;
    if samples.iter().any(|sample| sample.len() != expected) {
        return Err(invalid(format!(
            "serving step timing returned an unexpected stage count; expected {expected}"
        )));
    }
    let mut totals = vec![0u64; names.len()];
    let mut serving_elapsed_ns = 0u64;
    for sample in samples {
        for (total, &ns) in totals.iter_mut().zip(&sample[..names.len()]) {
            *total = total.saturating_add(ns);
        }
        serving_elapsed_ns = serving_elapsed_ns.saturating_add(sample[names.len()]);
    }
    let stages = names
        .into_iter()
        .zip(totals)
        .map(|(stage, total_ns)| StageTiming {
            stage,
            total_ns,
            mean_ns: total_ns / positions.max(1) as u64,
        })
        .collect();
    let second = |ns: u64| ns as f64 / 1_000_000_000.0;
    Ok(ServedForwardTiming {
        measurement: "single fresh, uninstrumented parity pass followed by separate fresh CPU-oracle and CPU-serving timing passes; no warm-up",
        cpu_model: cpu_model(),
        logical_cpus: std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1),
        timed_positions: positions,
        cpu_oracle_elapsed_ns,
        cpu_oracle_tokens_per_second: positions as f64 / second(cpu_oracle_elapsed_ns).max(1e-12),
        serving_elapsed_ns,
        serving_tokens_per_second: positions as f64 / second(serving_elapsed_ns).max(1e-12),
        stages,
    })
}

fn timing_stage_names(layers: usize) -> Vec<String> {
    let mut names = vec!["embedding".to_owned()];
    for layer in 0..layers {
        names.extend([
            format!("layer_{layer}_rec_norm_gain"),
            format!("layer_{layer}_rec_gathers"),
            format!("layer_{layer}_recurrence_core"),
            format!("layer_{layer}_rec_out_residual"),
            format!("layer_{layer}_mlp_norm_gain"),
            format!("layer_{layer}_mlp_gathers"),
            format!("layer_{layer}_gated_product"),
            format!("layer_{layer}_activation_quantize"),
            format!("layer_{layer}_mlp_down_residual"),
        ]);
    }
    names.extend([
        "final_norm_query".to_owned(),
        "readout".to_owned(),
        "bookkeeping".to_owned(),
    ]);
    names
}

fn cpu_model() -> String {
    std::fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|cpuinfo| {
            cpuinfo
                .lines()
                .find_map(|line| line.strip_prefix("model name\t: ").map(str::to_owned))
        })
        .unwrap_or_else(|| "unknown".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, synthetic_hamming, FbsSynth};
    use uor_r4_integer::stack::{
        add_point_key_uorlut01, export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot,
        PointKeyImage, MEMORY_PAD,
    };

    fn artifact_with(readout_nearest: bool, typed_hamming: bool) -> Vec<u8> {
        let mut snapshot = synthetic_fbs_trunk_with(
            701,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 3,
                context: 20,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = readout_nearest;
        let mut hamming = synthetic_hamming(snapshot.width, 3, 4, 702);
        if !typed_hamming {
            for read in &mut hamming.reads {
                read.weight[snapshot.width..].fill(0.0);
            }
        }
        snapshot.hamming = Some(hamming);
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 4,
            key_window: 20,
            opening: vec![FbsMemoryEntry {
                key: vec![0, 7],
                value: 42,
            }],
            opening_weight: vec![1.0; snapshot.width],
            reply: Some(uor_r4_integer::stack::FbsReplyTable {
                turn: 2,
                entries: vec![FbsMemoryEntry {
                    key: vec![0, 7, MEMORY_PAD, 1, MEMORY_PAD, 42],
                    value: 77,
                }],
                weight: vec![1.0; snapshot.width],
            }),
        });
        export_fbs_trunk_uorlut01(&snapshot).unwrap()
    }

    fn artifact() -> Vec<u8> {
        artifact_with(true, true)
    }

    #[test]
    fn contract_states_truncating_and_rounding_boundaries() {
        let contract = exact_fbs_contract();
        let rounding = contract.rounding.join("\n");
        for required in [
            "RMS mean division and reciprocal truncate",
            "isqrt returns floor",
            "Hamming",
            "both integer quotients truncate toward zero",
            "half-up",
        ] {
            assert!(rounding.contains(required), "missing {required:?}");
        }
    }

    #[test]
    fn deterministic_report_checks_all_decisions_and_accounts_cost() {
        let artifact = artifact();
        let config = ServedForwardAuditConfig {
            tokens: vec![0, 7, 11, 42, 77],
            reply_start: Some(3),
            measure_timing: false,
        };
        let first = audit_served_forward(&artifact, &config).unwrap();
        let second = audit_served_forward(&artifact, &config).unwrap();
        assert_eq!(
            serde_json::to_vec(&first).unwrap(),
            serde_json::to_vec(&second).unwrap()
        );
        assert!(!first.parity.instrument_fault);
        assert_eq!(first.parity.positions_checked, 5);
        assert_eq!(first.parity.query_coordinates_compared, 5 * 64);
        assert_eq!(first.parity.token_decisions_compared, 5);
        assert_eq!(first.parity.kept_cells_compared, 5 * 8);
        assert_eq!(
            first.parity.cpu_decision_sha256,
            first.parity.serving_decision_sha256
        );
        let totals = &first.modelled_sequence_cost;
        assert!(totals.iter().any(|stage| stage.stage == "hamming_readout"));
        assert!(totals.iter().any(|stage| stage.stage == "turn_memory"));
        assert_eq!(
            first
                .modelled_position_cost
                .iter()
                .flatten()
                .map(|stage| stage.hamming_candidate_comparisons)
                .sum::<u64>(),
            (2 * (4 + 8 + 12)) as u64,
            "one Hamming read after layer 1 and one at readout, four heads"
        );
        assert!(first.manifest.serialized_table_payload_bytes > 0);
        assert!(
            first.manifest.serialized_table_payload_bytes <= first.artifact.bytes,
            "serialized table payload cannot exceed the artifact"
        );
        assert!(
            first.manifest.decoded_resident_table_bytes
                >= first.manifest.serialized_table_payload_bytes
        );
    }

    #[test]
    fn timed_report_uses_named_serving_stages_and_keeps_parity() {
        let artifact = artifact();
        let report = audit_served_forward(
            &artifact,
            &ServedForwardAuditConfig {
                tokens: vec![0, 7, 11],
                reply_start: None,
                measure_timing: true,
            },
        )
        .unwrap();
        let timing = report.timing.unwrap();
        assert_eq!(timing.timed_positions, 3);
        assert_eq!(timing.stages.len(), 9 * 3 + 4);
        assert!(timing.stages.iter().any(|stage| stage.stage == "embedding"));
        assert!(timing.stages.iter().any(|stage| stage.stage == "readout"));
        assert!(!report.parity.instrument_fault);
    }

    #[test]
    fn hamming_copy_only_and_readout_modes_have_exact_costs() {
        let copy_only = artifact_with(true, false);
        let report = audit_served_forward(
            &copy_only,
            &ServedForwardAuditConfig::new(vec![0, 7, 11, 42, 77]),
        )
        .unwrap();
        assert_eq!(
            report
                .modelled_position_cost
                .iter()
                .flatten()
                .map(|stage| stage.hamming_candidate_comparisons)
                .sum::<u64>(),
            24,
            "two reads, two copy heads and six prior-position visits in the serving range"
        );
        assert!(report
            .modelled_position_cost
            .iter()
            .flatten()
            .filter(|stage| stage.stage.contains("hamming"))
            .all(|stage| stage.hamilton_products == 0));
        let typed = audit_served_forward(
            &artifact_with(true, true),
            &ServedForwardAuditConfig::new(vec![0, 7, 11, 42, 77]),
        )
        .unwrap();
        assert_eq!(
            typed
                .modelled_position_cost
                .iter()
                .flatten()
                .filter(|stage| stage.stage.contains("hamming"))
                .map(|stage| stage.hamilton_products)
                .sum::<u64>(),
            10 * 24,
            "two read calls at four heads contribute 24 Hamilton products each"
        );

        let writer = audit_served_forward(
            &artifact_with(false, true),
            &ServedForwardAuditConfig::new(vec![0, 7]),
        )
        .unwrap();
        let nearest = audit_served_forward(
            &artifact_with(true, true),
            &ServedForwardAuditConfig::new(vec![0, 7]),
        )
        .unwrap();
        let readout = |report: &ServedForwardAuditReport| {
            report
                .modelled_position_cost
                .iter()
                .flatten()
                .filter(|stage| stage.stage == "readout")
                .map(|stage| stage.readout_candidate_tokens)
                .sum::<u64>()
        };
        assert_eq!(readout(&writer), 2 * 16);
        assert_eq!(readout(&nearest), 2 * 512);
    }

    #[test]
    fn memory_accounting_is_an_opening_upper_bound_and_skips_missing_reply_tables() {
        let mut snapshot = synthetic_fbs_trunk_with(
            704,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 2,
            key_window: 8,
            opening: vec![FbsMemoryEntry {
                key: vec![3, 4],
                value: 42,
            }],
            opening_weight: vec![1.0; snapshot.width],
            reply: None,
        });
        let bytes = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let bytes = add_point_key_uorlut01(
            &bytes,
            &PointKeyImage {
                width: snapshot.width,
                points: vec![0; snapshot.width * 2],
                targets: vec![42, 42],
            },
        )
        .unwrap();
        let report = audit_served_forward(
            &bytes,
            &ServedForwardAuditConfig {
                tokens: vec![0, 7, 11, 42],
                reply_start: Some(2),
                measure_timing: false,
            },
        )
        .unwrap();
        assert!(!report.parity.instrument_fault);
        let memory = report
            .modelled_sequence_cost
            .iter()
            .find(|stage| stage.stage == "turn_memory")
            .unwrap();
        assert_eq!(memory.memory_key_lookups, 1);
        assert_eq!(memory.point_table_rows_on_opening_miss, 2);
        assert_eq!(report.manifest.memory.as_ref().unwrap().reply_rows, 0);
    }

    #[test]
    fn malformed_artifact_and_bad_token_sequence_are_refused() {
        assert!(audit_served_forward(
            b"not a UORLUT artifact",
            &ServedForwardAuditConfig::new(vec![0])
        )
        .is_err());
        let artifact = artifact();
        assert!(
            audit_served_forward(&artifact, &ServedForwardAuditConfig::new(Vec::new())).is_err()
        );
        assert!(
            audit_served_forward(&artifact, &ServedForwardAuditConfig::new(vec![512])).is_err()
        );
        assert!(audit_served_forward(
            &artifact,
            &ServedForwardAuditConfig {
                tokens: vec![0],
                reply_start: Some(2),
                measure_timing: false,
            }
        )
        .is_err());
    }

    #[test]
    fn unsupported_retention_mode_is_refused() {
        let mut snapshot = synthetic_fbs_trunk_with(
            703,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 8,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.retain_rates = true;
        let artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let error =
            audit_served_forward(&artifact, &ServedForwardAuditConfig::new(vec![0])).unwrap_err();
        assert!(error.to_string().contains("CPU FBS forward refusal"));
    }

    #[test]
    fn planted_parity_failure_is_an_instrument_fault() {
        let mut parity = ParityAccumulator::default();
        let cpu = CpuFbsStep {
            token: 2,
            kept_cells: vec![1],
            query_q12: vec![5, 6],
        };
        let serving = NativeStep {
            token: 2,
            kept_cells: vec![1],
            query_q12: vec![5, 7],
        };
        parity.compare(0, &cpu, &serving);
        let report = parity.finish(1, 2, 1);
        assert!(report.instrument_fault);
        assert_eq!(report.query_coordinate_mismatches, 1);
        assert_eq!(report.mismatches[0].component, "query_q12");
    }
}
