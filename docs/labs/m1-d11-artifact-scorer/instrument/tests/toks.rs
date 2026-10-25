//! Stage 3 tests: the step-6 timing statistics and the serving loop.
use m1_d11_artifact_scorer_instrument::toks::{
    default_prompt, duration_stats, time_serving, Protocol, STEP6,
};
use uor_r4_discrete::d11_export::export_d11;
use uor_r4_discrete::metadata::BinderPlan;
use uor_r4_discrete::normalization_mechanism::NormalizationMode;
use uor_r4_discrete::q4_import::{COMPOSITION_KAPPA, OPERATOR_KAPPA};
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::{ObjectivePolicy, ReadInitialization, ReadSpec, Spec, SCHEMA};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

/// Six of the 16 orbit roots, as `discrete_d11.rs` lists them.
const ROOTS: [[i32; 8]; 6] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
];

/// A flock-read artifact over 11 tokens and 2 blocks.
fn model(context: u32) -> IntegerDiscreteModel {
    let spec = Spec {
        vocabulary: 11,
        blocks: 2,
        context,
        kept_cells: 3,
    };
    let read_spec = ReadSpec::Flock { kept: 2 };
    let stages = BinderPlan::new(context as usize).expect("plan").stages() as u32;
    let gate_len = read_spec.gate_count(spec.blocks, stages).expect("gates") as usize;
    let snapshot = Snapshot {
        schema: SCHEMA.to_owned(),
        spec,
        read_spec,
        objective_policy: ObjectivePolicy::NearestAddressMarginV1,
        read_initialization: ReadInitialization::ActiveLeafV1,
        normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        seed: 3,
        epoch: 1,
        table: (0..22).flat_map(|i| ROOTS[(i * 5 + i / 3) % 6]).collect(),
        routes: (0..2 * stages).map(|i| (i * 11) % 64).collect(),
        gates: (0..gate_len).map(|i| (i % 3) as i32 - 1).collect(),
        operator_kappa: OPERATOR_KAPPA.to_owned(),
        composition_kappa: COMPOSITION_KAPPA.to_owned(),
        kernel_sha256: "11".repeat(32),
        source_sha256: "22".repeat(32),
        geometry_sha256: "33".repeat(32),
    };
    IntegerDiscreteModel::from_bytes(&export_d11(&snapshot).expect("export")).expect("reader")
}

#[test]
fn statistics_follow_the_registered_definitions() {
    // 20 samples, 1..=20 ms, out of order.
    let samples: Vec<u64> = [
        7, 3, 20, 1, 15, 9, 11, 2, 18, 5, 13, 4, 19, 6, 17, 8, 14, 10, 16, 12,
    ]
    .iter()
    .map(|ms| ms * 1_000_000)
    .collect();
    let stats = duration_stats(192, &samples).expect("stats");
    // Median of 20: the mean of the 10th and 11th (10 and 11 ms).
    assert_eq!(stats["median_ns"], 10_500_000);
    // Nearest-rank p95 of 20: rank 19, the 19 ms sample.
    assert_eq!(stats["p95_rank"], 19);
    assert_eq!(stats["p95_ns"], 19_000_000);
    assert_eq!(
        stats["median_tokens_per_s"],
        192u64 * 1_000_000_000 / 10_500_000
    );
    assert_eq!(
        stats["p95_tokens_per_s"],
        192u64 * 1_000_000_000 / 19_000_000
    );
    assert_eq!(stats["raw_ns"].as_array().map(Vec::len), Some(20));
    assert_eq!(stats["raw_us"][0], 7_000);
    assert!(duration_stats(1, &[]).is_err());
    assert!(
        duration_stats(1, &[5, 0, 7]).is_err(),
        "a 0 ns sample is refused"
    );
}

#[test]
fn the_decode_loop_appends_predictions_and_slides_the_window() {
    let model = model(192);
    let protocol = Protocol {
        warmup: 1,
        runs: 3,
        prompt_tokens: 192,
        decode_tokens: 8,
    };
    let prompt = default_prompt(11, 192);
    let timing = time_serving(&model, &prompt, protocol).expect("timing");
    // The same reply, written step by step against evaluate_prefix.
    let mut window = prompt.clone();
    let mut expected = Vec::new();
    for _ in 0..8 {
        let next = model.evaluate_prefix(&window).expect("eval").prediction;
        expected.push(next);
        window.push(next);
        window.remove(0);
        assert_eq!(window.len(), 192);
    }
    assert_eq!(timing["reply"], serde_json::json!(expected));
    // The reply is not one token repeated, so appending a constant instead of
    // each prediction would fail the comparison above.
    assert!(
        expected.iter().any(|&t| t != expected[0]),
        "the fixture writes more than one distinct token"
    );
    assert_eq!(timing["prefill"]["runs"], 3);
    assert_eq!(timing["decode"]["runs"], 3);
    assert_eq!(timing["decode"]["tokens"], 8);
    assert_eq!(timing["protocol"]["prompt_tokens"], 192);
}

#[test]
fn the_protocol_refuses_a_short_context_or_prompt() {
    assert_eq!(
        (
            STEP6.warmup,
            STEP6.runs,
            STEP6.prompt_tokens,
            STEP6.decode_tokens
        ),
        (1, 20, 192, 64)
    );
    let short = model(81);
    assert!(time_serving(&short, &default_prompt(11, 192), STEP6)
        .expect_err("refused")
        .contains("context"));
    let full = model(192);
    assert!(time_serving(&full, &default_prompt(11, 100), STEP6)
        .expect_err("refused")
        .contains("prompt"));
}
