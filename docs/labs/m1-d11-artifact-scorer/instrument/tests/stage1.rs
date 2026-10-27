//! Stage 1 tests: the export round-trip and the DEV row builder.

use m1_d11_artifact_scorer_instrument as inst;
use serde_json::json;
use uor_r4_discrete::corpus::{sha256, BosIdentity, CorpusSource, FilePin, PreparedCorpus, Split};
use uor_r4_discrete::d11_export::export_d11;
use uor_r4_discrete::metadata::BinderPlan;
use uor_r4_discrete::normalization_mechanism::NormalizationMode;
use uor_r4_discrete::q4_import::{COMPOSITION_KAPPA, OPERATOR_KAPPA};
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::training::CHECKPOINT_SCHEMA;
use uor_r4_discrete::{ObjectivePolicy, ReadInitialization, ReadSpec, Spec, SCHEMA};

const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
const C_ROOT: [i32; 8] = [-2, 2, 0, 0, 0, 0, 0, 0];
const NEG_B: [i32; 8] = [0, 0, -2, 0, -2, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];
const D_ROOT: [i32; 8] = [2, -2, 0, 0, 0, 0, 0, 0];

/// The `d11_export.rs` test fixture, built through public paths only.
fn sample_snapshot() -> Snapshot {
    let tokens = [
        [B, A],
        [A, D_ROOT],
        [NEG_A, NEG_B],
        [C_ROOT, A],
        [D_ROOT, A],
    ];
    let spec = Spec {
        vocabulary: tokens.len() as u32,
        blocks: 2,
        context: 6,
        kept_cells: 2,
    };
    let read_spec = ReadSpec::Relation { kept: 1, sweeps: 1 };
    let stages = BinderPlan::new(spec.context as usize)
        .expect("binder plan")
        .stages() as u32;
    let gate_len = read_spec
        .gate_count(spec.blocks, stages)
        .expect("gate count") as usize;
    let mut gates = vec![0i32; gate_len];
    gates[2] = 1;
    gates[6] = 1;
    Snapshot {
        schema: SCHEMA.to_owned(),
        spec,
        read_spec,
        objective_policy: ObjectivePolicy::NearestAddressMarginV1,
        read_initialization: ReadInitialization::ActiveLeafV1,
        normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        seed: 7,
        epoch: 3,
        table: tokens.iter().flatten().flatten().copied().collect(),
        routes: vec![0u32; (spec.blocks * stages) as usize],
        gates,
        operator_kappa: OPERATOR_KAPPA.to_owned(),
        composition_kappa: COMPOSITION_KAPPA.to_owned(),
        kernel_sha256: "11".repeat(32),
        source_sha256: "22".repeat(32),
        geometry_sha256: "33".repeat(32),
    }
}

fn checkpoint_for(snapshot: &Snapshot, accepted: u64) -> serde_json::Value {
    let model = serde_json::to_value(snapshot).expect("model value");
    let stored = sha256(&serde_json::to_vec(&model).expect("model bytes"));
    json!({"schema": CHECKPOINT_SCHEMA, "model_sha256": stored, "model": model,
           "cursor": {"accepted_updates": accepted}})
}

#[test]
fn export_is_export_d11_unchanged_and_the_integer_reader_accepts_it() {
    let snapshot = sample_snapshot();
    let expected = export_d11(&snapshot).expect("export_d11");
    let exported = inst::export_snapshot(&snapshot).expect("export");
    assert_eq!(
        exported.bytes, expected,
        "export must write export_d11's bytes, unchanged"
    );
    assert_eq!(exported.report["file_sha256"], sha256(&expected));
    assert_eq!(exported.report["bytes"], expected.len());
    let through_checkpoint =
        inst::export_checkpoint(&checkpoint_for(&snapshot, 3)).expect("checkpoint");
    assert_eq!(through_checkpoint.bytes, expected);
    assert_eq!(through_checkpoint.report["accepted_updates"], 3);
}

