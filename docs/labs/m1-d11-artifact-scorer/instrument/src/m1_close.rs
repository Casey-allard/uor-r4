//! Step 8 (`m1-close-saved-model`, `#2837` / `#2029`): five-bar M1 closing-bar
//! evaluator, prospective fresh-transfer gate receipt builder, and canonical
//! saved-model audit across all discrete checkpoints and exported `UORLUT01`
//! artifacts on `main`.
//!
//! All arithmetic is exact integer: zero floating-point types or literals, zero
//! squashing, zero matrix multiplication. Under `CHARTER.md` §2 and
//! `docs/roadmap/items/m1-close-saved-model.md`, `NOT_RUN` is not `PASS`, and
//! the sealed `Fresh V2` panel (`0b7e754c…` / `a1cda529…`, seal `df5d074a…`)
//! remains unopened (`UNAVAILABLE_NOT_RUN`) whenever Bars (a)–(d) do not pass
//! across two seeds on `ft/dev`.

use serde_json::{json, Value};

use crate::fresh_transfer::{
    DEV_ELIGIBLE_TARGETS, DEV_MASK_SHA256, DEV_T2_TOP1, DEV_TOKENS_SHA256, DEV_UNIGRAM_TOP1,
    FRESH_TRANSFER_GATE_SCHEMA, FRESH_TRANSFER_REPORT_SCHEMA, FRESH_V2_ELIGIBLE_TARGETS,
    FRESH_V2_MANIFEST_SHA256, FRESH_V2_MASK_SHA256, FRESH_V2_PAYLOAD_TOKENS, FRESH_V2_PSUCC_ROWS,
    FRESH_V2_ROW_BINDING_SHA256, FRESH_V2_SEAL_SHA256, FRESH_V2_TOKENS_SHA256,
    FULL_PANEL_NOISE_FLOOR, PSUCC_NOISE_FLOOR, TRAIN_MASK_SHA256, TRAIN_TOKENS_SHA256,
};

pub const M1_CLOSE_SCHEMA: &str = "uor-r4.m1-close-saved-model/1";
pub const DEV_PSUCC_ROWS: u64 = 3_581;
pub const DEV_PSUCC_UNIGRAM_TOP1: u64 = 169;
pub const DEV_PSUCC_T2_TOP1: u64 = 1_729;

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Per-seed measurements for a two-seed discrete candidate evaluated against the
/// M1 closing bars on `ft/dev` (`14,287` rows).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M1SeedMetrics {
    pub seed: u64,
    pub checkpoint_sha256: String,
    pub model_snapshot_sha256: String,
    pub artifact_sha256: String,
    pub artifact_bytes: u64,
    pub d11_reader_accepted: bool,
    pub d11_export_includes_full_scoring_path: bool,
    pub dev_rows: u64,
    pub d11_dev_top1: u64,
    pub d11_dev_top8: u64,
    pub d11_dev_admitted: u64,
    pub train_eval_dev_top1: u64,
    pub train_eval_dev_top8: u64,
    pub train_eval_dev_admitted: u64,
    pub row_mismatches: u64,
    pub export_gap_b2_pass: bool,
    pub planted_fault_b4_pass: bool,
    pub laptop_prefill_median_tok_s: u64,
    pub laptop_decode_median_tok_s: u64,
    pub gpu_context_tok_s: u64,
    pub gpu_response_tok_s: u64,
    pub psucc_rows: u64,
    pub psucc_d11_top1: Option<u64>,
    pub stage3_trunk_level3_dev_top1: Option<u64>,
    pub stage3_trunk_level3_psucc_top1: Option<u64>,
}

impl M1SeedMetrics {
    pub fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("seed".to_owned(), json!(self.seed));
        obj.insert("checkpoint_sha256".to_owned(), json!(self.checkpoint_sha256));
        obj.insert(
            "model_snapshot_sha256".to_owned(),
            json!(self.model_snapshot_sha256),
        );
        obj.insert("artifact_sha256".to_owned(), json!(self.artifact_sha256));
        obj.insert("artifact_bytes".to_owned(), json!(self.artifact_bytes));
        obj.insert(
            "d11_reader_accepted".to_owned(),
            json!(self.d11_reader_accepted),
        );
        obj.insert(
            "d11_export_includes_full_scoring_path".to_owned(),
            json!(self.d11_export_includes_full_scoring_path),
        );
        obj.insert("dev_rows".to_owned(), json!(self.dev_rows));
        obj.insert("d11_dev_top1".to_owned(), json!(self.d11_dev_top1));
        obj.insert("d11_dev_top8".to_owned(), json!(self.d11_dev_top8));
        obj.insert("d11_dev_admitted".to_owned(), json!(self.d11_dev_admitted));
        obj.insert(
            "train_eval_dev_top1".to_owned(),
            json!(self.train_eval_dev_top1),
        );
        obj.insert(
            "train_eval_dev_top8".to_owned(),
            json!(self.train_eval_dev_top8),
        );
        obj.insert(
            "train_eval_dev_admitted".to_owned(),
            json!(self.train_eval_dev_admitted),
        );
        obj.insert("row_mismatches".to_owned(), json!(self.row_mismatches));
        obj.insert(
            "export_gap_b2_pass".to_owned(),
            json!(self.export_gap_b2_pass),
        );
        obj.insert(
            "planted_fault_b4_pass".to_owned(),
            json!(self.planted_fault_b4_pass),
        );
        obj.insert(
            "laptop_prefill_median_tok_s".to_owned(),
            json!(self.laptop_prefill_median_tok_s),
        );
        obj.insert(
            "laptop_decode_median_tok_s".to_owned(),
            json!(self.laptop_decode_median_tok_s),
        );
        obj.insert(
            "gpu_context_tok_s".to_owned(),
            json!(self.gpu_context_tok_s),
        );
        obj.insert(
            "gpu_response_tok_s".to_owned(),
            json!(self.gpu_response_tok_s),
        );
        obj.insert("psucc_rows".to_owned(), json!(self.psucc_rows));
        obj.insert("psucc_d11_top1".to_owned(), json!(self.psucc_d11_top1));
        obj.insert(
            "stage3_trunk_level3_dev_top1".to_owned(),
            json!(self.stage3_trunk_level3_dev_top1),
        );
        obj.insert(
            "stage3_trunk_level3_psucc_top1".to_owned(),
            json!(self.stage3_trunk_level3_psucc_top1),
        );
        Value::Object(obj)
    }
}

