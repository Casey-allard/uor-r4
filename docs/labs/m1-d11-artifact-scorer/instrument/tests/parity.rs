//! Stage 2 finite parity tests: the scorer's row body, built for the host,
//! against `IntegerDiscreteModel::evaluate_prefix` on fixture artifacts in all
//! three read modes, up to the 243-token context and the 16-block bound.
use m1_d11_artifact_scorer_instrument::artifact::ArtifactTables;
use m1_d11_artifact_scorer_instrument::kernel::{
    compare_row, host_record_sizes, host_score, pack_rows, pack_rows_with_targets, row_plans,
    PackedRows, RowScore, PLAN_WORDS, ROW_OUT_WORDS, SCRATCH_WORDS,
};
use m1_d11_artifact_scorer_instrument::{
    compare_with_train_eval, planted_fault_control, reference_check, score_summary, DevRow,
};
use uor_r4_discrete::d11_export::export_d11;
use uor_r4_discrete::metadata::BinderPlan;
use uor_r4_discrete::normalization_mechanism::NormalizationMode;
use uor_r4_discrete::q4_import::{COMPOSITION_KAPPA, OPERATOR_KAPPA};
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::{ObjectivePolicy, ReadInitialization, ReadSpec, Spec, SCHEMA};
use uor_r4_integer::discrete_d11::{DiscretePrefixEvaluation, IntegerDiscreteModel};

/// The 16-root orbit, as `discrete_d11.rs` lists it. A wrong entry here makes
/// the integer reader refuse the fixture, so it cannot pass silently.
const ROOTS: [[i32; 8]; 16] = [
    [2, 2, 0, 0, 0, 0, 0, 0],
    [-2, -2, 0, 0, 0, 0, 0, 0],
    [-2, 2, 0, 0, 0, 0, 0, 0],
    [0, 0, -2, 0, -2, 0, 0, 0],
    [0, 0, -2, 0, 2, 0, 0, 0],
    [0, 0, 0, -2, 0, 0, 0, -2],
    [0, 0, 0, -2, 0, 0, 0, 2],
    [0, 0, 0, 0, 0, -2, -2, 0],
    [0, 0, 0, 0, 0, -2, 2, 0],
    [0, 0, 0, 0, 0, 2, -2, 0],
    [0, 0, 0, 0, 0, 2, 2, 0],
    [0, 0, 0, 2, 0, 0, 0, -2],
    [0, 0, 0, 2, 0, 0, 0, 2],
    [0, 0, 2, 0, -2, 0, 0, 0],
    [0, 0, 2, 0, 2, 0, 0, 0],
    [2, -2, 0, 0, 0, 0, 0, 0],
];

/// xorshift64: fixture data only.
struct Stream(u64);

impl Stream {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    seed: u64,
    read_spec: ReadSpec,
    vocabulary: u32,
    blocks: u32,
    context: u32,
    kept_cells: u32,
    rows: usize,
    /// Token blocks are drawn from the first `roots` orbit roots.
    roots: u64,
}

fn snapshot(case: &Case) -> Snapshot {
    let mut s = Stream(case.seed);
    let spec = Spec {
        vocabulary: case.vocabulary,
        blocks: case.blocks,
        context: case.context,
        kept_cells: case.kept_cells,
    };
    let stages = BinderPlan::new(spec.context as usize)
        .expect("binder plan")
        .stages() as u32;
    let gate_len = case
        .read_spec
        .gate_count(spec.blocks, stages)
        .expect("gate count") as usize;
    // A few distinct roots per block, so cells and token distances tie often.
    let table = (0..spec.vocabulary * spec.blocks)
        .flat_map(|_| ROOTS[s.below(case.roots) as usize])
        .collect();
    let routes = (0..spec.blocks * stages)
        .map(|_| s.below(64) as u32)
        .collect();
    let gates = (0..gate_len).map(|_| s.below(3) as i32 - 1).collect();
    let read_initialization = match case.read_spec {
        ReadSpec::Route => ReadInitialization::ZeroGatesV1,
        _ => ReadInitialization::ActiveLeafV1,
    };
    Snapshot {
        schema: SCHEMA.to_owned(),
        spec,
        read_spec: case.read_spec,
        objective_policy: ObjectivePolicy::NearestAddressMarginV1,
        read_initialization,
        normalization_mode: NormalizationMode::ScaleMatchedSnapV1,
        position: None,
        address_metric: Default::default(),
        seed: case.seed as u32,
        epoch: 1,
        table,
        routes,
        gates,
        operator_kappa: OPERATOR_KAPPA.to_owned(),
        composition_kappa: COMPOSITION_KAPPA.to_owned(),
        kernel_sha256: "11".repeat(32),
        source_sha256: "22".repeat(32),
        geometry_sha256: "33".repeat(32),
    }
}