#[test]
fn a_checkpoint_with_a_wrong_hash_epoch_or_schema_is_refused() {
    let snapshot = sample_snapshot();
    let mut tampered = checkpoint_for(&snapshot, 3);
    tampered["model_sha256"] = json!("00".repeat(32));
    assert!(inst::export_checkpoint(&tampered)
        .err()
        .expect("refused")
        .contains("hash mismatch"));
    let wrong_epoch = checkpoint_for(&snapshot, 4);
    assert!(inst::export_checkpoint(&wrong_epoch)
        .err()
        .expect("refused")
        .contains("epoch"));
    let mut not_a_checkpoint = checkpoint_for(&snapshot, 3);
    not_a_checkpoint["schema"] = json!("uor-r4.discrete-corpus-driver/1");
    assert!(inst::export_checkpoint(&not_a_checkpoint)
        .err()
        .expect("refused")
        .contains("checkpoint"));
}

/// The `corpus.rs` test fixture: an 8-token stream with two documents.
fn fixture_corpus() -> PreparedCorpus {
    let tokens: [u16; 8] = [0, 1, 2, 3, 0, 4, 5, 6];
    let mask = [1u8, 1, 1, 1, 1, 0, 1, 1];
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
    for t in tokens {
        bytes.extend_from_slice(&t.to_le_bytes());
    }
    let source = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: "fixture.uort".into(),
            sha256: sha256(&bytes),
        },
        mask: FilePin {
            path: "fixture.mask".into(),
            sha256: sha256(&mask),
        },
        expected_payload_tokens: tokens.len() as u64,
        expected_eligible_targets: 5,
    };
    let bos = BosIdentity {
        token: "<|bos|>".into(),
        id: 0,
    };
    PreparedCorpus::from_bytes(source, &bytes, &mask, 8, bos).expect("fixture corpus")
}

#[test]
fn rows_follow_the_eval_walk_and_do_not_depend_on_the_batch_size() {
    let corpus = fixture_corpus();
    let context = 4;
    let one = inst::rows_from_corpus(&corpus, context, 1).expect("rows at batch 1");
    let two = inst::rows_from_corpus(&corpus, context, 2).expect("rows at batch 2");
    let all = inst::rows_from_corpus(&corpus, context, 64).expect("rows at batch 64");
    assert_eq!(one, two);
    assert_eq!(one, all);
    assert_eq!(one.len(), corpus.eligible_len());
    let stream = corpus.tokens();
    for (i, row) in one.iter().enumerate() {
        let position = row.target_position as usize;
        assert_eq!(row.ordinal, i as u64);
        assert_eq!(
            row.target_token, stream[position],
            "the target is the token at its position"
        );
        assert!(!row.prefix.is_empty() && row.prefix.len() <= context as usize);
        assert_eq!(
            row.prefix.last().copied(),
            Some(stream[position - 1]),
            "the prefix ends just before its target"
        );
    }
    assert_eq!(inst::rows_sha256(&one), inst::rows_sha256(&all));
    let mut changed = one.clone();
    changed[0].target_token ^= 1;
    assert_ne!(
        inst::rows_sha256(&one),
        inst::rows_sha256(&changed),
        "the hash covers each row"
    );
}

/// The registered DEV panel: 14,287 rows. Set `UOR_DEV_CONFIG` to a driver
/// config whose DEV files exist (for example on a pod). Run without it, the
/// test fails: an ignored test that was asked for never passes by skipping.
#[test]
#[ignore = "needs the DEV files: set UOR_DEV_CONFIG"]
fn the_registered_dev_panel_has_14287_rows() {
    let path = std::env::var("UOR_DEV_CONFIG")
        .expect("set UOR_DEV_CONFIG to a driver config whose DEV files exist");
    let bytes = std::fs::read(&path).expect("config");
    let config = serde_json::from_slice(&bytes).expect("driver config");
    let rows = inst::dev_rows(&config).expect("dev rows");
    assert_eq!(rows.len(), 14_287);
}

