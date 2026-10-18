//! CUDA driver boundary for the integer SpiralCore/FBS path.
//! Host work admits dimensions/identities and handles storage/observations.
//! Initialization, encoding, routing, scoring and proposal choice run on CUDA.
//! Module loading consumes exact AOT cubins assembled from admitted PTX; there
//! is no PTX-JIT fallback, runtime source compiler, or CPU model.

use crate::proposal_timing::{EventDuration, HostTimings, ProposalTimings, HOST_SPANS};
use crate::{
    metadata::{BinderPlan, ABSENT},
    objective::{self, ExactRatio, HingeDistribution},
    Error, Identity, ObjectivePolicy, Parameter, ReadInitialization, ReadSpec, Result, Spec,
    SCHEMA,
};
use cudarc::driver::{
    CudaContext, CudaEvent, CudaFunction, CudaSlice, CudaStream, LaunchConfig, Ptx, PushKernelArg,
};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::Instant;

mod geometry {
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/discrete.ptx"));
const ORDINARY_CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/ordinary.cubin"));
const ORDINARY_RESOURCE_REPORT: &str =
    include_str!(concat!(env!("OUT_DIR"), "/ordinary-resources.txt"));
const CUDA_IMAGE_TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
#[path = "decomposition_device.rs"]
mod decomposition_device;
pub use decomposition_device::DecompositionProfiler;
#[path = "baseline_reuse_device.rs"]
mod baseline_reuse_device;
pub use baseline_reuse_device::BaselineReuseProfiler;
#[path = "rows_device.rs"]
mod rows_device;
pub use rows_device::{compiled_rows_image_sha256, RowsCounterfactual};
#[path = "rowsk_device.rs"]
mod rowsk_device;
pub use rowsk_device::{
    compiled_rowsk_image_sha256, CoordinateScore, RowsKScoring, MAX_COORDINATE_BLOCK,
};
static NEXT_MODEL: AtomicU64 = AtomicU64::new(1);

/// Trace ABI from the production flock read, ordered by row then payload block:
/// header [frame_root, output_root, residual_terms, query_payload_root], followed
/// by one level record [gate+1, kept_count, consensus_root, eligible_count,
/// kept_node_ids[8], action_ids[8], carried_root_ids[8]]. Unused slots are ABSENT.
pub const FLOCK_TRACE_HEADER_WORDS: usize = 4;
pub const FLOCK_TRACE_LEVEL_WORDS: usize = 28;
pub const FLOCK_TRACE_KEPT_CAPACITY: usize = 8;
pub const EVALUATION_AGGREGATE_WORDS: usize = objective::AGGREGATE_WORDS;

/// Exact integer observations of the same production read and shared writer.
#[derive(Debug, serde::Serialize)]
pub struct EvaluationRow {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_position: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_token: Option<u32>,
    /// Global distance/token-ID order across all admitted tokens. ABSENT pads
    /// slots when fewer than eight identities are admitted.
    pub top8: [u32; 8],
    pub token_hinge: u64,
    pub address_hinge: u64,
    pub cell_rank_hinge: u64,
    pub target_cell_rank: u64,
    pub occupied_cells: u64,
    pub admitted_token_count: u64,
    pub loss_ta_numerator: u64,
    pub loss_tr_numerator: u64,
    pub objective_loss_numerator: u64,
    pub total_loss_numerator: u64,
    pub token_rivals: u64,
    pub address_rivals: u64,
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub admitted_cells: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EvaluationAggregate {
    pub scored_rows: u64,
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub token_hinge_sum: u64,
    pub address_hinge_sum: u64,
    pub total_loss_numerator_sum: u64,
    pub token_eligible_rows: u64,
    pub address_eligible_rows: u64,
    pub token_rival_count_sum: u64,
    pub address_rival_count_sum: u64,
    pub admitted_cell_count_sum: u64,
    pub eligible_any_rows: u64,
    pub eligible_unviolated_rows: u64,
    pub token_unviolated_eligible_rows: u64,
    pub address_unviolated_eligible_rows: u64,
    /// Strict target distance advantage over an admitted nonself rival. This
    /// diagnostic uses T<width; it does not change the margin objective/bar.
    pub token_strict_win_eligible_rows: u64,
    /// Exact common rational denominator for each row, not a floating mean.
    pub loss_denominator_per_row: u64,
    pub cell_rank_hinge_sum: u64,
    pub weighted_token_hinge_sum: u64,
    pub weighted_address_hinge_sum: u64,
    pub weighted_cell_rank_hinge_sum: u64,
    pub loss_ta_numerator_sum: u64,
    pub loss_tr_numerator_sum: u64,
    pub occupied_cells: u64,
    pub admitted_token_count_sum: u64,
    pub mean_admitted_token_count: ExactRatio,
    pub target_admitted_rows: u64,
    pub admitted_strict_win_rows: u64,
    pub cell_rank_zero_rows: u64,
    pub objective_loss_numerator_sum: u64,
    pub address_loss_share_ta: ExactRatio,
    pub rank_loss_share_tr: ExactRatio,
    pub token_hinge_distribution: HingeDistribution,
    pub address_hinge_distribution: HingeDistribution,
    pub cell_rank_hinge_distribution: HingeDistribution,
}
impl EvaluationAggregate {
    fn from_words(words: &[u64]) -> Result<Self> {
        let words: &[u64; EVALUATION_AGGREGATE_WORDS] = words
            .try_into()
            .map_err(|_| invalid("evaluation aggregate inventory differs from objective ABI"))?;
        let distribution =
            |start: usize, bins: usize, quantile: usize, sum: u64| HingeDistribution {
                min: words[quantile],
                p25: words[quantile + 1],
                p50: words[quantile + 2],
                p75: words[quantile + 3],
                p90: words[quantile + 4],
                p99: words[quantile + 5],
                max: words[quantile + 6],
                mean: ExactRatio {
                    numerator: sum,
                    denominator: words[0],
                },
                zero_rows: words[start],
                histogram: words[start..start + bins].to_vec(),
            };
        Ok(Self {
            scored_rows: words[0],
            top1_correct: words[1],
            top8_correct: words[2],
            token_hinge_sum: words[3],
            address_hinge_sum: words[4],
            total_loss_numerator_sum: words[5],
            token_eligible_rows: words[6],
            address_eligible_rows: words[7],
            token_rival_count_sum: words[8],
            address_rival_count_sum: words[9],
            admitted_cell_count_sum: words[10],
            eligible_any_rows: words[11],
            eligible_unviolated_rows: words[12],
            token_unviolated_eligible_rows: words[13],
            address_unviolated_eligible_rows: words[14],
            token_strict_win_eligible_rows: words[15],
            loss_denominator_per_row: words[16],
            cell_rank_hinge_sum: words[17],
            weighted_token_hinge_sum: words[18],
            weighted_address_hinge_sum: words[19],
            weighted_cell_rank_hinge_sum: words[20],
            loss_ta_numerator_sum: words[21],
            loss_tr_numerator_sum: words[22],
            occupied_cells: words[23],
            admitted_token_count_sum: words[24],
            mean_admitted_token_count: ExactRatio {
                numerator: words[24],
                denominator: words[0],
            },
            target_admitted_rows: words[25],
            admitted_strict_win_rows: words[26],
            cell_rank_zero_rows: words[27],
            objective_loss_numerator_sum: words[28],
            address_loss_share_ta: ExactRatio {
                numerator: words[19],
                denominator: words[21],
            },
            rank_loss_share_tr: ExactRatio {
                numerator: words[20],
                denominator: words[22],
            },
            token_hinge_distribution: distribution(29, 641, 999, words[3]),
            address_hinge_distribution: distribution(670, 73, 1006, words[4]),
            cell_rank_hinge_distribution: distribution(743, 256, 1013, words[17]),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct Evaluation {
    pub schema: &'static str,
    pub objective_policy: ObjectivePolicy,
    pub aggregate: EvaluationAggregate,
    pub rows: Vec<EvaluationRow>,
    #[serde(skip_serializing)]
    summary_words: Vec<u64>,
    #[serde(skip_serializing)]
    binding: Binding,
    #[serde(skip_serializing)]
    epoch: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct CountMetrics {
    pub token_counts: Vec<u64>,
    pub top8: [u32; 8],
    pub scored_rows: u64,
    pub top1_correct: u64,
    pub top8_correct: u64,
    pub zero_count_targets: u64,
    pub source_targets: u64,
    pub nonzero_tokens: u64,
}

#[derive(Debug, serde::Serialize)]
pub struct CountEvaluation {
    /// Caller supplies TRAIN mask/document-eligible target IDs. This API admits
    /// identities, not split lineage; the corpus packet owns TRAIN provenance.
    pub train_masked_reference: CountMetrics,
    /// Built from DEV targets in its own table, never the primary reference.
    pub dev_response_oracle: CountMetrics,
    pub dev_oracle_diagnostic_only: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct GpuMemory {
    pub ordinal: usize,
    pub free_bytes: usize,
    pub total_bytes: usize,
}

#[derive(Debug, serde::Serialize)]
pub struct DeviceResources {
    pub memory: GpuMemory,
    pub forward_local_bytes: u64,
    pub counterfactual_local_bytes: u64,
    pub evaluation_local_bytes: u64,
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

fn launch_config(items: u32) -> LaunchConfig {
    // The generic cudarc helper always requests 1024 threads, even for one
    // proposal. Bound blocks to this path's actual inventory and register use.
    let threads = items.clamp(1, 128);
    LaunchConfig {
        grid_dim: (items.div_ceil(threads), 1, 1),
        block_dim: (threads, 1, 1),
        shared_mem_bytes: 0,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    pub model: u64,
    pub spec: Spec,
    pub read_spec: ReadSpec,
    pub objective_policy: ObjectivePolicy,
    pub read_initialization: ReadInitialization,
    pub seed: u32,
    pub geometry_sha256: [u8; 32],
    pub source_sha256: [u8; 32],
}

/// A carried batch keeps exact identities and its device vectors together.
/// Vectors are valid only for the codebook epoch at which they were encoded.
pub struct CarriedBatch {
    vectors: CudaSlice<i32>,
    ids: CudaSlice<u32>,
    identities: Vec<Identity>,
    valid_lengths: Vec<u32>,
    plan: Option<CudaSlice<u32>>,
    binding: Binding,
    epoch: u64,
    rows: u32,
    stream: Arc<CudaStream>,
}
impl CarriedBatch {
    pub fn identities(&self) -> &[Identity] {
        &self.identities
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn rows(&self) -> u32 {
        self.rows
    }
    pub fn valid_lengths(&self) -> &[u32] {
        &self.valid_lengths
    }
    pub fn padded_count(&self) -> usize {
        self.ids.len()
    }
    /// Storage/qualification observation only; not a host model input path.
    pub fn values(&self) -> Result<Vec<i32>> {
        self.stream.clone_dtoh(&self.vectors).map_err(cuda)
    }
}

/// Completed GPU observations; the native output vector stays device-resident.
pub struct Forward {
    pub predictions: Vec<u32>,
    pub losses: Vec<u64>,
    pub token_rivals: Vec<u32>,
    pub address_rivals: Vec<u32>,
    /// Physical [rows,16] cells; ABSENT marks unused slots.
    pub admitted_cells: Vec<u32>,
    vectors: CudaSlice<i32>,
    trace: Option<CudaSlice<u32>>,
    trace_stride: Option<usize>,
    flock_trace_words: Option<usize>,
    binding: Binding,
    epoch: u64,
    stream: Arc<CudaStream>,
}
impl Forward {
    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    /// Storage/qualification observation only; no CPU scoring or transport.
    pub fn vectors(&self) -> Result<Vec<i32>> {
        self.stream.clone_dtoh(&self.vectors).map_err(cuda)
    }
    /// Optional completed device trace as storage observation only. This does
    /// not reconstruct, score, select or transport any model values on the host.
    /// Normal `forward` results return None and retain no trace allocation.
    pub fn flock_trace(&self) -> Result<Option<Vec<u32>>> {
        let Some(trace) = self.trace.as_ref() else {
            return Ok(None);
        };
        let mut words = self.stream.clone_dtoh(trace).map_err(cuda)?;
        if let Some(count) = self.flock_trace_words {
            words.truncate(count);
        }
        Ok(Some(words))
    }
    /// Optional completed device per-node relation scores (`rows * blocks * nodes_count`)
    /// written by `discrete_forward` in `ReadSpec::Relation` mode. Storage readback only;
    /// no host recurrence or score computation.
    pub fn relation_scores(&self) -> Result<Option<Vec<i32>>> {
        if !matches!(self.binding.read_spec, ReadSpec::Relation { .. }) {
            return Ok(None);
        }
        let (Some(trace), Some(offset)) = (self.trace.as_ref(), self.flock_trace_words) else {
            return Ok(None);
        };
        let words = self.stream.clone_dtoh(trace).map_err(cuda)?;
        let tail = words
            .get(offset..)
            .ok_or_else(|| invalid("relation trace buffer truncated"))?;
        Ok(Some(tail.iter().map(|&w| w as i32).collect()))
    }
    /// Words per (row, block), including its header and all model levels.
    pub fn trace_stride(&self) -> Option<usize> {
        self.trace_stride
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProposalObservation {
    /// Candidate ordinal, not a gate trit. Zero is always current/noop; for
    /// gates, 1/2 select the other trits in [0,-1,+1] order skipping current.
    pub candidate: u64,
    pub before: u64,
    pub after: u64,
    pub epoch: u64,
    /// CUDA compares every active candidate cost with CURRENT, including
    /// candidates worse than CURRENT. False therefore means all active costs
    /// tied, not merely that no update was accepted. Gate padding is excluded.
    pub any_candidate_cost_differs_from_identity: bool,
}

/// Artifact storage only. Values are admitted on the GPU when loaded; no
/// host score, route or transport is computed during serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: ReadSpec,
    #[serde(
        default,
        skip_serializing_if = "ObjectivePolicy::is_nearest_address_margin"
    )]
    pub objective_policy: ObjectivePolicy,
    #[serde(default, skip_serializing_if = "ReadInitialization::is_zero_gates")]
    pub read_initialization: ReadInitialization,
    pub seed: u32,
    pub epoch: u64,
    pub table: Vec<i32>,
    pub routes: Vec<u32>,
    /// Explicit full [blocks,stages] trit inventory in both modes. It is
    /// inactive in Route mode; there is no schema1/default-mode migration.
    pub gates: Vec<i32>,
    pub operator_kappa: String,
    pub composition_kappa: String,
    pub kernel_sha256: String,
    pub source_sha256: String,
    pub geometry_sha256: String,
}

/// Two encodings of one snapshot, never interchangeable with a raw FilePin or
/// the encoding of the enclosing DriverCheckpoint. This is storage metadata;
/// computing these hashes does not construct a Device or evaluate a model.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SnapshotHashDomains {
    /// Domain A: serialization of the serde_json::Value stored in a checkpoint.
    pub model_storage_sha256: String,
    /// Domain B: serialization of the typed Snapshot in declared field order.
    pub model_snapshot_sha256: String,
}

pub fn snapshot_hash_domains(snapshot: &Snapshot) -> Result<SnapshotHashDomains> {
    let stored = serde_json::to_value(snapshot).map_err(|e| invalid(e.to_string()))?;
    stored_snapshot_hash_domains(&stored)
}

pub fn stored_snapshot_hash_domains(stored: &serde_json::Value) -> Result<SnapshotHashDomains> {
    let snapshot: Snapshot =
        serde_json::from_value(stored.clone()).map_err(|e| invalid(e.to_string()))?;
    Ok(SnapshotHashDomains {
        // Hash the original stored Value, not a normalized typed round trip.
        model_storage_sha256: crate::corpus::sha256(
            &serde_json::to_vec(stored).map_err(|e| invalid(e.to_string()))?,
        ),
        model_snapshot_sha256: crate::corpus::sha256(
            &serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?,
        ),
    })
}

fn kernel_sha256() -> String {
    Sha256::digest(PTX.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn source_sha256() -> [u8; 32] {
    let mut source = Sha256::new();
    source.update(SCHEMA.as_bytes());
    source.update(include_bytes!("device.rs"));
    source.update(include_bytes!("discrete.cu"));
    source.update(include_bytes!("metadata.rs"));
    source.update(include_bytes!("lib.rs"));
    source.update(include_bytes!("objective.rs"));
    source.update(include_bytes!("proposal_timing.rs"));
    source.update(include_bytes!("../build.rs"));
    source.update(include_bytes!("decomposition_source.rs"));
    source.update(include_bytes!("decomposition.rs"));
    source.update(include_bytes!("decomposition_device.rs"));
    source.update(include_bytes!("baseline_reuse_source.rs"));
    source.update(include_bytes!("baseline_reuse.rs"));
    source.update(include_bytes!("baseline_reuse_device.rs"));
    source.update(decomposition_device::DIAGNOSTIC_PTX.as_bytes());
    source.update(PTX.as_bytes());
    // Preserve the PTX provenance hashes while also binding the actual loaded
    // objects, exact target and the reports describing those same objects.
    source.update(b"uor-r4.cuda-images/1");
    for (name, bytes) in [
        ("format", b"cubin".as_slice()),
        ("target", CUDA_IMAGE_TARGET.as_bytes()),
        ("ordinary.cubin", ORDINARY_CUBIN),
        (
            "decomposition.cubin",
            decomposition_device::DIAGNOSTIC_CUBIN,
        ),
        (
            "ordinary-resources.txt",
            ORDINARY_RESOURCE_REPORT.as_bytes(),
        ),
        (
            "decomposition-resources.txt",
            decomposition_device::DIAGNOSTIC_RESOURCE_REPORT.as_bytes(),
        ),
    ] {
        source.update((name.len() as u64).to_le_bytes());
        source.update(name.as_bytes());
        source.update((bytes.len() as u64).to_le_bytes());
        source.update(bytes);
    }
    // A separate versioned binding leaves the old U/D image record's meaning
    // intact while admitting every byte of the opt-in R image and provenance.
    source.update(b"uor-r4.baseline-reuse-image/1");
    for (name, bytes) in [
        (
            "baseline-reuse.cu",
            baseline_reuse_device::REUSE_SOURCE.as_bytes(),
        ),
        (
            "baseline-reuse.ptx",
            baseline_reuse_device::REUSE_PTX.as_bytes(),
        ),
        ("baseline-reuse.cubin", baseline_reuse_device::REUSE_CUBIN),
        (
            "baseline-reuse-resources.txt",
            baseline_reuse_device::REUSE_RESOURCE_REPORT.as_bytes(),
        ),
    ] {
        source.update((name.len() as u64).to_le_bytes());
        source.update(name.as_bytes());
        source.update((bytes.len() as u64).to_le_bytes());
        source.update(bytes);
    }
    // The opt-in row-parallel image is bound in its own versioned record.
    source.update(b"uor-r4.rows-image/1");
    for (name, bytes) in [
        (
            "rows_source.rs",
            include_bytes!("rows_source.rs").as_slice(),
        ),
        (
            "rows_device.rs",
            include_bytes!("rows_device.rs").as_slice(),
        ),
        ("rows.cu", rows_device::ROWS_SOURCE.as_bytes()),
        ("rows.ptx", rows_device::ROWS_PTX.as_bytes()),
        ("rows.cubin", rows_device::ROWS_CUBIN),
        (
            "rows-resources.txt",
            rows_device::ROWS_RESOURCE_REPORT.as_bytes(),
        ),
    ] {
        source.update((name.len() as u64).to_le_bytes());
        source.update(name.as_bytes());
        source.update((bytes.len() as u64).to_le_bytes());
        source.update(bytes);
    }
    source.finalize().into()
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// Identity of the exact embedded cubins/reports, without a CUDA context.
pub fn compiled_cuda_images() -> crate::decomposition::CudaImages {
    crate::decomposition::CudaImages {
        schema: "uor-r4.cuda-images/1".into(),
        format: "cubin".into(),
        target: CUDA_IMAGE_TARGET.into(),
        ordinary_sha256: format!("{:x}", Sha256::digest(ORDINARY_CUBIN)),
        decomposition_sha256: format!(
            "{:x}",
            Sha256::digest(decomposition_device::DIAGNOSTIC_CUBIN)
        ),
        ordinary_report_sha256: format!(
            "{:x}",
            Sha256::digest(ORDINARY_RESOURCE_REPORT.as_bytes())
        ),
        decomposition_report_sha256: format!(
            "{:x}",
            Sha256::digest(decomposition_device::DIAGNOSTIC_RESOURCE_REPORT.as_bytes())
        ),
    }
}

/// Independent identity of the exact opt-in R source/PTX/cubin/report.
pub fn compiled_baseline_reuse_image() -> crate::baseline_reuse::BaselineReuseImage {
    crate::baseline_reuse::BaselineReuseImage {
        schema: "uor-r4.baseline-reuse-image/1".into(),
        format: "cubin".into(),
        target: CUDA_IMAGE_TARGET.into(),
        source_sha256: format!(
            "{:x}",
            Sha256::digest(baseline_reuse_device::REUSE_SOURCE.as_bytes())
        ),
        ptx_sha256: format!(
            "{:x}",
            Sha256::digest(baseline_reuse_device::REUSE_PTX.as_bytes())
        ),
        cubin_sha256: format!("{:x}", Sha256::digest(baseline_reuse_device::REUSE_CUBIN)),
        report_sha256: format!(
            "{:x}",
            Sha256::digest(baseline_reuse_device::REUSE_RESOURCE_REPORT.as_bytes())
        ),
    }
}

fn admit_cuda_target(context: &CudaContext) -> Result<()> {
    compiled_cuda_images().validate().map_err(invalid)?;
    let (major, minor) = context.compute_capability().map_err(cuda)?;
    if major < 1 || !(0..=9).contains(&minor) || CUDA_IMAGE_TARGET != format!("sm_{major}{minor}") {
        return Err(invalid(format!(
            "AOT cubin target {CUDA_IMAGE_TARGET} differs from device capability {major}.{minor}; no PTX fallback"
        )));
    }
    Ok(())
}

/// Compiled artifact identity only; no CUDA context, model initialization or
/// numerical model operation is needed to validate an admission binding.
pub fn compiled_identity() -> serde_json::Value {
    serde_json::json!({
        "source_sha256": hex_digest(&source_sha256()),
        "kernel_sha256": kernel_sha256(),
        "geometry_sha256": hex_digest(&geometry_digest(&packed_geometry())),
        "decomposition_kernel_sha256": format!("{:x}",Sha256::digest(decomposition_device::DIAGNOSTIC_PTX.as_bytes())),
        "cuda_images": compiled_cuda_images(),
        "baseline_reuse_image": compiled_baseline_reuse_image(),
    })
}

pub struct ProposeProfiler {
    model: u64,
    stream: Arc<CudaStream>,
    events: [CudaEvent; 4],
    recorded: [bool; 4],
    started: Option<Instant>,
    boundary: Option<Instant>,
    host: HostTimings,
    counterfactual_status: Option<u32>,
    fault_losses: Option<Vec<u64>>,
}

impl ProposeProfiler {
    fn begin(&mut self) {
        self.host = HostTimings::default();
        self.recorded = [false; 4];
        self.counterfactual_status = None;
        self.fault_losses = None;
        let now = Instant::now();
        self.started = Some(now);
        self.boundary = Some(now);
    }

    fn mark(&mut self, span: usize) {
        let now = Instant::now();
        if let Some(boundary) = self.boundary {
            self.host.spans_ns[span] = now.duration_since(boundary).as_nanos();
            self.host.completed[span] = true;
        }
        self.boundary = Some(now);
    }

    fn event(&mut self, index: usize) -> Result<()> {
        self.events[index].record(&self.stream).map_err(cuda)?;
        self.recorded[index] = true;
        Ok(())
    }

    pub fn host_timings(&self) -> &HostTimings {
        &self.host
    }

    pub fn counterfactual_status(&self) -> Option<u32> {
        self.counterfactual_status
    }
    pub fn fault_losses(&self) -> Option<&[u64]> {
        self.fault_losses.as_deref()
    }

    pub fn timings(&self) -> Result<ProposalTimings> {
        self.host.validate_complete()?;
        if !self.recorded.iter().all(|recorded| *recorded)
            || !self.host.completed.iter().all(|completed| *completed)
        {
            return Err(invalid("proposal profile is incomplete"));
        }
        let started = Instant::now();
        let counterfactual = EventDuration::from_milliseconds_bits(
            self.events[0]
                .elapsed_ms(&self.events[1])
                .map_err(cuda)?
                .to_bits(),
        )?;
        let chooser = EventDuration::from_milliseconds_bits(
            self.events[2]
                .elapsed_ms(&self.events[3])
                .map_err(cuda)?
                .to_bits(),
        )?;
        Ok(ProposalTimings {
            schema: "uor-r4.proposal-subphase-timing/1",
            host_span_names: HOST_SPANS,
            host: self.host.clone(),
            counterfactual,
            chooser,
            event_readback_ns: started.elapsed().as_nanos(),
            accounting: "host spans partition the API call; GPU event durations overlap host waits and MUST NOT be added to host durations",
        })
    }
}

/// Device-chosen proposal. Private buffers prevent a host-authored update.
pub struct Proposal {
    data: CudaSlice<u64>,
    losses: CudaSlice<u64>,
    parameter: Parameter,
    binding: Binding,
    epoch: u64,
    stream: Arc<CudaStream>,
}
impl Proposal {
    pub fn parameter(&self) -> Parameter {
        self.parameter
    }
    /// Storage observation of all sixteen completed device counterfactual
    /// costs. Gate ordinals0..2 are active and ordinals3..15 are MAX padding.
    /// This performs no host scoring, ranking or candidate selection.
    pub fn candidate_losses(&self) -> Result<Vec<u64>> {
        self.stream.clone_dtoh(&self.losses).map_err(cuda)
    }
    pub fn observation(&self) -> Result<ProposalObservation> {
        let words = self.stream.clone_dtoh(&self.data).map_err(cuda)?;
        let words: [u64; 5] = words
            .try_into()
            .map_err(|_| invalid("proposal receipt must contain five words"))?;
        let any_candidate_cost_differs_from_identity = match words[4] {
            0 => false,
            1 => true,
            _ => return Err(invalid("proposal sensitivity receipt must be a boolean")),
        };
        Ok(ProposalObservation {
            candidate: words[0],
            before: words[1],
            after: words[2],
            epoch: words[3],
            any_candidate_cost_differs_from_identity,
        })
    }
}

struct Kernels {
    init: CudaFunction,
    encode: CudaFunction,
    export_token_cells: CudaFunction,
    forward: CudaFunction,
    counterfactual: CudaFunction,
    choose: CudaFunction,
    apply: CudaFunction,
    validate: CudaFunction,
    evaluate: CudaFunction,
    evaluation_aggregate: CudaFunction,
}

pub struct Device {
    context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    table: CudaSlice<i32>,
    routes: CudaSlice<u32>,
    gates: CudaSlice<i32>,
    epoch: CudaSlice<u64>,
    geometry: CudaSlice<u32>,
    metadata: BinderPlan,
    kernels: Kernels,
    binding: Binding,
    /// Lazily loaded opt-in row-parallel module, bound to this model/stream.
    rows_cache: std::sync::OnceLock<RowsCounterfactual>,
}

fn packed_geometry() -> Vec<u32> {
    // This is fixed generated geometry metadata, not carrier computation.
    let mut words = Vec::with_capacity(5120);
    words.extend(geometry::ROOTS.iter().flatten().map(|&x| (x as i32) as u32));
    for bytes in [
        &geometry::AXES[..],
        &geometry::COMPOSE[..],
        &geometry::BFS[..],
        &geometry::BRANCH[..],
        &geometry::GENERATORS[..],
        &geometry::INVERSE[..],
    ] {
        words.extend(bytes.iter().map(|&x| u32::from(x)));
    }
    words.push(u32::from(geometry::IDENTITY));
    words
}

fn make_binding(
    spec: Spec,
    read_spec: ReadSpec,
    objective_policy: ObjectivePolicy,
    read_initialization: ReadInitialization,
    seed: u32,
    geometry: &[u32],
) -> Result<Binding> {
    let model = NEXT_MODEL
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| x.checked_add(1))
        .map_err(|_| invalid("process model identity exhausted"))?;
    Ok(Binding {
        model,
        spec,
        read_spec,
        objective_policy,
        read_initialization,
        seed,
        geometry_sha256: geometry_digest(geometry),
        source_sha256: source_sha256(),
    })
}

fn geometry_digest(geometry: &[u32]) -> [u8; 32] {
    let mut geometric = Sha256::new();
    geometric.update(geometry::OPERATOR_KAPPA.as_bytes());
    geometric.update(geometry::COMPOSITION_KAPPA.as_bytes());
    for word in geometry {
        geometric.update(word.to_le_bytes());
    }
    geometric.finalize().into()
}

impl Device {
    pub fn new(spec: Spec, seed: u32, ordinal: usize) -> Result<Self> {
        Self::new_with_read(spec, ReadSpec::Route, seed, ordinal)
    }

    pub fn new_with_read(
        spec: Spec,
        read_spec: ReadSpec,
        seed: u32,
        ordinal: usize,
    ) -> Result<Self> {
        Self::new_with_objective(
            spec,
            read_spec,
            ObjectivePolicy::NearestAddressMarginV1,
            seed,
            ordinal,
        )
    }

    pub fn new_with_objective(
        spec: Spec,
        read_spec: ReadSpec,
        objective_policy: ObjectivePolicy,
        seed: u32,
        ordinal: usize,
    ) -> Result<Self> {
        Self::new_with_initialization(
            spec,
            read_spec,
            objective_policy,
            ReadInitialization::ZeroGatesV1,
            seed,
            ordinal,
        )
    }

    pub fn new_with_initialization(
        spec: Spec,
        read_spec: ReadSpec,
        objective_policy: ObjectivePolicy,
        read_initialization: ReadInitialization,
        seed: u32,
        ordinal: usize,
    ) -> Result<Self> {
        read_spec.validate(spec)?;
        read_initialization.validate(read_spec)?;
        let metadata = BinderPlan::new(spec.context as usize)?;
        let packed = packed_geometry();
        if packed.len() != 5120 {
            return Err(invalid("generated geometry ABI inventory differs"));
        }
        let binding = make_binding(
            spec,
            read_spec,
            objective_policy,
            read_initialization,
            seed,
            &packed,
        )?;
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        admit_cuda_target(&context)?;
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(ORDINARY_CUBIN.to_vec()))
            .map_err(cuda)?;
        let kernels = Kernels {
            init: module.load_function("discrete_init").map_err(cuda)?,
            encode: module.load_function("discrete_encode").map_err(cuda)?,
            export_token_cells: module
                .load_function("discrete_export_token_cells")
                .map_err(cuda)?,
            forward: module.load_function("discrete_forward").map_err(cuda)?,
            counterfactual: module
                .load_function("discrete_counterfactual")
                .map_err(cuda)?,
            choose: module.load_function("discrete_choose").map_err(cuda)?,
            apply: module.load_function("discrete_apply").map_err(cuda)?,
            validate: module.load_function("discrete_validate").map_err(cuda)?,
            evaluate: module.load_function("discrete_evaluate").map_err(cuda)?,
            evaluation_aggregate: module
                .load_function("discrete_evaluation_aggregate")
                .map_err(cuda)?,
        };
        let geometry = stream.clone_htod(&packed).map_err(cuda)?;
        let width = spec.blocks as usize * 8;
        let mut table = stream
            .alloc_zeros::<i32>(spec.vocabulary as usize * width)
            .map_err(cuda)?;
        let mut routes = stream
            .alloc_zeros::<u32>(spec.blocks as usize * metadata.stages())
            .map_err(cuda)?;
        let gate_count = read_spec.gate_count(spec.blocks, metadata.stages() as u32)? as usize;
        let mut gates = stream.alloc_zeros::<i32>(gate_count).map_err(cuda)?;
        let epoch = stream.alloc_zeros::<u64>(1).map_err(cuda)?;
        let words = read_spec.words();
        let spec_gpu = stream
            .clone_htod(&[
                spec.vocabulary,
                spec.blocks,
                spec.context,
                spec.kept_cells,
                1,
                metadata.stages() as u32,
                metadata.nodes_count() as u32,
                metadata.frontier().len() as u32,
                words[0],
                words[1],
                words[2],
                objective_policy.word(),
            ])
            .map_err(cuda)?;
        let mut status = stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let items = u32::try_from(table.len().max(routes.len()).max(gates.len()))
            .map_err(|_| invalid("initialization inventory exceeds CUDA index bound"))?;
        let mut launch = stream.launch_builder(&kernels.init);
        launch
            .arg(&mut table)
            .arg(&mut routes)
            .arg(&mut gates)
            .arg(&geometry)
            .arg(&spec_gpu)
            .arg(&seed)
            .arg(&mut status);
        // SAFETY: exact discrete_init ABI, checked allocation inventories,
        // immutable geometry/spec, writable buffers owned by this launch.
        unsafe { launch.launch(launch_config(items)) }.map_err(cuda)?;
        check_status(&stream, &status, "initialize")?;
        // A prospectively declared literal storage amendment after the unchanged
        // production initializer. This is data upload, not a host read/forward,
        // target-derived choice or learning apply; the model remains epoch zero.
        if read_initialization == ReadInitialization::ActiveLeafV1 {
            let mut declared = vec![0i32; gate_count];
            for block in 0..spec.blocks as usize {
                declared[block * metadata.stages()] = 1;
            }
            stream.memcpy_htod(&declared, &mut gates).map_err(cuda)?;
        }
        // Storage validation of a declared literal initialization, never a host
        // read/score/reference. Only declared atom combine gates may be nonzero.
        let initialized_gates = stream.clone_dtoh(&gates).map_err(cuda)?;
        for (index, observed) in initialized_gates.iter().enumerate() {
            let expected = i32::from(
                read_initialization == ReadInitialization::ActiveLeafV1
                    && index < spec.blocks as usize * metadata.stages()
                    && index.is_multiple_of(metadata.stages()),
            );
            if *observed != expected {
                return Err(invalid(
                    "CUDA initial gate storage differs from declared read initialization",
                ));
            }
        }
        Ok(Self {
            context,
            stream,
            table,
            routes,
            gates,
            epoch,
            geometry,
            metadata,
            kernels,
            binding,
            rows_cache: std::sync::OnceLock::new(),
        })
    }

    pub fn binding(&self) -> &Binding {
        &self.binding
    }
    pub fn spec(&self) -> Spec {
        self.binding.spec
    }
    pub fn read_spec(&self) -> ReadSpec {
        self.binding.read_spec
    }
    pub fn objective_policy(&self) -> ObjectivePolicy {
        self.binding.objective_policy
    }
    pub fn read_initialization(&self) -> ReadInitialization {
        self.binding.read_initialization
    }
    pub fn ordinal(&self) -> usize {
        self.context.ordinal()
    }
    /// Driver metadata only, for resource observations before constructing a
    /// model. Context initialization itself is included in the observed state.
    pub fn memory_info(ordinal: usize) -> Result<GpuMemory> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (free_bytes, total_bytes) = context.mem_get_info().map_err(cuda)?;
        Ok(GpuMemory {
            ordinal,
            free_bytes,
            total_bytes,
        })
    }
    /// Synchronize completed work before reporting memory and compiled local
    /// storage. This is driver/kernel metadata, not model arithmetic or a peak
    /// memory measurement; callers must label the before/after sampling points.
    pub fn resource_metadata(&self) -> Result<DeviceResources> {
        self.synchronize()?;
        let (free_bytes, total_bytes) = self.context.mem_get_info().map_err(cuda)?;
        let local = |kernel: &CudaFunction| -> Result<u64> {
            let bytes = kernel.get_attribute(cudarc::driver::sys::CUfunction_attribute_enum::CU_FUNC_ATTRIBUTE_LOCAL_SIZE_BYTES).map_err(cuda)?;
            u64::try_from(bytes).map_err(|_| invalid("CUDA reported negative kernel local storage"))
        };
        Ok(DeviceResources {
            memory: GpuMemory {
                ordinal: self.ordinal(),
                free_bytes,
                total_bytes,
            },
            forward_local_bytes: local(&self.kernels.forward)?,
            counterfactual_local_bytes: local(&self.kernels.counterfactual)?,
            evaluation_local_bytes: local(&self.kernels.evaluate)?,
        })
    }
    pub fn epoch(&self) -> Result<u64> {
        read_epoch(&self.stream, &self.epoch)
    }
    pub fn synchronize(&self) -> Result<()> {
        self.context.synchronize().map_err(cuda)
    }

    /// Storage-only readback for admission checks; copies no codebook or routes.
    pub fn read_gates(&self) -> Result<Vec<i32>> {
        self.stream.clone_dtoh(&self.gates).map_err(cuda)
    }

    /// Read-only CUDA export of the current checkpoint's token cells, indexed
    /// by token ID. Geometry is computed by the existing device token_cell
    /// identity-candidate helper; host work only converts checked storage.
    pub fn export_token_cells(&self) -> Result<Vec<u16>> {
        let vocabulary = self.spec().vocabulary;
        let spec = self.gpu_spec(0)?;
        let mut cells = self
            .stream
            .alloc_zeros::<u32>(vocabulary as usize)
            .map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.export_token_cells);
        launch
            .arg(&self.table)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&mut cells)
            .arg(&mut status);
        // SAFETY: the admitted spec and geometry belong to this model; one
        // output word exists per token. Only cells/status are writable, and
        // no routes, gates or epoch pointer is passed to this read-only entry.
        unsafe { launch.launch(launch_config(vocabulary)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "token-cell export")?;
        self.stream
            .clone_dtoh(&cells)
            .map_err(cuda)?
            .into_iter()
            .map(|cell| {
                u16::try_from(cell).map_err(|_| invalid("token-cell export exceeds u16 storage"))
            })
            .collect()
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        // This is storage readback, not a CPU reference or a learning update.
        Ok(Snapshot {
            schema: SCHEMA.to_owned(),
            spec: self.spec(),
            read_spec: self.read_spec(),
            objective_policy: self.objective_policy(),
            read_initialization: self.read_initialization(),
            seed: self.binding.seed,
            epoch: self.epoch()?,
            table: self.stream.clone_dtoh(&self.table).map_err(cuda)?,
            routes: self.stream.clone_dtoh(&self.routes).map_err(cuda)?,
            gates: self.stream.clone_dtoh(&self.gates).map_err(cuda)?,
            operator_kappa: geometry::OPERATOR_KAPPA.to_owned(),
            composition_kappa: geometry::COMPOSITION_KAPPA.to_owned(),
            kernel_sha256: kernel_sha256(),
            source_sha256: hex_digest(&self.binding.source_sha256),
            geometry_sha256: hex_digest(&self.binding.geometry_sha256),
        })
    }

    pub fn from_snapshot(snapshot: Snapshot, ordinal: usize) -> Result<Self> {
        snapshot.read_spec.validate(snapshot.spec)?;
        let metadata = BinderPlan::new(snapshot.spec.context as usize)?;
        if snapshot.schema != SCHEMA
            || snapshot.operator_kappa != geometry::OPERATOR_KAPPA
            || snapshot.composition_kappa != geometry::COMPOSITION_KAPPA
            || snapshot.kernel_sha256 != kernel_sha256()
            || snapshot.source_sha256 != hex_digest(&source_sha256())
            || snapshot.geometry_sha256 != hex_digest(&geometry_digest(&packed_geometry()))
        {
            return Err(invalid(
                "snapshot schema/geometry/AOT kernel/compiler source binding differs",
            ));
        }
        let gate_count = snapshot
            .read_spec
            .gate_count(snapshot.spec.blocks, metadata.stages() as u32)?
            as usize;
        if snapshot.table.len()
            != snapshot.spec.vocabulary as usize * snapshot.spec.blocks as usize * 8
            || snapshot.routes.len() != snapshot.spec.blocks as usize * metadata.stages()
            || snapshot.gates.len() != gate_count
        {
            return Err(invalid(
                "snapshot codebook/route/gate allocation inventory differs",
            ));
        }
        // Fresh process identity even for byte-identical reloads. No carried
        // batch/proposal from another model instance becomes valid by loading.
        let mut model = Self::new_with_initialization(
            snapshot.spec,
            snapshot.read_spec,
            snapshot.objective_policy,
            snapshot.read_initialization,
            snapshot.seed,
            ordinal,
        )?;
        model.table = model.stream.clone_htod(&snapshot.table).map_err(cuda)?;
        model.routes = model.stream.clone_htod(&snapshot.routes).map_err(cuda)?;
        model.gates = model.stream.clone_htod(&snapshot.gates).map_err(cuda)?;
        model.epoch = model.stream.clone_htod(&[snapshot.epoch]).map_err(cuda)?;
        let spec = model.gpu_spec(1)?;
        let mut status = model.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let items = u32::try_from(
            (model.table.len() / 8)
                .max(model.routes.len())
                .max(model.gates.len()),
        )
        .map_err(|_| invalid("snapshot validation inventory exceeds CUDA index bound"))?;
        let mut launch = model.stream.launch_builder(&model.kernels.validate);
        launch
            .arg(&model.table)
            .arg(&model.routes)
            .arg(&model.gates)
            .arg(&model.geometry)
            .arg(&spec)
            .arg(&mut status);
        // SAFETY: discrete_validate ABI, checked buffer shapes, fixed geometry
        // and spec. All table/root membership and route-state admission runs
        // on CUDA, before the loaded model can encode or forward any tokens.
        unsafe { launch.launch(launch_config(items)) }.map_err(cuda)?;
        check_status(&model.stream, &status, "snapshot validation")?;
        Ok(model)
    }

    fn gpu_spec(&self, rows: u32) -> Result<CudaSlice<u32>> {
        let spec = self.spec();
        let words = self.read_spec().words();
        self.stream
            .clone_htod(&[
                spec.vocabulary,
                spec.blocks,
                spec.context,
                spec.kept_cells,
                rows,
                self.metadata.stages() as u32,
                self.metadata.nodes_count() as u32,
                self.metadata.frontier().len() as u32,
                words[0],
                words[1],
                words[2],
                self.objective_policy().word(),
            ])
            .map_err(cuda)
    }

    fn admit_batch(&self, batch: &CarriedBatch, targets: &[u32]) -> Result<u64> {
        if batch.binding != self.binding {
            return Err(invalid(
                "carried batch belongs to another model/source/geometry",
            ));
        }
        if !(1..=64).contains(&batch.rows)
            || batch.plan.is_none()
            || batch.valid_lengths.len() != batch.rows as usize
            || batch
                .valid_lengths
                .iter()
                .any(|&n| n == 0 || n > self.spec().context)
            || batch.ids.len() != batch.rows as usize * self.spec().context as usize
        {
            return Err(invalid(
                "forward/proposal requires 1..64 admitted nonempty row plans",
            ));
        }
        if targets.len() != batch.rows as usize
            || targets.iter().any(|&x| x >= self.spec().vocabulary)
        {
            return Err(invalid(
                "targets require one valid token per carried context row",
            ));
        }
        let epoch = self.epoch()?;
        if batch.epoch != epoch {
            return Err(invalid(
                "carried vectors are stale at the current model epoch",
            ));
        }
        Ok(epoch)
    }

    pub fn carry(&self, identities: &[Identity]) -> Result<CarriedBatch> {
        let context = self.spec().context as usize;
        let lengths =
            if identities.len().is_multiple_of(context) && identities.len() / context <= 64 {
                vec![self.spec().context; identities.len() / context]
            } else {
                Vec::new()
            };
        self.encode_carried(identities, identities.to_vec(), lengths)
    }

    /// Each window is one caller-bounded causal prefix. No plan can select
    /// its padding; the last valid token is the row query. Document/BOS
    /// segmentation remains explicit in the caller, not inferred from IDs.
    pub fn carry_windows(&self, windows: &[Vec<Identity>]) -> Result<CarriedBatch> {
        let spec = self.spec();
        if windows.is_empty()
            || windows.len() > 64
            || windows.iter().any(|w| {
                w.is_empty()
                    || w.len() > spec.context as usize
                    || w.iter().any(|x| x.token >= spec.vocabulary)
            })
        {
            return Err(invalid("carry_windows requires 1..64 nonempty causal prefixes of length1..context with valid IDs"));
        }
        let mut padded = Vec::with_capacity(windows.len() * spec.context as usize);
        let mut identities = Vec::new();
        let mut lengths = Vec::with_capacity(windows.len());
        for window in windows {
            identities.extend_from_slice(window);
            lengths.push(window.len() as u32);
            padded.extend_from_slice(window);
            padded.extend(std::iter::repeat_n(
                window[0],
                spec.context as usize - window.len(),
            ));
        }
        self.encode_carried(&padded, identities, lengths)
    }

    fn row_plans(&self, lengths: &[u32]) -> Result<CudaSlice<u32>> {
        let max_nodes = self.metadata.nodes_count();
        let stride = 3 + max_nodes * 4 + 16;
        let mut packed = Vec::with_capacity(lengths.len() * stride);
        for &length in lengths {
            let plan = BinderPlan::new(length as usize)?;
            if plan.nodes_count() > max_nodes || plan.frontier().len() > 16 {
                return Err(invalid("row plan exceeds its model allocation inventory"));
            }
            packed.extend([
                length,
                plan.nodes_count() as u32,
                plan.frontier().len() as u32,
            ]);
            let nodes = plan.node_words();
            packed.extend_from_slice(&nodes);
            packed.extend(std::iter::repeat_n(ABSENT, max_nodes * 4 - nodes.len()));
            packed.extend_from_slice(plan.frontier());
            packed.extend(std::iter::repeat_n(ABSENT, 16 - plan.frontier().len()));
        }
        self.stream.clone_htod(&packed).map_err(cuda)
    }

    fn encode_carried(
        &self,
        physical: &[Identity],
        identities: Vec<Identity>,
        valid_lengths: Vec<u32>,
    ) -> Result<CarriedBatch> {
        let spec = self.spec();
        let count = physical.len();
        if count == 0 || physical.iter().any(|x| x.token >= spec.vocabulary) {
            return Err(invalid("carry requires nonempty valid token identities"));
        }
        let width = spec.blocks * 8;
        let count = u32::try_from(count)
            .map_err(|_| invalid("carry token inventory exceeds CUDA index bound"))?;
        let elements = count
            .checked_mul(width)
            .ok_or_else(|| invalid("carry vector inventory exceeds CUDA index bound"))?;
        let rows = valid_lengths.len() as u32;
        let plan = if rows > 0 {
            Some(self.row_plans(&valid_lengths)?)
        } else {
            None
        };
        let epoch = self.epoch()?;
        let ids = self
            .stream
            .clone_htod(&physical.iter().map(|x| x.token).collect::<Vec<_>>())
            .map_err(cuda)?;
        let mut vectors = self
            .stream
            .alloc_zeros::<i32>(elements as usize)
            .map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.encode);
        launch
            .arg(&self.table)
            .arg(&ids)
            .arg(&count)
            .arg(&width)
            .arg(&spec.vocabulary)
            .arg(&mut vectors)
            .arg(&mut status);
        // SAFETY: discrete_encode receives valid IDs and count*width buffers;
        // only output/status are writable, and all belong to this context.
        unsafe { launch.launch(launch_config(elements)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "encode")?;
        Ok(CarriedBatch {
            vectors,
            ids,
            identities,
            valid_lengths,
            plan,
            binding: self.binding.clone(),
            epoch,
            rows,
            stream: Arc::clone(&self.stream),
        })
    }

    pub fn forward(&self, batch: &CarriedBatch, targets: &[u32]) -> Result<Forward> {
        self.forward_internal(batch, targets, false)
    }

    /// Qualification observation of this same production forward path. The
    /// read/loss law is unchanged; only a bounded CUDA-written trace is retained.
    pub fn forward_traced(&self, batch: &CarriedBatch, targets: &[u32]) -> Result<Forward> {
        if self.read_spec() == ReadSpec::Route {
            return Err(invalid("forward_traced requires a flock model"));
        }
        self.forward_internal(batch, targets, true)
    }

    fn forward_internal(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        traced: bool,
    ) -> Result<Forward> {
        let epoch = self.admit_batch(batch, targets)?;
        let target = self.stream.clone_htod(targets).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let mut out = self
            .stream
            .alloc_zeros::<i32>(batch.rows as usize * self.spec().blocks as usize * 8)
            .map_err(cuda)?;
        let mut predictions = self
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut losses = self
            .stream
            .alloc_zeros::<u64>(batch.rows as usize)
            .map_err(cuda)?;
        let mut token_rivals = self
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut address_rivals = self
            .stream
            .alloc_zeros::<u32>(batch.rows as usize)
            .map_err(cuda)?;
        let mut admitted_cells = self
            .stream
            .alloc_zeros::<u32>(batch.rows as usize * 16)
            .map_err(cuda)?;
        let trace_stride = traced
            .then_some(FLOCK_TRACE_HEADER_WORDS + self.metadata.stages() * FLOCK_TRACE_LEVEL_WORDS);
        let flock_trace_words = trace_stride
            .map(|stride| {
                (batch.rows as usize)
                    .checked_mul(self.spec().blocks as usize)
                    .and_then(|blocks| blocks.checked_mul(stride))
                    .ok_or_else(|| invalid("flock trace inventory exceeds storage index bound"))
            })
            .transpose()?;
        let relation_score_words =
            if traced && matches!(self.read_spec(), ReadSpec::Relation { .. }) {
                Some(
                    (batch.rows as usize)
                        .checked_mul(self.spec().blocks as usize)
                        .and_then(|blocks| blocks.checked_mul(self.metadata.nodes_count()))
                        .ok_or_else(|| {
                            invalid("relation trace inventory exceeds storage index bound")
                        })?,
                )
            } else {
                None
            };
        let mut trace = if let Some(base_words) = flock_trace_words {
            let words = base_words
                .checked_add(relation_score_words.unwrap_or(0))
                .ok_or_else(|| invalid("flock trace inventory exceeds storage index bound"))?;
            Some(self.stream.alloc_zeros::<u32>(words).map_err(cuda)?)
        } else {
            None
        };
        // CUDA driver pointer parameters are CUdeviceptr-sized words. Passing
        // the address of this zero WORD marshals a null device pointer, not a
        // host pointer. The CUDA read tests trace for null before every write.
        let null_trace: cudarc::driver::sys::CUdeviceptr = 0;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.forward);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&target)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&mut out)
            .arg(&mut predictions)
            .arg(&mut losses)
            .arg(&mut token_rivals)
            .arg(&mut address_rivals)
            .arg(&mut admitted_cells);
        if let Some(trace) = trace.as_mut() {
            launch.arg(trace);
        } else {
            launch.arg(&null_trace);
        }
        launch.arg(&mut status);
        // SAFETY: discrete_forward ABI and bounded rows/width/context checked
        // before launch; immutable inputs, disjoint owned result buffers. Trace
        // is either the exact rows*blocks*(4+stages*28) allocation (plus
        // rows*blocks*nodes_count in Relation mode) or a null
        // CUdeviceptr, which the kernel never dereferences. Its U* argument is
        // immediately before status in the explicit 64-bit driver ABI.
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "forward")?;
        let predictions = self.stream.clone_dtoh(&predictions).map_err(cuda)?;
        let losses = self.stream.clone_dtoh(&losses).map_err(cuda)?;
        let token_rivals = self.stream.clone_dtoh(&token_rivals).map_err(cuda)?;
        let address_rivals = self.stream.clone_dtoh(&address_rivals).map_err(cuda)?;
        let admitted_cells = self.stream.clone_dtoh(&admitted_cells).map_err(cuda)?;
        Ok(Forward {
            predictions,
            losses,
            token_rivals,
            address_rivals,
            admitted_cells,
            vectors: out,
            trace,
            trace_stride,
            flock_trace_words,
            binding: self.binding.clone(),
            epoch,
            stream: Arc::clone(&self.stream),
        })
    }

    /// Evaluate exact hits, component hinges and eligibility on CUDA. Top-eight
    /// ranking is global over admitted identities; the target is never inserted.
    pub fn evaluate(&self, batch: &CarriedBatch, targets: &[u32]) -> Result<Evaluation> {
        let epoch = self.admit_batch(batch, targets)?;
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        let mut top8 = self
            .stream
            .alloc_zeros::<u32>(batch.rows as usize * 8)
            .map_err(cuda)?;
        let mut metrics = self
            .stream
            .alloc_zeros::<u64>(batch.rows as usize * objective::ROW_WORDS)
            .map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.evaluate);
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&mut top8)
            .arg(&mut metrics)
            .arg(&mut status);
        // SAFETY: same immutable model/input ABI as forward; bounded rows and
        // per-row eight-word result allocations. The read/writer runs on CUDA.
        unsafe { launch.launch(launch_config(batch.rows)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "evaluation")?;
        let summary_words = aggregate_gpu(
            &self.stream,
            &self.kernels.evaluation_aggregate,
            &metrics,
            batch.rows,
            self.spec().blocks * 8,
            false,
        )?;
        let aggregate = EvaluationAggregate::from_words(&summary_words)?;
        let top8 = self.stream.clone_dtoh(&top8).map_err(cuda)?;
        let metrics = self.stream.clone_dtoh(&metrics).map_err(cuda)?;
        // Storage decoding only. Scores, ranks, eligibility and all sums above
        // were completed on CUDA; no numerical reductions occur in this loop.
        let rows = top8
            .chunks_exact(8)
            .zip(metrics.chunks_exact(objective::ROW_WORDS))
            .map(|(top, p)| {
                let top8 = top
                    .try_into()
                    .map_err(|_| invalid("top8 row must contain eight IDs"))?;
                Ok(EvaluationRow {
                    target_position: None,
                    target_token: None,
                    top8,
                    token_hinge: p[0],
                    address_hinge: p[1],
                    cell_rank_hinge: p[8],
                    target_cell_rank: p[9],
                    occupied_cells: p[10],
                    admitted_token_count: p[11],
                    loss_ta_numerator: p[2],
                    loss_tr_numerator: p[12],
                    objective_loss_numerator: p[13],
                    total_loss_numerator: p[2],
                    token_rivals: p[3],
                    address_rivals: p[4],
                    top1_correct: p[5],
                    top8_correct: p[6],
                    admitted_cells: p[7],
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Evaluation {
            schema: objective::EVALUATION_SCHEMA,
            objective_policy: self.objective_policy(),
            aggregate,
            rows,
            summary_words,
            binding: self.binding.clone(),
            epoch,
        })
    }

    /// CUDA aggregation of completed batch receipts at one model/epoch. Private
    /// words are re-uploaded unchanged; public report-field edits cannot author
    /// an aggregate. No host score/count/rank/reduction is performed.
    pub fn aggregate_evaluations(&self, evaluations: &[Evaluation]) -> Result<EvaluationAggregate> {
        if evaluations.is_empty() {
            return Err(invalid(
                "evaluation aggregation requires nonempty completed receipts",
            ));
        }
        let count = u32::try_from(evaluations.len())
            .map_err(|_| invalid("evaluation receipt inventory exceeds CUDA index bound"))?;
        let epoch = self.epoch()?;
        let capacity = evaluations
            .len()
            .checked_mul(EVALUATION_AGGREGATE_WORDS)
            .ok_or_else(|| invalid("evaluation receipt allocation exceeds storage index bound"))?;
        let mut words = Vec::new();
        words
            .try_reserve_exact(capacity)
            .map_err(|e| invalid(format!("evaluation receipt allocation: {e}")))?;
        for evaluation in evaluations {
            if evaluation.binding != self.binding || evaluation.epoch != epoch {
                return Err(invalid(
                    "evaluation receipt belongs to another model/source/epoch",
                ));
            }
            if evaluation.summary_words.len() != EVALUATION_AGGREGATE_WORDS {
                return Err(invalid(
                    "evaluation receipt inventory differs from objective ABI",
                ));
            }
            words.extend_from_slice(&evaluation.summary_words);
        }
        let metrics = self.stream.clone_htod(&words).map_err(cuda)?;
        EvaluationAggregate::from_words(&aggregate_gpu(
            &self.stream,
            &self.kernels.evaluation_aggregate,
            &metrics,
            count,
            self.spec().blocks * 8,
            true,
        )?)
    }

    /// Raw-frequency reference from caller-prepared masked TRAIN target IDs.
    /// Host admission checks identities/lengths only. Both histograms, global
    /// top8 rankings and hit counters are computed on CUDA. The separate DEV
    /// response oracle is diagnostic and cannot affect the TRAIN reference.
    pub fn count_reference(
        vocabulary: u32,
        training_targets: &[u32],
        eval_targets: &[u32],
        ordinal: usize,
    ) -> Result<CountEvaluation> {
        if !(2..=8192).contains(&vocabulary)
            || training_targets.is_empty()
            || eval_targets.is_empty()
            || training_targets
                .iter()
                .chain(eval_targets)
                .any(|&id| id >= vocabulary)
        {
            return Err(invalid("count reference requires vocabulary2..8192 and nonempty valid TRAIN/DEV target IDs"));
        }
        let _training_count = u32::try_from(training_targets.len())
            .map_err(|_| invalid("TRAIN count inventory exceeds CUDA index bound"))?;
        let eval_count = u32::try_from(eval_targets.len())
            .map_err(|_| invalid("DEV count inventory exceeds CUDA index bound"))?;
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        admit_cuda_target(&context)?;
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(ORDINARY_CUBIN.to_vec()))
            .map_err(cuda)?;
        let histogram = module.load_function("discrete_count").map_err(cuda)?;
        let score = module.load_function("discrete_count_score").map_err(cuda)?;
        let targets = stream.clone_htod(eval_targets).map_err(cuda)?;
        let train_masked_reference = count_metrics(
            &stream,
            &histogram,
            &score,
            vocabulary,
            training_targets,
            &targets,
            eval_count,
        )?;
        let dev_response_oracle = count_metrics(
            &stream,
            &histogram,
            &score,
            vocabulary,
            eval_targets,
            &targets,
            eval_count,
        )?;
        Ok(CountEvaluation {
            train_masked_reference,
            dev_response_oracle,
            dev_oracle_diagnostic_only: true,
        })
    }

    fn parameter(&self, parameter: Parameter) -> Result<(u32, u32)> {
        match parameter {
            Parameter::Route(_) if matches!(self.read_spec(), ReadSpec::Flock { .. }) => {
                Err(invalid("route parameters are inactive in a flock model"))
            }
            Parameter::Route(_) if matches!(self.read_spec(), ReadSpec::Relation { .. }) => {
                Err(invalid("route parameters are inactive in a relation model"))
            }
            Parameter::Route(index) if (index as usize) < self.routes.len() => Ok((0, index)),
            Parameter::VectorBlock(index)
                if index < self.spec().vocabulary * self.spec().blocks =>
            {
                Ok((1, index))
            }
            Parameter::Gate(index) if self.read_spec() == ReadSpec::Route => {
                Err(Error::UnimplementedParameter { kind: 3, index })
            }
            Parameter::Gate(index) if (index as usize) < self.gates.len() => Ok((3, index)),
            _ => Err(invalid(
                "proposal parameter index exceeds its declared inventory",
            )),
        }
    }

    pub fn proposal_profiler(&self) -> Result<ProposeProfiler> {
        let event = || {
            self.context
                .new_event(Some(cudarc::driver::sys::CUevent_flags::CU_EVENT_DEFAULT))
                .map_err(cuda)
        };
        Ok(ProposeProfiler {
            model: self.binding.model,
            stream: Arc::clone(&self.stream),
            events: [event()?, event()?, event()?, event()?],
            recorded: [false; 4],
            started: None,
            boundary: None,
            host: HostTimings::default(),
            counterfactual_status: None,
            fault_losses: None,
        })
    }

    pub fn propose_profiled(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profiler: &mut ProposeProfiler,
    ) -> Result<Proposal> {
        profiler.begin();
        if profiler.model != self.binding.model || !Arc::ptr_eq(&profiler.stream, &self.stream) {
            return Err(invalid("proposal profiler belongs to another model/stream"));
        }
        let result = self.propose_inner(batch, targets, parameter, Some(profiler), None);
        if result.is_ok() {
            profiler.mark(8);
        }
        if let Some(started) = profiler.started {
            profiler.host.call_ns = if result.is_ok() {
                profiler
                    .boundary
                    .map(|end| end.duration_since(started).as_nanos())
                    .unwrap_or(0)
            } else {
                started.elapsed().as_nanos()
            };
        }
        result
    }

    pub fn propose(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
    ) -> Result<Proposal> {
        self.propose_inner(batch, targets, parameter, None, None)
    }

    fn propose_inner(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        profiler: Option<&mut ProposeProfiler>,
        decomposition: Option<&mut DecompositionProfiler>,
    ) -> Result<Proposal> {
        self.propose_inner_with_reuse(batch, targets, parameter, profiler, decomposition, None)
    }

    fn propose_inner_with_reuse(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
        mut profiler: Option<&mut ProposeProfiler>,
        decomposition: Option<&mut DecompositionProfiler>,
        mut reuse: Option<&mut BaselineReuseProfiler>,
    ) -> Result<Proposal> {
        let (kind, index) = self.parameter(parameter)?;
        let epoch = self.admit_batch(batch, targets)?;
        if let Some(profile) = profiler.as_deref_mut() {
            profile.mark(0);
        }
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        if let Some(profile) = profiler.as_deref_mut() {
            profile.mark(1);
        }
        // Fixed sixteen-word ABI: Gate3 writes active ordinals0..2 and pads
        // 3..15 with MAX. Its chooser inspects only the three active entries;
        // geometric parameter kinds use all sixteen. Every slot is written.
        let mut losses = self.stream.alloc_zeros::<u64>(16).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        if let Some(profile) = profiler.as_deref_mut() {
            profile.mark(2);
        }
        let diagnostic_kernel = decomposition.as_ref().map(|probe| probe.kernel.clone());
        let reuse_kernel = reuse.as_ref().map(|probe| probe.kernel.clone());
        if let Some(probe) = reuse.as_deref_mut() {
            // Reset cost is inside both the enclosing call and R event span.
            // The same audit buffer/image is used in controls and timing.
            if let Some(profile) = profiler.as_deref_mut() {
                profile.event(0)?;
            }
            self.stream.memset_zeros(&mut probe.records).map_err(cuda)?;
        }
        let mut launch = self.stream.launch_builder(
            diagnostic_kernel
                .as_ref()
                .or(reuse_kernel.as_ref())
                .unwrap_or(&self.kernels.counterfactual),
        );
        launch
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&batch.ids)
            .arg(&batch.vectors)
            .arg(&targets)
            .arg(
                batch
                    .plan
                    .as_ref()
                    .ok_or_else(|| invalid("missing admitted row plans"))?,
            )
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&kind)
            .arg(&index)
            .arg(&mut losses)
            .arg(&mut status);
        if let Some(probe) = decomposition {
            launch.arg(&mut probe.records);
            probe.submitted = true;
        }
        if let Some(probe) = reuse.as_deref_mut() {
            launch.arg(&mut probe.records);
        }
        // SAFETY: ordinary ABI, diagnostic ABI with 16*40 extra words, or
        // opt-in reuse ABI with 160 extra audit words and static shared512.
        // Same-model admission binds every input and the exact one-block,
        // sixteen-thread launch; no dynamic shared memory is requested.
        if reuse_kernel.is_none() {
            if let Some(profile) = profiler.as_deref_mut() {
                profile.event(0)?;
            }
        }
        unsafe { launch.launch(launch_config(16)) }.map_err(cuda)?;
        if let Some(probe) = reuse {
            probe.submitted = true;
        }
        if let Some(profile) = profiler.as_deref_mut() {
            profile.event(1)?;
            profile.mark(3);
        }
        if let Some(profile) = profiler.as_deref_mut() {
            let words = self.stream.clone_dtoh(&status).map_err(cuda)?;
            let code = *words
                .first()
                .ok_or_else(|| invalid("missing CUDA status metadata"))?;
            profile.counterfactual_status = Some(code);
            if code != 0 {
                profile.fault_losses = Some(self.stream.clone_dtoh(&losses).map_err(cuda)?);
                return Err(invalid(format!(
                    "discrete CUDA counterfactual refused with status {code}"
                )));
            }
        } else {
            check_status(&self.stream, &status, "counterfactual")?;
        }
        if let Some(profile) = profiler.as_deref_mut() {
            profile.mark(4);
        }
        let mut proposal = self.stream.alloc_zeros::<u64>(5).map_err(cuda)?;
        if let Some(profile) = profiler.as_deref_mut() {
            profile.mark(5);
        }
        let mut launch = self.stream.launch_builder(&self.kernels.choose);
        launch
            .arg(&losses)
            .arg(&self.epoch)
            .arg(&self.table)
            .arg(&self.routes)
            .arg(&self.gates)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&kind)
            .arg(&index)
            .arg(&mut proposal);
        // SAFETY: fixed sixteen losses with a kind-bound active count, current
        // parameter/table/routes/gates,
        // geometry/spec and epoch, with five writable proposal words. Word4
        // is a device-computed sensitivity observation, never an apply input.
        if let Some(profile) = profiler.as_deref_mut() {
            profile.event(2)?;
        }
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        if let Some(profile) = profiler.as_deref_mut() {
            profile.event(3)?;
            profile.mark(6);
        }
        self.stream.synchronize().map_err(cuda)?;
        if let Some(profile) = profiler {
            profile.mark(7);
        }
        Ok(Proposal {
            data: proposal,
            losses,
            parameter,
            binding: self.binding.clone(),
            epoch,
            stream: Arc::clone(&self.stream),
        })
    }