/// Prefixes of every length class: 1, the full context, and lengths between,
/// over a small token subset so repeated tokens (relation edge 3) occur.
fn prefixes(case: &Case) -> Vec<Vec<u32>> {
    let mut s = Stream(case.seed ^ 0x9e37_79b9_7f4a_7c15);
    let alphabet = (case.vocabulary / 3).max(2) as u64;
    (0..case.rows)
        .map(|row| {
            let length = match row {
                0 => 1,
                1 => case.context as usize,
                _ => 1 + s.below(u64::from(case.context)) as usize,
            };
            (0..length).map(|_| s.below(alphabet) as u32).collect()
        })
        .collect()
}

/// A case's artifact (read back by the scorer's reader), model and rows.
fn fixture(
    case: &Case,
) -> (
    ArtifactTables,
    IntegerDiscreteModel,
    Vec<Vec<u32>>,
    PackedRows,
) {
    let bytes = export_d11(&snapshot(case)).expect("export_d11");
    let (tables, model) = ArtifactTables::from_bytes(&bytes).expect("artifact tables");
    let rows = prefixes(case);
    let views: Vec<&[u32]> = rows.iter().map(Vec::as_slice).collect();
    let packed = pack_rows(&views).expect("pack");
    (tables, model, rows, packed)
}

fn verify(case: &Case, model: &IntegerDiscreteModel, rows: &[Vec<u32>], scores: &[RowScore]) {
    assert_eq!(scores.len(), rows.len());
    for (i, (prefix, score)) in rows.iter().zip(scores).enumerate() {
        assert_eq!(score.status, 0, "{}: row {i} refused", case.name);
        if let Err(e) = compare_row(model, prefix, score) {
            panic!("{}: row {i} (length {}): {e}", case.name, prefix.len());
        }
    }
}

fn check(case: &Case) {
    let (tables, model, rows, packed) = fixture(case);
    verify(
        case,
        &model,
        &rows,
        &host_score(&tables, &packed).expect("host score"),
    );
}

/// Route, Flock, Relation, and the largest shape the reader admits (16
/// blocks, 16 kept cells, the 243-token context, kept 8 and 5 sweeps).
const CASES: [Case; 4] = [
    Case {
        name: "route",
        seed: 0x5eed_0001,
        read_spec: ReadSpec::Route,
        vocabulary: 37,
        blocks: 4,
        context: 81,
        kept_cells: 3,
        rows: 256,
        roots: 16,
    },
    Case {
        name: "flock",
        seed: 0x5eed_0002,
        read_spec: ReadSpec::Flock { kept: 3 },
        vocabulary: 41,
        blocks: 2,
        context: 81,
        kept_cells: 4,
        rows: 48,
        roots: 12,
    },
    Case {
        name: "relation",
        seed: 0x5eed_0003,
        read_spec: ReadSpec::Relation { kept: 2, sweeps: 3 },
        vocabulary: 29,
        blocks: 4,
        context: 27,
        kept_cells: 2,
        rows: 48,
        roots: 12,
    },
    Case {
        name: "bounds",
        seed: 0x5eed_0004,
        read_spec: ReadSpec::Relation { kept: 8, sweeps: 5 },
        vocabulary: 64,
        blocks: 16,
        context: 243,
        kept_cells: 16,
        rows: 8,
        roots: 12,
    },
];

