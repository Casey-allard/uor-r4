use std::path::PathBuf;

use m1_d11_artifact_scorer_instrument::fresh_transfer::{
    evaluate_fresh_transfer_two_seeds, validate_fresh_transfer_gate, FreshSeedScore,
    FRESH_V2_ELIGIBLE_TARGETS, FRESH_V2_MASK_SHA256, FRESH_V2_PAYLOAD_TOKENS, FRESH_V2_PSUCC_ROWS,
    FRESH_V2_TOKENS_SHA256,
};
use m1_d11_artifact_scorer_instrument::m1_close::{
    build_fresh_transfer_gate_receipt, canonical_m1_saved_model_audit,
    canonical_stage_t_fz_candidate, canonical_stage_t_rel_candidate,
    canonical_trunk_joint_pointer_fz_candidate, evaluate_m1_closing_bars, M1CloseCandidateInput,
    DEV_PSUCC_ROWS, DEV_PSUCC_ROWS_C192, DEV_PSUCC_T2_TOP1, DEV_PSUCC_UNIGRAM_TOP1_C192,
    M1_CLOSE_SCHEMA,
};
use uor_r4_discrete::corpus::{CorpusSource, FilePin, Split};

fn fresh_v2_source() -> CorpusSource {
    CorpusSource {
        split: Split::Fresh,
        tokens: FilePin {
            path: PathBuf::from("fresh_v2/tokens.bin"),
            sha256: FRESH_V2_TOKENS_SHA256.to_owned(),
        },
        mask: FilePin {
            path: PathBuf::from("fresh_v2/eval_mask.bin"),
            sha256: FRESH_V2_MASK_SHA256.to_owned(),
        },
        expected_payload_tokens: FRESH_V2_PAYLOAD_TOKENS,
        expected_eligible_targets: FRESH_V2_ELIGIBLE_TARGETS,
    }
}

fn synthetic_passing_candidate() -> M1CloseCandidateInput {
    let mut base = canonical_stage_t_fz_candidate();
    base.candidate_id = "synthetic-passing-two-seed".to_owned();
    base.seed_1.d11_dev_top1 = 6_350;
    base.seed_1.train_eval_dev_top1 = 6_350;
    base.seed_1.psucc_rows = DEV_PSUCC_ROWS;
    base.seed_1.psucc_d11_top1 = Some(1_760);

    base.seed_2.d11_dev_top1 = 6_340;
    base.seed_2.train_eval_dev_top1 = 6_340;
    base.seed_2.psucc_rows = DEV_PSUCC_ROWS;
    base.seed_2.psucc_d11_top1 = Some(1_755);

    let fresh_s1 = FreshSeedScore {
        seed: 1,
        artifact_sha256: base.seed_1.artifact_sha256.clone(),
        rows: 14_287,
        psucc_rows: FRESH_V2_PSUCC_ROWS,
        refused_rows: 0,
        reference_mismatches: 0,
        b4_pass: true,
        model_top1: 6_360,
        model_top8: 9_100,
        unigram_top1: 1_210,
        t2_top1: 6_250,
        b_vs_unigram: 5_200,
        c_vs_unigram: 50,
        b_vs_t2: 310,
        c_vs_t2: 200,
        psucc_model_top1: 1_780,
        psucc_unigram_top1: 170,
        psucc_t2_top1: 1_710,
        psucc_b_vs_t2: 110,
        psucc_c_vs_t2: 40,
    };
    let fresh_s2 = FreshSeedScore {
        seed: 2,
        artifact_sha256: base.seed_2.artifact_sha256.clone(),
        rows: 14_287,
        psucc_rows: FRESH_V2_PSUCC_ROWS,
        refused_rows: 0,
        reference_mismatches: 0,
        b4_pass: true,
        model_top1: 6_352,
        model_top8: 9_080,
        unigram_top1: 1_210,
        t2_top1: 6_250,
        b_vs_unigram: 5_192,
        c_vs_unigram: 50,
        b_vs_t2: 302,
        c_vs_t2: 200,
        psucc_model_top1: 1_775,
        psucc_unigram_top1: 170,
        psucc_t2_top1: 1_710,
        psucc_b_vs_t2: 105,
        psucc_c_vs_t2: 40,
    };
    let fresh_rep = evaluate_fresh_transfer_two_seeds(&fresh_s1, &fresh_s2).unwrap();
    assert_eq!(fresh_rep["bar_e_pass"], true);
    base.fresh_transfer_report = Some(fresh_rep);
    base
}

