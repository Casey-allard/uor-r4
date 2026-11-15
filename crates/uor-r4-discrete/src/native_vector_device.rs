//! AOT CUDA handle for native Q31 Hamilton vector transport and training steps.
//! The module contains only admitted integer entry points and no PTX/JIT fallback.

use crate::native_vector::{
    q31_batch_len_admitted, q31_coordinate_admitted, q31_hamilton_rotate, q31_hamilton_train_step,
    q31_quaternion_admitted, q31_read_credit_shift, Q31FirstPickRead, Q31HamiltonModel,
    Q31ModelStep, Q31Quaternion, Q31ReadOutput, Q31ReadTrainStep, Q31TrainingStep, Q31Transport,
    MAX_Q31_BATCH_ITEMS, MAX_Q31_STEP_SHIFT,
};
use crate::{Error, Result};
use cudarc::driver::{
    CudaContext, CudaFunction, CudaSlice, CudaStream, LaunchConfig, Ptx, PushKernelArg,
};
use sha2::{Digest, Sha256};
use std::sync::Arc;

const SOURCE: &str = include_str!("native_vector.cu");
const PTX: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector.ptx"));
const CUBIN: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/native-vector.cubin"));
const RESOURCES: &str = include_str!(concat!(env!("OUT_DIR"), "/native-vector-resources.txt"));
const TARGET: &str = include_str!(concat!(env!("OUT_DIR"), "/cuda-image-target.txt"));
const ENTRY_ROTATE: &str = "native_q31_hamilton_rotate";
const ENTRY_TRAIN_STEP: &str = "native_q31_hamilton_train_step";
const ENTRY_READ_ADDRESS: &str = "native_q31_read_address";
const ENTRY_FIRST_PICK_READ: &str = "native_q31_first_pick_read";
const ENTRY_READ_PICK_LOSS: &str = "native_q31_read_pick_loss";
const ENTRY_READ_GATE_CREDIT_PARTIAL: &str = "native_q31_read_gate_credit_partial";
const ENTRY_READ_GATE_APPLY: &str = "native_q31_read_gate_apply";
/// Threads per block for every native-vector launch.
const BLOCK_THREADS: u32 = 256;

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}

fn cuda(error: impl std::fmt::Display) -> Error {
    Error::Cuda(error.to_string())
}

/// Source, PTX, cubin and ptxas-report identities for the embedded image.
pub fn compiled_native_vector_image() -> [String; 4] {
    [
        format!("{:x}", Sha256::digest(SOURCE.as_bytes())),
        format!("{:x}", Sha256::digest(PTX.as_bytes())),
        format!("{:x}", Sha256::digest(CUBIN)),
        format!("{:x}", Sha256::digest(stable_resource_report().as_bytes())),
    ]
}

/// The compiler's stopwatch is not part of the artifact identity.
fn stable_resource_report() -> String {
    RESOURCES
        .lines()
        .filter(|line| !line.contains("Compile time ="))
        .collect::<Vec<_>>()
        .join("\n")
}

/// One explicit RTX-class CUDA context for native integer vector training.
pub struct NativeVectorEngine {
    _context: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    rotate: CudaFunction,
    train_step: CudaFunction,
    read_address: CudaFunction,
    first_pick_read: CudaFunction,
    read_pick_loss: CudaFunction,
    read_gate_credit_partial: CudaFunction,
    read_gate_apply: CudaFunction,
}

/// One batch's addresses and pick loss under one pair of gate sets.
struct PickLoss {
    loss: u128,
    rivals: CudaSlice<u32>,
    residuals: CudaSlice<i64>,
}

fn row_config(rows: u32) -> LaunchConfig {
    LaunchConfig {
        grid_dim: (rows.div_ceil(BLOCK_THREADS), 1, 1),
        block_dim: (BLOCK_THREADS, 1, 1),
        shared_mem_bytes: 0,
    }
}