#[test]
fn record_sizes_match_the_host_build() {
    let (scratch, row_out, plan) = host_record_sizes();
    assert_eq!(scratch, (SCRATCH_WORDS * 4) as i64);
    assert_eq!(row_out, (ROW_OUT_WORDS * 4) as i64);
    assert_eq!(plan, (PLAN_WORDS * 4) as i64);
}

#[test]
fn row_plans_follow_the_ternary_binder() {
    let plans = row_plans(243).expect("plans");
    let record = |length: usize| &plans[(length - 1) * PLAN_WORDS..length * PLAN_WORDS];
    // 9 atoms merge into 3 level-1 nodes and one level-2 root.
    let nine = record(9);
    assert_eq!((nine[0], nine[1], nine[2]), (13, 1, 12));
    assert_eq!(&nine[34 + 12 * 7..34 + 13 * 7], &[3, 7, 11, -1, 2, 0, 8]);
    // 243 = 3^5: one root over 364 nodes; 242 leaves two nodes per level.
    assert_eq!((record(243)[0], record(243)[1]), (364, 1));
    assert_eq!(record(242)[1], 10);
}

#[test]
fn route_reads_agree_with_the_reference() {
    check(&CASES[0]);
}

#[test]
fn flock_reads_agree_with_the_reference() {
    check(&CASES[1]);
}

#[test]
fn relation_reads_agree_with_the_reference() {
    check(&CASES[2]);
}

#[test]
fn the_largest_shape_agrees_with_the_reference() {
    check(&CASES[3]);
}

#[test]
fn score_totals_count_the_reference_predictions() {
    let (tables, model, prefixes, _packed) = fixture(&CASES[1]);
    let references: Vec<_> = prefixes
        .iter()
        .map(|p| model.evaluate_prefix(p).expect("reference"))
        .collect();
    // Even rows target the reference prediction, odd rows a fixed token.
    let rows: Vec<DevRow> = prefixes
        .iter()
        .zip(&references)
        .enumerate()
        .map(|(i, (prefix, reference))| DevRow {
            ordinal: i as u64,
            target_position: i as u64,
            target_token: if i % 2 == 0 { reference.prediction } else { 5 },
            prefix: prefix.clone(),
        })
        .collect();
    let views: Vec<&[u32]> = rows.iter().map(|r| r.prefix.as_slice()).collect();
    let targets: Vec<u32> = rows.iter().map(|r| r.target_token).collect();
    let packed = pack_rows_with_targets(&views, &targets).expect("pack with targets");
    let scores = host_score(&tables, &packed).expect("host score");
    let (summary, lines) = score_summary(&tables, &rows, &scores).expect("summary");
    let count = |hit: &dyn Fn(&DevRow, &DiscretePrefixEvaluation) -> bool| {
        rows.iter()
            .zip(&references)
            .filter(|(row, reference)| hit(row, reference))
            .count()
    };
    let top1 = count(&|row, r| r.prediction == row.target_token);
    let top8 = count(&|row, r| r.top8.contains(&row.target_token));
    let admitted = count(&|row, r| {
        r.admitted_cells
            .contains(&model.cells()[row.target_token as usize])
    });
    assert!(
        top1 >= rows.len() / 2,
        "the fixture has top-1 hits to count"
    );
    assert_eq!(
        (
            summary["top1"].as_u64(),
            summary["top8"].as_u64(),
            summary["admitted"].as_u64()
        ),
        (Some(top1 as u64), Some(top8 as u64), Some(admitted as u64))
    );
    assert_eq!(summary["rows"], rows.len());
    assert_eq!(summary["refused"], 0);
    assert_eq!(lines.iter().filter(|&&b| b == b'\n').count(), rows.len());

    let check = reference_check(&model, &rows, &scores);
    assert_eq!(check["agreed"], rows.len());
    let mut wrong = scores.clone();
    wrong[3].top8.swap(0, 1);
    let check = reference_check(&model, &rows, &wrong);
    assert_eq!(check["agreed"], rows.len() - 1);
    assert_eq!(check["first_mismatch"]["ordinal"], 3);

    // Negative control: perturbing loss_ta on row 2 must also fail reference_check.
    let mut wrong_loss = scores.clone();
    wrong_loss[2].loss_ta += 1;
    let check_loss = reference_check(&model, &rows, &wrong_loss);
    assert_eq!(check_loss["agreed"], rows.len() - 1);
    assert_eq!(check_loss["first_mismatch"]["ordinal"], 2);
}