#[test]
fn canonical_saved_models_pass_bars_a_b_c_and_fail_bar_d_leaving_fresh_seal_intact() {
    let (audit, trunk_gate_receipt) = canonical_m1_saved_model_audit().unwrap();
    assert_eq!(audit["schema"], M1_CLOSE_SCHEMA);
    assert_eq!(audit["m1_close_pass"], true);
    assert_eq!(audit["m1_milestone_status"], "CLOSED_PASS");
    assert_eq!(audit["not_run_is_not_pass"], true);
    assert_eq!(audit["fresh_v2_seal_intact"], true);
    assert_eq!(audit["fresh_v2_authorized_by_trunk_bars_a_to_d"], true);
    assert_eq!(audit["registered_bars"]["dev_psucc_t2_top1"], 1_245);
    assert_eq!(audit["registered_bars"]["dev_psucc_rows_c192"], 3_847);
    assert_eq!(audit["registered_bars"]["dev_psucc_unigram_top1_c192"], 194);
    let inv = audit["all_saved_discrete_models_inventory"]
        .as_array()
        .unwrap();
    assert_eq!(inv.len(), 9);
    assert_eq!(inv[8]["id"], "trunk-joint-pointer-v1-d11");
    assert_eq!(inv[8]["bar_a_pass"], true);
    assert_eq!(inv[8]["bar_b_pass"], true);
    assert_eq!(inv[8]["bar_c_pass"], true);
    assert_eq!(inv[8]["bar_d_pass"], true);
    assert_eq!(inv[8]["bar_e_pass"], true);
    assert_eq!(inv[8]["m1_close_pass"], true);
    assert_eq!(inv[8]["clean_room_19table_d11_accepted"], true);
    assert_eq!(DEV_PSUCC_T2_TOP1, 1_245);
    assert_eq!(DEV_PSUCC_ROWS_C192, 3_847);
    assert_eq!(DEV_PSUCC_UNIGRAM_TOP1_C192, 194);

    let trunk = canonical_trunk_joint_pointer_fz_candidate();
    let trunk_v = evaluate_m1_closing_bars(&trunk).unwrap();
    assert!(trunk_v.bar_a_d11_self_contained);
    assert!(trunk_v.bar_b_export_gap_and_b4);
    assert!(trunk_v.bar_c_tokens_per_s_reported);
    assert!(trunk_v.bar_d_dev_beats_unigram_and_t2_two_seeds);
    assert!(trunk_v.bars_a_to_d_pass);
    assert!(trunk_v.bar_e_fresh_transfer_two_seeds);
    assert!(trunk_v.m1_close_pass);
    assert_eq!(trunk_v.outcome, "PASS_M1_ALL_FIVE_BARS");
    assert_eq!(
        trunk_gate_receipt["dev_decision"],
        "PASS_BARS_A_TO_D_TWO_SEEDS"
    );
    validate_fresh_transfer_gate(
        &trunk_gate_receipt,
        &trunk.seed_1.artifact_sha256,
        &fresh_v2_source(),
    )
    .unwrap();
    validate_fresh_transfer_gate(
        &trunk_gate_receipt,
        &trunk.seed_2.artifact_sha256,
        &fresh_v2_source(),
    )
    .unwrap();

    let fz = canonical_stage_t_fz_candidate();
    let fz_v = evaluate_m1_closing_bars(&fz).unwrap();
    assert!(fz_v.bar_a_d11_self_contained);
    assert!(fz_v.bar_b_export_gap_and_b4);
    assert!(fz_v.bar_c_tokens_per_s_reported);
    assert!(!fz_v.bar_d_dev_beats_unigram_and_t2_two_seeds);
    assert!(!fz_v.bars_a_to_d_pass);
    assert_eq!(fz_v.dev_required_top1_target, 6_324);
    assert_eq!(fz_v.seed_1_deficit_to_t2_floor, 5_869);
    assert_eq!(fz_v.seed_2_deficit_to_t2_floor, 5_869);
    assert_eq!(fz_v.fresh_transfer_status, "UNAVAILABLE_NOT_RUN");
    assert!(!fz_v.bar_e_fresh_transfer_two_seeds);
    assert!(!fz_v.m1_close_pass);

    let rel = canonical_stage_t_rel_candidate();
    let rel_v = evaluate_m1_closing_bars(&rel).unwrap();
    assert!(rel_v.bar_a_d11_self_contained);
    assert!(rel_v.bar_b_export_gap_and_b4);
    assert!(rel_v.bar_c_tokens_per_s_reported);
    assert!(!rel_v.bar_d_dev_beats_unigram_and_t2_two_seeds);
    assert!(!rel_v.bars_a_to_d_pass);
    assert_eq!(rel_v.seed_1_deficit_to_t2_floor, 5_953);
    assert_eq!(rel_v.fresh_transfer_status, "UNAVAILABLE_NOT_RUN");
    assert!(!rel_v.bar_e_fresh_transfer_two_seeds);
    assert!(!rel_v.m1_close_pass);

    // Prospective fresh-seal check: the 4-table Stage T receipt has dev_decision = FAIL_BAR_D_HOLD_FRESH_SEAL,
    // which validate_fresh_transfer_gate refuses before touching disk.
    let fz_gate_receipt = build_fresh_transfer_gate_receipt(&fz, &fz_v);
    assert_eq!(
        fz_gate_receipt["dev_decision"],
        "FAIL_BAR_D_HOLD_FRESH_SEAL"
    );
    let err = validate_fresh_transfer_gate(
        &fz_gate_receipt,
        &fz.seed_1.artifact_sha256,
        &fresh_v2_source(),
    )
    .unwrap_err();
    assert!(err.starts_with("SEALED_FRESH_PANEL_REFUSED:"));
}

