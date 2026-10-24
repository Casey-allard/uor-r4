//! Attention preparation/readback and explicit read-only GPU cell instruments.
//! Metadata entry points never construct a model. Token-cell export restores an
//! exact checkpoint; DEV capture adds a zero-apply forward pass. Neither issues
//! a training capability or implements host admission geometry.
use super::*;

fn input<T: serde::de::DeserializeOwned>(path: &Path) -> Result<(T, FilePin)> {
    let bytes = fs::read(path).map_err(|e| invalid(format!("{}: {e}", path.display())))?;
    let value = serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
    Ok((
        value,
        FilePin {
            path: fs::canonicalize(path).map_err(|e| invalid(e.to_string()))?,
            sha256: corpus::sha256(&bytes),
        },
    ))
}

/// Storage encoding only. Cell identities have already been computed on CUDA;
/// this function neither maps token vectors nor recreates model admissions.
#[cfg(any(feature = "cuda", test))]
fn token_cell_bytes(cells: &[u16]) -> Result<Vec<u8>> {
    if cells.len() != 4096 || cells.iter().any(|&cell| cell >= 256) {
        return Err(invalid(
            "token-cell export requires4096 device cell IDs in0..255",
        ));
    }
    Ok(cells.iter().flat_map(|cell| cell.to_le_bytes()).collect())
}

#[cfg(not(feature = "cuda"))]
pub fn export_token_cells(
    _config_path: &Path,
    _checkpoint_path: &Path,
    _output_directory: &Path,
) -> Result<Value> {
    Err(invalid(
        "token-cell export requires CUDA; no host geometry implementation exists",
    ))
}

