use std::path::Path;
use uor_r4_integer::{IntegerModel, ReadGeometry, ReadMode};

const LORENTZ_MODEL_PATH: &str =
    "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/qat-lorentzflat_s1";
const DOT_MODEL_PATH: &str =
    "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/qat-dot_s1";
const TABLES_PATH: &str = "../../docs/evidence/native-lorentz-packet-2026-09-26/packed/tables";
const TOTAL: u64 = 1u64 << 48;

#[test]
fn test_cross_lab_lorentz_loading_and_inference() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(LORENTZ_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        eprintln!("Skipping: lorentz packet not present at {model_path:?}");
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-lorentzflat_s1 with arcosh tables");

    assert_eq!(model.config().read_geometry, ReadGeometry::Lorentz);
    assert_eq!(model.config().width, 128);
    assert_eq!(model.config().context, 256);

    let mut session = model.new_session();
    let step1 = model
        .step(&mut session, 42, ReadMode::Enabled)
        .expect("step 1 with lorentz read enabled");

    let sum1: u64 = step1.probabilities.iter().sum();
    assert_eq!(sum1, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");

    let step2 = model
        .step(&mut session, 100, ReadMode::Enabled)
        .expect("step 2 with lorentz read enabled");

    let sum2: u64 = step2.probabilities.iter().sum();
    assert_eq!(sum2, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");
    assert_eq!(session.len(), 2);

    // Causal memory ablation on Lorentz model: NoRead must disable read masses
    let step_noread = model
        .step(&mut session, 200, ReadMode::NoRead)
        .expect("step with NoRead");
    assert_eq!(step_noread.no_read_mass, TOTAL);
    assert!(step_noread.read_masses.iter().all(|&m| m == 0));
}

#[test]
fn test_cross_lab_dot_loading_and_inference() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(DOT_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        eprintln!("Skipping: dot packet not present at {model_path:?}");
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-dot_s1 with tables");

    assert_eq!(model.config().read_geometry, ReadGeometry::Dot);
    assert_eq!(model.config().width, 128);
    assert_eq!(model.config().context, 256);

    let mut session = model.new_session();
    let step = model
        .step(&mut session, 42, ReadMode::Enabled)
        .expect("step with dot read enabled");

    let sum: u64 = step.probabilities.iter().sum();
    assert_eq!(sum, TOTAL, "probabilities must sum to 1.0 (Q48 TOTAL)");
}

#[test]
fn test_cross_lab_lorentz_causal_nll_advantage() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let model_path = manifest_dir.join(LORENTZ_MODEL_PATH);
    let tables_path = manifest_dir.join(TABLES_PATH);

    if !model_path.exists() || !tables_path.exists() {
        return;
    }

    let model = IntegerModel::load_with_tables(&model_path, &tables_path)
        .expect("must load qat-lorentzflat_s1");

    // Sequence of repeated/pattern tokens to give memory read an active causal signal
    let sequence: Vec<u32> = vec![
        10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 10, 20,
        30, 40, 50, 60, 70, 80, 90, 100,
    ];

    // Compute NLL under ReadMode::Enabled
    let mut session_read = model.new_session();
    let mut nll_read = 0.0f64;
    for i in 0..sequence.len() - 1 {
        let current = sequence[i];
        let target = sequence[i + 1] as usize;
        let step = model
            .step(&mut session_read, current, ReadMode::Enabled)
            .expect("step read");
        let prob = (step.probabilities[target] as f64) / (TOTAL as f64);
        let clipped = prob.max(1e-12);
        nll_read -= clipped.ln();
    }

    // Compute NLL under ReadMode::NoRead
    let mut session_noread = model.new_session();
    let mut nll_noread = 0.0f64;
    for i in 0..sequence.len() - 1 {
        let current = sequence[i];
        let target = sequence[i + 1] as usize;
        let step = model
            .step(&mut session_noread, current, ReadMode::NoRead)
            .expect("step noread");
        let prob = (step.probabilities[target] as f64) / (TOTAL as f64);
        let clipped = prob.max(1e-12);
        nll_noread -= clipped.ln();
    }

    let avg_nll_read = nll_read / ((sequence.len() - 1) as f64);
    let avg_nll_noread = nll_noread / ((sequence.len() - 1) as f64);
    let delta_nll = avg_nll_noread - avg_nll_read;

    println!(
        "Lorentz Causal Ablation: Read NLL = {avg_nll_read:.4}, NoRead NLL = {avg_nll_noread:.4}, Delta NLL = {delta_nll:.4} nats"
    );

    // Memory read must improve prediction significantly on repeated sequence (> 5.0 nats advantage)
    assert!(
        delta_nll > 5.0,
        "Lorentz memory read must provide substantial causal predictive benefit (> 5.0 nats), got {delta_nll:.4}"
    );
}

/// Assert empirical transfer invariants recorded in sealed cross-lab evidence receipts
/// (`radial-parameter-transfer-validation-2026-09-27.json` and `dot-reset-validation-2026-09-27.json`).
///
/// Note (Scope Boundary): This test verifies artifact schema, source/parent SHA256 hashes,
/// transfer tensor counts, clean Adam clock/moment resets, and empirical startup disturbance
/// metrics recorded in sealed evidence. It is a receipt regression check, not a newly executed
/// live model fit or independent serving witness.
#[test]
fn test_cross_lab_radial_transfer_witness_invariants() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let radial_evidence_path = manifest_dir
        .join("../../docs/evidence/radial-parameter-transfer-validation-2026-09-27.json");
    let dot_evidence_path =
        manifest_dir.join("../../docs/evidence/dot-reset-validation-2026-09-27.json");

    assert!(
        radial_evidence_path.exists(),
        "radial evidence JSON missing at {:?}",
        radial_evidence_path
    );
    assert!(
        dot_evidence_path.exists(),
        "dot evidence JSON missing at {:?}",
        dot_evidence_path
    );

    let radial_data: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&radial_evidence_path).expect("read radial evidence"),
    )
    .expect("parse radial evidence JSON");

    let dot_data: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(&dot_evidence_path).expect("read dot evidence"),
    )
    .expect("parse dot evidence JSON");

    // Invariant 0: Receipt Provenance & Schema Integrity
    assert_eq!(
        radial_data["schema"],
        "uor-r4.radial-parameter-transfer-validation/1"
    );
    assert_eq!(dot_data["schema"], "uor-r4.dot-reset-validation/1");
    assert_eq!(
        radial_data["startup"]["evaluator_sha256"],
        "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62"
    );
    assert_eq!(
        dot_data["observations"]["evaluator_sha256"],
        "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62"
    );
    assert_eq!(
        dot_data["transfer"]["parent_model_sha256"],
        "6defec21fc2be395a9505b9f10301c03aeb3c23529c6e1be4e2e1639c7ce6d79"
    );

    // Invariant 1: Startup verification status and parent equality
    assert_eq!(
        radial_data["startup"]["status"],
        "DESCRIPTIVE_STARTUP_COMPLETE"
    );
    assert_eq!(
        radial_data["startup"]["shared_all_parameter_arrays_identical_before_observation"],
        true
    );
    assert_eq!(
        radial_data["startup"]["parameters_unchanged_after_generation"],
        true
    );
    assert_eq!(
        dot_data["observations"]["status"],
        "UNCHANGED_DOT_OUTPUT_WITH_FRESH_OPTIMIZER"
    );
    assert_eq!(
        dot_data["observations"]["parent_and_reset_predictions_identical"],
        true
    );
    assert_eq!(
        dot_data["observations"]["parent_and_reset_generation_identical"],
        true
    );
    assert_eq!(dot_data["observations"]["parameters_unchanged"], true);

    // Invariant 2: Parameter lineage transfer & zero moment pollution
    assert_eq!(dot_data["transfer"]["shared_array_count"], 21);
    assert_eq!(dot_data["transfer"]["copied_scalar_count"], 1678466);
    assert_eq!(
        dot_data["transfer"]["all_optimizer_parameter_clocks_zero"],
        true
    );
    assert_eq!(
        dot_data["transfer"]["named_moment_values_sha256"],
        "4091023f8b11f160787ab17ebd32568464507d8a2f5cec6b18b2df9ff7882116"
    );
    assert_eq!(
        dot_data["transfer"]["same_copied_parameter_manifest_as_both_prior_radial_arms"],
        true
    );
    assert_eq!(dot_data["transfer"]["historical_visits"], 64192512);

    // Invariant 3: Dot reset vs Radial transfer NLL disturbance and ordering
    let dot_nll = dot_data["observations"]["initial_next_token_nll"]
        .as_f64()
        .expect("dot initial nll");
    let lorentz_nll = radial_data["observations"]["lorentz"]["observed_initial_next_token_nll"]
        .as_f64()
        .expect("lorentz initial nll");
    let affine_nll = radial_data["observations"]["lorentz_affine"]
        ["observed_initial_next_token_nll"]
        .as_f64()
        .expect("affine initial nll");

    assert!(
        (dot_nll - 1.996717).abs() < 1e-4,
        "Dot reset initial NLL must match reference 1.996717"
    );
    assert!(
        (lorentz_nll - 2.389537).abs() < 1e-4,
        "Lorentz initial NLL must match reference 2.389537"
    );
    assert!(
        (affine_nll - 2.400457).abs() < 1e-4,
        "LorentzAffine initial NLL must match reference 2.400457"
    );
    assert!(
        affine_nll > lorentz_nll,
        "Affine initial disturbance ({affine_nll:.6}) should exceed Lorentz ({lorentz_nll:.6})"
    );
    assert!(
        lorentz_nll > dot_nll,
        "Lorentz initial NLL ({lorentz_nll:.6}) should show positive disturbance over Dot parent ({dot_nll:.6})"
    );

    // Invariant 4: Connected finite gradients across all 6 gradient families with expected dimensions
    let check_gradient_dimensions = |arm_data: &serde_json::Value| {
        assert_eq!(
            arm_data["all_required_gradient_arrays_have_nonzero_coordinates"],
            true
        );
        let grads = &arm_data["gradients"];
        assert_eq!(grads["read.query.weight"]["coordinates"], 16384);
        assert_eq!(grads["read.query.weight"]["finite"], true);
        assert_eq!(grads["read.key.weight"]["coordinates"], 16384);
        assert_eq!(grads["read.key.weight"]["finite"], true);
        assert_eq!(grads["read.value.weight"]["coordinates"], 65536);
        assert_eq!(grads["read.value.weight"]["finite"], true);
        assert_eq!(grads["read.no_read.weight"]["coordinates"], 256);
        assert_eq!(grads["read.no_read.weight"]["finite"], true);
        assert_eq!(grads["read.lorentz_log_beta"]["coordinates"], 1);
        assert_eq!(grads["read.lorentz_log_beta"]["finite"], true);
        assert_eq!(grads["read.lorentz_offset"]["coordinates"], 1);
        assert_eq!(grads["read.lorentz_offset"]["finite"], true);
    };
    check_gradient_dimensions(&radial_data["observations"]["lorentz"]);
    check_gradient_dimensions(&radial_data["observations"]["lorentz_affine"]);

    // Invariant 5: Causal read mass, conditional entropy, and no zero-read positions across 1,020 positions
    let lorentz_summary =
        &radial_data["observations"]["lorentz"]["summary_excludes_empty_history_position0"];
    assert_eq!(lorentz_summary["all_causal_read_positions_nonzero"], true);
    assert_eq!(lorentz_summary["zero_read_positions"], 0);
    assert_eq!(lorentz_summary["defined_entropy_positions"], 1020);
    let lorentz_read_mass = lorentz_summary["mean_read_mass"]
        .as_f64()
        .expect("lorentz mean read mass");
    let lorentz_entropy = lorentz_summary["mean_conditional_read_entropy_nats"]
        .as_f64()
        .expect("lorentz entropy");
    assert!(
        (lorentz_read_mass - 0.120842).abs() < 1e-4,
        "Lorentz mean read mass must match 0.120842"
    );
    assert!(
        (lorentz_entropy - 4.203201).abs() < 1e-4,
        "Lorentz entropy must match 4.203201"
    );

    let affine_summary =
        &radial_data["observations"]["lorentz_affine"]["summary_excludes_empty_history_position0"];
    assert_eq!(affine_summary["all_causal_read_positions_nonzero"], true);
    assert_eq!(affine_summary["zero_read_positions"], 0);
    assert_eq!(affine_summary["defined_entropy_positions"], 1020);
    let affine_read_mass = affine_summary["mean_read_mass"]
        .as_f64()
        .expect("affine mean read mass");
    let affine_entropy = affine_summary["mean_conditional_read_entropy_nats"]
        .as_f64()
        .expect("affine entropy");
    assert!(
        (affine_read_mass - 0.117484).abs() < 1e-4,
        "Affine mean read mass must match 0.117484"
    );
    assert!(
        (affine_entropy - 4.206362).abs() < 1e-4,
        "Affine entropy must match 4.206362"
    );

    // Invariant 6: Bit-identical post-token0 states & single-key score law difference
    assert_eq!(
        radial_data["first_common_read"]["post_token0_states_bit_identical"],
        true
    );
    let pos1_diff = radial_data["first_common_read"]["observations"][0]
        ["lorentz_minus_affine_read_log_odds"]
        .as_f64()
        .expect("pos 1 log-odds diff");
    assert!(
        (pos1_diff - 0.067784).abs() < 1e-4,
        "Position 1 Lorentz vs Affine log-odds diff must match 0.067784 nats"
    );

    // Invariant 7: Language continuation token IDs and response text behavior
    assert_eq!(
        dot_data["generation"]["response_text"],
        "toys and read them all day."
    );
    assert_eq!(
        dot_data["generation"]["generated_token_ids"],
        serde_json::json!([624, 269, 1298, 495, 434, 329, 16, 201])
    );
    assert_eq!(
        radial_data["generated_outputs"][0]["response_text"],
        "toys. One day, he found a"
    );
    assert_eq!(
        radial_data["generated_outputs"][0]["generated_token_ids"],
        serde_json::json!([624, 16, 530, 329, 14, 288, 505, 261])
    );
    assert_eq!(
        radial_data["generated_outputs"][1]["response_text"],
        "toys. One day, he found a"
    );
    assert_eq!(
        radial_data["generated_outputs"][1]["generated_token_ids"],
        serde_json::json!([624, 16, 530, 329, 14, 288, 505, 261])
    );
}