#[test]
fn dev_rows_and_rows_from_corpus_refuse_sealed_fresh_panel_before_sealed_dev_decision() {
    let tokens: [u16; 8] = [0, 1, 2, 3, 0, 4, 5, 6];
    let mask = [1u8, 1, 1, 1, 1, 0, 1, 1];
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
    for t in tokens {
        bytes.extend_from_slice(&t.to_le_bytes());
    }
    let fresh_source = CorpusSource {
        split: Split::Fresh,
        tokens: FilePin {
            path: "fresh.uort".into(),
            sha256: sha256(&bytes),
        },
        mask: FilePin {
            path: "fresh.mask".into(),
            sha256: sha256(&mask),
        },
        expected_payload_tokens: tokens.len() as u64,
        expected_eligible_targets: 5,
    };
    let bos = BosIdentity {
        token: "<|bos|>".into(),
        id: 0,
    };
    let fresh_corpus =
        PreparedCorpus::from_bytes(fresh_source, &bytes, &mask, 8, bos).expect("fresh corpus");
    let err = inst::rows_from_corpus(&fresh_corpus, 4, 1)
        .expect_err("rows_from_corpus must refuse Split::Fresh without a sealed DEV decision");
    assert!(
        err.contains("SEALED_FRESH_PANEL_REFUSED"),
        "expected SEALED_FRESH_PANEL_REFUSED, got: {err}"
    );

    // Also verify that a Split::Dev CorpusSource carrying the sealed Fresh V2 or K16
    // hashes is refused before touching disk.
    for (tok_sha, mask_sha) in [
        (
            inst::fresh_transfer::FRESH_V2_TOKENS_SHA256,
            inst::fresh_transfer::FRESH_V2_MASK_SHA256,
        ),
        (
            inst::fresh_transfer::K16_FRESH_TOKENS_SHA256,
            inst::fresh_transfer::K16_FRESH_MASK_SHA256,
        ),
    ] {
        let disguised_dev = CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: "/nonexistent/tokens.u16".into(),
                sha256: tok_sha.into(),
            },
            mask: FilePin {
                path: "/nonexistent/response_mask.u8".into(),
                sha256: mask_sha.into(),
            },
            expected_payload_tokens: 44_678,
            expected_eligible_targets: 14_479,
        };
        let err = inst::fresh_transfer::refuse_sealed_fresh_source(&disguised_dev)
            .expect_err("disguised fresh source must be refused");
        assert!(err.contains("SEALED_FRESH_PANEL_REFUSED"));
    }
}