/// Read-only export from one exact typed DriverCheckpoint. The caller reserves
/// its report directory before constructing a CUDA context. Snapshot equality
/// is storage validation, never a host recomputation of the exported geometry.
#[cfg(feature = "cuda")]
pub fn export_token_cells(
    config_path: &Path,
    checkpoint_path: &Path,
    output_directory: &Path,
) -> Result<Value> {
    let (config, config_file): (DriverConfig, _) = input(config_path)?;
    validate_config(&config)?;
    let driver = PreparedDriver::load(config.clone())?;
    let (checkpoint, checkpoint_file): (training::DriverCheckpoint, _) = input(checkpoint_path)?;
    driver.validate_checkpoint(&checkpoint)?;
    let hashes = checkpoint_hash_domains(&checkpoint)?;
    // Reserve the new output before any CUDA allocation or model restore.
    // A later failure leaves this partial file with the caller's FAILED report.
    let output_path = output_directory.join("token_cells.u16");
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output_path)
        .map_err(|e| invalid(format!("exclusive {}: {e}", output_path.display())))?;
    let checkpoint_sha256 = corpus::sha256(&encode(&checkpoint)?);
    let epoch = checkpoint.cursor.accepted_updates;
    let (model, cursor) = driver.restore(checkpoint.clone(), 0)?;
    if cursor != checkpoint.cursor
        || model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != checkpoint.model
    {
        return Err(invalid(
            "token-cell export restore differs from the exact checkpoint",
        ));
    }
    let cells = model.export_token_cells()?;
    if model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != checkpoint.model
    {
        return Err(invalid("token-cell export mutated the checkpoint"));
    }
    let bytes = token_cell_bytes(&cells)?;
    output
        .write_all(&bytes)
        .and_then(|_| output.sync_all())
        .map_err(|e| invalid(e.to_string()))?;
    let stored = fs::read(&output_path).map_err(|e| invalid(e.to_string()))?;
    if stored != bytes {
        return Err(invalid("token-cell export storage readback differs"));
    }
    let output_file = FilePin {
        path: fs::canonicalize(&output_path).map_err(|e| invalid(e.to_string()))?,
        sha256: corpus::sha256(&stored),
    };
    Ok(json!({"schema":"uor-r4.attention-token-cells/1",
        "status":"DEVICE_TOKEN_CELL_MAP_EXPORTED_ZERO_APPLY",
        "config_file":config_file,"config_sha256":corpus::sha256(&encode(&config)?),
        "lineage_sha256":driver.lineage_sha256()?,"checkpoint_file":checkpoint_file,
        "driver_checkpoint_sha256":checkpoint_sha256,
        "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
        "model_bindings":{"source_sha256":checkpoint.model["source_sha256"],
            "kernel_sha256":checkpoint.model["kernel_sha256"],"geometry_sha256":checkpoint.model["geometry_sha256"],
            "spec":config.spec,"read_spec":config.read_spec,"objective_policy":config.objective_policy,
            "read_initialization":config.read_initialization,"seed":config.seed},
        "model_epoch":epoch,"completed_batches":cursor.completed_batches,
        "token_cells_file":output_file,"token_count":cells.len(),"byte_count":bytes.len(),
        "encoding":"raw_u16_le_no_header","ordering":"ascending_token_id_0_through4095",
        "cell_range":{"min":0,"max":255},"absent_cells_allowed":false,
        "cell_ids_source":"existing CUDA token_cell(table,geometry,spec,token,2,0,0); identity candidate",
        "model_restores":1,"export_kernel_calls":1,"token_cell_evaluations":4096,
        "gpu_forward_calls":0,"proposals":0,"updates_applied":0,"snapshot_unchanged":true,
        "training_capability_issued":false,"host_cell_reconstruction":false,
        "execution_scope":"read-only GPU instrument; separate applicable FIT and PM release required",
        "training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}))
}

fn probe_indices(rows: &[(u64, u32, bool)]) -> Result<Vec<usize>> {
    if rows.windows(2).any(|pair| pair[0].0 >= pair[1].0) {
        return Err(invalid(
            "preflight rows must be unique and in canonical position order",
        ));
    }
    let selected = rows
        .iter()
        .enumerate()
        .filter_map(|(index, row)| row.2.then_some(index))
        .take(64)
        .collect::<Vec<_>>();
    if selected.len() != 64 {
        return Err(invalid(
            "preflight requires64 copy-successor rows for the fixed probe",
        ));
    }
    Ok(selected)
}

fn validate_window(window: &[crate::Identity], target_position: u64, context: u32) -> Result<()> {
    let first = window
        .first()
        .ok_or_else(|| invalid("probe causal window is empty"))?;
    let last = window
        .last()
        .ok_or_else(|| invalid("probe causal window is empty"))?;
    if window.len() > context as usize
        || last.occurrence.checked_add(1) != Some(target_position)
        || window.iter().any(|identity| {
            identity.occurrence >= target_position
                || identity.token >= 4096
                || identity.record != first.record
                || identity.version != 0
        })
        || window
            .windows(2)
            .any(|pair| pair[0].occurrence.checked_add(1) != Some(pair[1].occurrence))
    {
        return Err(invalid(
            "probe must contain only the bounded contiguous in-document prefix before its target",
        ));
    }
    Ok(())
}

/// Prepare the canonical panel and the first64 P_succ causal windows before any
/// model outcomes. Labels classify the registered panel, never choose a model.
pub fn panel(config_path: &Path) -> Result<Value> {
    let (config, config_file): (DriverConfig, _) = input(config_path)?;
    validate_config(&config)?;
    let driver = PreparedDriver::load(config.clone())?;
    let rows = dev_row_identities(&config)?;
    let selected = probe_indices(&rows)?;
    let source = corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
    let mut windows = Vec::with_capacity(64);
    let mut targets = Vec::with_capacity(64);
    for &ordinal in &selected {
        let batch = source.batch(ordinal as u64, config.spec.context, 1)?;
        let (position, target, _) = rows[ordinal];
        if batch.target_positions != [position]
            || batch.targets != [target]
            || batch.windows.len() != 1
        {
            return Err(invalid("probe request differs from the canonical DEV row"));
        }
        let window = batch
            .windows
            .into_iter()
            .next()
            .ok_or_else(|| invalid("probe window missing"))?;
        validate_window(&window, position, config.spec.context)?;
        if !evidence::is_copy_successor(&window, target) {
            return Err(invalid(
                "selected probe is not a registered copy-successor row",
            ));
        }
        windows.push(window);
        targets.push(target);
    }
    let canonical_rows = rows.iter().enumerate().map(|(ordinal, &(position, token, psucc))| {
        json!({"row_ordinal":ordinal,"target_position":position,"target_token":token,"is_psucc":psucc})
    }).collect::<Vec<_>>();
    Ok(json!({"schema":"uor-r4.attention-preflight-panel/1",
        "status":"PINNED_PANEL_AND_PROBE_METADATA_ONLY",
        "config_file":config_file,"config_sha256":corpus::sha256(&encode(&config)?),
        "lineage_sha256":driver.lineage_sha256()?,"dev_binding":source.binding(),
        "rows":canonical_rows,"row_count":rows.len(),"psucc_count":rows.iter().filter(|row|row.2).count(),
        "probe_selection_policy":"first64_P_succ_rows_in_canonical_ascending_eligible_DEV_order_before_model_outcomes",
        "probe_row_ordinals":selected,"probe_request":{"windows":windows,"targets":targets},
        "scoring_performed":false,"cuda_context_created":false,"model_constructed":false,
        "updates_applied":0,"release":"STILL_REQUIRED","training":"HELD","q4":"NOT_EXECUTABLE"}))
}

#[cfg(not(feature = "cuda"))]
pub fn dev_b3(
    _config_path: &Path,
    _checkpoint_path: &Path,
    _evaluation_path: &Path,
) -> Result<Value> {
    Err(invalid("DEV B3 readback needs the CUDA build's typed Snapshot metadata; no model or CUDA context was created"))
}

/// Validate an existing raw eval-corpus report against its DriverCheckpoint and
/// canonical DEV rows. This only decodes stored CUDA observations and identities.
#[cfg(feature = "cuda")]
pub fn dev_b3(config_path: &Path, checkpoint_path: &Path, evaluation_path: &Path) -> Result<Value> {
    let inputs = dev_inputs(
        DevLock::Attention,
        config_path,
        checkpoint_path,
        evaluation_path,
    )?;
    let DevInputs {
        config,
        config_file,
        driver,
        checkpoint,
        checkpoint_file,
        evaluation_file,
        b3,
        ..
    } = inputs;
    let hashes = checkpoint_hash_domains(&checkpoint)?;
    Ok(json!({"schema":"uor-r4.attention-preflight-dev-b3/1",
        "status":"VALIDATED_RETAINED_DEV_OBSERVATIONS",
        "config_file":config_file,"config_sha256":corpus::sha256(&encode(&config)?),
        "lineage_sha256":driver.lineage_sha256()?,"checkpoint_file":checkpoint_file,
        "driver_checkpoint_sha256":corpus::sha256(&encode(&checkpoint)?),"evaluation_file":evaluation_file,
        "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
        "compiled_identity_checked":true,"model_epoch":checkpoint.cursor.accepted_updates,
        "completed_batches":checkpoint.cursor.completed_batches,"b3":b3,
        "admission_contract":"checked EvaluationRow.admitted_cells minus address_rivals in {0,1}",
        "actual_admitted_slot_ids":{"status":"UNAVAILABLE","reason":"eval-corpus reports counts; slot IDs are not derived from those counts"},
        "evaluation_mutates_model":false,"updates_applied":0,"scoring_performed_by_this_command":false,
        "model_constructed":false,"cuda_context_created":false,"release":"STILL_REQUIRED",
        "training":"HELD","q4":"NOT_EXECUTABLE"}))
}

#[cfg(feature = "cuda")]
struct DevInputs {
    config: DriverConfig,
    config_file: FilePin,
    driver: PreparedDriver,
    checkpoint: training::DriverCheckpoint,
    checkpoint_file: FilePin,
    evaluation_file: FilePin,
    rows: Vec<(u64, u32, bool)>,
    b3: evidence::B3Summary,
    hits: Vec<u64>,
}

#[cfg(any(feature = "cuda", test))]
fn validate_dev_b3_config(config: &DriverConfig) -> Result<()> {
    config.validate()?;
    validate_registered_corpus(config)?;
    if config.spec.kept_cells != 8
        || config.objective_policy != crate::ObjectivePolicy::NearestAddressMarginV1
        || !matches!(config.seed, 1 | 2)
        || !config.normalization_mode.is_scale_matched_snap()
    {
        return Err(invalid(
            "DEV B3 requires registered K8 seed-1 or seed-2 nearest-address scale-matched config",
        ));
    }
    let valid_arm = match config.read_spec {
        ReadSpec::Flock { kept: 8 } | ReadSpec::Relation { kept: 8, sweeps: 1 } => {
            config.schedule == training::SchedulePolicy::InterleavedReadEvery8V1
                && matches!(
                    config.read_initialization,
                    ReadInitialization::ActiveLeafV1 | ReadInitialization::ZeroGatesV1
                )
        }
        ReadSpec::Route => {
            config.schedule == training::SchedulePolicy::CanonicalVectorThenReadV1
                && matches!(
                    config.read_initialization,
                    ReadInitialization::ZeroGatesV1
                        | ReadInitialization::RouteBypassV1
                        | ReadInitialization::RoutePermutedActionsV1
                )
        }
        _ => false,
    };
    if !valid_arm {
        return Err(invalid(
            "DEV B3 requires a registered Flock8/Relation8 or Route8 read-on/read-off/permuted arm",
        ));
    }
    Ok(())
}

#[cfg(feature = "cuda")]
fn dev_inputs(
    lock: DevLock,
    config_path: &Path,
    checkpoint_path: &Path,
    evaluation_path: &Path,
) -> Result<DevInputs> {
    let (config, config_file): (DriverConfig, _) = input(config_path)?;
    dev_config_lock(lock, &config)?;
    let driver = PreparedDriver::load(config.clone())?;
    let (checkpoint, checkpoint_file): (training::DriverCheckpoint, _) = input(checkpoint_path)?;
    driver.validate_checkpoint(&checkpoint)?;
    let (evaluation, evaluation_file): (Value, _) = input(evaluation_path)?;
    if evaluation["schema"] != json!(training::DRIVER_SCHEMA)
        || evaluation["evaluation_lineage"] != json!(driver.lineage())
        || evaluation.get("transfer_corpus").is_some()
        || evaluation.get("evaluation").is_some()
    {
        return Err(invalid("DEV B3 requires the raw registered DEV eval-corpus report, not transfer/enveloped evidence"));
    }
    check_compiled_identity(&checkpoint, "DEV B3")?;
    verify_evaluation(&evaluation, &checkpoint)?;
    let rows = dev_row_identities(&config)?;
    let b3 = evidence::summarize(&evaluation, &rows)?;
    let hits = dev_hit_words(&evaluation)?;
    Ok(DevInputs {
        config,
        config_file,
        driver,
        checkpoint,
        checkpoint_file,
        evaluation_file,
        rows,
        b3,
        hits,
    })
}

/// Compiled identity is metadata; no Device/context is constructed and no
/// bridge can silently promote a report from another numerical source.
#[cfg(feature = "cuda")]
fn check_compiled_identity(checkpoint: &training::DriverCheckpoint, label: &str) -> Result<()> {
    let compiled = crate::device::compiled_identity();
    for key in ["source_sha256", "kernel_sha256", "geometry_sha256"] {
        if checkpoint.model[key] != compiled[key] {
            return Err(invalid(format!(
                "{label} checkpoint {key} differs from the current compiled identity"
            )));
        }
    }
    Ok(())
}

/// Which registered config lock a DEV readback uses. `Attention` is the
/// unchanged A8 DEV-B3 lock (scale-matched only); `Normalization` admits the A8
/// learned-Relation proxy trunk in any of the three registered modes.
#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DevLock {
    Attention,
    Normalization,
}

#[cfg(any(feature = "cuda", test))]
fn dev_config_lock(lock: DevLock, config: &DriverConfig) -> Result<()> {
    match lock {
        DevLock::Attention => validate_dev_b3_config(config),
        DevLock::Normalization => validate_normalization_config(config),
    }
}

/// Raw per-row `top1_correct` words in report order. Values are not clamped
/// here so the paired scorer can refuse anything outside {0,1}.
#[cfg(feature = "cuda")]
fn dev_hit_words(evaluation: &Value) -> Result<Vec<u64>> {
    let mut hits = Vec::new();
    for batch in evaluation["batches"]
        .as_array()
        .ok_or_else(|| invalid("DEV batches missing"))?
    {
        for row in batch["rows"]
            .as_array()
            .ok_or_else(|| invalid("DEV rows missing"))?
        {
            hits.push(
                row["top1_correct"]
                    .as_u64()
                    .ok_or_else(|| invalid("DEV top1_correct must be an unsigned integer"))?,
            );
        }
    }
    Ok(hits)
}

#[cfg(not(feature = "cuda"))]
pub fn normalization_dev_b3(
    _config_path: &Path,
    _checkpoint_path: &Path,
    _evaluation_path: &Path,
) -> Result<Value> {
    Err(invalid("normalization DEV B3 readback needs the CUDA build's typed Snapshot metadata; no model or CUDA context was created"))
}

/// DEV B3 readback for one normalization arm: the same immutable checks as
/// `dev_b3`, under the normalization config lock instead of the attention one.
#[cfg(feature = "cuda")]
pub fn normalization_dev_b3(
    config_path: &Path,
    checkpoint_path: &Path,
    evaluation_path: &Path,
) -> Result<Value> {
    let DevInputs {
        config,
        config_file,
        driver,
        checkpoint,
        checkpoint_file,
        evaluation_file,
        b3,
        ..
    } = dev_inputs(
        DevLock::Normalization,
        config_path,
        checkpoint_path,
        evaluation_path,
    )?;
    let hashes = checkpoint_hash_domains(&checkpoint)?;
    Ok(json!({"schema":"uor-r4.normalization-dev-b3/1",
        "status":"VALIDATED_RETAINED_DEV_OBSERVATIONS",
        "normalization_mode":config.normalization_mode,"normalization_scope":normalization_scope(&config)?,
        "config_file":config_file,"config_sha256":corpus::sha256(&encode(&config)?),
        "lineage_sha256":driver.lineage_sha256()?,"checkpoint_file":checkpoint_file,
        "driver_checkpoint_sha256":corpus::sha256(&encode(&checkpoint)?),"evaluation_file":evaluation_file,
        "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
        "compiled_identity_checked":true,"model_epoch":checkpoint.cursor.accepted_updates,
        "completed_batches":checkpoint.cursor.completed_batches,"b3":b3,
        "admission_contract":"checked EvaluationRow.admitted_cells minus address_rivals in {0,1}",
        "actual_admitted_slot_ids":{"status":"UNAVAILABLE","reason":"eval-corpus reports counts; slot IDs are not derived from those counts"},
        "evaluation_mutates_model":false,"updates_applied":0,"scoring_performed_by_this_command":false,
        "model_constructed":false,"cuda_context_created":false,"release":"STILL_REQUIRED",
        "training":"HELD","q4":"NOT_EXECUTABLE"}))
}

#[cfg(not(feature = "cuda"))]
pub fn normalization_dev_structure(_config_path: &Path, _checkpoint_path: &Path) -> Result<Value> {
    Err(invalid("normalization DEV structure readback requires a CUDA traced forward pass; no host reconstruction is available; no model or CUDA context was created"))
}

/// Structural-norm stability readback for one normalization arm: restores the
/// exact checkpoint, runs zero-apply traced Flock forwards over every DEV row,
/// and scores the CUDA-written trace and output blocks with the pure
/// `structural_summary`. The bar term is judged on `ScaleMatchedSnapV1` only.
#[cfg(feature = "cuda")]
pub fn normalization_dev_structure(config_path: &Path, checkpoint_path: &Path) -> Result<Value> {
    use crate::normalization_mechanism::{
        structural_summary, NormalizationMode, StructuralSummary, STRUCTURE_TRACE_HEADER_WORDS,
        STRUCTURE_TRACE_LEVEL_WORDS,
    };
    if crate::device::FLOCK_TRACE_HEADER_WORDS != STRUCTURE_TRACE_HEADER_WORDS
        || crate::device::FLOCK_TRACE_LEVEL_WORDS != STRUCTURE_TRACE_LEVEL_WORDS
    {
        return Err(invalid(
            "structural scorer trace layout differs from the device Flock trace layout",
        ));
    }
    let (config, config_file): (DriverConfig, _) = input(config_path)?;
    dev_config_lock(DevLock::Normalization, &config)?;
    let driver = PreparedDriver::load(config.clone())?;
    let (checkpoint, checkpoint_file): (training::DriverCheckpoint, _) = input(checkpoint_path)?;
    driver.validate_checkpoint(&checkpoint)?;
    check_compiled_identity(&checkpoint, "DEV structure")?;
    validate_config(&config)?;
    let identities = dev_row_identities(&config)?;
    let stages = BinderPlan::new(config.spec.context as usize)?.stages();
    let blocks = config.spec.blocks as usize;
    let source = corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
    if source.eligible_len() != identities.len() {
        return Err(invalid(
            "structure DEV source inventory differs from the canonical DEV rows",
        ));
    }
    let epoch = checkpoint.cursor.accepted_updates;
    let (model, cursor) = driver.restore(checkpoint.clone(), 0)?;
    if cursor != checkpoint.cursor
        || model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != checkpoint.model
    {
        return Err(invalid(
            "structure restore differs from the exact admitted checkpoint",
        ));
    }
    let mut summary = StructuralSummary::default();
    let mut rows = 0usize;
    let mut first = 0;
    let mut forward_calls = 0u64;
    while first < source.eligible_len() as u64 {
        let batch = source.batch(first, config.spec.context, config.batch_rows)?;
        let count = batch.targets.len();
        if batch.windows.len() != count
            || batch.target_positions.len() != count
            || count == 0
            || count > 64
        {
            return Err(invalid(
                "structure batch shape differs from the canonical DEV pass",
            ));
        }
        for (index, ((window, &position), &target)) in batch
            .windows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
            .enumerate()
        {
            let ordinal = rows
                .checked_add(index)
                .ok_or_else(|| invalid("structure DEV row ordinal overflow"))?;
            let &(expected_position, expected_target, _) = identities
                .get(ordinal)
                .ok_or_else(|| invalid("structure DEV has an extra row"))?;
            if position != expected_position || target != expected_target {
                return Err(invalid(
                    "structure DEV row differs from the canonical DEV rows",
                ));
            }
            validate_window(window, position, config.spec.context)?;
        }
        let carried = model.carry_windows(&batch.windows)?;
        if carried.rows() as usize != count
            || carried.valid_lengths().len() != count
            || carried.epoch() != epoch
            || carried
                .valid_lengths()
                .iter()
                .zip(&batch.windows)
                .any(|(&length, window)| length as usize != window.len())
        {
            return Err(invalid("structure carry shape/valid length/epoch differs"));
        }
        let forward = model.forward_traced(&carried, &batch.targets)?;
        if forward.epoch() != epoch || model.epoch()? != epoch {
            return Err(invalid("structure traced Forward epoch differs"));
        }
        let trace = forward
            .flock_trace()?
            .ok_or_else(|| invalid("structure traced Forward returned no Flock trace"))?;
        let stride = forward
            .trace_stride()
            .ok_or_else(|| invalid("structure traced Forward returned no trace stride"))?;
        let level_words = stride
            .checked_sub(STRUCTURE_TRACE_HEADER_WORDS)
            .filter(|words| words % STRUCTURE_TRACE_LEVEL_WORDS == 0)
            .ok_or_else(|| invalid("structure trace stride is not header plus whole levels"))?;
        if level_words / STRUCTURE_TRACE_LEVEL_WORDS != stages {
            return Err(invalid(
                "structure trace level count differs from the config stage count",
            ));
        }
        let vectors = forward.vectors()?;
        let expected_words = count
            .checked_mul(blocks)
            .and_then(|words| words.checked_mul(8))
            .ok_or_else(|| invalid("structure output inventory overflow"))?;
        if vectors.len() != expected_words {
            return Err(invalid(
                "structure output vectors differ from rows x blocks x 8",
            ));
        }
        let outputs = vectors
            .chunks_exact(8)
            .map(<[i32; 8]>::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| invalid(e.to_string()))?;
        let part = structural_summary(&trace, stride, count, blocks, stages, &outputs)?;
        summary.absorb(&part)?;
        rows = rows
            .checked_add(count)
            .ok_or_else(|| invalid("structure DEV row count overflow"))?;
        forward_calls = forward_calls
            .checked_add(1)
            .ok_or_else(|| invalid("structure forward call count overflow"))?;
        first = batch.next_target;
    }
    if rows != identities.len()
        || model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != checkpoint.model
    {
        return Err(invalid(
            "structure complete inventory or unchanged snapshot check failed",
        ));
    }
    let judged = config.normalization_mode == NormalizationMode::ScaleMatchedSnapV1;
    let met = if judged {
        json!(summary.met)
    } else {
        Value::Null
    };
    let hashes = checkpoint_hash_domains(&checkpoint)?;
    Ok(json!({"schema":"uor-r4.normalization-dev-structure/1",
        "status":"ACTUAL_GPU_TRACED_FORWARD_STRUCTURE_SCORED_ZERO_APPLY",
        "normalization_mode":config.normalization_mode,"normalization_scope":normalization_scope(&config)?,
        "config_file":config_file,"config_sha256":corpus::sha256(&encode(&config)?),
        "lineage_sha256":driver.lineage_sha256()?,"checkpoint_file":checkpoint_file,
        "driver_checkpoint_sha256":corpus::sha256(&encode(&checkpoint)?),
        "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
        "compiled_identity_checked":true,"dev_binding":source.binding(),
        "model_epoch":epoch,"completed_batches":checkpoint.cursor.completed_batches,
        "rows":rows,"stages":stages,"blocks":blocks,"gpu_forward_calls":forward_calls,
        "summary":summary,
        "bar_item":"structural norm stability (judged on n_snap only)",
        "judged":judged,"met":met,
        "trace_source":"existing production Device::forward_traced Flock trace and output blocks; scored by the pure structural_summary",
        "snapshot_unchanged":true,"updates_applied":0,"evaluation_mutates_model":false,
        "release":"STILL_REQUIRED","training":"HELD"}))
}

/// Registered DEV panel: P_succ rows and the Q1 admission floor on them.
#[cfg(feature = "cuda")]
const DEV_PSUCC_ROWS: u64 = 3847;
#[cfg(feature = "cuda")]
const Q1_ADMISSION_FLOOR: u64 = 1351;

/// Paired one-sided McNemar contrast counts on one row population.
#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
struct DevPairedCount {
    rows: u64,
    /// Reference (claimed better) correct and control wrong.
    b: u64,
    /// Reference wrong and control correct.
    c: u64,
    both_correct: u64,
    both_wrong: u64,
    f_i: u64,
    b_minus_c: i64,
    beats: bool,
}

#[cfg(any(feature = "cuda", test))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
struct DevPaired {
    all: DevPairedCount,
    psucc: DevPairedCount,
}

/// `F_I = max(24, ceil(2 * sqrt(b + c)))`, exactly, via the integer square
/// root of `4 * (b + c)`.
#[cfg(any(feature = "cuda", test))]
fn dev_paired_floor(discordant: u64) -> Result<u64> {
    let square = discordant
        .checked_mul(4)
        .ok_or_else(|| invalid("DEV paired floor radicand overflow"))?;
    let root = square.isqrt();
    let exact = root
        .checked_mul(root)
        .ok_or_else(|| invalid("DEV paired floor square overflow"))?;
    let ceiling = root
        .checked_add(u64::from(exact < square))
        .ok_or_else(|| invalid("DEV paired floor ceiling overflow"))?;
    Ok(24.max(ceiling))
}

#[cfg(any(feature = "cuda", test))]
fn dev_count(count: &mut u64) -> Result<()> {
    *count = count
        .checked_add(1)
        .ok_or_else(|| invalid("DEV paired count overflow"))?;
    Ok(())
}

#[cfg(any(feature = "cuda", test))]
fn dev_paired_close(mut count: DevPairedCount) -> Result<DevPairedCount> {
    let discordant = count
        .b
        .checked_add(count.c)
        .ok_or_else(|| invalid("DEV discordance overflow"))?;
    count.f_i = dev_paired_floor(discordant)?;
    let b = i64::try_from(count.b).map_err(|_| invalid("DEV b exceeds i64"))?;
    let c = i64::try_from(count.c).map_err(|_| invalid("DEV c exceeds i64"))?;
    count.b_minus_c = b
        .checked_sub(c)
        .ok_or_else(|| invalid("DEV b - c overflow"))?;
    let f_i = i64::try_from(count.f_i).map_err(|_| invalid("DEV F_I exceeds i64"))?;
    count.beats = count.b_minus_c >= f_i;
    Ok(count)
}

/// Pair two DEV readbacks row by row. `reference` is the arm claimed better;
/// `control` the arm it must beat. Rows must be the identical canonical panel.
#[cfg(any(feature = "cuda", test))]
fn dev_paired(
    reference: &evidence::B3Summary,
    reference_hits: &[u64],
    control: &evidence::B3Summary,
    control_hits: &[u64],
) -> Result<DevPaired> {
    if reference.rows.len() != control.rows.len()
        || reference_hits.len() != reference.rows.len()
        || control_hits.len() != control.rows.len()
    {
        return Err(invalid("DEV paired row or hit inventories differ"));
    }
    let mut all = DevPairedCount::default();
    let mut psucc = DevPairedCount::default();
    for (((r, c), &rhit), &chit) in reference
        .rows
        .iter()
        .zip(&control.rows)
        .zip(reference_hits)
        .zip(control_hits)
    {
        if (r.row_ordinal, r.target_position, r.target_token, r.is_psucc)
            != (c.row_ordinal, c.target_position, c.target_token, c.is_psucc)
        {
            return Err(invalid("DEV paired row identities differ"));
        }
        if rhit > 1 || chit > 1 {
            return Err(invalid("DEV top1_correct must be 0 or 1"));
        }
        if rhit > u64::from(r.target_admitted) || chit > u64::from(c.target_admitted) {
            return Err(invalid(
                "DEV correct hit cannot exceed CUDA target admission",
            ));
        }
        let tally = |count: &mut DevPairedCount| -> Result<()> {
            dev_count(&mut count.rows)?;
            match (rhit, chit) {
                (1, 0) => dev_count(&mut count.b),
                (0, 1) => dev_count(&mut count.c),
                (1, 1) => dev_count(&mut count.both_correct),
                _ => dev_count(&mut count.both_wrong),
            }
        };
        tally(&mut all)?;
        if r.is_psucc {
            tally(&mut psucc)?;
        }
    }
    Ok(DevPaired {
        all: dev_paired_close(all)?,
        psucc: dev_paired_close(psucc)?,
    })
}

/// Two normalization configs pair only when they differ in mode alone.
#[cfg(any(feature = "cuda", test))]
fn require_mode_only_difference(reference: &DriverConfig, control: &DriverConfig) -> Result<()> {
    if reference.normalization_mode == control.normalization_mode {
        return Err(invalid(
            "DEV paired arms must use different normalization modes",
        ));
    }
    if reference.seed != control.seed {
        return Err(invalid("DEV paired arms must share one seed"));
    }
    let mut aligned = control.clone();
    aligned.normalization_mode = reference.normalization_mode;
    if &aligned != reference {
        return Err(invalid(
            "DEV paired configs must be equal except normalization_mode",
        ));
    }
    Ok(())
}

#[cfg(not(feature = "cuda"))]
pub fn normalization_dev_paired(
    _reference_config: &Path,
    _reference_checkpoint: &Path,
    _reference_evaluation: &Path,
    _control_config: &Path,
    _control_checkpoint: &Path,
    _control_evaluation: &Path,
) -> Result<Value> {
    Err(invalid("normalization DEV paired scoring needs the CUDA build's typed Snapshot metadata; no model or CUDA context was created"))
}

/// Paired one-sided McNemar top-1 contrast between two normalization arms of
/// one seed on the registered DEV panel, plus each arm's Q1 admission floor.
#[cfg(feature = "cuda")]
pub fn normalization_dev_paired(
    reference_config: &Path,
    reference_checkpoint: &Path,
    reference_evaluation: &Path,
    control_config: &Path,
    control_checkpoint: &Path,
    control_evaluation: &Path,
) -> Result<Value> {
    let reference = dev_inputs(
        DevLock::Normalization,
        reference_config,
        reference_checkpoint,
        reference_evaluation,
    )?;
    let control = dev_inputs(
        DevLock::Normalization,
        control_config,
        control_checkpoint,
        control_evaluation,
    )?;
    require_mode_only_difference(&reference.config, &control.config)?;
    let paired = dev_paired(&reference.b3, &reference.hits, &control.b3, &control.hits)?;
    let mut arms = Vec::with_capacity(2);
    for arm in [&reference, &control] {
        if arm.b3.psucc.rows != DEV_PSUCC_ROWS {
            return Err(invalid(format!(
                "DEV P_succ rows {} differ from the registered {DEV_PSUCC_ROWS}",
                arm.b3.psucc.rows
            )));
        }
        let hashes = checkpoint_hash_domains(&arm.checkpoint)?;
        arms.push(json!({"normalization_mode":arm.config.normalization_mode,
            "normalization_scope":normalization_scope(&arm.config)?,
            "config_file":arm.config_file,"config_sha256":corpus::sha256(&encode(&arm.config)?),
            "lineage_sha256":arm.driver.lineage_sha256()?,"checkpoint_file":arm.checkpoint_file,
            "driver_checkpoint_sha256":corpus::sha256(&encode(&arm.checkpoint)?),
            "evaluation_file":arm.evaluation_file,
            "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
            "model_epoch":arm.checkpoint.cursor.accepted_updates,
            "completed_batches":arm.checkpoint.cursor.completed_batches,
            "psucc_rows":arm.b3.psucc.rows,"psucc_target_admitted":arm.b3.psucc.target_admitted,
            "q1_admission_floor":Q1_ADMISSION_FLOOR,
            "q1_admission_floor_met":arm.b3.psucc.target_admitted >= Q1_ADMISSION_FLOOR}));
    }
    if paired.psucc.rows != DEV_PSUCC_ROWS {
        return Err(invalid(
            "DEV paired P_succ rows differ from the registered panel",
        ));
    }
    Ok(json!({"schema":"uor-r4.normalization-dev-paired/1",
        "status":"SCORED_RETAINED_DEV_OBSERVATIONS",
        "seed":reference.config.seed,"reference":arms[0],"control":arms[1],
        "contrast":"paired one-sided McNemar top-1: b = reference correct and control wrong, c = reference wrong and control correct; beats iff b - c >= F_I = max(24, ceil(2*sqrt(b + c)))",
        "all":paired.all,"psucc":paired.psucc,
        "non_inferiority":"UNSCORED: no margin registered (claude-57 NOT_FIT prerequisite 3)",
        "structural_norm_stability":"UNSCORED: needs per-stage output traces, not in an eval-corpus report",
        "seeds":"one seed per call; the bar needs both seeds",
        "compiled_identity_checked":true,"evaluation_mutates_model":false,"updates_applied":0,
        "model_constructed":false,"cuda_context_created":false,
        "release":"STILL_REQUIRED","training":"HELD","q4":"NOT_EXECUTABLE"}))
}

#[cfg(not(feature = "cuda"))]
pub fn capture_dev(
    _config_path: &Path,
    _checkpoint_path: &Path,
    _evaluation_path: &Path,
) -> Result<Value> {
    Err(invalid("actual DEV cell-ID capture requires a separately admitted CUDA forward pass; no host reconstruction is available"))
}

/// Actual production GPU Forward cell-ID capture for same-cell controls. Source
/// availability grants no execution permission: a prospective packet must price
/// and release this additional full-DEV zero-apply forward pass explicitly.
#[cfg(feature = "cuda")]
pub fn capture_dev(
    config_path: &Path,
    checkpoint_path: &Path,
    evaluation_path: &Path,
) -> Result<Value> {
    // All checkpoint, raw evaluation, compiled source and canonical ordered DEV
    // identities are validated before restore creates the first CUDA context.
    let inputs = dev_inputs(
        DevLock::Attention,
        config_path,
        checkpoint_path,
        evaluation_path,
    )?;
    let config = &inputs.config;
    validate_config(config)?;
    let source = corpus::PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)?;
    if source.eligible_len() != inputs.rows.len() {
        return Err(invalid(
            "capture DEV source inventory differs from the bound evaluation",
        ));
    }
    let epoch = inputs.checkpoint.cursor.accepted_updates;
    let (model, cursor) = inputs.driver.restore(inputs.checkpoint.clone(), 0)?;
    if cursor != inputs.checkpoint.cursor
        || model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != inputs.checkpoint.model
    {
        return Err(invalid(
            "capture restore differs from the exact admitted checkpoint",
        ));
    }
    let mut rows = Vec::with_capacity(inputs.rows.len());
    let mut first = 0;
    let mut forward_calls = 0u64;
    while first < source.eligible_len() as u64 {
        let batch = source.batch(first, config.spec.context, config.batch_rows)?;
        let count = batch.targets.len();
        if batch.windows.len() != count
            || batch.target_positions.len() != count
            || count == 0
            || count > 64
        {
            return Err(invalid(
                "capture batch shape differs from the canonical DEV pass",
            ));
        }
        // Corpus identities are metadata. No target cell, eligibility, score,
        // ranking or writer output is reconstructed from them on the host.
        for (index, ((window, &position), &target)) in batch
            .windows
            .iter()
            .zip(&batch.target_positions)
            .zip(&batch.targets)
            .enumerate()
        {
            let ordinal = rows.len() + index;
            let &(expected_position, expected_target, _) = inputs
                .rows
                .get(ordinal)
                .ok_or_else(|| invalid("capture DEV has an extra row"))?;
            if position != expected_position || target != expected_target {
                return Err(invalid("capture DEV row differs from the bound evaluation"));
            }
            validate_window(window, position, config.spec.context)?;
        }
        let carried = model.carry_windows(&batch.windows)?;
        if carried.rows() as usize != count
            || carried.valid_lengths().len() != count
            || carried.epoch() != epoch
            || carried
                .valid_lengths()
                .iter()
                .zip(&batch.windows)
                .any(|(&length, window)| length as usize != window.len())
        {
            return Err(invalid("capture carry shape/valid length/epoch differs"));
        }
        let forward = model.forward(&carried, &batch.targets)?;
        if forward.admitted_cells.len() != count * 16
            || forward.epoch() != epoch
            || model.epoch()? != epoch
        {
            return Err(invalid(
                "capture Forward physical16-cell inventory/epoch differs",
            ));
        }
        for (index, cells) in forward.admitted_cells.chunks_exact(16).enumerate() {
            let ordinal = rows.len();
            let (position, target, psucc) = inputs.rows[ordinal];
            rows.push(json!({"row_ordinal":ordinal,"target_position":position,
                "target_token":target,"is_psucc":psucc,
                "eval_target_admitted":inputs.b3.rows[ordinal].target_admitted,
                "valid_length":carried.valid_lengths()[index],"model_epoch":epoch,
                "admitted_cells":cells}));
        }
        forward_calls += 1;
        first = batch.next_target;
    }
    if rows.len() != inputs.rows.len()
        || forward_calls != (rows.len() as u64).div_ceil(64)
        || model.epoch()? != epoch
        || serde_json::to_value(model.snapshot()?).map_err(|e| invalid(e.to_string()))?
            != inputs.checkpoint.model
    {
        return Err(invalid(
            "capture complete inventory or unchanged snapshot check failed",
        ));
    }
    let hashes = checkpoint_hash_domains(&inputs.checkpoint)?;
    Ok(json!({"schema":"uor-r4.attention-preflight-dev-capture/1",
        "status":"ACTUAL_GPU_FORWARD_CELL_IDS_CAPTURED_ZERO_APPLY",
        "config_file":inputs.config_file,"config_sha256":corpus::sha256(&encode(config)?),
        "lineage_sha256":inputs.driver.lineage_sha256()?,"checkpoint_file":inputs.checkpoint_file,
        "driver_checkpoint_sha256":corpus::sha256(&encode(&inputs.checkpoint)?),
        "model_storage_sha256":hashes.model_storage_sha256,"model_snapshot_sha256":hashes.model_snapshot_sha256,
        "evaluation_file":inputs.evaluation_file,"dev_binding":source.binding(),
        "model_epoch":epoch,"completed_batches":inputs.checkpoint.cursor.completed_batches,
        "rows":rows,"b3":inputs.b3,"row_count":inputs.rows.len(),
        "physical_cell_words_per_row":16,"unused_slot_marker":u32::MAX,
        "cell_ids_source":"existing production Device::forward; physical16-cell words copied without host reconstruction",
        "b3_source":"separately pinned eval-corpus EvaluationRow counts, not reconstructed from captured cell IDs",
        "additional_zero_apply_gpu_forward_passes":1,"gpu_forward_calls":forward_calls,"model_restores":1,
        "snapshot_unchanged":true,"updates_applied":0,"proposals":0,"training_capability_issued":false,
        "execution_scope":"additional full DEV CUDA forward pass; prospective packet/FIT/PM release and cost disposition required",
        "training":"HELD","m2":"HELD","stage_b":"HELD","q4":"NOT_EXECUTABLE"}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_cell_encoding_preserves_device_order_and_little_endian_bytes() -> Result<()> {
        let mut cells = vec![0u16; 4096];
        cells[0] = 255;
        cells[4095] = 17;
        let bytes = token_cell_bytes(&cells)?;
        assert_eq!(bytes.len(), 8192);
        assert_eq!(&bytes[..4], &[255, 0, 0, 0]);
        assert_eq!(&bytes[8190..], &[17, 0]);
        assert!(token_cell_bytes(&cells[..4095]).is_err());
        cells[1] = 256;
        assert!(token_cell_bytes(&cells).is_err());
        cells[1] = u16::MAX;
        assert!(token_cell_bytes(&cells).is_err());
        Ok(())
    }
    #[cfg(not(feature = "cuda"))]
    #[test]
    fn token_cell_export_has_no_cpu_fallback_or_input_reads() {
        assert!(export_token_cells(
            Path::new("missing-config"),
            Path::new("missing-checkpoint"),
            Path::new("missing-output")
        )
        .is_err());
    }
    #[test]
    fn probe_uses_first64_declared_successors_without_outcome_selection() -> Result<()> {
        let rows = (0..200).map(|i| (i, 0, i % 2 == 1)).collect::<Vec<_>>();
        assert_eq!(
            probe_indices(&rows)?,
            (0..64).map(|i| 2 * i + 1).collect::<Vec<_>>()
        );
        assert!(probe_indices(&rows[..126]).is_err());
        let mut duplicate = rows;
        duplicate[1].0 = 0;
        assert!(probe_indices(&duplicate).is_err());
        Ok(())
    }
    #[test]
    fn probe_window_excludes_target_and_future_occurrences() -> Result<()> {
        let window = (10..13)
            .map(|occurrence| crate::Identity {
                token: 0,
                occurrence,
                record: 1,
                version: 0,
            })
            .collect::<Vec<_>>();
        validate_window(&window, 13, 192)?;
        assert!(validate_window(&window, 12, 192).is_err());
        assert!(validate_window(&window, 14, 192).is_err());
        let mut crossed_document = window.clone();
        crossed_document[1].record = 2;
        assert!(validate_window(&crossed_document, 13, 192).is_err());
        assert!(validate_window(&window, 13, 2).is_err());
        Ok(())
    }
    #[test]
    fn dev_b3_admits_paired_read_on_read_off_and_permuted_arms() -> Result<()> {
        let base: DriverConfig = serde_json::from_slice(include_bytes!(
            "../../../docs/labs/address-cells-preflight-configs-2026-10-19/evidence/a8.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        validate_dev_b3_config(&base)?;
        let mut f_off = base.clone();
        f_off.read_initialization = ReadInitialization::ZeroGatesV1;
        validate_dev_b3_config(&f_off)?;
        for init in [
            ReadInitialization::ZeroGatesV1,
            ReadInitialization::RouteBypassV1,
            ReadInitialization::RoutePermutedActionsV1,
        ] {
            let mut route = base.clone();
            route.read_spec = ReadSpec::Route;
            route.schedule = training::SchedulePolicy::CanonicalVectorThenReadV1;
            route.read_initialization = init;
            validate_dev_b3_config(&route)?;
        }
        let mut bad_route = base.clone();
        bad_route.read_spec = ReadSpec::Route;
        bad_route.schedule = training::SchedulePolicy::InterleavedReadEvery8V1;
        bad_route.read_initialization = ReadInitialization::ZeroGatesV1;
        assert!(validate_dev_b3_config(&bad_route).is_err());
        Ok(())
    }
    #[test]
    fn normalization_dev_b3_lock_admits_three_modes_attention_lock_does_not() -> Result<()> {
        let base: DriverConfig = serde_json::from_slice(include_bytes!(
            "../../../docs/labs/address-cells-preflight-configs-2026-10-19/evidence/a8.json"
        ))
        .map_err(|e| invalid(e.to_string()))?;
        for mode in [
            NormalizationMode::ScaleMatchedSnapV1,
            NormalizationMode::UnitSnapControlV1,
            NormalizationMode::UnnormalizedControlV1,
        ] {
            let mut arm = base.clone();
            arm.normalization_mode = mode;
            dev_config_lock(DevLock::Normalization, &arm)?;
            let attention = dev_config_lock(DevLock::Attention, &arm);
            assert_eq!(attention.is_ok(), mode.is_scale_matched_snap());
        }
        let mut route = base.clone();
        route.read_spec = ReadSpec::Route;
        route.schedule = training::SchedulePolicy::CanonicalVectorThenReadV1;
        route.read_initialization = ReadInitialization::ZeroGatesV1;
        dev_config_lock(DevLock::Attention, &route)?;
        assert!(dev_config_lock(DevLock::Normalization, &route).is_err());
        let mut control = base.clone();
        control.normalization_mode = NormalizationMode::UnnormalizedControlV1;
        require_mode_only_difference(&base, &control)?;
        assert!(require_mode_only_difference(&base, &base).is_err());
        let mut other_seed = control.clone();
        other_seed.seed = 2;
        assert!(require_mode_only_difference(&base, &other_seed).is_err());
        let mut other_init = control;
        other_init.read_initialization = ReadInitialization::ZeroGatesV1;
        assert!(require_mode_only_difference(&base, &other_init).is_err());
        Ok(())
    }

    fn summary(rows: &[(bool, bool)]) -> evidence::B3Summary {
        evidence::B3Summary {
            rows: rows
                .iter()
                .enumerate()
                .map(|(i, &(is_psucc, target_admitted))| evidence::AdmissionRow {
                    row_ordinal: i as u64,
                    target_position: 10 + i as u64,
                    target_token: 7,
                    is_psucc,
                    target_admitted,
                })
                .collect(),
            full: evidence::B3Count::default(),
            psucc: evidence::B3Count::default(),
            complement: evidence::B3Count::default(),
        }
    }

    /// `b` reference-only hits, `c` control-only, `both`, `neither`; the first
    /// `psucc_b` of the `b` rows are P_succ, every other row is complement.
    fn paired_case(
        b: usize,
        c: usize,
        both: usize,
        neither: usize,
        psucc_b: usize,
    ) -> Result<DevPaired> {
        let n = b + c + both + neither;
        let rows = (0..n).map(|i| (i < psucc_b, true)).collect::<Vec<_>>();
        let s = summary(&rows);
        let mut rh = vec![0u64; n];
        let mut ch = vec![0u64; n];
        rh[..b].fill(1);
        ch[b..b + c].fill(1);
        rh[b + c..b + c + both].fill(1);
        ch[b + c..b + c + both].fill(1);
        dev_paired(&s, &rh, &s, &ch)
    }

    #[test]
    fn dev_paired_counts_and_floor() -> Result<()> {
        assert_eq!(dev_paired_floor(0)?, 24);
        assert_eq!(dev_paired_floor(10)?, 24);
        assert_eq!(dev_paired_floor(143)?, 24);
        assert_eq!(dev_paired_floor(144)?, 24);
        assert_eq!(dev_paired_floor(145)?, 25);
        assert_eq!(dev_paired_floor(200)?, 29);
        assert_eq!(dev_paired_floor(225)?, 30);
        assert!(dev_paired_floor(u64::MAX).is_err());
        let small = paired_case(5, 2, 3, 4, 2)?;
        assert_eq!(
            small.all,
            DevPairedCount {
                rows: 14,
                b: 5,
                c: 2,
                both_correct: 3,
                both_wrong: 4,
                f_i: 24,
                b_minus_c: 3,
                beats: false,
            }
        );
        assert_eq!(
            (
                small.psucc.rows,
                small.psucc.b,
                small.psucc.c,
                small.psucc.f_i
            ),
            (2, 2, 0, 24)
        );
        // b + c = 24 + 0: b - c == F_I == 24 beats at the minimum floor.
        let at_floor = paired_case(24, 0, 0, 0, 0)?;
        assert_eq!((at_floor.all.f_i, at_floor.all.beats), (24, true));
        assert!(!paired_case(23, 0, 0, 0, 0)?.all.beats);
        // b + c = 200: F_I = ceil(2*sqrt(200)) = 29 (28^2 = 784 < 800 <= 841).
        let above = paired_case(115, 85, 0, 0, 0)?;
        assert_eq!(
            (above.all.f_i, above.all.b_minus_c, above.all.beats),
            (29, 30, true)
        );
        let edge = paired_case(114, 86, 0, 0, 0)?;
        assert_eq!(
            (edge.all.f_i, edge.all.b_minus_c, edge.all.beats),
            (29, 28, false)
        );
        let reverse = paired_case(0, 30, 0, 0, 0)?;
        assert_eq!((reverse.all.b_minus_c, reverse.all.beats), (-30, false));
        Ok(())
    }

    #[test]
    fn dev_paired_refuses_mismatched_rows_and_impossible_hits() -> Result<()> {
        let s = summary(&[(true, true), (false, false), (true, true)]);
        let hits = [1, 0, 0];
        dev_paired(&s, &hits, &s, &hits)?;
        let short = summary(&[(true, true), (false, false)]);
        assert!(dev_paired(&s, &hits, &short, &hits[..2]).is_err());
        assert!(dev_paired(&s, &hits[..2], &s, &hits).is_err());
        let mut moved = s.clone();
        moved.rows[2].target_token = 8;
        assert!(dev_paired(&s, &hits, &moved, &hits).is_err());
        let mut relabelled = s.clone();
        relabelled.rows[0].is_psucc = false;
        assert!(dev_paired(&s, &hits, &relabelled, &hits).is_err());
        assert!(dev_paired(&s, &[0, 1, 0], &s, &hits).is_err());
        assert!(dev_paired(&s, &[2, 0, 0], &s, &hits).is_err());
        Ok(())
    }
}