/// Complete two-seed input for evaluating a saved discrete candidate against
/// the five M1 closing bars (a)–(e).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M1CloseCandidateInput {
    pub candidate_id: String,
    pub family: String,
    pub relation_policy: String,
    pub fit_review_url: String,
    pub prerun_review_url: String,
    pub seed_1: M1SeedMetrics,
    pub seed_2: M1SeedMetrics,
    /// Optional `uor-r4.m1-fresh-transfer-report/1` JSON. Must be `None` when
    /// `bars_a_to_d_pass == false` (enforcing the prospective fresh seal).
    pub fresh_transfer_report: Option<Value>,
}

impl M1CloseCandidateInput {
    pub fn to_json(&self) -> Value {
        json!({
            "candidate_id": self.candidate_id,
            "family": self.family,
            "relation_policy": self.relation_policy,
            "fit_review_url": self.fit_review_url,
            "prerun_review_url": self.prerun_review_url,
            "seed_1": self.seed_1.to_json(),
            "seed_2": self.seed_2.to_json(),
            "fresh_transfer_report": self.fresh_transfer_report,
        })
    }
}

/// Five-bar M1 closing verdict for a two-seed discrete candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct M1BarVerdicts {
    pub candidate_id: String,
    pub family: String,
    pub relation_policy: String,
    pub bar_a_d11_self_contained: bool,
    pub bar_b_export_gap_and_b4: bool,
    pub bar_c_tokens_per_s_reported: bool,
    pub bar_d_dev_beats_unigram_and_t2_two_seeds: bool,
    pub bars_a_to_d_pass: bool,
    pub dev_two_seed_spread: u64,
    pub dev_required_floor: u64,
    pub dev_required_top1_target: u64,
    pub seed_1_deficit_to_t2_floor: u64,
    pub seed_2_deficit_to_t2_floor: u64,
    pub fresh_transfer_status: String,
    pub bar_e_fresh_transfer_two_seeds: bool,
    pub m1_close_pass: bool,
    pub outcome: String,
}

impl M1BarVerdicts {
    pub fn to_json(&self) -> Value {
        json!({
            "candidate_id": self.candidate_id,
            "family": self.family,
            "relation_policy": self.relation_policy,
            "bar_a_d11_self_contained": self.bar_a_d11_self_contained,
            "bar_b_export_gap_and_b4": self.bar_b_export_gap_and_b4,
            "bar_c_tokens_per_s_reported": self.bar_c_tokens_per_s_reported,
            "bar_d_dev_beats_unigram_and_t2_two_seeds": self.bar_d_dev_beats_unigram_and_t2_two_seeds,
            "bars_a_to_d_pass": self.bars_a_to_d_pass,
            "dev_two_seed_spread": self.dev_two_seed_spread,
            "dev_required_floor": self.dev_required_floor,
            "dev_required_top1_target": self.dev_required_top1_target,
            "seed_1_deficit_to_t2_floor": self.seed_1_deficit_to_t2_floor,
            "seed_2_deficit_to_t2_floor": self.seed_2_deficit_to_t2_floor,
            "fresh_transfer_status": self.fresh_transfer_status,
            "bar_e_fresh_transfer_two_seeds": self.bar_e_fresh_transfer_two_seeds,
            "m1_close_pass": self.m1_close_pass,
            "outcome": self.outcome,
        })
    }
}

fn check_seed_bar_a(s: &M1SeedMetrics) -> bool {
    is_lower_hex_64(&s.checkpoint_sha256)
        && is_lower_hex_64(&s.model_snapshot_sha256)
        && is_lower_hex_64(&s.artifact_sha256)
        && s.artifact_bytes > 0
        && s.d11_reader_accepted
        && s.d11_export_includes_full_scoring_path
}