#[test]
fn planted_fault_b4_and_train_eval_gap_verify_and_catch_faults() {
    let bytes = export_d11(&snapshot(&CASES[1])).expect("export_d11");
    let (tables, _model) = ArtifactTables::from_bytes(&bytes).expect("artifact tables");
    let prefixes = prefixes(&CASES[1]);
    let rows: Vec<DevRow> = prefixes
        .iter()
        .enumerate()
        .map(|(i, p)| DevRow {
            ordinal: i as u64,
            target_position: (100 + i) as u64,
            target_token: (i as u32) % CASES[1].vocabulary,
            prefix: p.clone(),
        })
        .collect();
    let views: Vec<&[u32]> = rows.iter().map(|r| r.prefix.as_slice()).collect();
    let targets: Vec<u32> = rows.iter().map(|r| r.target_token).collect();
    let packed = pack_rows_with_targets(&views, &targets).expect("pack");
    let scores = host_score(&tables, &packed).expect("host score");
    let (summary, _lines) = score_summary(&tables, &rows, &scores).expect("summary");

    let b4 = planted_fault_control(&bytes, &tables, &rows, &scores, |fault_tables| {
        host_score(fault_tables, &packed)
    })
    .expect("planted fault B4 passes");
    assert_eq!(b4["b4_pass"], true);

    // Negative control: if the scorer ignores table faults, B4 must fail.
    assert!(
        planted_fault_control(&bytes, &tables, &rows, &scores, |_fault_tables| {
            Ok(scores.clone())
        })
        .is_err(),
        "B4 must fail when faults produce 0 row mismatches"
    );

    let batch_rows: Vec<serde_json::Value> = rows
        .iter()
        .zip(&scores)
        .map(|(r, s)| {
            serde_json::json!({
                "target_position": r.target_position,
                "target_token": r.target_token,
                "top8": s.top8,
                "top1_correct": u64::from(s.prediction == r.target_token),
                "top8_correct": u64::from(s.top8.contains(&r.target_token)),
                "token_hinge": s.token_hinge,
                "address_hinge": s.address_hinge,
                "cell_rank_hinge": s.rank_hinge,
                "loss_ta_numerator": s.loss_ta,
                "loss_tr_numerator": s.loss_tr,
            })
        })
        .collect();
    let mut train_eval = serde_json::json!({
        "accepted_updates_in_model": 1,
        "aggregate": {
            "scored_rows": rows.len(),
            "top1_correct": summary["top1"],
            "top8_correct": summary["top8"],
            "target_admitted_rows": summary["admitted"],
            "token_hinge_sum": summary["token_hinge_sum"],
            "address_hinge_sum": summary["address_hinge_sum"],
            "cell_rank_hinge_sum": summary["cell_rank_hinge_sum"],
            "loss_ta_numerator_sum": summary["loss_ta_numerator_sum"],
            "loss_tr_numerator_sum": summary["loss_tr_numerator_sum"],
        },
        "batches": [{"rows": batch_rows}],
    });
    let gap = compare_with_train_eval(&summary, &rows, &scores, &train_eval, &b4).expect("gap");
    assert_eq!(gap["exact_zero_gap"], true);
    assert_eq!(gap["b2_pass"], true);
    assert_eq!(gap["b4_pass"], true);

    // Negative control: a 1-unit discrepancy in row 0's loss_ta_numerator sets exact_zero_gap false.
    train_eval["batches"][0]["rows"][0]["loss_ta_numerator"] =
        serde_json::json!(scores[0].loss_ta + 1);
    let bad_gap =
        compare_with_train_eval(&summary, &rows, &scores, &train_eval, &b4).expect("bad gap");
    assert_eq!(bad_gap["exact_zero_gap"], false);
    assert_eq!(bad_gap["row_agreement"]["mismatched_rows"], 1);
}

