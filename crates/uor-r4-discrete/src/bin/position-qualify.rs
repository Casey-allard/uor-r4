//! Finite CUDA position-mechanism qualification and stack/throughput profiler (`#2843`).
//! A separate binary so the training driver's source identity (`DRIVER_SOURCE_SHA256`)
//! is unchanged. No floating-point types or literals (D32), no dense matrix
//! multiplication (D25), and no squashing functions (D23) appear in this file.
//!
//! Usage: position-qualify REPORT_JSON   (requires `--features cuda` and a GPU)

#[cfg(feature = "cuda")]
mod cuda_run {
    use serde::Serialize;
    use std::fs;
    use std::path::Path;
    use std::time::Instant;
    use uor_r4_discrete::corpus::sha256;
    use uor_r4_discrete::device::{
        compiled_identity, Device, Snapshot, FLOCK_TRACE_HEADER_WORDS, FLOCK_TRACE_LEVEL_WORDS,
    };
    use uor_r4_discrete::normalization_mechanism::NormalizationMode;
    use uor_r4_discrete::position_mechanism::{
        apply_frame_to_root, candidate_to_frame, relative_holonomy, PositionConfig, PositionMode,
        PositionStepTable, ORBIT_ROOTS,
    };
    use uor_r4_discrete::{
        driver_source_pin::DRIVER_SOURCE_SHA256, Error, Identity, ObjectivePolicy, Parameter,
        ReadInitialization, ReadSpec, Result, Spec,
    };

    fn invalid(msg: impl Into<String>) -> Error {
        Error::Invalid(msg.into())
    }

    fn patterned_table(blocks: u32) -> Result<PositionStepTable> {
        let mut candidates = Vec::with_capacity((blocks as usize) * ORBIT_ROOTS);
        for b in 0..blocks as usize {
            for r in 0..ORBIT_ROOTS {
                candidates.push(((b * 5 + r * 3) % 15 + 1) as u8);
            }
        }
        PositionStepTable::new(blocks, candidates)
    }

    fn root_coords(r: u8) -> Result<[i8; 8]> {
        let (_, coords) = apply_frame_to_root(candidate_to_frame(0), r)?;
        Ok(coords)
    }

    fn root_index_of(coords: &[i32]) -> Result<u8> {
        let target: [i8; 8] = coords
            .iter()
            .map(|&v| i8::try_from(v).map_err(|_| invalid("coordinate outside i8")))
            .collect::<Result<Vec<i8>>>()?
            .try_into()
            .map_err(|_| invalid("block slice is not 8 coordinates"))?;
        for r in 0..ORBIT_ROOTS as u8 {
            if root_coords(r)? == target {
                return Ok(r);
            }
        }
        Err(invalid("table block is not one of the 16 orbit roots"))
    }

    fn row_identities(tokens: &[u32], base_occ: u64) -> Vec<Identity> {
        tokens
            .iter()
            .enumerate()
            .map(|(idx, &token)| Identity {
                token,
                occurrence: base_occ + idx as u64,
                record: 17,
                version: 9,
            })
            .collect()
    }

    #[derive(Debug, Serialize)]
    pub struct ThroughputArm {
        pub read_mode: String,
        pub position_mode: String,
        pub s14_word: u32,
        pub batch_rows: u32,
        pub context: u32,
        pub iterations: u32,
        pub total_tokens: u64,
        pub forward_elapsed_us: u64,
        pub forward_tokens_per_sec: u64,
        pub evaluate_elapsed_us: u64,
        pub evaluate_tokens_per_sec: u64,
        pub vectors_sha256: String,
    }

    #[derive(Debug, Serialize)]
    pub struct PositionQualificationReport {
        pub schema: &'static str,
        pub driver_source_sha256: &'static str,
        pub compiled_identity: serde_json::Value,
        pub forward_local_bytes: u64,
        pub counterfactual_local_bytes: u64,
        pub evaluation_local_bytes: u64,
        pub checked_leaf_traces: usize,
        pub identity_control_matches_identity_table: bool,
        pub endpoint_control_interior_swap_invariant: bool,
        pub prefix_holonomy_interior_swap_sensitive: bool,
        pub snapshot_roundtrip_modes_verified: usize,
        pub step_16_cuda_refusal_message: String,
        pub route_position_refused: bool,
        pub throughput_arms: Vec<ThroughputArm>,
    }