#[test]
fn fresh_transfer_gate_refuses_until_two_seed_dev_bar_passes_and_evaluates_exact_mcnemar_bars() {
    use inst::fresh_transfer::{
        build_train_count_tables, evaluate_fresh_transfer_two_seeds, mcnemar_floor,
        score_fresh_against_count_tables, validate_fresh_transfer_gate, DEV_ELIGIBLE_TARGETS,
        FRESH_TRANSFER_GATE_SCHEMA, FRESH_V2_ELIGIBLE_TARGETS, FRESH_V2_MASK_SHA256,
        FRESH_V2_PAYLOAD_TOKENS, FRESH_V2_ROW_BINDING_SHA256, FRESH_V2_SEAL_SHA256,
        FRESH_V2_TOKENS_SHA256, TRAIN_MASK_SHA256, TRAIN_TOKENS_SHA256,
    };

    let fresh_v2_source = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: "/nonexistent/fresh_v2/tokens.u16".into(),
            sha256: FRESH_V2_TOKENS_SHA256.into(),
        },
        mask: FilePin {
            path: "/nonexistent/fresh_v2/response_mask.u8".into(),
            sha256: FRESH_V2_MASK_SHA256.into(),
        },
        expected_payload_tokens: FRESH_V2_PAYLOAD_TOKENS,
        expected_eligible_targets: FRESH_V2_ELIGIBLE_TARGETS,
    };
    let art1 = "aa".repeat(32);
    let art2 = "bb".repeat(32);

    let valid_receipt = json!({
        "schema": FRESH_TRANSFER_GATE_SCHEMA,
        "dev_decision": "PASS_BARS_A_TO_D_TWO_SEEDS",
        "fresh_seal_sha256": FRESH_V2_SEAL_SHA256,
        "fresh_row_binding_sha256": FRESH_V2_ROW_BINDING_SHA256,
        "train_tokens_sha256": TRAIN_TOKENS_SHA256,
        "train_mask_sha256": TRAIN_MASK_SHA256,
        "fit_review_url": "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-1",
        "prerun_review_url": "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-2",
        "seeds": [
            {
                "seed": 1,
                "dev_rows": DEV_ELIGIBLE_TARGETS,
                "d11_top1": 6_400,
                "export_gap_b2_pass": true,
                "planted_fault_b4_pass": true,
                "tokens_per_s_reported": true,
                "artifact_sha256": art1,
            },
            {
                "seed": 2,
                "dev_rows": DEV_ELIGIBLE_TARGETS,
                "d11_top1": 6_380,
                "export_gap_b2_pass": true,
                "planted_fault_b4_pass": true,
                "tokens_per_s_reported": true,
                "artifact_sha256": art2,
            }
        ]
    });

    // Valid two-seed PASS receipt passes gate validation for both bound artifacts.
    validate_fresh_transfer_gate(&valid_receipt, &art1, &fresh_v2_source)
        .expect("valid receipt for seed 1");
    validate_fresh_transfer_gate(&valid_receipt, &art2, &fresh_v2_source)
        .expect("valid receipt for seed 2");

    // Unbound artifact SHA-256 is refused.
    assert!(
        validate_fresh_transfer_gate(&valid_receipt, &"cc".repeat(32), &fresh_v2_source).is_err()
    );

    // Current Stage T checkpoints (455 and 506 on DEV, below T2 6,300) are refused.
    let mut below_bar = valid_receipt.clone();
    below_bar["seeds"][0]["d11_top1"] = json!(455);
    below_bar["seeds"][1]["d11_top1"] = json!(506);
    let err = validate_fresh_transfer_gate(&below_bar, &art1, &fresh_v2_source)
        .expect_err("DEV scores below 6,300 T2 bar must be refused");
    assert!(err.contains("SEALED_FRESH_PANEL_REFUSED"));

    // Non-PASS dev_decision is refused.
    let mut held = valid_receipt.clone();
    held["dev_decision"] = json!("NOT_TRIGGERED");
    assert!(validate_fresh_transfer_gate(&held, &art1, &fresh_v2_source).is_err());

    // Exact integer McNemar floor checks:
    // discordant = 100 -> 2 * sqrt(100) = 20 -> max(24, 20) = 24
    assert_eq!(mcnemar_floor(24, 100), 24);
    // discordant = 145 -> 4 * 145 = 580 -> 24^2 = 576 < 580 <= 25^2 = 625 -> 25
    assert_eq!(mcnemar_floor(24, 145), 25);
    // P_succ floor: min 20, discordant = 100 -> max(20, 20) = 20; discordant = 101 -> 21
    assert_eq!(mcnemar_floor(20, 100), 20);
    assert_eq!(mcnemar_floor(20, 101), 21);

    // Build count tables from a Split::Train fixture corpus and score synthetic fresh rows.
    let tokens: [u16; 10] = [0, 1, 2, 1, 2, 1, 2, 0, 1, 2];
    let mask = [1u8; 10];
    let mut bytes = vec![0u8; 64];
    bytes[..4].copy_from_slice(b"UORT");
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
    bytes[16..20].copy_from_slice(&8u32.to_le_bytes());
    for t in tokens {
        bytes.extend_from_slice(&t.to_le_bytes());
    }
    let train_source = CorpusSource {
        split: Split::Train,
        tokens: FilePin {
            path: "train.uort".into(),
            sha256: sha256(&bytes),
        },
        mask: FilePin {
            path: "train.mask".into(),
            sha256: sha256(&mask),
        },
        expected_payload_tokens: tokens.len() as u64,
        expected_eligible_targets: 8,
    };
    let bos = BosIdentity {
        token: "<|bos|>".into(),
        id: 0,
    };
    let train_corpus =
        PreparedCorpus::from_bytes(train_source, &bytes, &mask, 8, bos).expect("train corpus");
    let tables = build_train_count_tables(&train_corpus, 4, 4, 4).expect("train count tables");

    let sample_rows = vec![inst::DevRow {
        ordinal: 0,
        target_position: 4,
        target_token: 3,
        prefix: vec![0, 1, 3, 1],
    }];
    let sample_scores = vec![inst::kernel::RowScore {
        status: 0,
        prediction: 3,
        top8: vec![3, 2, 1, 0, 4, 5, 6, 7],
        admitted_cells: vec![0],
        read_vector: vec![0; 32],
        token_hinge: 0,
        address_hinge: 0,
        rank_hinge: 0,
        loss_ta: 0,
        loss_tr: 0,
    }];
    let s1 =
        score_fresh_against_count_tables(1, &art1, &tables, &sample_rows, &sample_scores, 0, true)
            .expect("score s1");
    assert_eq!(s1.rows, 1);
    assert_eq!(s1.psucc_rows, 1);
    assert_eq!(s1.model_top1, 1);
    assert_eq!(s1.t2_top1, 0);

    // Two-seed evaluator requires the full floor (24 on full panel, 20 on P_succ).
    let mut s1_full = s1.clone();
    s1_full.rows = 14_479;
    s1_full.psucc_rows = 3_594;
    s1_full.unigram_top1 = 1_240;
    s1_full.t2_top1 = 6_380;
    s1_full.model_top1 = 6_450;
    s1_full.b_vs_unigram = 5_310;
    s1_full.c_vs_unigram = 100;
    s1_full.b_vs_t2 = 120;
    s1_full.c_vs_t2 = 50;
    s1_full.psucc_unigram_top1 = 300;
    s1_full.psucc_t2_top1 = 2_100;
    s1_full.psucc_model_top1 = 2_140;
    s1_full.psucc_b_vs_t2 = 60;
    s1_full.psucc_c_vs_t2 = 20;

    let mut s2_full = s1_full.clone();
    s2_full.seed = 2;
    s2_full.artifact_sha256 = art2;
    s2_full.model_top1 = 6_440;
    s2_full.psucc_model_top1 = 2_135;

    let report = evaluate_fresh_transfer_two_seeds(&s1_full, &s2_full).expect("two-seed report");
    assert_eq!(report["bar_e_pass"], json!(true));
    assert_eq!(report["outcome"], json!("PASS_BAR_E_FRESH_TRANSFER"));
}

