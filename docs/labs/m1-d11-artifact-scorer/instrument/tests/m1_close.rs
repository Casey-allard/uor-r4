use std::path::PathBuf;

use m1_d11_artifact_scorer_instrument::fresh_transfer::{
    evaluate_fresh_transfer_two_seeds, validate_fresh_transfer_gate, FreshSeedScore,
    FRESH_V2_ELIGIBLE_TARGETS, FRESH_V2_MASK_SHA256, FRESH_V2_PAYLOAD_TOKENS, FRESH_V2_PSUCC_ROWS,
    FRESH_V2_TOKENS_SHA256,
};
use m1_d11_artifact_scorer_instrument::m1_close::{
    build_fresh_transfer_gate_receipt, canonical_m1_saved_model_audit,
    canonical_stage_t_fz_candidate, canonical_stage_t_rel_candidate, evaluate_m1_closing_bars,
    M1CloseCandidateInput, DEV_PSUCC_ROWS, M1_CLOSE_SCHEMA,
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
    let (audit, fz_gate_receipt) = canonical_m1_saved_model_audit().unwrap();
    assert_eq!(audit["schema"], M1_CLOSE_SCHEMA);
    assert_eq!(audit["m1_close_pass"], false);
    assert_eq!(
        audit["m1_milestone_status"],
        "OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED"
    );
    assert_eq!(audit["not_run_is_not_pass"], true);
    assert_eq!(audit["fresh_v2_seal_intact"], true);
    assert_eq!(
        audit["all_saved_discrete_models_inventory"]
            .as_array()
            .unwrap()
            .len(),
        8
    );

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

    // Prospective fresh-seal check: the Stage T receipt has dev_decision = FAIL_BAR_D_HOLD_FRESH_SEAL,
    // which validate_fresh_transfer_gate refuses before touching disk.
    assert_eq!(fz_gate_receipt["dev_decision"], "FAIL_BAR_D_HOLD_FRESH_SEAL");
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