    pub fn run(out_path: &Path) -> Result<()> {
        let small_spec = Spec {
            vocabulary: 6,
            blocks: 2,
            context: 4,
            kept_cells: 4,
        };
        let steps = patterned_table(small_spec.blocks)?;
        let id_steps = PositionStepTable::identity(small_spec.blocks)?;
        let rows_tokens: [[u32; 4]; 2] = [[0, 1, 2, 3], [4, 2, 5, 1]];
        let mut batch_identities = row_identities(&rows_tokens[0], 0);
        batch_identities.extend(row_identities(&rows_tokens[1], 10));

        let mut checked_leaf_traces = 0usize;
        let mut identity_control_matches_identity_table = false;

        for read_spec in [
            ReadSpec::Flock { kept: 3 },
            ReadSpec::Relation { kept: 3, sweeps: 1 },
        ] {
            let id_table_dev = Device::new_with_position(
                small_spec,
                read_spec,
                ObjectivePolicy::NearestAddressMarginV1,
                ReadInitialization::ActiveLeafV1,
                NormalizationMode::ScaleMatchedSnapV1,
                Some(PositionConfig::new(
                    PositionMode::PrefixHolonomyV1,
                    id_steps.clone(),
                )),
                7,
                0,
            )?;
            let id_table_fwd =
                id_table_dev.forward_traced(&id_table_dev.carry(&batch_identities)?, &[0, 1])?;

            for mode in [
                PositionMode::PrefixHolonomyV1,
                PositionMode::EndpointControlV1,
                PositionMode::IdentityControlV1,
            ] {
                let device = Device::new_with_position(
                    small_spec,
                    read_spec,
                    ObjectivePolicy::NearestAddressMarginV1,
                    ReadInitialization::ActiveLeafV1,
                    NormalizationMode::ScaleMatchedSnapV1,
                    Some(PositionConfig::new(mode, steps.clone())),
                    7,
                    0,
                )?;
                let snap = device.snapshot()?;
                let carried = device.carry(&batch_identities)?;
                let fwd = device.forward_traced(&carried, &[0, 1])?;
                let eval = device.evaluate(&carried, &[0, 1])?;
                if eval.aggregate.scored_rows != 2 {
                    return Err(invalid("expected 2 scored rows in small batch evaluation"));
                }
                let prop = device.propose(&carried, &[0, 1], Parameter::Gate(0))?;
                if prop.candidate_losses()?.len() != 16 {
                    return Err(invalid("expected 16 proposal candidate losses"));
                }

                let trace = fwd
                    .flock_trace()?
                    .ok_or_else(|| invalid("missing flock trace"))?;
                let stride = fwd
                    .trace_stride()
                    .ok_or_else(|| invalid("missing trace stride"))?;
                let blocks = small_spec.blocks as usize;

                for (row_idx, row_toks) in rows_tokens.iter().enumerate() {
                    for block in 0..blocks {
                        let window_roots: Vec<u8> = row_toks
                            .iter()
                            .map(|&tok| {
                                let start = (tok as usize * blocks + block) * 8;
                                root_index_of(&snap.table[start..start + 8])
                            })
                            .collect::<Result<Vec<u8>>>()?;
                        let rec_base = (row_idx * blocks + block) * stride;
                        let rec = &trace[rec_base..rec_base + stride];
                        if rec[3] != u32::from(window_roots[3]) {
                            return Err(invalid(
                                "trace header rec[3] differs from query payload root",
                            ));
                        }
                        let lv0 = &rec[FLOCK_TRACE_HEADER_WORDS
                            ..FLOCK_TRACE_HEADER_WORDS + FLOCK_TRACE_LEVEL_WORDS];
                        let found = lv0[1] as usize;
                        let eligible = lv0[3] as usize;
                        if found != 3 || eligible != 3 {
                            return Err(invalid("expected 3 kept and 3 eligible level-0 leaves"));
                        }
                        for k in 0..found {
                            let node = lv0[4 + k] as usize;
                            if node >= 3 {
                                return Err(invalid("level-0 leaf node out of 0..3"));
                            }
                            let gpu_action = u8::try_from(lv0[12 + k])
                                .map_err(|_| invalid("action out of u8"))?;
                            let gpu_carried = u8::try_from(lv0[20 + k])
                                .map_err(|_| invalid("carried root out of u8"))?;
                            let expected_action =
                                relative_holonomy(mode, &steps, block, &window_roots, node, 3)?;
                            let (expected_carried, _) =
                                apply_frame_to_root(expected_action, window_roots[node])?;
                            if gpu_action != expected_action || gpu_carried != expected_carried {
                                return Err(invalid(format!(
                                    "GPU vs host position mismatch for {mode:?} row {row_idx} block {block} leaf {node}"
                                )));
                            }
                            checked_leaf_traces += 1;
                        }
                    }
                }

                if mode == PositionMode::IdentityControlV1 {
                    let id_trace = id_table_fwd
                        .flock_trace()?
                        .ok_or_else(|| invalid("missing id trace"))?;
                    if fwd.vectors()? != id_table_fwd.vectors()? || trace != id_trace {
                        return Err(invalid(
                            "IdentityControlV1 differs from PrefixHolonomyV1 with identity table",
                        ));
                    }
                    identity_control_matches_identity_table = true;
                }
            }
        }

        // Control 2: interior token swap on GPU
        let swap_spec = Spec {
            vocabulary: 4,
            blocks: 2,
            context: 4,
            kept_cells: 4,
        };
        let mut chosen_roots = [0u8, 1u8, 2u8, 3u8];
        let mut found_witness = false;
        for r1 in 0..ORBIT_ROOTS as u8 {
            for r2 in 0..ORBIT_ROOTS as u8 {
                let win_ab = [0u8, r1, r2, 3u8];
                let win_ba = [0u8, r2, r1, 3u8];
                let mut sum_ab = [0i32; 8];
                let mut sum_ba = [0i32; 8];
                for s in 0..3usize {
                    let ha = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &steps,
                        0,
                        &win_ab,
                        s,
                        3,
                    )?;
                    let hb = relative_holonomy(
                        PositionMode::PrefixHolonomyV1,
                        &steps,
                        0,
                        &win_ba,
                        s,
                        3,
                    )?;
                    let (_, ca) = apply_frame_to_root(ha, win_ab[s])?;
                    let (_, cb) = apply_frame_to_root(hb, win_ba[s])?;
                    for d in 0..8 {
                        sum_ab[d] += i32::from(ca[d]);
                        sum_ba[d] += i32::from(cb[d]);
                    }
                }
                if sum_ab != sum_ba {
                    chosen_roots = [0u8, r1, r2, 3u8];
                    found_witness = true;
                    break;
                }
            }
            if found_witness {
                break;
            }
        }
        if !found_witness {
            return Err(invalid("failed to find interior swap witness"));
        }

