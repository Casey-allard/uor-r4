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
    let inputs = dev_inputs(config_path, checkpoint_path, evaluation_path)?;
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
    config_path: &Path,
    checkpoint_path: &Path,
    evaluation_path: &Path,
) -> Result<DevInputs> {
    let (config, config_file): (DriverConfig, _) = input(config_path)?;
    validate_dev_b3_config(&config)?;
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
    // Compiled identity is metadata; no Device/context is constructed and no
    // bridge can silently promote a report from another numerical source.
    let compiled = crate::device::compiled_identity();
    for key in ["source_sha256", "kernel_sha256", "geometry_sha256"] {
        if checkpoint.model[key] != compiled[key] {
            return Err(invalid(format!(
                "DEV B3 checkpoint {key} differs from the current compiled identity"
            )));
        }
    }
    verify_evaluation(&evaluation, &checkpoint)?;
    let rows = dev_row_identities(&config)?;
    let b3 = evidence::summarize(&evaluation, &rows)?;
    Ok(DevInputs {
        config,
        config_file,
        driver,
        checkpoint,
        checkpoint_file,
        evaluation_file,
        rows,
        b3,
    })
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
    let inputs = dev_inputs(config_path, checkpoint_path, evaluation_path)?;
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
}