#[test]
fn fresh_transfer_pins_match_sealed_corpora_and_export_trunk_with_scorer_roundtrips() {
    use inst::fresh_transfer::{
        refuse_sealed_fresh_source, DEV_MASK_SHA256, DEV_TOKENS_SHA256, TRAIN_MASK_SHA256,
        TRAIN_TOKENS_SHA256,
    };
    assert_eq!(
        DEV_TOKENS_SHA256,
        "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b"
    );
    assert_eq!(
        DEV_MASK_SHA256,
        "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078"
    );
    assert_eq!(
        TRAIN_TOKENS_SHA256,
        "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8"
    );
    assert_eq!(
        TRAIN_MASK_SHA256,
        "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3"
    );

    // Disguised Stage 3 cut_fresh_slice digests must also be refused on the DEV path.
    let stage3_cut_disguised = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: "/nonexistent/tokens.u16".into(),
            sha256: "1f8bf88d8deee51574c68ffd8976ce4415fc8aed5602213d6905a37a138517ea".into(),
        },
        mask: FilePin {
            path: "/nonexistent/response_mask.u8".into(),
            sha256: "fe87363920e19bf072e9fe3a7289e6cd2f1abde6c2431caae8f5c8494d3829d5".into(),
        },
        expected_payload_tokens: 46_669,
        expected_eligible_targets: 14_287,
    };
    assert!(refuse_sealed_fresh_source(&stage3_cut_disguised).is_err());
    assert!(inst::fresh_transfer::is_prefix_copy_successor(&[0, 7, 12, 7], 12));
    assert!(
        !inst::fresh_transfer::is_prefix_copy_successor(&[0, 7, 12, 7, 19, 7], 12),
        "copy-successor checks only the most recent prior occurrence of query (rposition)"
    );

    // Exercise `export_trunk_with_scorer` end-to-end on a 4-block, 4096-vocab fixture.
    use uor_r4_discrete::normalization_mechanism::orbit_root_coords;
    use uor_r4_discrete::training::{
        CodebookInitialization, CounterfactualPath, DriverConfig, RelationLearningPolicy,
        SchedulePolicy,
    };
    use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

    let spec = Spec {
        vocabulary: 4096,
        blocks: 4,
        context: 6,
        kept_cells: 8,
    };
    let stages = BinderPlan::new(spec.context as usize).unwrap().stages() as u32;
    let read_spec = ReadSpec::Relation { kept: 1, sweeps: 1 };
    let gate_len = read_spec.gate_count(spec.blocks, stages).unwrap() as usize;
    let mut table = Vec::with_capacity(4096 * 32);
    for v in 0..4096u32 {
        let r_arr = [
            (v & 0xF) as u8,
            ((v >> 4) & 0xF) as u8,
            ((v >> 8) & 0xF) as u8,
            ((v.wrapping_mul(7) ^ (v >> 3)) & 0xF) as u8,
        ];
        for b in 0..4 {
            table.extend_from_slice(&orbit_root_coords(r_arr[b]).unwrap());
        }
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
        epoch: 3,
        table,
        routes: vec![0u32; (spec.blocks * stages) as usize],
        gates: vec![0i32; gate_len],
        operator_kappa: OPERATOR_KAPPA.to_owned(),
        composition_kappa: COMPOSITION_KAPPA.to_owned(),
        kernel_sha256: "11".repeat(32),
        source_sha256: "22".repeat(32),
        geometry_sha256: "33".repeat(32),
    };
    let ckpt = checkpoint_for(&snapshot, 3);

    let tmp_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../local/antigravity/test-export-trunk-fixture");
    std::fs::create_dir_all(&tmp_dir).unwrap();
    let tok_words: [u16; 8] = [0, 7, 12, 7, 19, 0, 7, 12];
    let dev_tok_words: [u16; 8] = [0, 11, 23, 11, 5, 0, 11, 23];
    let mask_bytes = [1u8; 8];
    let mut uort_bytes = vec![0u8; 64];
    uort_bytes[..4].copy_from_slice(b"UORT");
    uort_bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    uort_bytes[8..16].copy_from_slice(&(tok_words.len() as u64).to_le_bytes());
    uort_bytes[16..20].copy_from_slice(&4096u32.to_le_bytes());
    for t in tok_words {
        uort_bytes.extend_from_slice(&t.to_le_bytes());
    }
    let mut dev_uort_bytes = vec![0u8; 64];
    dev_uort_bytes[..4].copy_from_slice(b"UORT");
    dev_uort_bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    dev_uort_bytes[8..16].copy_from_slice(&(dev_tok_words.len() as u64).to_le_bytes());
    dev_uort_bytes[16..20].copy_from_slice(&4096u32.to_le_bytes());
    for t in dev_tok_words {
        dev_uort_bytes.extend_from_slice(&t.to_le_bytes());
    }
    let train_tok = tmp_dir.join("train_tokens.u16");
    let train_mask = tmp_dir.join("train_mask.u8");
    let dev_tok = tmp_dir.join("dev_tokens.u16");
    let dev_mask = tmp_dir.join("dev_mask.u8");
    let tok_json = tmp_dir.join("tokenizer.json");
    let tbl_json = tmp_dir.join("table.json");
    std::fs::write(&train_tok, &uort_bytes).unwrap();
    std::fs::write(&train_mask, &mask_bytes).unwrap();
    std::fs::write(&dev_tok, &dev_uort_bytes).unwrap();
    std::fs::write(&dev_mask, &mask_bytes).unwrap();
    std::fs::write(&tok_json, b"{}").unwrap();
    std::fs::write(&tbl_json, b"[]").unwrap();

    let cfg = DriverConfig {
        schema: "uor-r4.discrete-corpus-driver/1".into(),
        spec,
        read_spec,
        objective_policy: ObjectivePolicy::NearestAddressMarginV1,
        read_initialization: ReadInitialization::ActiveLeafV1,
        normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        relation_learning_policy: RelationLearningPolicy::FrozenZeroV1,
        counterfactual_path: CounterfactualPath::RowsV1,
        codebook_initialization: CodebookInitialization::PinnedTableV1 {
            table: FilePin {
                path: tbl_json,
                sha256: sha256(b"[]"),
            },
        },
        coordinate_block: uor_r4_discrete::training::CoordinateBlock(16),
        seed: 1,
        batch_rows: 4,
        train: CorpusSource {
            split: Split::Train,
            tokens: FilePin {
                path: train_tok,
                sha256: sha256(&uort_bytes),
            },
            mask: FilePin {
                path: train_mask,
                sha256: sha256(&mask_bytes),
            },
            expected_payload_tokens: 8,
            expected_eligible_targets: 6,
        },
        dev: CorpusSource {
            split: Split::Dev,
            tokens: FilePin {
                path: dev_tok,
                sha256: sha256(&dev_uort_bytes),
            },
            mask: FilePin {
                path: dev_mask,
                sha256: sha256(&mask_bytes),
            },
            expected_payload_tokens: 8,
            expected_eligible_targets: 6,
        },
        tokenizer: FilePin {
            path: tok_json,
            sha256: sha256(b"{}"),
        },
        bos: BosIdentity {
            token: "<|bos|>".into(),
            id: 0,
        },
        schedule: SchedulePolicy::InterleavedReadEvery8V1,
    };
    let exported = inst::export_trunk_with_scorer(&ckpt, &cfg, 4, |tables, packed| {
        inst::kernel::host_score(tables, packed)
    })
    .expect("export_trunk_with_scorer");
    let _ = std::fs::remove_dir_all(&tmp_dir);
    assert_eq!(exported.report["round_trip_verified"], json!(true));
    assert_eq!(
        exported.report["trunk_dev_sample_reference_mismatches"],
        json!(0)
    );
    let model = IntegerDiscreteModel::from_bytes(&exported.bytes).expect("trunk model");
    assert!(model.is_trunk_d11());
}