        let base_dev = Device::new_with_position(
            swap_spec,
            ReadSpec::Flock { kept: 3 },
            ObjectivePolicy::NearestAddressMarginV1,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::UnnormalizedControlV1,
            Some(PositionConfig::new(
                PositionMode::PrefixHolonomyV1,
                steps.clone(),
            )),
            7,
            0,
        )?;
        let mut hol_snap = base_dev.snapshot()?;
        for (tok, &root) in chosen_roots.iter().enumerate() {
            let coords = root_coords(root)?;
            for b in 0..swap_spec.blocks as usize {
                let start = (tok * swap_spec.blocks as usize + b) * 8;
                for d in 0..8 {
                    hol_snap.table[start + d] = i32::from(coords[d]);
                }
            }
        }
        let mut end_snap = hol_snap.clone();
        end_snap.position = Some(PositionConfig::new(
            PositionMode::EndpointControlV1,
            steps.clone(),
        ));

        let hol_dev = Device::from_snapshot(hol_snap, 0)?;
        let end_dev = Device::from_snapshot(end_snap, 0)?;
        let mut swap_rows = row_identities(&[0, 1, 2, 3], 0);
        swap_rows.extend(row_identities(&[0, 2, 1, 3], 10));
        let hol_vecs = hol_dev
            .forward_traced(&hol_dev.carry(&swap_rows)?, &[0, 0])?
            .vectors()?;
        let end_vecs = end_dev
            .forward_traced(&end_dev.carry(&swap_rows)?, &[0, 0])?
            .vectors()?;
        let endpoint_control_interior_swap_invariant = end_vecs[0..16] == end_vecs[16..32];
        let prefix_holonomy_interior_swap_sensitive = hol_vecs[0..16] != hol_vecs[16..32];
        if !endpoint_control_interior_swap_invariant || !prefix_holonomy_interior_swap_sensitive {
            return Err(invalid(
                "interior swap GPU control failed to separate PrefixHolonomyV1 from EndpointControlV1",
            ));
        }

