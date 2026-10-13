//! CUDA driver boundary for the integer SpiralCore/FBS path.
//! Host work admits dimensions/identities and handles storage/observations.
//! Initialization, encoding, routing, scoring and proposal choice run on CUDA.
//! Module loading consumes AOT PTX; there is no runtime compiler or CPU model.

use crate::{
    metadata::{BinderPlan, ABSENT},
    Error, Identity, Parameter, ReadSpec, Result, Spec, SCHEMA,
};
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, LaunchConfig, Ptx, PushKernelArg,
};
use sha2::{Digest, Sha256};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

mod geometry {
    include!(concat!(env!("OUT_DIR"), "/geometry.rs"));
}
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/discrete.ptx"));
static NEXT_MODEL: AtomicU64 = AtomicU64::new(1);

/// Trace ABI from the production flock read, ordered by row then payload block:
/// header [frame_root, output_root, residual_terms, query_payload_root], followed
/// by one level record [gate+1, kept_count, consensus_root, eligible_count,
/// kept_node_ids[8], action_ids[8], carried_root_ids[8]]. Unused slots are ABSENT.
pub const FLOCK_TRACE_HEADER_WORDS: usize = 4;
pub const FLOCK_TRACE_LEVEL_WORDS: usize = 28;
pub const FLOCK_TRACE_KEPT_CAPACITY: usize = 8;

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
        self.trace
            .as_ref()
            .map(|trace| self.stream.clone_dtoh(trace).map_err(cuda))
            .transpose()
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
}

/// Artifact storage only. Values are admitted on the GPU when loaded; no
/// host score, route or transport is computed during serialization.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub schema: String,
    pub spec: Spec,
    pub read_spec: ReadSpec,
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
    source.update(include_bytes!("../build.rs"));
    source.update(PTX.as_bytes());
    source.finalize().into()
}

fn hex_digest(digest: &[u8; 32]) -> String {
    digest.iter().map(|b| format!("{b:02x}")).collect()
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
        let words: [u64; 4] = words
            .try_into()
            .map_err(|_| invalid("proposal receipt must contain four words"))?;
        Ok(ProposalObservation {
            candidate: words[0],
            before: words[1],
            after: words[2],
            epoch: words[3],
        })
    }
}