fn check_seed_bar_b(s: &M1SeedMetrics) -> bool {
    s.dev_rows == DEV_ELIGIBLE_TARGETS
        && s.d11_dev_top1 == s.train_eval_dev_top1
        && s.d11_dev_top8 == s.train_eval_dev_top8
        && s.d11_dev_admitted == s.train_eval_dev_admitted
        && s.row_mismatches == 0
        && s.export_gap_b2_pass
        && s.planted_fault_b4_pass
}

fn check_seed_bar_c(s: &M1SeedMetrics) -> bool {
    s.laptop_prefill_median_tok_s > 0
        && s.laptop_decode_median_tok_s > 0
        && s.gpu_context_tok_s > 0
        && s.gpu_response_tok_s > 0
}

/// Evaluates all five M1 closing bars (a)–(e) on a two-seed candidate input.
/// Refuses any input where `fresh_transfer_report.is_some()` while
/// `bars_a_to_d_pass == false` so that a premature unsealing of `Fresh V2`
/// is rejected as a hard error.
pub fn evaluate_m1_closing_bars(input: &M1CloseCandidateInput) -> Result<M1BarVerdicts, String> {
    if input.seed_1.seed != 1 || input.seed_2.seed != 2 {
        return Err("evaluate_m1_closing_bars requires seed_1.seed == 1 and seed_2.seed == 2".into());
    }
    let s1 = &input.seed_1;
    let s2 = &input.seed_2;

    let bar_a = check_seed_bar_a(s1)
        && check_seed_bar_a(s2)
        && s1.artifact_sha256 != s2.artifact_sha256;
    let bar_b = check_seed_bar_b(s1) && check_seed_bar_b(s2);
    let bar_c = check_seed_bar_c(s1) && check_seed_bar_c(s2);

    let dev_two_seed_spread = s1.d11_dev_top1.abs_diff(s2.d11_dev_top1);
    let dev_required_floor = FULL_PANEL_NOISE_FLOOR.max(dev_two_seed_spread);
    let dev_required_top1_target = DEV_T2_TOP1.saturating_add(dev_required_floor);

    let psucc_ok = match (s1.psucc_d11_top1, s2.psucc_d11_top1) {
        (Some(p1), Some(p2)) => {
            let p_spread = p1.abs_diff(p2);
            let p_floor = PSUCC_NOISE_FLOOR.max(p_spread);
            let p_target = DEV_PSUCC_T2_TOP1.saturating_add(p_floor);
            s1.psucc_rows == DEV_PSUCC_ROWS
                && s2.psucc_rows == DEV_PSUCC_ROWS
                && p1 > DEV_PSUCC_UNIGRAM_TOP1
                && p2 > DEV_PSUCC_UNIGRAM_TOP1
                && p1 >= p_target
                && p2 >= p_target
        }
        (None, None) => true,
        _ => false,
    };

    let bar_d = s1.dev_rows == DEV_ELIGIBLE_TARGETS
        && s2.dev_rows == DEV_ELIGIBLE_TARGETS
        && s1.d11_dev_top1 > DEV_UNIGRAM_TOP1
        && s2.d11_dev_top1 > DEV_UNIGRAM_TOP1
        && s1.d11_dev_top1 >= dev_required_top1_target
        && s2.d11_dev_top1 >= dev_required_top1_target
        && psucc_ok;

    let bars_a_to_d_pass = bar_a && bar_b && bar_c && bar_d;
    let seed_1_deficit_to_t2_floor = dev_required_top1_target.saturating_sub(s1.d11_dev_top1);
    let seed_2_deficit_to_t2_floor = dev_required_top1_target.saturating_sub(s2.d11_dev_top1);

    if !bars_a_to_d_pass && input.fresh_transfer_report.is_some() {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: fresh_transfer_report provided while bars_a_to_d_pass is false"
                .into(),
        );
    }

    let (fresh_transfer_status, bar_e) = match &input.fresh_transfer_report {
        None => ("UNAVAILABLE_NOT_RUN".to_owned(), false),
        Some(rep) => {
            if rep["schema"].as_str() != Some(FRESH_TRANSFER_REPORT_SCHEMA) {
                return Err(format!(
                    "fresh_transfer_report schema must be {FRESH_TRANSFER_REPORT_SCHEMA}"
                ));
            }
            if rep["fresh_tokens_sha256"].as_str() != Some(FRESH_V2_TOKENS_SHA256)
                || rep["fresh_mask_sha256"].as_str() != Some(FRESH_V2_MASK_SHA256)
                || rep["fresh_seal_sha256"].as_str() != Some(FRESH_V2_SEAL_SHA256)
                || rep["fresh_row_binding_sha256"].as_str() != Some(FRESH_V2_ROW_BINDING_SHA256)
            {
                return Err("fresh_transfer_report fresh pins do not match sealed Fresh V2".into());
            }
            if rep["seed_1"]["artifact_sha256"].as_str() != Some(s1.artifact_sha256.as_str())
                || rep["seed_2"]["artifact_sha256"].as_str() != Some(s2.artifact_sha256.as_str())
            {
                return Err(
                    "fresh_transfer_report seed artifact_sha256 does not match candidate seeds"
                        .into(),
                );
            }
            let pass = rep["bar_e_pass"].as_bool() == Some(true)
                && rep["outcome"].as_str() == Some("PASS_BAR_E_FRESH_TRANSFER");
            let status = if pass {
                "PASS_BAR_E_FRESH_TRANSFER".to_owned()
            } else {
                "FAIL_BAR_E_CLOSE_CONFIGURATION".to_owned()
            };
            (status, pass)
        }
    };

    let m1_close_pass = bars_a_to_d_pass && bar_e;
    let outcome = if m1_close_pass {
        "PASS_M1_ALL_FIVE_BARS".to_owned()
    } else if !bars_a_to_d_pass {
        "HOLD_M1_OPEN_BAR_D_UNMET_FRESH_SEAL_INTACT".to_owned()
    } else {
        "HOLD_M1_OPEN_BAR_E_UNMET".to_owned()
    };

    Ok(M1BarVerdicts {
        candidate_id: input.candidate_id.clone(),
        family: input.family.clone(),
        relation_policy: input.relation_policy.clone(),
        bar_a_d11_self_contained: bar_a,
        bar_b_export_gap_and_b4: bar_b,
        bar_c_tokens_per_s_reported: bar_c,
        bar_d_dev_beats_unigram_and_t2_two_seeds: bar_d,
        bars_a_to_d_pass,
        dev_two_seed_spread,
        dev_required_floor,
        dev_required_top1_target,
        seed_1_deficit_to_t2_floor,
        seed_2_deficit_to_t2_floor,
        fresh_transfer_status,
        bar_e_fresh_transfer_two_seeds: bar_e,
        m1_close_pass,
        outcome,
    })
}

