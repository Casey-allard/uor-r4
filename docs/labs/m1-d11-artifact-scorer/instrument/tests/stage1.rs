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