    /// Future authorized commit path. Corpus-training CLI remains held;
    /// finite proposal qualification must not call this method.
    pub(crate) fn apply(&mut self, proposal: &Proposal) -> Result<bool> {
        let (kind, index) = self.parameter(proposal.parameter)?;
        if proposal.binding != self.binding {
            return Err(invalid("proposal belongs to another model/source/geometry"));
        }
        let before = self.epoch()?;
        if proposal.epoch != before || proposal.observation()?.epoch != before {
            return Err(invalid("proposal is stale at the current model epoch"));
        }
        let spec = self.gpu_spec(1)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.apply);
        launch
            .arg(&mut self.table)
            .arg(&mut self.routes)
            .arg(&mut self.gates)
            .arg(&self.geometry)
            .arg(&spec)
            .arg(&proposal.data)
            .arg(&kind)
            .arg(&index)
            .arg(&mut self.epoch)
            .arg(&mut status);
        // SAFETY: opaque same-model/epoch proposal, admitted parameter index,
        // and exclusively borrowed table/routes/epoch buffers. The CUDA
        // kernel decides whether to commit; host code never selects a winner.
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "apply")?;
        Ok(self.epoch()? != before)
    }
}

fn aggregate_gpu(
    stream: &Arc<CudaStream>,
    kernel: &CudaFunction,
    metrics: &CudaSlice<u64>,
    count: u32,
    width: u32,
    summaries: bool,
) -> Result<Vec<u64>> {
    let summaries = u32::from(summaries);
    let mut out = stream
        .alloc_zeros::<u64>(EVALUATION_AGGREGATE_WORDS)
        .map_err(cuda)?;
    let mut status = stream.alloc_zeros::<u32>(1).map_err(cuda)?;
    let mut launch = stream.launch_builder(kernel);
    launch
        .arg(metrics)
        .arg(&count)
        .arg(&width)
        .arg(&summaries)
        .arg(&mut out)
        .arg(&mut status);
    // SAFETY: aggregate's exact six-argument ABI; count*14/count*1020 immutable
    // receipt words and 1020 writable outputs, owned by this CUDA context.
    unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
    check_status(stream, &status, "evaluation aggregation")?;
    stream.clone_dtoh(&out).map_err(cuda)
}