impl NativeVectorEngine {
    /// Load the admitted `native-vector.cubin`; no PTX fallback is attempted.
    pub fn new(ordinal: usize) -> Result<Self> {
        let context = CudaContext::new(ordinal).map_err(cuda)?;
        let (major, minor) = context.compute_capability().map_err(cuda)?;
        if TARGET.trim() != format!("sm_{major}{minor}") {
            return Err(invalid(format!(
                "native-vector cubin target {} differs from device {major}.{minor}",
                TARGET.trim()
            )));
        }
        let stream = context.default_stream();
        let module = context
            .load_module(Ptx::from_binary(CUBIN.to_vec()))
            .map_err(cuda)?;
        let rotate = module.load_function(ENTRY_ROTATE).map_err(cuda)?;
        let train_step = module.load_function(ENTRY_TRAIN_STEP).map_err(cuda)?;
        let read_address = module.load_function(ENTRY_READ_ADDRESS).map_err(cuda)?;
        let first_pick_read = module.load_function(ENTRY_FIRST_PICK_READ).map_err(cuda)?;
        let read_pick_loss = module.load_function(ENTRY_READ_PICK_LOSS).map_err(cuda)?;
        let read_gate_credit_partial = module
            .load_function(ENTRY_READ_GATE_CREDIT_PARTIAL)
            .map_err(cuda)?;
        let read_gate_apply = module.load_function(ENTRY_READ_GATE_APPLY).map_err(cuda)?;
        Ok(Self {
            _context: context,
            stream,
            rotate,
            train_step,
            read_address,
            first_pick_read,
            read_pick_loss,
            read_gate_credit_partial,
            read_gate_apply,
        })
    }