fn seed_gate_json(s: &M1SeedMetrics) -> Value {
    json!({
        "seed": s.seed,
        "checkpoint_sha256": s.checkpoint_sha256,
        "model_snapshot_sha256": s.model_snapshot_sha256,
        "artifact_sha256": s.artifact_sha256,
        "artifact_bytes": s.artifact_bytes,
        "dev_rows": s.dev_rows,
        "d11_top1": s.d11_dev_top1,
        "d11_top8": s.d11_dev_top8,
        "d11_admitted": s.d11_dev_admitted,
        "export_gap_b2_pass": s.export_gap_b2_pass && s.row_mismatches == 0,
        "planted_fault_b4_pass": s.planted_fault_b4_pass,
        "tokens_per_s_reported": check_seed_bar_c(s),
        "laptop_prefill_median_tok_s": s.laptop_prefill_median_tok_s,
        "laptop_decode_median_tok_s": s.laptop_decode_median_tok_s,
        "gpu_context_tok_s": s.gpu_context_tok_s,
        "gpu_response_tok_s": s.gpu_response_tok_s,
    })
}

/// Builds a `uor-r4.m1-fresh-transfer-gate/1` receipt from a two-seed candidate
/// input and its `M1BarVerdicts`. When `verdicts.bars_a_to_d_pass` is `true`,
/// `dev_decision` is `"PASS_BARS_A_TO_D_TWO_SEEDS"` (accepted by
/// `validate_fresh_transfer_gate`). When `false`, `dev_decision` is
/// `"FAIL_BAR_D_HOLD_FRESH_SEAL"` (refused by `validate_fresh_transfer_gate`).
pub fn build_fresh_transfer_gate_receipt(
    input: &M1CloseCandidateInput,
    verdicts: &M1BarVerdicts,
) -> Value {
    let dev_decision = if verdicts.bars_a_to_d_pass {
        "PASS_BARS_A_TO_D_TWO_SEEDS"
    } else {
        "FAIL_BAR_D_HOLD_FRESH_SEAL"
    };
    let s1 = seed_gate_json(&input.seed_1);
    let s2 = seed_gate_json(&input.seed_2);
    let mut obj = serde_json::Map::new();
    obj.insert("schema".to_owned(), json!(FRESH_TRANSFER_GATE_SCHEMA));
    obj.insert("candidate_id".to_owned(), json!(input.candidate_id));
    obj.insert("family".to_owned(), json!(input.family));
    obj.insert("relation_policy".to_owned(), json!(input.relation_policy));
    obj.insert("dev_decision".to_owned(), json!(dev_decision));
    obj.insert(
        "fresh_transfer_status".to_owned(),
        json!(verdicts.fresh_transfer_status),
    );
    obj.insert("fresh_seal_sha256".to_owned(), json!(FRESH_V2_SEAL_SHA256));
    obj.insert(
        "fresh_row_binding_sha256".to_owned(),
        json!(FRESH_V2_ROW_BINDING_SHA256),
    );
    obj.insert(
        "fresh_manifest_sha256".to_owned(),
        json!(FRESH_V2_MANIFEST_SHA256),
    );
    obj.insert(
        "fresh_tokens_sha256".to_owned(),
        json!(FRESH_V2_TOKENS_SHA256),
    );
    obj.insert("fresh_mask_sha256".to_owned(), json!(FRESH_V2_MASK_SHA256));
    obj.insert(
        "fresh_expected_payload_tokens".to_owned(),
        json!(FRESH_V2_PAYLOAD_TOKENS),
    );
    obj.insert(
        "fresh_expected_eligible_targets".to_owned(),
        json!(FRESH_V2_ELIGIBLE_TARGETS),
    );
    obj.insert(
        "fresh_expected_psucc_rows".to_owned(),
        json!(FRESH_V2_PSUCC_ROWS),
    );
    obj.insert("dev_tokens_sha256".to_owned(), json!(DEV_TOKENS_SHA256));
    obj.insert("dev_mask_sha256".to_owned(), json!(DEV_MASK_SHA256));
    obj.insert("train_tokens_sha256".to_owned(), json!(TRAIN_TOKENS_SHA256));
    obj.insert("train_mask_sha256".to_owned(), json!(TRAIN_MASK_SHA256));
    obj.insert("fit_review_url".to_owned(), json!(input.fit_review_url));
    obj.insert(
        "prerun_review_url".to_owned(),
        json!(input.prerun_review_url),
    );
    obj.insert("dev_unigram_top1".to_owned(), json!(DEV_UNIGRAM_TOP1));
    obj.insert("dev_t2_top1".to_owned(), json!(DEV_T2_TOP1));
    obj.insert(
        "dev_two_seed_spread".to_owned(),
        json!(verdicts.dev_two_seed_spread),
    );
    obj.insert(
        "dev_required_floor".to_owned(),
        json!(verdicts.dev_required_floor),
    );
    obj.insert(
        "dev_required_top1_target".to_owned(),
        json!(verdicts.dev_required_top1_target),
    );
    obj.insert("seeds".to_owned(), json!([s1, s2]));
    Value::Object(obj)
}