/// The GPU qualification's finite test: on a CUDA device, the cubin scores
/// every fixture row bit-identically to the host build and to the reference,
/// including across a launch-chunk boundary.
#[cfg(feature = "cuda")]
#[test]
#[ignore = "needs a CUDA device; run on the pod"]
fn gpu_scores_are_bit_identical_to_the_reference_and_the_host_build() {
    use m1_d11_artifact_scorer_instrument::device::DeviceScorer;
    let chunked = Case {
        name: "relation-chunked",
        rows: 4100,
        ..CASES[2]
    };
    for case in CASES.iter().chain([&chunked]) {
        let (tables, model, rows, packed) = fixture(case);
        let scorer = DeviceScorer::new(0, &tables).expect("device scorer");
        let gpu = scorer.score(&packed).expect("GPU score");
        let host = host_score(&tables, &packed).expect("host score");
        assert_eq!(gpu, host, "{}: the GPU and host builds differ", case.name);
        verify(case, &model, &rows, &gpu);
    }
}

#[test]
fn rows_the_reference_refuses_are_refused() {
    let case = Case {
        name: "refusals",
        seed: 0x5eed_0005,
        read_spec: ReadSpec::Flock { kept: 2 },
        vocabulary: 9,
        blocks: 2,
        context: 9,
        kept_cells: 2,
        rows: 0,
        roots: 12,
    };
    let bytes = export_d11(&snapshot(&case)).expect("export_d11");
    let (tables, model) = ArtifactTables::from_bytes(&bytes).expect("artifact tables");
    let rows: Vec<Vec<u32>> = vec![vec![], vec![9], vec![0; 10], vec![1, 2, 3]];
    let views: Vec<&[u32]> = rows.iter().map(Vec::as_slice).collect();
    let scores = host_score(&tables, &pack_rows(&views).expect("pack")).expect("host score");
    assert_eq!(
        scores.iter().map(|s| s.status).collect::<Vec<_>>(),
        [1, 2, 1, 0],
        "empty, out-of-vocabulary and over-long prefixes are refused"
    );
    for (prefix, score) in rows.iter().zip(&scores) {
        compare_row(&model, prefix, score).expect("kernel and reference agree");
    }
    // A refusal for another reason than the reference's is not agreement.
    let mut wrong_reason = scores[1].clone();
    wrong_reason.status = 1;
    assert!(compare_row(&model, &rows[1], &wrong_reason)
        .expect_err("a different refusal")
        .contains("different reasons"));
    // A hand-built row longer than its stride is refused before scoring.
    let mut overlong = pack_rows(&views).expect("pack");
    overlong.lengths[0] = overlong.stride + 1;
    assert!(host_score(&tables, &overlong)
        .expect_err("refused")
        .contains("0..=stride"));
}

#[test]
fn a_tampered_artifact_is_refused_before_scoring() {
    let case = Case {
        name: "tamper",
        seed: 0x5eed_0006,
        read_spec: ReadSpec::Route,
        vocabulary: 9,
        blocks: 2,
        context: 9,
        kept_cells: 2,
        rows: 0,
        roots: 12,
    };
    let mut bytes = export_d11(&snapshot(&case)).expect("export_d11");
    let last = bytes.len() - 1;
    bytes[last] ^= 0x40;
    assert!(ArtifactTables::from_bytes(&bytes)
        .expect_err("refused")
        .contains("integer reader"));
}