        // Control 3: snapshot roundtrips and CUDA status 10 refusal for step 16
        let single_row = row_identities(&[0, 1, 2, 3], 0);
        let mut snapshot_roundtrip_modes_verified = 0usize;
        for mode in [
            PositionMode::PrefixHolonomyV1,
            PositionMode::EndpointControlV1,
            PositionMode::IdentityControlV1,
        ] {
            let dev = Device::new_with_position(
                small_spec,
                ReadSpec::Flock { kept: 2 },
                ObjectivePolicy::NearestAddressMarginV1,
                ReadInitialization::ActiveLeafV1,
                NormalizationMode::ScaleMatchedSnapV1,
                Some(PositionConfig::new(mode, steps.clone())),
                7,
                0,
            )?;
            let before = dev.forward_traced(&dev.carry(&single_row)?, &[0])?;
            let snap = dev.snapshot()?;
            let reloaded = Device::from_snapshot(snap, 0)?;
            let after = reloaded.forward_traced(&reloaded.carry(&single_row)?, &[0])?;
            if before.vectors()? != after.vectors()?
                || before.flock_trace()? != after.flock_trace()?
            {
                return Err(invalid("snapshot roundtrip changed positioned GPU output"));
            }
            snapshot_roundtrip_modes_verified += 1;
        }

        let route_position_refused = Device::new_with_position(
            small_spec,
            ReadSpec::Route,
            ObjectivePolicy::NearestAddressMarginV1,
            ReadInitialization::ZeroGatesV1,
            NormalizationMode::ScaleMatchedSnapV1,
            Some(PositionConfig::new(
                PositionMode::PrefixHolonomyV1,
                steps.clone(),
            )),
            7,
            0,
        )
        .is_err();
        if !route_position_refused {
            return Err(invalid("Route with PositionConfig was not refused"));
        }

        let valid_dev = Device::new_with_position(
            small_spec,
            ReadSpec::Flock { kept: 2 },
            ObjectivePolicy::NearestAddressMarginV1,
            ReadInitialization::ActiveLeafV1,
            NormalizationMode::ScaleMatchedSnapV1,
            Some(PositionConfig::new(
                PositionMode::PrefixHolonomyV1,
                steps.clone(),
            )),
            7,
            0,
        )?;
        let valid_snap = valid_dev.snapshot()?;
        let mut json_val = serde_json::to_value(&valid_snap).map_err(|e| invalid(e.to_string()))?;
        json_val["position"]["steps"]["candidates"][0] = serde_json::json!(16);
        let malformed_snap: Snapshot =
            serde_json::from_value(json_val).map_err(|e| invalid(e.to_string()))?;
        let step_16_cuda_refusal_message = match Device::from_snapshot(malformed_snap, 0) {
            Err(Error::Invalid(msg))
                if msg == "discrete CUDA snapshot validation refused with status 10" =>
            {
                msg
            }
            Err(other) => {
                return Err(invalid(format!(
                    "expected CUDA snapshot validation status 10, got {other}"
                )))
            }
            Ok(_) => return Err(invalid("Device::from_snapshot accepted step candidate 16")),
        };

        // Control 4: full M1 shape stack bytes and tok/s across modes 0..=3
        let bench_spec = Spec {
            vocabulary: 4096,
            blocks: 4,
            context: 192,
            kept_cells: 8,
        };
        let bench_steps = patterned_table(bench_spec.blocks)?;
        let batch_rows = 64u32;
        let iterations = 10u32;
        let total_tokens =
            u64::from(batch_rows) * u64::from(bench_spec.context) * u64::from(iterations);
        let mut bench_identities = Vec::with_capacity((batch_rows * bench_spec.context) as usize);
        let mut bench_targets = Vec::with_capacity(batch_rows as usize);
        for r in 0..batch_rows {
            bench_targets.push((r * 13 + 7) % bench_spec.vocabulary);
            for p in 0..bench_spec.context {
                bench_identities.push(Identity {
                    token: (r * 31 + p * 17 + 3) % bench_spec.vocabulary,
                    occurrence: u64::from(r) * 1000 + u64::from(p),
                    record: u64::from(r) + 1,
                    version: 1,
                });
            }
        }

        let mut forward_local_bytes = 0u64;
        let mut counterfactual_local_bytes = 0u64;
        let mut evaluation_local_bytes = 0u64;
        let mut throughput_arms = Vec::new();