#[test]
fn negative_gate_checks_on_each_of_five_m1_bars_and_prospective_fresh_seal() {
    let passing = synthetic_passing_candidate();
    let v_pass = evaluate_m1_closing_bars(&passing).unwrap();
    assert!(v_pass.bar_a_d11_self_contained);
    assert!(v_pass.bar_b_export_gap_and_b4);
    assert!(v_pass.bar_c_tokens_per_s_reported);
    assert!(v_pass.bar_d_dev_beats_unigram_and_t2_two_seeds);
    assert!(v_pass.bars_a_to_d_pass);
    assert!(v_pass.bar_e_fresh_transfer_two_seeds);
    assert!(v_pass.m1_close_pass);
    assert_eq!(v_pass.outcome, "PASS_M1_ALL_FIVE_BARS");

    let gate_pass = build_fresh_transfer_gate_receipt(&passing, &v_pass);
    assert_eq!(gate_pass["dev_decision"], "PASS_BARS_A_TO_D_TWO_SEEDS");
    validate_fresh_transfer_gate(
        &gate_pass,
        &passing.seed_1.artifact_sha256,
        &fresh_v2_source(),
    )
    .unwrap();
    validate_fresh_transfer_gate(
        &gate_pass,
        &passing.seed_2.artifact_sha256,
        &fresh_v2_source(),
    )
    .unwrap();

    // 1. Break Bar (a): side-table omitted from D11 export (as in #3267 M_code or #2844 TrunkTables)
    let mut break_a = passing.clone();
    break_a.seed_1.d11_export_includes_full_scoring_path = false;
    // With fresh_transfer_report still attached, evaluate_m1_closing_bars must refuse premature unsealing:
    let err_a = evaluate_m1_closing_bars(&break_a).unwrap_err();
    assert!(err_a.starts_with("SEALED_FRESH_PANEL_REFUSED:"));
    break_a.fresh_transfer_report = None;
    let v_break_a = evaluate_m1_closing_bars(&break_a).unwrap();
    assert!(!v_break_a.bar_a_d11_self_contained);
    assert!(!v_break_a.bars_a_to_d_pass);
    assert!(!v_break_a.m1_close_pass);

    // 2. Break Bar (b): nonzero export gap (`row_mismatches = 1`)
    let mut break_b = passing.clone();
    break_b.fresh_transfer_report = None;
    break_b.seed_2.row_mismatches = 1;
    let v_break_b = evaluate_m1_closing_bars(&break_b).unwrap();
    assert!(!v_break_b.bar_b_export_gap_and_b4);
    assert!(!v_break_b.bars_a_to_d_pass);
    assert!(!v_break_b.m1_close_pass);

    // 3. Break Bar (c): missing tokens/s (`laptop_decode_median_tok_s = 0`)
    let mut break_c = passing.clone();
    break_c.fresh_transfer_report = None;
    break_c.seed_1.laptop_decode_median_tok_s = 0;
    let v_break_c = evaluate_m1_closing_bars(&break_c).unwrap();
    assert!(!v_break_c.bar_c_tokens_per_s_reported);
    assert!(!v_break_c.bars_a_to_d_pass);
    assert!(!v_break_c.m1_close_pass);

    // 4. Break Bar (d): DEV top-1 fails to clear T2 + 24 (`6,320 < 6,324`)
    let mut break_d = passing.clone();
    break_d.fresh_transfer_report = None;
    break_d.seed_2.d11_dev_top1 = 6_320;
    break_d.seed_2.train_eval_dev_top1 = 6_320;
    let v_break_d = evaluate_m1_closing_bars(&break_d).unwrap();
    assert!(!v_break_d.bar_d_dev_beats_unigram_and_t2_two_seeds);
    assert!(!v_break_d.bars_a_to_d_pass);
    assert!(!v_break_d.m1_close_pass);

    // 5. Break Bar (e): `NOT_RUN` (`fresh_transfer_report = None`) is never `PASS` even when Bars (a)-(d) pass
    let mut break_e_not_run = passing.clone();
    break_e_not_run.fresh_transfer_report = None;
    let v_break_e = evaluate_m1_closing_bars(&break_e_not_run).unwrap();
    assert!(v_break_e.bars_a_to_d_pass);
    assert_eq!(v_break_e.fresh_transfer_status, "UNAVAILABLE_NOT_RUN");
    assert!(!v_break_e.bar_e_fresh_transfer_two_seeds);
    assert!(!v_break_e.m1_close_pass);
    assert_eq!(v_break_e.outcome, "HOLD_M1_OPEN_BAR_E_UNMET");
}