struct Kernels {
    init: CudaFunction,
    encode: CudaFunction,
    forward: CudaFunction,
    counterfactual: CudaFunction,
    choose: CudaFunction,
    apply: CudaFunction,
    validate: CudaFunction,
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

fn make_binding(spec: Spec, read_spec: ReadSpec, seed: u32, geometry: &[u32]) -> Result<Binding> {
    let model = NEXT_MODEL
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| x.checked_add(1))
        .map_err(|_| invalid("process model identity exhausted"))?;
    Ok(Binding {
        model,
        spec,
        read_spec,
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
        read_spec.validate(spec)?;
        let metadata = BinderPlan::new(spec.context as usize)?;
        let packed = packed_geometry();
        if packed.len() != 5120 {
            return Err(invalid("generated geometry ABI inventory differs"));
        }
        let binding = make_binding(spec, read_spec, seed, &packed)?;
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let stream = context.default_stream();
        let module = context.load_module(Ptx::from_src(PTX)).map_err(cuda)?;
        let kernels = Kernels {
            init: module.load_function("discrete_init").map_err(cuda)?,
            encode: module.load_function("discrete_encode").map_err(cuda)?,
            forward: module.load_function("discrete_forward").map_err(cuda)?,
            counterfactual: module
                .load_function("discrete_counterfactual")
                .map_err(cuda)?,
            choose: module.load_function("discrete_choose").map_err(cuda)?,
            apply: module.load_function("discrete_apply").map_err(cuda)?,
            validate: module.load_function("discrete_validate").map_err(cuda)?,
        };
        let geometry = stream.clone_htod(&packed).map_err(cuda)?;
        let width = spec.blocks as usize * 8;
        let mut table = stream
            .alloc_zeros::<i32>(spec.vocabulary as usize * width)
            .map_err(cuda)?;
        let mut routes = stream
            .alloc_zeros::<u32>(spec.blocks as usize * metadata.stages())
            .map_err(cuda)?;
        let mut gates = stream
            .alloc_zeros::<i32>(spec.blocks as usize * metadata.stages())
            .map_err(cuda)?;
        let epoch = stream.alloc_zeros::<u64>(1).map_err(cuda)?;
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
                read_spec.words()[0],
                read_spec.words()[1],
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
    pub fn epoch(&self) -> Result<u64> {
        read_epoch(&self.stream, &self.epoch)
    }
    pub fn synchronize(&self) -> Result<()> {
        self.context.synchronize().map_err(cuda)
    }

    pub fn snapshot(&self) -> Result<Snapshot> {
        // This is storage readback, not a CPU reference or a learning update.
        Ok(Snapshot {
            schema: SCHEMA.to_owned(),
            spec: self.spec(),
            read_spec: self.read_spec(),
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
        if snapshot.table.len()
            != snapshot.spec.vocabulary as usize * snapshot.spec.blocks as usize * 8
            || snapshot.routes.len() != snapshot.spec.blocks as usize * metadata.stages()
            || snapshot.gates.len() != snapshot.spec.blocks as usize * metadata.stages()
        {
            return Err(invalid(
                "snapshot codebook/route/gate allocation inventory differs",
            ));
        }
        // Fresh process identity even for byte-identical reloads. No carried
        // batch/proposal from another model instance becomes valid by loading.
        let mut model =
            Self::new_with_read(snapshot.spec, snapshot.read_spec, snapshot.seed, ordinal)?;
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
                self.read_spec().words()[0],
                self.read_spec().words()[1],
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
        let lengths = if identities.len() % context == 0 && identities.len() / context <= 64 {
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
        let mut trace = if let Some(stride) = trace_stride {
            let words = (batch.rows as usize)
                .checked_mul(self.spec().blocks as usize)
                .and_then(|blocks| blocks.checked_mul(stride))
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
        // is either the exact rows*blocks*(4+stages*28) allocation or a null
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
            binding: self.binding.clone(),
            epoch,
            stream: Arc::clone(&self.stream),
        })
    }

    fn parameter(&self, parameter: Parameter) -> Result<(u32, u32)> {
        match parameter {
            Parameter::Route(_) if matches!(self.read_spec(), ReadSpec::Flock { .. }) => {
                Err(invalid("route parameters are inactive in a flock model"))
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

    pub fn propose(
        &self,
        batch: &CarriedBatch,
        targets: &[u32],
        parameter: Parameter,
    ) -> Result<Proposal> {
        let (kind, index) = self.parameter(parameter)?;
        let epoch = self.admit_batch(batch, targets)?;
        let targets = self.stream.clone_htod(targets).map_err(cuda)?;
        let spec = self.gpu_spec(batch.rows)?;
        // Fixed sixteen-word ABI: Gate3 writes active ordinals0..2 and pads
        // 3..15 with MAX. Its chooser inspects only the three active entries;
        // geometric parameter kinds use all sixteen. Every slot is written.
        let mut losses = self.stream.alloc_zeros::<u64>(16).map_err(cuda)?;
        let mut status = self.stream.alloc_zeros::<u32>(1).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.kernels.counterfactual);
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
        // SAFETY: exact counterfactual ABI; parameter and input inventories
        // admitted; sixteen loss slots are scored/padded on CUDA as above.
        unsafe { launch.launch(launch_config(16)) }.map_err(cuda)?;
        check_status(&self.stream, &status, "counterfactual")?;
        let mut proposal = self.stream.alloc_zeros::<u64>(4).map_err(cuda)?;
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
        // geometry/spec and epoch, with four writable proposal words.
        unsafe { launch.launch(launch_config(1)) }.map_err(cuda)?;
        self.stream.synchronize().map_err(cuda)?;
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
    pub fn apply(&mut self, proposal: &Proposal) -> Result<bool> {
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