        for (read_name, read_spec) in [
            ("flock_k8", ReadSpec::Flock { kept: 8 }),
            ("relation_k8_s1", ReadSpec::Relation { kept: 8, sweeps: 1 }),
        ] {
            for (pos_name, pos_cfg, s14_word) in [
                ("legacy_bfs", None, 0u32),
                (
                    "prefix_holonomy_v1",
                    Some(PositionConfig::new(
                        PositionMode::PrefixHolonomyV1,
                        bench_steps.clone(),
                    )),
                    1u32,
                ),
                (
                    "endpoint_control_v1",
                    Some(PositionConfig::new(
                        PositionMode::EndpointControlV1,
                        bench_steps.clone(),
                    )),
                    2u32,
                ),
                (
                    "identity_control_v1",
                    Some(PositionConfig::new(
                        PositionMode::IdentityControlV1,
                        bench_steps.clone(),
                    )),
                    3u32,
                ),
            ] {
                let dev = Device::new_with_position(
                    bench_spec,
                    read_spec,
                    ObjectivePolicy::NearestAddressMarginV1,
                    ReadInitialization::ActiveLeafV1,
                    NormalizationMode::ScaleMatchedSnapV1,
                    pos_cfg,
                    7,
                    0,
                )?;
                let res = dev.resource_metadata()?;
                forward_local_bytes = res.forward_local_bytes;
                counterfactual_local_bytes = res.counterfactual_local_bytes;
                evaluation_local_bytes = res.evaluation_local_bytes;

                let carried = dev.carry(&bench_identities)?;
                // Warmup (2 calls)
                let mut vec_bytes = Vec::new();
                for _ in 0..2 {
                    let fwd = dev.forward(&carried, &bench_targets)?;
                    vec_bytes = fwd
                        .vectors()?
                        .into_iter()
                        .flat_map(|v| v.to_le_bytes())
                        .collect();
                    let _ = dev.evaluate(&carried, &bench_targets)?;
                }
                dev.synchronize()?;

                let start_fwd = Instant::now();
                for _ in 0..iterations {
                    let _ = dev.forward(&carried, &bench_targets)?;
                }
                dev.synchronize()?;
                let forward_elapsed_us = u64::try_from(start_fwd.elapsed().as_micros())
                    .unwrap_or(1)
                    .max(1);
                let forward_tokens_per_sec = (total_tokens * 1_000_000) / forward_elapsed_us;

                let start_eval = Instant::now();
                for _ in 0..iterations {
                    let _ = dev.evaluate(&carried, &bench_targets)?;
                }
                dev.synchronize()?;
                let evaluate_elapsed_us = u64::try_from(start_eval.elapsed().as_micros())
                    .unwrap_or(1)
                    .max(1);
                let evaluate_tokens_per_sec = (total_tokens * 1_000_000) / evaluate_elapsed_us;

                throughput_arms.push(ThroughputArm {
                    read_mode: read_name.to_owned(),
                    position_mode: pos_name.to_owned(),
                    s14_word,
                    batch_rows,
                    context: bench_spec.context,
                    iterations,
                    total_tokens,
                    forward_elapsed_us,
                    forward_tokens_per_sec,
                    evaluate_elapsed_us,
                    evaluate_tokens_per_sec,
                    vectors_sha256: sha256(&vec_bytes),
                });
            }
        }

        let report = PositionQualificationReport {
            schema: "uor-r4.position-cuda-qualification/1",
            driver_source_sha256: DRIVER_SOURCE_SHA256,
            compiled_identity: compiled_identity(),
            forward_local_bytes,
            counterfactual_local_bytes,
            evaluation_local_bytes,
            checked_leaf_traces,
            identity_control_matches_identity_table,
            endpoint_control_interior_swap_invariant,
            prefix_holonomy_interior_swap_sensitive,
            snapshot_roundtrip_modes_verified,
            step_16_cuda_refusal_message,
            route_position_refused,
            throughput_arms,
        };
        let encoded = serde_json::to_vec_pretty(&report).map_err(|e| invalid(e.to_string()))?;
        if let Some(parent) = out_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| invalid(e.to_string()))?;
            }
        }
        fs::write(out_path, encoded).map_err(|e| invalid(e.to_string()))?;
        Ok(())
    }
}

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 1 {
        eprintln!("usage: position-qualify REPORT_JSON");
        return std::process::ExitCode::from(2);
    }
    #[cfg(feature = "cuda")]
    {
        match cuda_run::run(std::path::Path::new(&args[0])) {
            Ok(()) => std::process::ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                std::process::ExitCode::FAILURE
            }
        }
    }
    #[cfg(not(feature = "cuda"))]
    {
        eprintln!("position-qualify requires the cuda feature; nothing ran");
        std::process::ExitCode::from(2)
    }
}