#[test]
fn clean_room_integer_discrete_model_loads_19_table_trunk_d11_with_zero_export_gap_and_serving_toks(
) {
    use m1_d11_artifact_scorer_instrument::toks::{default_prompt, time_serving, Protocol};
    use uor_r4_discrete::{
        d11_export::export_d11,
        metadata::BinderPlan,
        normalization_mechanism::{identify_orbit_root, orbit_root_coords, NormalizationMode},
        snapshot::Snapshot,
        trunk::{
            build_trunk_norm_frames, build_trunk_step_table, compute_recurrence_state,
            evaluate_joint_pointer_full_writer, export_trunk_d11, RecurrenceMode, TrunkTables,
            TrunkTrainRow,
        },
        w2_capacity::pack_code,
        ObjectivePolicy, ReadInitialization, ReadSpec, Spec, SCHEMA,
    };
    use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

    const READ_WIDTH: usize = 32;
    let spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 192,
        kept_cells: 8,
    };
    let stages = BinderPlan::new(spec.context as usize).unwrap().stages() as u32;
    let read_spec = ReadSpec::Flock { kept: 8 };
    let gate_len = read_spec.gate_count(spec.blocks, stages).unwrap() as usize;
    let mut table = Vec::with_capacity(4096 * READ_WIDTH);
    let mut roots = vec![[0u8; 4]; 4096];
    let mut coords = vec![[0i32; READ_WIDTH]; 4096];
    let mut cell_tokens: [Vec<u16>; 256] = std::array::from_fn(|_| Vec::new());
    for v in 0..4096u32 {
        let r0 = (v & 0xF) as u8;
        let r1 = ((v >> 4) & 0xF) as u8;
        let r2 = ((v >> 8) & 0xF) as u8;
        let r3 = ((v.wrapping_mul(7) ^ (v >> 3)) & 0xF) as u8;
        let r_arr = [r0, r1, r2, r3];
        roots[v as usize] = r_arr;
        let cell = (usize::from(r0) << 4) | usize::from(r3);
        cell_tokens[cell].push(v as u16);
        let mut vec32 = [0i32; READ_WIDTH];
        for b in 0..4 {
            let blk = orbit_root_coords(r_arr[b]).unwrap();
            vec32[b * 8..(b + 1) * 8].copy_from_slice(&blk);
        }
        coords[v as usize] = vec32;
        table.extend_from_slice(&vec32);
    }
    let snapshot = Snapshot {
        schema: SCHEMA.to_owned(),
        spec,
        read_spec,
        objective_policy: ObjectivePolicy::NearestAddressMarginV1,
        read_initialization: ReadInitialization::ActiveLeafV1,
        normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        seed: 1,
        epoch: 5,
        table,
        routes: vec![0u32; (spec.blocks * stages) as usize],
        gates: vec![1i32; gate_len],
        operator_kappa: "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b"
            .to_owned(),
        composition_kappa:
            "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0".to_owned(),
        kernel_sha256: "11".repeat(32),
        source_sha256: "22".repeat(32),
        geometry_sha256: "33".repeat(32),
    };

    let base_bytes = export_d11(&snapshot).unwrap();
    let base_model = IntegerDiscreteModel::from_bytes(&base_bytes).unwrap();
    assert!(!base_model.is_trunk_d11());

    let step_table = build_trunk_step_table(spec.blocks, 0).unwrap();
    let norm_frames = build_trunk_norm_frames(spec.blocks, 0).unwrap();

    let prefixes: [Vec<u32>; 5] = [
        vec![7, 12, 7, 19],
        vec![7, 19, 7, 12],
        vec![3, 7, 12],
        vec![19, 3, 7, 40, 9],
        (0..64).map(|i| ((i * 73 + 11) % 4096) as u32).collect(),
    ];
    let mut train_rows = Vec::new();
    for (idx, pfx) in prefixes.iter().enumerate() {
        let base_eval = base_model.evaluate_prefix(pfx).unwrap();
        let mut r4 = [0u8; 4];
        for b in 0..4 {
            let blk: [i32; 8] = base_eval.read_vector[b * 8..(b + 1) * 8]
                .try_into()
                .unwrap();
            r4[b] = identify_orbit_root(&blk).unwrap();
        }
        let read_code = pack_code(r4);
        let win_u16: Vec<u16> = pfx.iter().map(|&t| t as u16).collect();
        let (_, rec_code) = compute_recurrence_state(
            &win_u16,
            &roots,
            &step_table,
            &norm_frames,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            0,
        )
        .unwrap();
        let query_token = *win_u16.last().unwrap();
        let target_token = (1000 + idx * 317) as u16;
        for _ in 0..3 {
            train_rows.push(TrunkTrainRow {
                read_code,
                rec_code,
                rec_shuf_code: rec_code,
                rec_unit_code: rec_code,
                query_token,
                target_token,
            });
        }
        for _ in 0..4 {
            train_rows.push(TrunkTrainRow {
                read_code: read_code ^ 0x0001,
                rec_code,
                rec_shuf_code: rec_code,
                rec_unit_code: rec_code,
                query_token,
                target_token: target_token + 1,
            });
        }
    }
    let trunk_tables = TrunkTables::build(&train_rows).unwrap();
    let trunk_bytes =
        export_trunk_d11(&snapshot, &step_table, &norm_frames, &trunk_tables).unwrap();
    let trunk_model = IntegerDiscreteModel::from_bytes(&trunk_bytes).unwrap();
    assert!(trunk_model.is_trunk_d11());

    for pfx in &prefixes {
        let d11_eval = trunk_model.evaluate_prefix(pfx).unwrap();
        let base_eval = base_model.evaluate_prefix(pfx).unwrap();
        let q_read: [i32; READ_WIDTH] = base_eval.read_vector.clone().try_into().unwrap();
        let mut r4 = [0u8; 4];
        for b in 0..4 {
            let blk: [i32; 8] = q_read[b * 8..(b + 1) * 8].try_into().unwrap();
            r4[b] = identify_orbit_root(&blk).unwrap();
        }
        let read_code = pack_code(r4);
        let win_u16: Vec<u16> = pfx.iter().map(|&t| t as u16).collect();
        let (_, rec_code) = compute_recurrence_state(
            &win_u16,
            &roots,
            &step_table,
            &norm_frames,
            RecurrenceMode::ActiveHolonomyV1,
            NormalizationMode::ScaleMatchedSnapV1,
            0,
        )
        .unwrap();
        let (ref_chosen_tok, ref_ptr, ref_q_trunk, _, ref_top8, ref_admitted) =
            evaluate_joint_pointer_full_writer(
                &q_read,
                &win_u16,
                read_code,
                rec_code,
                RecurrenceMode::ActiveHolonomyV1,
                NormalizationMode::ScaleMatchedSnapV1,
                &trunk_tables,
                &roots,
                &coords,
                &cell_tokens,
            )
            .unwrap();

        assert_eq!(
            d11_eval.read_vector, ref_q_trunk,
            "pfx={pfx:?} read_code={read_code:#06x} rec_code={rec_code:#06x} ref_chosen_tok={ref_chosen_tok} ref_ptr={ref_ptr:?}"
        );
        let ref_admitted_u32: Vec<u32> = ref_admitted
            .iter()
            .take(spec.kept_cells as usize)
            .map(|&c| u32::from(c))
            .collect();
        assert_eq!(d11_eval.admitted_cells, ref_admitted_u32);
        assert_eq!(d11_eval.top8, ref_top8);
        assert_eq!(d11_eval.prediction, ref_top8[0]);
    }

    let mut bad_magic = trunk_bytes.clone();
    bad_magic[0] ^= 1;
    assert!(IntegerDiscreteModel::from_bytes(&bad_magic).is_err());

    let mut bad_trailing = trunk_bytes.clone();
    bad_trailing.push(0);
    assert!(IntegerDiscreteModel::from_bytes(&bad_trailing).is_err());

    let mut bad_tail = trunk_bytes.clone();
    let last_tok_hi = bad_tail.len() - 5;
    bad_tail[last_tok_hi] ^= 0xff;
    assert!(IntegerDiscreteModel::from_bytes(&bad_tail).is_err());

    let prompt = default_prompt(spec.vocabulary, 192);
    let protocol = Protocol {
        warmup: 1,
        runs: 3,
        prompt_tokens: 192,
        decode_tokens: 16,
    };
    let toks = time_serving(&trunk_model, &prompt, protocol).unwrap();
    assert!(toks["prefill"]["median_tokens_per_s"].as_u64().unwrap() > 0);
    assert!(toks["decode"]["median_tokens_per_s"].as_u64().unwrap() > 0);

    // Verify `ArtifactTables::from_bytes`, C++/CUDA kernel body (`host_score`),
    // `reference_check` (`evaluate_prefix` + `observe_target`), and `planted_fault_control` (`B4`)
    // on the 19-table `uor-r4.lut-discrete-trunk/1` artifact.
    let (tables, trunk_model_2) =
        m1_d11_artifact_scorer_instrument::artifact::ArtifactTables::from_bytes(&trunk_bytes)
            .unwrap();
    assert_eq!(tables.trunk[0], 1);
    let dev_rows: Vec<m1_d11_artifact_scorer_instrument::DevRow> = prefixes
        .iter()
        .enumerate()
        .map(|(idx, pfx)| m1_d11_artifact_scorer_instrument::DevRow {
            ordinal: idx as u64,
            target_position: pfx.len() as u64,
            target_token: (1000 + idx * 317) as u32,
            prefix: pfx.clone(),
        })
        .collect();
    let views: Vec<&[u32]> = dev_rows.iter().map(|r| r.prefix.as_slice()).collect();
    let targets: Vec<u32> = dev_rows.iter().map(|r| r.target_token).collect();
    let packed =
        m1_d11_artifact_scorer_instrument::kernel::pack_rows_with_targets(&views, &targets)
            .unwrap();

    // Failing-first check: with the 19-table trunk gate set aside (`trunk = vec![0]`),
    // `host_score` evaluates the unrotated 4-table vector and disagrees with `trunk_model_2`.
    let mut tables_no_trunk = tables.clone();
    tables_no_trunk.trunk = vec![0u32];
    let scores_no_trunk =
        m1_d11_artifact_scorer_instrument::kernel::host_score(&tables_no_trunk, &packed).unwrap();
    let ref_no_trunk = m1_d11_artifact_scorer_instrument::reference_check(
        &trunk_model_2,
        &dev_rows,
        &scores_no_trunk,
    );
    assert_eq!(ref_no_trunk["checked"], prefixes.len() as u64);
    assert_eq!(ref_no_trunk["agreed"], 0);

    // Restored check: with the 19-table trunk gate active (`tables.trunk[0] == 1`),
    // `host_score` matches `trunk_model_2::observe_target` on all 5 rows (`agreed == checked`)
    // and passes `planted_fault_control` (`B4`).
    let scores = m1_d11_artifact_scorer_instrument::kernel::host_score(&tables, &packed).unwrap();
    let ref_ok =
        m1_d11_artifact_scorer_instrument::reference_check(&trunk_model_2, &dev_rows, &scores);
    assert_eq!(ref_ok["checked"], prefixes.len() as u64);
    assert_eq!(
        ref_ok["agreed"],
        prefixes.len() as u64,
        "first_mismatch={}",
        ref_ok["first_mismatch"]
    );
    let b4 = m1_d11_artifact_scorer_instrument::planted_fault_control(
        &trunk_bytes,
        &tables,
        &dev_rows,
        &scores,
        |ft| m1_d11_artifact_scorer_instrument::kernel::host_score(ft, &packed),
    )
    .unwrap();
    assert_eq!(b4["b4_pass"], true);
}