/// Canonical two-seed Stage T `T16_fz` (`FrozenZeroV1`, `Flock` read, `B = 4`,
/// `C = 256`, `16,404` batches) candidate input from sealed `#2836`, `#2839`,
/// and `#2844` evidence.
pub fn canonical_stage_t_fz_candidate() -> M1CloseCandidateInput {
    M1CloseCandidateInput {
        candidate_id: "stage-t-t16-fz".to_owned(),
        family: "Stage T B=4 C=256 Flock read (FrozenZeroV1)".to_owned(),
        relation_policy: "FrozenZeroV1".to_owned(),
        fit_review_url:
            "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-4323500163"
                .to_owned(),
        prerun_review_url:
            "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-4323500163"
                .to_owned(),
        seed_1: M1SeedMetrics {
            seed: 1,
            checkpoint_sha256:
                "3bd76bc81fa20e5da460eb7fb2ba32036475890cb860a0d6e60a6c3069f9d604".to_owned(),
            model_snapshot_sha256:
                "c4da33b56fb03b145155da23d5eb84188055e507880f659347db3d26f37bb9eb".to_owned(),
            artifact_sha256:
                "8a2c3cb4da4c80643ed992f886ff858ba2d739bc901ab23c84805619da1dd7f5".to_owned(),
            artifact_bytes: 533_248,
            d11_reader_accepted: true,
            d11_export_includes_full_scoring_path: true,
            dev_rows: DEV_ELIGIBLE_TARGETS,
            d11_dev_top1: 455,
            d11_dev_top8: 1_484,
            d11_dev_admitted: 5_009,
            train_eval_dev_top1: 455,
            train_eval_dev_top8: 1_484,
            train_eval_dev_admitted: 5_009,
            row_mismatches: 0,
            export_gap_b2_pass: true,
            planted_fault_b4_pass: true,
            laptop_prefill_median_tok_s: 12_551,
            laptop_decode_median_tok_s: 12_630,
            gpu_context_tok_s: 56_836_282,
            gpu_response_tok_s: 232_879,
            psucc_rows: DEV_PSUCC_ROWS,
            psucc_d11_top1: Some(107),
            stage3_trunk_level3_dev_top1: Some(2_988),
            stage3_trunk_level3_psucc_top1: Some(511),
        },
        seed_2: M1SeedMetrics {
            seed: 2,
            checkpoint_sha256:
                "bd692f3eb1f89598d88cc5be98b349a2ba597dd1cb38c860f70c7cd09bc6610c".to_owned(),
            model_snapshot_sha256:
                "c4da33b56fb03b145155da23d5eb84188055e507880f659347db3d26f37bb9eb".to_owned(),
            artifact_sha256:
                "33ce2c5531ceac304823879c84417811408446c509665ef4c2f62196859078fd".to_owned(),
            artifact_bytes: 533_248,
            d11_reader_accepted: true,
            d11_export_includes_full_scoring_path: true,
            dev_rows: DEV_ELIGIBLE_TARGETS,
            d11_dev_top1: 455,
            d11_dev_top8: 1_484,
            d11_dev_admitted: 5_009,
            train_eval_dev_top1: 455,
            train_eval_dev_top8: 1_484,
            train_eval_dev_admitted: 5_009,
            row_mismatches: 0,
            export_gap_b2_pass: true,
            planted_fault_b4_pass: true,
            laptop_prefill_median_tok_s: 12_663,
            laptop_decode_median_tok_s: 12_523,
            gpu_context_tok_s: 56_652_733,
            gpu_response_tok_s: 232_126,
            psucc_rows: DEV_PSUCC_ROWS,
            psucc_d11_top1: Some(107),
            stage3_trunk_level3_dev_top1: Some(2_989),
            stage3_trunk_level3_psucc_top1: Some(511),
        },
        fresh_transfer_report: None,
    }
}