fn count_metrics(
    stream: &Arc<CudaStream>,
    histogram: &CudaFunction,
    score: &CudaFunction,
    vocabulary: u32,
    source: &[u32],
    targets: &CudaSlice<u32>,
    eval_count: u32,
) -> Result<CountMetrics> {
    let count = u32::try_from(source.len())
        .map_err(|_| invalid("count inventory exceeds CUDA index bound"))?;
    let source = stream.clone_htod(source).map_err(cuda)?;
    let mut counts = stream
        .alloc_zeros::<u64>(vocabulary as usize)
        .map_err(cuda)?;
    let mut status = stream.alloc_zeros::<u32>(1).map_err(cuda)?;
    let mut launch = stream.launch_builder(histogram);
    launch
        .arg(&source)
        .arg(&count)
        .arg(&vocabulary)
        .arg(&mut counts)
        .arg(&mut status);
    // SAFETY: count valid IDs; zeroed vocabulary-sized u64 histogram, immutable
    // IDs and disjoint status. Each CUDA thread increments its admitted token.
    unsafe { launch.launch(launch_config(count)) }.map_err(cuda)?;
    check_status(stream, &status, "count histogram")?;
    let mut top8 = stream.alloc_zeros::<u32>(8).map_err(cuda)?;
    let mut metrics = stream.alloc_zeros::<u64>(6).map_err(cuda)?;
    let mut launch = stream.launch_builder(score);
    launch
        .arg(&counts)
        .arg(&vocabulary)
        .arg(targets)
        .arg(&eval_count)
        .arg(&mut top8)
        .arg(&mut metrics)
        .arg(&mut status);
    // SAFETY: same context/checked inventories, eight IDs and six exact metric
    // outputs. Count sorting and scored-target reductions run only on CUDA.
    unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
    check_status(stream, &status, "count scoring")?;
    let metrics = stream.clone_dtoh(&metrics).map_err(cuda)?;
    let metrics: [u64; 6] = metrics
        .try_into()
        .map_err(|_| invalid("count receipt must contain six words"))?;
    let top8 = stream
        .clone_dtoh(&top8)
        .map_err(cuda)?
        .try_into()
        .map_err(|_| invalid("count top8 must contain eight IDs"))?;
    Ok(CountMetrics {
        token_counts: stream.clone_dtoh(&counts).map_err(cuda)?,
        top8,
        scored_rows: metrics[0],
        top1_correct: metrics[1],
        top8_correct: metrics[2],
        zero_count_targets: metrics[3],
        source_targets: metrics[4],
        nonzero_tokens: metrics[5],
    })
}