    /// Rotate one batch of Q31 quaternions on CUDA and return raw Q31 words.
    pub fn hamilton_rotate(
        &self,
        gates: &[Q31Quaternion],
        values: &[Q31Quaternion],
    ) -> Result<Vec<Q31Transport>> {
        if gates.is_empty() || gates.len() != values.len() {
            return Err(invalid(
                "native-vector transport needs a nonempty equal-length batch",
            ));
        }
        if !q31_batch_len_admitted(gates.len()) {
            return Err(invalid(
                "native-vector batch exceeds the admitted 2^30-item bound",
            ));
        }
        if gates
            .iter()
            .chain(values)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(invalid(
                "native-vector Q31 coordinates must be in -2^31..=2^31",
            ));
        }
        let count = u32::try_from(gates.len())
            .map_err(|_| invalid("native-vector batch exceeds the CUDA index bound"))?;
        let gate_words: Vec<i64> = gates.iter().flatten().copied().collect();
        let value_words: Vec<i64> = values.iter().flatten().copied().collect();
        let gate_device = self.stream.clone_htod(&gate_words).map_err(cuda)?;
        let value_device = self.stream.clone_htod(&value_words).map_err(cuda)?;
        let mut output = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.rotate);
        launch
            .arg(&gate_device)
            .arg(&value_device)
            .arg(&mut output)
            .arg(&count);
        // SAFETY: every buffer carries exactly `count * 4` words on this stream.
        unsafe {
            launch.launch(LaunchConfig {
                grid_dim: (count.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
        }
        .map_err(cuda)?;
        let words: Vec<i64> = self.stream.clone_dtoh(&output).map_err(cuda)?;
        Ok(words
            .chunks_exact(4)
            .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
            .collect())
    }

    /// Run the Rust oracle on one quaternion in the same batch representation.
    pub fn hamilton_rotate_reference(
        gate: Q31Quaternion,
        value: Q31Quaternion,
    ) -> Result<Q31Transport> {
        Ok(q31_hamilton_rotate(gate, value)?)
    }

    /// Apply one bounded Q31 Hamilton training step per item on CUDA.
    pub fn hamilton_train_step(
        &self,
        gates: &[Q31Quaternion],
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
        step_shift: u32,
    ) -> Result<Vec<Q31TrainingStep>> {
        if gates.is_empty() || gates.len() != values.len() || gates.len() != targets.len() {
            return Err(invalid(
                "native-vector training needs nonempty equal-length gate, value and target batches",
            ));
        }
        if !q31_batch_len_admitted(gates.len()) {
            return Err(invalid(
                "native-vector training batch exceeds the admitted 2^30-item bound",
            ));
        }
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(invalid("native-vector step_shift must be in 0..=31"));
        }
        if gates
            .iter()
            .chain(values)
            .chain(targets)
            .any(|quaternion| !q31_quaternion_admitted(*quaternion))
        {
            return Err(invalid(
                "native-vector Q31 training words must be in -2^31..=2^31",
            ));
        }
        let count = u32::try_from(gates.len())
            .map_err(|_| invalid("native-vector training batch exceeds the CUDA index bound"))?;
        let gate_words: Vec<i64> = gates.iter().flatten().copied().collect();
        let value_words: Vec<i64> = values.iter().flatten().copied().collect();
        let target_words: Vec<i64> = targets.iter().flatten().copied().collect();
        let gate_device = self.stream.clone_htod(&gate_words).map_err(cuda)?;
        let value_device = self.stream.clone_htod(&value_words).map_err(cuda)?;
        let target_device = self.stream.clone_htod(&target_words).map_err(cuda)?;
        let mut predictions = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut candidates = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut accepted = self
            .stream
            .alloc_zeros::<i64>(gates.len() * 4)
            .map_err(cuda)?;
        let mut loss_before = self.stream.alloc_zeros::<u64>(gates.len()).map_err(cuda)?;
        let mut loss_after = self.stream.alloc_zeros::<u64>(gates.len()).map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.train_step);
        launch
            .arg(&gate_device)
            .arg(&value_device)
            .arg(&target_device)
            .arg(&count)
            .arg(&step_shift)
            .arg(&mut predictions)
            .arg(&mut candidates)
            .arg(&mut accepted)
            .arg(&mut loss_before)
            .arg(&mut loss_after);
        unsafe {
            launch.launch(LaunchConfig {
                grid_dim: (count.div_ceil(256), 1, 1),
                block_dim: (256, 1, 1),
                shared_mem_bytes: 0,
            })
        }
        .map_err(cuda)?;
        let predictions: Vec<i64> = self.stream.clone_dtoh(&predictions).map_err(cuda)?;
        let candidates: Vec<i64> = self.stream.clone_dtoh(&candidates).map_err(cuda)?;
        let accepted: Vec<i64> = self.stream.clone_dtoh(&accepted).map_err(cuda)?;
        let loss_before: Vec<u64> = self.stream.clone_dtoh(&loss_before).map_err(cuda)?;
        let loss_after: Vec<u64> = self.stream.clone_dtoh(&loss_after).map_err(cuda)?;
        Ok((0..gates.len())
            .map(|index| Q31TrainingStep {
                prediction: [
                    predictions[index * 4],
                    predictions[index * 4 + 1],
                    predictions[index * 4 + 2],
                    predictions[index * 4 + 3],
                ],
                candidate_gate: [
                    candidates[index * 4],
                    candidates[index * 4 + 1],
                    candidates[index * 4 + 2],
                    candidates[index * 4 + 3],
                ],
                accepted_gate: [
                    accepted[index * 4],
                    accepted[index * 4 + 1],
                    accepted[index * 4 + 2],
                    accepted[index * 4 + 3],
                ],
                loss_before: loss_before[index],
                loss_after: loss_after[index],
                accepted: loss_after[index] < loss_before[index],
            })
            .collect())
    }

    /// Run the Rust training-step oracle for one item.
    pub fn hamilton_train_step_reference(
        gate: Q31Quaternion,
        value: Q31Quaternion,
        target: Q31Quaternion,
        step_shift: u32,
    ) -> Result<Q31TrainingStep> {
        Ok(q31_hamilton_train_step(gate, value, target, step_shift)?)
    }

    /// Apply one shared Q31 model step: CUDA per-item credit, then deterministic aggregation.
    pub fn train_model_step(
        &self,
        model: &mut Q31HamiltonModel,
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        if values.is_empty() || values.len() != targets.len() {
            return Err(invalid(
                "native Q31 model step needs nonempty equal-length values and targets",
            ));
        }
        let gates = vec![model.gate(); values.len()];
        let records = self.hamilton_train_step(&gates, values, targets, model.step_shift())?;
        Ok(model.step_from_records(&records, values, targets)?)
    }

    /// Run the shared model step through the Rust oracle.
    pub fn train_model_step_reference(
        model: &mut Q31HamiltonModel,
        values: &[Q31Quaternion],
        targets: &[Q31Quaternion],
    ) -> Result<Q31ModelStep> {
        Ok(model.step_reference(values, targets)?)
    }

    /// Run one first-pick read on CUDA: one thread per row computes both
    /// addresses, then one thread per row picks, reads and adds. The refusals
    /// and their messages are those of [`Q31FirstPickRead::apply`].
    pub fn first_pick_read(
        &self,
        read: &Q31FirstPickRead,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
    ) -> Result<Q31ReadOutput> {
        let width = read.width();
        let rows = batch
            .checked_mul(time)
            .filter(|&rows| rows <= MAX_Q31_BATCH_ITEMS)
            .ok_or_else(|| invalid("native Q31 read admits at most 2^30 rows"))?;
        let words = rows
            .checked_mul(width)
            .ok_or_else(|| invalid("native Q31 read stream length overflows usize"))?;
        if stream.len() != words {
            return Err(invalid(
                "native Q31 read stream must hold batch * time * width words",
            ));
        }
        if ids.len() != rows {
            return Err(invalid("native Q31 read needs batch * time ids"));
        }
        if table.is_empty() || !table.len().is_multiple_of(width) {
            return Err(invalid(
                "native Q31 read table must be a nonempty multiple of width",
            ));
        }
        if !stream
            .iter()
            .chain(table)
            .all(|&word| q31_coordinate_admitted(word))
        {
            return Err(invalid(
                "native Q31 read stream and table words must be in -2^31..=2^31",
            ));
        }
        if rows == 0 {
            return Ok(Q31ReadOutput {
                picks: Vec::new(),
                out: Vec::new(),
            });
        }
        let vocab = (table.len() / width) as u64;
        // `rows <= 2^30` and `width <= 4096`, so all three fit `u32`.
        let rows_u32 = rows as u32;
        let time_u32 = time as u32;
        let width_u32 = width as u32;
        let shift = 31 - read.gain_shift();
        let query_words: Vec<i64> = read.query_gates().iter().flatten().copied().collect();
        let key_words: Vec<i64> = read.key_gates().iter().flatten().copied().collect();
        let stream_device = self.stream.clone_htod(stream).map_err(cuda)?;
        let query_gates = self.stream.clone_htod(&query_words).map_err(cuda)?;
        let key_gates = self.stream.clone_htod(&key_words).map_err(cuda)?;
        let ids_device = self.stream.clone_htod(ids).map_err(cuda)?;
        let table_device = self.stream.clone_htod(table).map_err(cuda)?;
        let weights_device = self.stream.clone_htod(read.weights()).map_err(cuda)?;
        let mut queries = self.stream.alloc_zeros::<i64>(rows * 16).map_err(cuda)?;
        let mut keys = self.stream.alloc_zeros::<i64>(rows * 16).map_err(cuda)?;
        let mut picks = self.stream.alloc_zeros::<u32>(rows).map_err(cuda)?;
        let mut out = self.stream.alloc_zeros::<i64>(words).map_err(cuda)?;
        let config = LaunchConfig {
            grid_dim: (rows_u32.div_ceil(BLOCK_THREADS), 1, 1),
            block_dim: (BLOCK_THREADS, 1, 1),
            shared_mem_bytes: 0,
        };
        let mut launch = self.stream.launch_builder(&self.read_address);
        launch
            .arg(&stream_device)
            .arg(&query_gates)
            .arg(&key_gates)
            .arg(&rows_u32)
            .arg(&width_u32)
            .arg(&mut queries)
            .arg(&mut keys);
        // SAFETY: the stream holds `rows * width` words, each gate buffer
        // `width` words, and each address buffer `rows * 16` words.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.first_pick_read);
        launch
            .arg(&queries)
            .arg(&keys)
            .arg(&stream_device)
            .arg(&ids_device)
            .arg(&table_device)
            .arg(&weights_device)
            .arg(&rows_u32)
            .arg(&time_u32)
            .arg(&width_u32)
            .arg(&vocab)
            .arg(&shift)
            .arg(&mut picks)
            .arg(&mut out);
        // SAFETY: same stream, so the addresses are written before this read;
        // ids and picks hold `rows` words, the table `vocab * width`, and the
        // kernel reads a table row only for an id below `vocab`.
        unsafe { launch.launch(config) }.map_err(cuda)?;
        let picks: Vec<u32> = self.stream.clone_dtoh(&picks).map_err(cuda)?;
        let out: Vec<i64> = self.stream.clone_dtoh(&out).map_err(cuda)?;
        // The CPU refuses at the first row whose picked id is outside the
        // vocab; any such row refuses the whole read here too.
        let refused = (0..rows).any(|row| {
            let t = row % time;
            t != 0 && u64::from(ids[row - t + picks[row] as usize]) >= vocab
        });
        if refused {
            return Err(invalid("native Q31 read id must be below the table vocab"));
        }
        Ok(Q31ReadOutput { picks, out })
    }

    /// Run the first-pick read through the Rust oracle.
    pub fn first_pick_read_reference(
        read: &Q31FirstPickRead,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
    ) -> Result<Q31ReadOutput> {
        Ok(read.apply(stream, ids, table, batch, time)?)
    }

    /// Addresses under `query_words`/`key_words`, then each row's rival,
    /// hinge and residuals; the host sums the loss words exactly in `u128`.
    #[allow(clippy::too_many_arguments)]
    fn pick_loss(
        &self,
        stream: &CudaSlice<i64>,
        query_words: &[i64],
        key_words: &[i64],
        targets: &CudaSlice<u32>,
        rows: u32,
        time: u32,
        width: u32,
        margin: u64,
    ) -> Result<PickLoss> {
        let rows_usize = rows as usize;
        let query_gates = self.stream.clone_htod(query_words).map_err(cuda)?;
        let key_gates = self.stream.clone_htod(key_words).map_err(cuda)?;
        let mut queries = self
            .stream
            .alloc_zeros::<i64>(rows_usize * 16)
            .map_err(cuda)?;
        let mut keys = self
            .stream
            .alloc_zeros::<i64>(rows_usize * 16)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.read_address);
        launch
            .arg(stream)
            .arg(&query_gates)
            .arg(&key_gates)
            .arg(&rows)
            .arg(&width)
            .arg(&mut queries)
            .arg(&mut keys);
        // SAFETY: the stream holds `rows * width` words, each gate buffer
        // `width` words, and each address buffer `rows * 16` words.
        unsafe { launch.launch(row_config(rows)) }.map_err(cuda)?;
        let mut rivals = self.stream.alloc_zeros::<u32>(rows_usize).map_err(cuda)?;
        let mut loss = self
            .stream
            .alloc_zeros::<u64>(rows_usize * 2)
            .map_err(cuda)?;
        let mut residuals = self
            .stream
            .alloc_zeros::<i64>(rows_usize * 48)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.read_pick_loss);
        launch
            .arg(&queries)
            .arg(&keys)
            .arg(targets)
            .arg(&rows)
            .arg(&time)
            .arg(&margin)
            .arg(&mut rivals)
            .arg(&mut loss)
            .arg(&mut residuals);
        // SAFETY: same stream, so the addresses are written first; targets,
        // rivals hold `rows` words, loss `2 * rows`, residuals `48 * rows`,
        // and every target was checked to be an earlier row of its window.
        unsafe { launch.launch(row_config(rows)) }.map_err(cuda)?;
        let words: Vec<u64> = self.stream.clone_dtoh(&loss).map_err(cuda)?;
        let loss = words
            .chunks_exact(2)
            .map(|pair| u128::from(pair[0]) | (u128::from(pair[1]) << 64))
            .sum();
        Ok(PickLoss {
            loss,
            rivals,
            residuals,
        })
    }

    /// One pick-margin update of the read's gates on CUDA, the twin of
    /// [`Q31FirstPickRead::train_pick_step`]: the same refusals in the same
    /// order, the rivals, hinges and residuals per row on the device, each
    /// candidate gate word on the device, the loss sum and the accept on
    /// the host, and both pick lists from [`Self::first_pick_read`].
    #[allow(clippy::too_many_arguments)]
    pub fn read_train_pick_step(
        &self,
        read: &Q31FirstPickRead,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
        targets: &[u32],
        margin: u64,
        step_shift: u32,
    ) -> Result<Q31ReadTrainStep> {
        let picks_before = self
            .first_pick_read(read, stream, ids, table, batch, time)?
            .picks;
        if targets.len() != picks_before.len() {
            return Err(invalid(
                "native Q31 read training needs batch * time targets",
            ));
        }
        if step_shift > MAX_Q31_STEP_SHIFT {
            return Err(invalid(
                "native Q31 read training step shift must be in 0..=31",
            ));
        }
        for (row, &target) in targets.iter().enumerate() {
            if target as usize > row % time {
                return Err(invalid(
                    "native Q31 read training target must be an earlier row of its window",
                ));
            }
        }
        if targets.is_empty() {
            // No rows: no credit, a zero loss on both sides, and no step.
            return Ok(Q31ReadTrainStep {
                read: read.clone(),
                accepted: false,
                loss_before: 0,
                loss_after: 0,
                picks_before: Vec::new(),
                picks_after: Vec::new(),
                rivals: Vec::new(),
            });
        }
        let width = read.width();
        // `first_pick_read` admitted `rows <= 2^30` and `width <= 4096`.
        let rows = targets.len() as u32;
        let time_u32 = time as u32;
        let width_u32 = width as u32;
        let credit_shift = q31_read_credit_shift(stream);
        let words =
            |gates: &[Q31Quaternion]| -> Vec<i64> { gates.iter().flatten().copied().collect() };
        let query_words = words(read.query_gates());
        let key_words = words(read.key_gates());
        let stream_device = self.stream.clone_htod(stream).map_err(cuda)?;
        let targets_device = self.stream.clone_htod(targets).map_err(cuda)?;

        let before = self.pick_loss(
            &stream_device,
            &query_words,
            &key_words,
            &targets_device,
            rows,
            time_u32,
            width_u32,
            margin,
        )?;
        let query_gates = self.stream.clone_htod(&query_words).map_err(cuda)?;
        let key_gates = self.stream.clone_htod(&key_words).map_err(cuda)?;
        let mut candidate_query = self.stream.alloc_zeros::<i64>(width).map_err(cuda)?;
        let mut candidate_key = self.stream.alloc_zeros::<i64>(width).map_err(cuda)?;
        // claude-393: each window's exact credit sum per gate word, then the
        // windows' sums added in window order; integer sums cannot move a word.
        let windows = batch as u32;
        let partial_items = (2 * width)
            .checked_mul(batch)
            .and_then(|items| u32::try_from(items).ok())
            .ok_or_else(|| {
                invalid("native Q31 read training on CUDA needs 2 * width * batch to fit u32")
            })?;
        let mut partials = self
            .stream
            .alloc_zeros::<u64>(2 * partial_items as usize)
            .map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.read_gate_credit_partial);
        launch
            .arg(&stream_device)
            .arg(&before.residuals)
            .arg(&targets_device)
            .arg(&before.rivals)
            .arg(&windows)
            .arg(&time_u32)
            .arg(&width_u32)
            .arg(&credit_shift)
            .arg(&mut partials);
        // SAFETY: same stream, so the residuals and rivals are written first;
        // `rows = windows * time` (checked by `first_pick_read`), each item
        // writes its own two words of `partials`, and a row's target and rival
        // index rows of its own window of the stream.
        unsafe { launch.launch(row_config(partial_items)) }.map_err(cuda)?;
        let mut launch = self.stream.launch_builder(&self.read_gate_apply);
        launch
            .arg(&partials)
            .arg(&query_gates)
            .arg(&key_gates)
            .arg(&windows)
            .arg(&width_u32)
            .arg(&step_shift)
            .arg(&mut candidate_query)
            .arg(&mut candidate_key);
        // SAFETY: same stream, so every partial is written first; each gate
        // and candidate buffer holds `width` words.
        unsafe { launch.launch(row_config(2 * width_u32)) }.map_err(cuda)?;
        let rivals: Vec<u32> = self.stream.clone_dtoh(&before.rivals).map_err(cuda)?;
        let candidate_query: Vec<i64> = self.stream.clone_dtoh(&candidate_query).map_err(cuda)?;
        let candidate_key: Vec<i64> = self.stream.clone_dtoh(&candidate_key).map_err(cuda)?;

        let quaternions = |words: &[i64]| -> Vec<Q31Quaternion> {
            words
                .chunks_exact(4)
                .map(|chunk| [chunk[0], chunk[1], chunk[2], chunk[3]])
                .collect()
        };
        let candidate = Q31FirstPickRead::new(
            quaternions(&candidate_query),
            quaternions(&candidate_key),
            read.weights().to_vec(),
            read.gain_shift(),
        )?;
        let loss_after = self
            .pick_loss(
                &stream_device,
                &candidate_query,
                &candidate_key,
                &targets_device,
                rows,
                time_u32,
                width_u32,
                margin,
            )?
            .loss;
        let accepted = loss_after < before.loss;
        let kept = if accepted { candidate } else { read.clone() };
        let picks_after = self
            .first_pick_read(&kept, stream, ids, table, batch, time)?
            .picks;
        Ok(Q31ReadTrainStep {
            read: kept,
            accepted,
            loss_before: before.loss,
            loss_after,
            picks_before,
            picks_after,
            rivals,
        })
    }

    /// Run the read's gate update through the Rust oracle.
    #[allow(clippy::too_many_arguments)]
    pub fn read_train_pick_step_reference(
        read: &Q31FirstPickRead,
        stream: &[i64],
        ids: &[u32],
        table: &[i64],
        batch: usize,
        time: usize,
        targets: &[u32],
        margin: u64,
        step_shift: u32,
    ) -> Result<Q31ReadTrainStep> {
        Ok(read.train_pick_step(stream, ids, table, batch, time, targets, margin, step_shift)?)
    }
}

/// Compile-time proof that the native device API remains integer-only.
pub fn native_vector_entry() -> &'static str {
    ENTRY_ROTATE
}

/// The second admitted entry: one bounded Q31 training step per batch item.
pub fn native_vector_training_entry() -> &'static str {
    ENTRY_TRAIN_STEP
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_identity_excludes_the_compiler_stopwatch() {
        assert!(!stable_resource_report().contains("Compile time ="));
        assert!(compiled_native_vector_image()
            .iter()
            .all(|digest| digest.len() == 64));
    }
}