/// Canonical two-seed Stage T `T16_rel` (`LearnV1`, `Relation` read, `B = 4`,
/// `C = 256`, `16,404` batches) candidate input from sealed `#2836`, `#2839`,
/// and `#2844` evidence.
pub fn canonical_stage_t_rel_candidate() -> M1CloseCandidateInput {
    M1CloseCandidateInput {
        candidate_id: "stage-t-t16-rel".to_owned(),
        family: "Stage T B=4 C=256 Relation read (LearnV1)".to_owned(),
        relation_policy: "LearnV1".to_owned(),
        fit_review_url:
            "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-4323500163"
                .to_owned(),
        prerun_review_url:
            "https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-4323500163"
                .to_owned(),
        seed_1: M1SeedMetrics {
            seed: 1,
            checkpoint_sha256:
                "ec80dd872af8914393448b532325410e5890b6a6bdac495a1140d03c2a1a1d44".to_owned(),
            model_snapshot_sha256:
                "6c36d97e29a8f2de1e4511a707090701604954c791ce32e546d5f603718b2798".to_owned(),
            artifact_sha256:
                "166403ca59268a16b0c5c7f7c225a15a3f1dbef797dfba4d38daec5c9b04fb5d".to_owned(),
            artifact_bytes: 533_248,
            d11_reader_accepted: true,
            d11_export_includes_full_scoring_path: true,
            dev_rows: DEV_ELIGIBLE_TARGETS,
            d11_dev_top1: 371,
            d11_dev_top8: 1_517,
            d11_dev_admitted: 5_009,
            train_eval_dev_top1: 371,
            train_eval_dev_top8: 1_517,
            train_eval_dev_admitted: 5_009,
            row_mismatches: 0,
            export_gap_b2_pass: true,
            planted_fault_b4_pass: true,
            laptop_prefill_median_tok_s: 1_237,
            laptop_decode_median_tok_s: 1_197,
            gpu_context_tok_s: 56_836_282,
            gpu_response_tok_s: 232_879,
            psucc_rows: DEV_PSUCC_ROWS,
            psucc_d11_top1: Some(78),
            stage3_trunk_level3_dev_top1: Some(2_227),
            stage3_trunk_level3_psucc_top1: Some(466),
        },
        seed_2: M1SeedMetrics {
            seed: 2,
            checkpoint_sha256:
                "9f56c3a4ab6998a180cfbe1cb6cb428749c6984ef11484abff2f5e625a703165".to_owned(),
            model_snapshot_sha256:
                "6c36d97e29a8f2de1e4511a707090701604954c791ce32e546d5f603718b2798".to_owned(),
            artifact_sha256:
                "3e52357b3b7fdcb123db339c5767b5685d7ccbc80511a8431b538ce0936de3a0".to_owned(),
            artifact_bytes: 533_248,
            d11_reader_accepted: true,
            d11_export_includes_full_scoring_path: true,
            dev_rows: DEV_ELIGIBLE_TARGETS,
            d11_dev_top1: 371,
            d11_dev_top8: 1_517,
            d11_dev_admitted: 5_009,
            train_eval_dev_top1: 371,
            train_eval_dev_top8: 1_517,
            train_eval_dev_admitted: 5_009,
            row_mismatches: 0,
            export_gap_b2_pass: true,
            planted_fault_b4_pass: true,
            laptop_prefill_median_tok_s: 1_179,
            laptop_decode_median_tok_s: 1_118,
            gpu_context_tok_s: 56_935_299,
            gpu_response_tok_s: 233_284,
            psucc_rows: DEV_PSUCC_ROWS,
            psucc_d11_top1: Some(78),
            stage3_trunk_level3_dev_top1: Some(2_226),
            stage3_trunk_level3_psucc_top1: Some(465),
        },
        fresh_transfer_report: None,
    }
}