fn read_epoch(stream: &Arc<CudaStream>, epoch: &CudaSlice<u64>) -> Result<u64> {
    stream
        .clone_dtoh(epoch)
        .map_err(cuda)?
        .first()
        .copied()
        .ok_or_else(|| invalid("missing model epoch metadata"))
}

fn check_status(stream: &Arc<CudaStream>, status: &CudaSlice<u32>, operation: &str) -> Result<()> {
    let code = stream
        .clone_dtoh(status)
        .map_err(cuda)?
        .first()
        .copied()
        .ok_or_else(|| invalid("missing CUDA status metadata"))?;
    if code == 0 {
        Ok(())
    } else {
        Err(invalid(format!(
            "discrete CUDA {operation} refused with status {code}"
        )))
    }
}

#[cfg(test)]
mod snapshot_hash_tests {
    use super::*;

    // Serializer fixture only. These literal arrays are not evaluated or
    // admitted as a model, and no CUDA context or Device is constructed.
    fn snapshot() -> Snapshot {
        Snapshot {
            schema: SCHEMA.into(),
            spec: Spec {
                vocabulary: 2,
                blocks: 2,
                context: 2,
                kept_cells: 2,
            },
            read_spec: ReadSpec::Route,
            objective_policy: ObjectivePolicy::NearestAddressMarginV1,
            read_initialization: ReadInitialization::ZeroGatesV1,
            seed: 1,
            epoch: 0,
            table: vec![0; 16],
            routes: vec![0; 2],
            gates: vec![0; 2],
            operator_kappa: "literal-metadata".into(),
            composition_kappa: "literal-metadata".into(),
            kernel_sha256: "1".repeat(64),
            source_sha256: "2".repeat(64),
            geometry_sha256: "3".repeat(64),
        }
    }

