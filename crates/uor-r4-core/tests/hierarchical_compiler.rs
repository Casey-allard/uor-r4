use uor_r4_core::transformerless::compiler::{
    calibrate_hamming_regions_from_signatures, induce_hierarchical_codes, Corpus, K, SIG_BYTES,
    STAGES,
};

#[test]
fn test_induce_hierarchical_codes() {
    let n = 20;
    let story = vec![1u32; n];
    let mut next = vec![0u32; n];

    // Pattern: 10 -> 20 -> 30 repeated 6 times
    for chunk in 0..6 {
        let base = chunk * 3;
        if base + 2 < n {
            next[base] = 10;
            next[base + 1] = 20;
            next[base + 2] = 30;
        }
    }

    let corpus = Corpus {
        n,
        stories: 1,
        story,
        input: vec![1u32; n],
        next,
        t_argmax: vec![0u32; n],
        top_tokens: vec![[0u32; 8]; n],
        top_weights: vec![[0u32; 8]; n],
        span_start: (0..n).map(|idx| idx as u32).collect(),
        span_end: (0..n).map(|idx| idx as u32 + 1).collect(),
        byte_start: vec![u32::MAX; n],
        byte_end: vec![u32::MAX; n],
        hidden: None,
    };

    let vocab = 100;
    let mut token_codes = vec![0u8; vocab * STAGES];
    token_codes[10 * STAGES] = 1;
    token_codes[10 * STAGES + 1] = 2;
    token_codes[10 * STAGES + 2] = 3;
    token_codes[10 * STAGES + 3] = 4;

    let hc = induce_hierarchical_codes(&token_codes, vocab, &corpus);

    let first_json = serde_json::to_vec(&hc).expect("hierarchical codes serialize");
    let second_json = serde_json::to_vec(&hc).expect("hierarchical codes serialize");
    assert_eq!(first_json, second_json, "hierarchical JSON is canonical");

    // Verify stable type prefixes
    assert_eq!(hc.token_type_prefixes.get("10"), Some(&vec![1, 2, 3, 4]));

    // Verify relational prefixes (transition pair and triplet)
    assert!(
        !hc.relational_prefixes.is_empty(),
        "Relational prefixes should not be empty"
    );

    let has_pair = hc.relational_prefixes.iter().any(|path| path == &[10, 20]);
    let has_triplet = hc
        .relational_prefixes
        .iter()
        .any(|path| path == &[10, 20, 30]);
    assert!(has_pair, "Should contain transition pair [10, 20]");
    assert!(
        has_triplet,
        "Should contain transition triplet [10, 20, 30]"
    );
}

#[test]
fn hamming_calibration_emits_histograms_and_radii() {
    let mut class_sigs = vec![vec![0u8; K * SIG_BYTES]; STAGES];
    class_sigs[0][SIG_BYTES] = 1;
    let mut signatures = vec![[0u8; SIG_BYTES]; 2];
    signatures[1][0] = 1;

    let report = calibrate_hamming_regions_from_signatures(&class_sigs, &signatures);
    assert_eq!(report.signature_bits, (SIG_BYTES * 8) as u16);
    assert_eq!(report.quantile_numerator, 95);
    assert_eq!(report.quantile_denominator, 100);
    assert_eq!(report.regions.len(), STAGES * K);

    let stage0_class0 = report
        .regions
        .iter()
        .find(|region| region.stage == 0 && region.class == 0)
        .expect("stage 0 class 0");
    assert_eq!(stage0_class0.sample_count, 1);
    assert_eq!(stage0_class0.acceptance_radius, 0);
    assert_eq!(stage0_class0.hamming_histogram[0], 1);

    let stage0_class1 = report
        .regions
        .iter()
        .find(|region| region.stage == 0 && region.class == 1)
        .expect("stage 0 class 1");
    assert_eq!(stage0_class1.sample_count, 1);
    assert_eq!(stage0_class1.acceptance_radius, 0);
    assert_eq!(stage0_class1.hamming_histogram[0], 1);
}

#[test]
fn hamming_calibration_ignores_invalid_stage_layout() {
    let class_sigs = vec![vec![]; STAGES];
    let signatures = vec![[0u8; SIG_BYTES]; 1];

    let report = calibrate_hamming_regions_from_signatures(&class_sigs, &signatures);
    assert_eq!(report.regions.len(), STAGES * K);
    assert!(report.regions.iter().all(|region| region.sample_count == 0));
    assert!(report
        .regions
        .iter()
        .all(|region| region.acceptance_radius == 0));
}