/// Builds the complete `uor-r4.m1-close-saved-model/1` five-bar audit and its
/// paired Stage T `T16_fz` `FreshTransferGateReceipt` JSON.
pub fn canonical_m1_saved_model_audit() -> Result<(Value, Value), String> {
    let fz_input = canonical_stage_t_fz_candidate();
    let rel_input = canonical_stage_t_rel_candidate();
    let fz_verdicts = evaluate_m1_closing_bars(&fz_input)?;
    let rel_verdicts = evaluate_m1_closing_bars(&rel_input)?;
    let fz_gate_receipt = build_fresh_transfer_gate_receipt(&fz_input, &fz_verdicts);
    let rel_gate_receipt = build_fresh_transfer_gate_receipt(&rel_input, &rel_verdicts);

    let any_pass = fz_verdicts.m1_close_pass || rel_verdicts.m1_close_pass;
    let m1_milestone_status = if any_pass {
        "CLOSED_PASS"
    } else {
        "OPEN_HELD_AT_STEP_8_PIVOT_REQUIRED"
    };

    let audit = json!({
        "schema": M1_CLOSE_SCHEMA,
        "item": "m1-close-saved-model",
        "issue": 2837,
        "milestone_issue": 2029,
        "m1_close_pass": any_pass,
        "m1_milestone_status": m1_milestone_status,
        "not_run_is_not_pass": true,
        "fresh_v2_seal_intact": !fz_verdicts.bars_a_to_d_pass && !rel_verdicts.bars_a_to_d_pass,
        "registered_bars": {
            "dev_rows": DEV_ELIGIBLE_TARGETS,
            "dev_unigram_top1": DEV_UNIGRAM_TOP1,
            "dev_t2_top1": DEV_T2_TOP1,
            "dev_full_panel_noise_floor": FULL_PANEL_NOISE_FLOOR,
            "dev_psucc_rows": DEV_PSUCC_ROWS,
            "dev_psucc_unigram_top1": DEV_PSUCC_UNIGRAM_TOP1,
            "dev_psucc_t2_top1": DEV_PSUCC_T2_TOP1,
            "dev_psucc_noise_floor": PSUCC_NOISE_FLOOR,
            "fresh_v2_tokens_sha256": FRESH_V2_TOKENS_SHA256,
            "fresh_v2_mask_sha256": FRESH_V2_MASK_SHA256,
            "fresh_v2_seal_sha256": FRESH_V2_SEAL_SHA256,
            "fresh_v2_row_binding_sha256": FRESH_V2_ROW_BINDING_SHA256,
        },
        "two_seed_exported_d11_candidates": [
            {
                "input": fz_input.to_json(),
                "verdicts": fz_verdicts.to_json(),
                "fresh_transfer_gate_receipt": fz_gate_receipt,
            },
            {
                "input": rel_input.to_json(),
                "verdicts": rel_verdicts.to_json(),
                "fresh_transfer_gate_receipt": rel_gate_receipt,
            }
        ],
        "all_saved_discrete_models_inventory": [
            {
                "id": "k16-a-route-b2",
                "source_unit": "archive/2026-10-16-m1-closures/k16-train-and-serve.md",
                "read_mode": "Route (B=2, C=64)",
                "seeds_evaluated": 1,
                "dev_top1": 747,
                "dev_top8": 2_492,
                "dev_admitted": 2_906,
                "d11_exported": true,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": false,
                "bar_d_pass": false,
                "bar_e_status": "HISTORICAL_2000_ROW_SLICE_116_TOP1_BELOW_UNIGRAM_166",
                "blocking_reason": "Single seed; B=2 C=64 dev_top1 = 747 < Unigram 1,226 and T2 6,300"
            },
            {
                "id": "k16-r-relation-b2",
                "source_unit": "archive/2026-10-16-m1-closures/k16-train-and-serve.md",
                "read_mode": "Relation (B=2, C=64)",
                "seeds_evaluated": 1,
                "dev_top1": 359,
                "dev_top8": 2_094,
                "dev_admitted": 2_906,
                "d11_exported": true,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": false,
                "bar_d_pass": false,
                "bar_e_status": "HISTORICAL_2000_ROW_SLICE_55_TOP1_BELOW_UNIGRAM_166",
                "blocking_reason": "Single seed; B=2 C=64 dev_top1 = 359 < Unigram 1,226 and T2 6,300"
            },
            {
                "id": "rung0-q4-flock-b4",
                "source_unit": "docs/labs/m1-w2-capacity-capture/README.md (#2833)",
                "read_mode": "Flock (B=4, C=256)",
                "seeds_evaluated": 1,
                "checkpoint_sha256": "eb43ff383c596071190690546b3e1d675b9122f52cf8580e1640304f7de06ac0",
                "dev_top1": 367,
                "dev_top8": 1_244,
                "dev_admitted": 3_426,
                "unconstrained_read_table_t_top1": 3_414,
                "restricted_t_restr_top1": 1_692,
                "restricted_g_restr_top1": 2_172,
                "d11_exported": false,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Single seed; dev_top1 = 367 and unconstrained read table T(c) = 3,414 < T2 6,300"
            },
            {
                "id": "a8-q1-flock-b4",
                "source_unit": "docs/labs/m1-a8-unfreeze-read-move-order/README.md (#2831)",
                "read_mode": "Flock (B=4, C=256)",
                "seeds_evaluated": 1,
                "checkpoint_sha256": "c6882e4348739e239aa0714b6a67d47855250e4d6c2c348188424d3b83476926",
                "dev_top1": 765,
                "dev_top8": 2_122,
                "dev_admitted": 4_292,
                "d11_exported": false,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Single seed; dev_top1 = 765 and shortlist admission A_W = 4,292 < T2 6,300"
            },
            {
                "id": "stage-t-t16-fz",
                "source_unit": "docs/labs/m1-export-gap-and-toks/README.md (#2836, #2839)",
                "read_mode": "Flock (B=4, C=256, FrozenZeroV1)",
                "seeds_evaluated": 2,
                "seed_1_artifact_sha256": "8a2c3cb4da4c80643ed992f886ff858ba2d739bc901ab23c84805619da1dd7f5",
                "seed_2_artifact_sha256": "33ce2c5531ceac304823879c84417811408446c509665ef4c2f62196859078fd",
                "dev_top1": 455,
                "dev_top8": 1_484,
                "dev_admitted": 5_009,
                "unconstrained_read_table_t_top1": 4_085,
                "restricted_t_restr_top1": 2_140,
                "restricted_g_restr_top1": 3_084,
                "d11_exported": true,
                "bar_a_pass": true,
                "bar_b_pass": true,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Passes Bars (a), (b), (c); fails Bar (d) (455 < 1,226 Unigram and 6,300 T2; deficit 5,869)"
            },
            {
                "id": "stage-t-t16-rel",
                "source_unit": "docs/labs/m1-export-gap-and-toks/README.md (#2836, #2839)",
                "read_mode": "Relation (B=4, C=256, LearnV1)",
                "seeds_evaluated": 2,
                "seed_1_artifact_sha256": "166403ca59268a16b0c5c7f7c225a15a3f1dbef797dfba4d38daec5c9b04fb5d",
                "seed_2_artifact_sha256": "3e52357b3b7fdcb123db339c5767b5685d7ccbc80511a8431b538ce0936de3a0",
                "dev_top1": 371,
                "dev_top8": 1_517,
                "dev_admitted": 5_009,
                "d11_exported": true,
                "bar_a_pass": true,
                "bar_b_pass": true,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Passes Bars (a), (b), (c); fails Bar (d) (371 < 1,226 Unigram and 6,300 T2; deficit 5,953)"
            },
            {
                "id": "within-cell-code-reranker-v1",
                "source_unit": "docs/labs/r-within-cell-ranking/README.md (#3267, #3295)",
                "read_mode": "Flock (B=4, C=256) + WithinCellCodeRerankerV1 (M_code 203,316 B)",
                "seeds_evaluated": 1,
                "dev_top1": 2_135,
                "dev_top8": 3_762,
                "dev_admitted": 5_009,
                "d11_exported": false,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Recovers 99.77% of T_restr(A_W) = 2,140 (beats Unigram 1,226 by +909), but M_code is not in 4-table UORLUT01 and pre-rotation A_W = 5,009 < T2 6,300"
            },
            {
                "id": "trunk-assembly-v1-level3",
                "source_unit": "docs/labs/m1-trunk-assembly-and-training/README.md (#2844)",
                "read_mode": "Flock/Relation (B=4, C=256) + DiscreteHamiltonGateV1 + DiscreteQuaternionRecurrenceV1",
                "seeds_evaluated": 2,
                "fz_dev_top1": [2_988, 2_989],
                "rel_dev_top1": [2_227, 2_226],
                "dev_admitted": 5_009,
                "d11_exported": false,
                "bar_a_pass": false,
                "bar_b_pass": false,
                "bar_c_pass": true,
                "bar_d_pass": false,
                "bar_e_status": "UNAVAILABLE_NOT_RUN",
                "blocking_reason": "Beats Unigram (1,226) by +1,762/+1,763 (T16_fz) and +1,001/+1,000 (T16_rel) across 2 seeds, but base trunk-model.d11 omits trunk tables and pre-rotation A_W(q_read) = 5,009 + marginal c_q weight (64 vs 32) cap Level 3 at 2,989 < T2 6,300"
            }
        ],
        "structural_diagnosis_and_pivot": {
            "bottleneck_1_pre_rotation_shortlist": "In 4-table UORLUT01 (IntegerDiscreteModel::evaluate_prefix), the 8-cell shortlist A_W(q_read) admits at most 5,009 / 14,287 DEV targets (35.06% < T2 = 6,300), and G_restr(A_W) = 3,084 < 6,300.",
            "bottleneck_2_marginal_read_code_collision": "A single 16-bit read code c_read in O_16^4 without query/recurrence conditioning has unconstrained table ceiling T(c_read) = 4,085 on Stage T T16_fz (3,414 on Rung-0 Q4, projected ceiling 3,552 in #3179 QB=B2) < T2 = 6,300.",
            "bottleneck_3_level3_marginal_query_inversion_and_d11_export": "In #2844 TrunkAssemblyV1, select_level3_full_trunk weighted marginal query count 64 * c_q above joint (query, read) 32 * c_qr and (query, rec) 16 * c_qc while restricting candidates to pre-rotation A_W(q_read), and trunk-model.d11 exported only the 4-table Stage 2 snapshot.",
            "pivot_path_a_discrete_joint_post_rotation_d11": "Select target via joint-first hierarchy (select_level3_joint_hierarchy_global, weighting 64 * c_qr + 32 * c_qc + 8 * c_rr + 2 * c_r + c_q), rotate q_read -> q_trunk via DiscreteHamiltonGateV1, score q_trunk through post-rotation full OuterL1V1 writer (A_W(q_trunk)), and serialize trunk tables into the D11 container.",
            "pivot_path_b_offline_float_trunk_export_d11": "Offline float-trained SpiralCore / FBS multi-layer trunk exported to integer UORLUT01 via d11-exporter-float-trunk (#2818, CHARTER.md §3.6 / D37)."
        }
    });

    Ok((audit, fz_gate_receipt))
}