    #[test]
    fn one_snapshot_retains_both_encoding_domains() -> Result<()> {
        let snapshot = snapshot();
        let stored = serde_json::to_value(&snapshot).map_err(|e| invalid(e.to_string()))?;
        let hashes = snapshot_hash_domains(&snapshot)?;
        assert_eq!(hashes, stored_snapshot_hash_domains(&stored)?);
        assert_eq!(
            hashes.model_storage_sha256,
            crate::corpus::sha256(
                &serde_json::to_vec(&stored).map_err(|e| invalid(e.to_string()))?
            )
        );
        assert_eq!(
            hashes.model_snapshot_sha256,
            crate::corpus::sha256(
                &serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?
            )
        );
        assert_ne!(hashes.model_storage_sha256, hashes.model_snapshot_sha256);
        Ok(())
    }

    #[test]
    fn stored_domain_is_not_normalized_and_malformed_snapshot_refuses() -> Result<()> {
        let mut stored = serde_json::to_value(snapshot()).map_err(|e| invalid(e.to_string()))?;
        let original = stored_snapshot_hash_domains(&stored)?;
        // Explicitly writing a default changes Domain A, while typed encoding
        // omits that default and therefore leaves Domain B unchanged.
        stored["read_initialization"] = serde_json::json!(ReadInitialization::ZeroGatesV1);
        let explicit = stored_snapshot_hash_domains(&stored)?;
        assert_ne!(original.model_storage_sha256, explicit.model_storage_sha256);
        assert_eq!(
            original.model_snapshot_sha256,
            explicit.model_snapshot_sha256
        );
        stored["unexpected"] = serde_json::json!(true);
        assert!(stored_snapshot_hash_domains(&stored).is_err());
        assert!(stored_snapshot_hash_domains(&serde_json::json!({})).is_err());
        Ok(())
    }
}