#[test]
fn m1_fresh_panel_transfer_evidence_and_sheet_pins_pass_bar_e() {
    use inst::fresh_transfer::{
        validate_fresh_transfer_gate, DEV_MASK_SHA256, DEV_TOKENS_SHA256,
        FRESH_V2_ELIGIBLE_TARGETS, FRESH_V2_MASK_SHA256, FRESH_V2_PAYLOAD_TOKENS,
        FRESH_V2_PSUCC_ROWS, FRESH_V2_TOKENS_SHA256, TRAIN_MASK_SHA256, TRAIN_TOKENS_SHA256,
    };

    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let ev_dir = root.join("docs/labs/m1-fresh-panel-transfer/evidence");
    let gate_receipt: serde_json::Value = serde_json::from_slice(
        &std::fs::read(ev_dir.join("trunk-joint-pointer-v1-d11-fresh-gate-receipt.json"))
            .expect("fresh gate receipt in docs/labs/m1-fresh-panel-transfer/evidence"),
    )
    .expect("parse fresh gate receipt");
    let rpos_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(ev_dir.join("t16-frozen-zero-trunk-fresh-score-rposition.json"))
            .expect("rposition fresh score in docs/labs/m1-fresh-panel-transfer/evidence"),
    )
    .expect("parse rposition fresh score");
    let win2_report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(ev_dir.join("t16-frozen-zero-trunk-fresh-score.json"))
            .expect("windows2 fresh score in docs/labs/m1-fresh-panel-transfer/evidence"),
    )
    .expect("parse windows2 fresh score");

    let fresh_v2_source = CorpusSource {
        split: Split::Dev,
        tokens: FilePin {
            path: "/nonexistent/fresh_v2/tokens.u16".into(),
            sha256: FRESH_V2_TOKENS_SHA256.into(),
        },
        mask: FilePin {
            path: "/nonexistent/fresh_v2/response_mask.u8".into(),
            sha256: FRESH_V2_MASK_SHA256.into(),
        },
        expected_payload_tokens: FRESH_V2_PAYLOAD_TOKENS,
        expected_eligible_targets: FRESH_V2_ELIGIBLE_TARGETS,
    };
    let s1_art = "04652a3de8e966a23539b04fbd77c7d9872d9f13a541fcb224d48e766e2c147d";
    let s2_art = "1b49e8bd823a1021e31de55c767111f9cf43807d3e6d007644e49281935877de";
    validate_fresh_transfer_gate(&gate_receipt, s1_art, &fresh_v2_source)
        .expect("gate receipt validates seed 1");
    validate_fresh_transfer_gate(&gate_receipt, s2_art, &fresh_v2_source)
        .expect("gate receipt validates seed 2");

    for rep in [&rpos_report, &win2_report] {
        assert_eq!(rep["bar_e_pass"], json!(true));
        assert_eq!(rep["outcome"], json!("PASS_BAR_E_FRESH_TRANSFER"));
        assert_eq!(rep["seed_1"]["rows"], json!(FRESH_V2_ELIGIBLE_TARGETS));
        assert_eq!(rep["seed_1"]["model_top1"], json!(7_215));
        assert_eq!(rep["seed_2"]["model_top1"], json!(7_228));
        assert_eq!(rep["seed_1"]["unigram_top1"], json!(1_279));
        assert_eq!(rep["seed_1"]["t2_top1"], json!(6_582));
        assert_eq!(rep["seed_1"]["refused_rows"], json!(0));
        assert_eq!(rep["seed_2"]["refused_rows"], json!(0));
        assert_eq!(rep["seed_1"]["reference_mismatches"], json!(0));
        assert_eq!(rep["seed_2"]["reference_mismatches"], json!(0));
        assert_eq!(rep["seed_1"]["b4_pass"], json!(true));
        assert_eq!(rep["seed_2"]["b4_pass"], json!(true));
    }
    assert_eq!(rpos_report["seed_1"]["psucc_rows"], json!(FRESH_V2_PSUCC_ROWS));
    assert_eq!(rpos_report["seed_1"]["psucc_model_top1"], json!(1_713));
    assert_eq!(rpos_report["seed_2"]["psucc_model_top1"], json!(1_711));
    assert_eq!(rpos_report["seed_1"]["psucc_t2_top1"], json!(1_240));
    assert_eq!(win2_report["seed_1"]["psucc_rows"], json!(4_900));
    assert_eq!(win2_report["seed_1"]["psucc_model_top1"], json!(2_333));
    assert_eq!(win2_report["seed_2"]["psucc_model_top1"], json!(2_329));
    assert_eq!(win2_report["seed_1"]["psucc_t2_top1"], json!(1_863));

    let sheet = std::fs::read_to_string(
        root.join("docs/harness/prep/m1-fresh-panel-transfer-2026-10-26.md"),
    )
    .expect("read prep sheet");
    for pin in [
        TRAIN_TOKENS_SHA256,
        TRAIN_MASK_SHA256,
        DEV_TOKENS_SHA256,
        DEV_MASK_SHA256,
        FRESH_V2_TOKENS_SHA256,
        FRESH_V2_MASK_SHA256,
    ] {
        assert!(
            sheet.contains(pin),
            "prep sheet must contain registered corpus SHA-256 pin {pin}"
        );
    }
}

