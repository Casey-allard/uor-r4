//! Step 7 (`m1-fresh-panel-transfer`, #2841): prospective decision gate,
//! sealed-fresh refusal guard, and paired fresh-panel count-table evaluator for
//! the exported `UORLUT01` integer artifact.
//!
//! All arithmetic is exact integer: zero floating-point types or literals, zero
//! squashing, zero matrix multiplication. Irrational paired McNemar floors
//! `ceil(2 * sqrt(b + c))` are evaluated by exact integer squaring
//! (`k * k >= 4 * (b + c)`).

use serde_json::{json, Value};
use uor_r4_discrete::corpus::{CorpusSource, PreparedCorpus, Split};
use uor_r4_discrete::w2_capacity::{
    build_tables, predict, CapacityError, Tables, TrainRow, VOCABULARY,
};

use crate::kernel::RowScore;
use crate::DevRow;

pub const FRESH_TRANSFER_GATE_SCHEMA: &str = "uor-r4.m1-fresh-transfer-gate/1";
pub const FRESH_TRANSFER_REPORT_SCHEMA: &str = "uor-r4.m1-fresh-transfer-report/1";

/// Sealed Fresh V2 panel (`codex-attention-dose1-fresh-v2-preservation.tar`,
/// MD5 `318fb430c9726c9600a897eb1e3d767a`, SHA-256
/// `2cd13ef672990a01f7a9e63023e03afacecb9ba8883cd9d7d844ca12cfb60143`).
pub const FRESH_V2_TOKENS_SHA256: &str =
    "0b7e754cfd242bf53145fe9cbca563be8503e6cd75df5449abb325b5c51ee1b8";
pub const FRESH_V2_MASK_SHA256: &str =
    "a1cda5295547bb641b1ea71ba4ebabb58d58c40e6c048cda91f35a878fecbf5e";
pub const FRESH_V2_SEAL_SHA256: &str =
    "df5d074ae09c743aa586e3463ffe87faca5703f44f7802bff8ac49ee2990395f";
pub const FRESH_V2_ROW_BINDING_SHA256: &str =
    "c2d4c7e512379650ae3d72703c1affc9a0e85d754da7b73703dc5aad2cee6c06";
pub const FRESH_V2_MANIFEST_SHA256: &str =
    "b41b8344cd4ca162ab2ea7d6749b826041d06b660546ed69c8205a5f64caca44";
pub const FRESH_V2_PAYLOAD_TOKENS: u64 = 44_678;
pub const FRESH_V2_ELIGIBLE_TARGETS: u64 = 14_479;
pub const FRESH_V2_PSUCC_ROWS: u64 = 3_594;

/// K16 / admission-redirect 2,000-row fresh slice
/// (`admission-redirect-transfer-20261015-codex-v2.tar`,
/// MD5 `8fb6a9e0d9282ef9ac869a595ce8ff33`, SHA-256
/// `004523df17711fc5eed4afa4ebadd2ce27a3fa001d3c327873fdb6cbdb3212fb`).
pub const K16_FRESH_BINDING_SHA256: &str =
    "4bd108ad4b1ee4673f87e3b87ef89a01a563a556330fa577d06e675f6be7f460";
pub const K16_FRESH_TOKENS_SHA256: &str =
    "82ead8db9584544a4cc461c394aaf9b00fa6c9fd8fdbf82e316392a71ecabd7b";
pub const K16_FRESH_MASK_SHA256: &str =
    "8287d20d2c76d388ab7ed9becda6f209d224574a933fa2d14c7bdda3663f5ce4";

/// Registered DEV and TRAIN corpus pins (`CHARTER.md` §4.8).
pub const DEV_TOKENS_SHA256: &str =
    "0919e90f813bf02f6f0528ebcf1f34c97987113b679619b0b44e065fa90b2546";
pub const DEV_MASK_SHA256: &str =
    "a838bb1ae20c05a1b68265a7aa11c3dd146200b7d1ff6ef185ff999e442f5327";
pub const DEV_ELIGIBLE_TARGETS: u64 = 14_287;
pub const DEV_UNIGRAM_TOP1: u64 = 1_226;
pub const DEV_T2_TOP1: u64 = 6_300;
pub const TRAIN_TOKENS_SHA256: &str =
    "3dee858fe16ffba3eb32ffbe2ffacbcfa139aa3b9cc8d9b4bc8fb8a34f56c35a";
pub const TRAIN_MASK_SHA256: &str =
    "b5e7352dd22bd1d168d92761d7eb6703198931a752f8bc74ef43734d7441bf32";
pub const MATCHED_TRAIN_BATCHES: usize = 16_404;
pub const MATCHED_TRAIN_ROWS: usize = 1_049_856;
pub const FULL_PANEL_NOISE_FLOOR: u64 = 24;
pub const PSUCC_NOISE_FLOOR: u64 = 20;

fn is_lower_hex_64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Refuses any `CorpusSource` that points to a `Split::Fresh` slice or to the
/// pinned sealed fresh digests (`0b7e754c…` / `a1cda529…` or `82ead8db…` /
/// `8287d20d…`) on the unscoped DEV scoring path (`dev_rows` and
/// `rows_from_corpus`).
pub fn refuse_sealed_fresh_source(source: &CorpusSource) -> Result<(), String> {
    let tok_sha = source.tokens.sha256.as_str();
    let mask_sha = source.mask.sha256.as_str();
    let tok_path = source.tokens.path.to_string_lossy();
    let mask_path = source.mask.path.to_string_lossy();
    let is_fresh_hash = tok_sha == FRESH_V2_TOKENS_SHA256
        || mask_sha == FRESH_V2_MASK_SHA256
        || tok_sha == K16_FRESH_TOKENS_SHA256
        || mask_sha == K16_FRESH_MASK_SHA256;
    let is_fresh_path = tok_path.contains("fresh_v2")
        || mask_path.contains("fresh_v2")
        || tok_path.contains("fresh-sealed")
        || mask_path.contains("fresh-sealed");
    if source.split == Split::Fresh || is_fresh_hash || is_fresh_path {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: dev_rows / rows_from_corpus may never open or score \
             the sealed fresh panel directly; use fresh_rows / d11-scorer fresh-score with a \
             sealed two-seed DEV bar-(a)-(d) PASS_BARS_A_TO_D_TWO_SEEDS decision receipt"
                .into(),
        );
    }
    Ok(())
}

/// Validates a `FreshTransferGateReceipt` JSON before the fresh slice can be
/// opened or scored. Any missing field, non-PASS DEV decision, single-seed
/// record, failed DEV bar (d) (`<= 1,226` unigram or `<= 6,300` T2), failed
/// export-gap `B2` / planted-fault `B4`, or unpinned fresh digest is refused
/// before touching the fresh corpus bytes on disk.
pub fn validate_fresh_transfer_gate(
    decision_receipt: &Value,
    artifact_sha256: &str,
    fresh_source: &CorpusSource,
) -> Result<(), String> {
    if decision_receipt["schema"].as_str() != Some(FRESH_TRANSFER_GATE_SCHEMA) {
        return Err(format!(
            "SEALED_FRESH_PANEL_REFUSED: decision receipt schema must be {FRESH_TRANSFER_GATE_SCHEMA}"
        ));
    }
    if decision_receipt["dev_decision"].as_str() != Some("PASS_BARS_A_TO_D_TWO_SEEDS") {
        return Err(format!(
            "SEALED_FRESH_PANEL_REFUSED: dev_decision is {:?}, expected PASS_BARS_A_TO_D_TWO_SEEDS",
            decision_receipt["dev_decision"].as_str()
        ));
    }
    if decision_receipt["fresh_seal_sha256"].as_str() != Some(FRESH_V2_SEAL_SHA256) {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: fresh_seal_sha256 does not match pinned df5d074a…".into(),
        );
    }
    if decision_receipt["fresh_row_binding_sha256"].as_str() != Some(FRESH_V2_ROW_BINDING_SHA256) {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: fresh_row_binding_sha256 does not match pinned c2d4c7e5…"
                .into(),
        );
    }
    if decision_receipt["train_tokens_sha256"].as_str() != Some(TRAIN_TOKENS_SHA256)
        || decision_receipt["train_mask_sha256"].as_str() != Some(TRAIN_MASK_SHA256)
    {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: train corpus pins do not match registered mw-balp/train"
                .into(),
        );
    }
    for url_key in ["fit_review_url", "prerun_review_url"] {
        let url = decision_receipt[url_key].as_str().unwrap_or("");
        if !url.starts_with("https://github.com/UOR-Foundation/uor-r4/issues/2841#issuecomment-") {
            return Err(format!(
                "SEALED_FRESH_PANEL_REFUSED: {url_key} must reference an #2841 issue comment"
            ));
        }
    }
    let seeds = decision_receipt["seeds"]
        .as_array()
        .ok_or("SEALED_FRESH_PANEL_REFUSED: decision receipt lacks seeds array")?;
    if seeds.len() != 2 {
        return Err("SEALED_FRESH_PANEL_REFUSED: decision receipt requires exactly 2 seeds".into());
    }
    let seed1 = &seeds[0];
    let seed2 = &seeds[1];
    if seed1["seed"].as_u64() != Some(1) || seed2["seed"].as_u64() != Some(2) {
        return Err("SEALED_FRESH_PANEL_REFUSED: seeds must be ordered [seed 1, seed 2]".into());
    }
    let s1_top1 = seed1["d11_top1"]
        .as_u64()
        .ok_or("SEALED_FRESH_PANEL_REFUSED: seed 1 lacks d11_top1")?;
    let s2_top1 = seed2["d11_top1"]
        .as_u64()
        .ok_or("SEALED_FRESH_PANEL_REFUSED: seed 2 lacks d11_top1")?;
    let two_seed_spread = s1_top1.abs_diff(s2_top1);
    let dev_floor = FULL_PANEL_NOISE_FLOOR.max(two_seed_spread);
    for (idx, s) in [(1u64, seed1), (2u64, seed2)] {
        if s["dev_rows"].as_u64() != Some(DEV_ELIGIBLE_TARGETS) {
            return Err(format!(
                "SEALED_FRESH_PANEL_REFUSED: seed {idx} dev_rows != {DEV_ELIGIBLE_TARGETS}"
            ));
        }
        let top1 = s["d11_top1"].as_u64().unwrap_or(0);
        if top1 <= DEV_UNIGRAM_TOP1 || top1 < DEV_T2_TOP1 + dev_floor {
            return Err(format!(
                "SEALED_FRESH_PANEL_REFUSED: seed {idx} d11_top1 ({top1}) does not beat DEV T2 ({DEV_T2_TOP1}) by >= {dev_floor}"
            ));
        }
        if s["export_gap_b2_pass"].as_bool() != Some(true)
            || s["planted_fault_b4_pass"].as_bool() != Some(true)
            || s["tokens_per_s_reported"].as_bool() != Some(true)
        {
            return Err(format!(
                "SEALED_FRESH_PANEL_REFUSED: seed {idx} lacks export_gap_b2_pass, planted_fault_b4_pass, or tokens_per_s_reported"
            ));
        }
        let art = s["artifact_sha256"].as_str().unwrap_or("");
        if !is_lower_hex_64(art) {
            return Err(format!(
                "SEALED_FRESH_PANEL_REFUSED: seed {idx} lacks valid 64-hex artifact_sha256"
            ));
        }
    }
    let art1 = seed1["artifact_sha256"].as_str().unwrap_or("");
    let art2 = seed2["artifact_sha256"].as_str().unwrap_or("");
    if art1 == art2 {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: seed 1 and seed 2 artifact_sha256 must be distinct".into(),
        );
    }
    if !is_lower_hex_64(artifact_sha256) || (artifact_sha256 != art1 && artifact_sha256 != art2) {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: artifact_sha256 is not bound in the approved two-seed DEV decision receipt"
                .into(),
        );
    }
    if fresh_source.tokens.sha256 != FRESH_V2_TOKENS_SHA256
        || fresh_source.mask.sha256 != FRESH_V2_MASK_SHA256
        || fresh_source.expected_payload_tokens != FRESH_V2_PAYLOAD_TOKENS
        || fresh_source.expected_eligible_targets != FRESH_V2_ELIGIBLE_TARGETS
    {
        return Err(
            "SEALED_FRESH_PANEL_REFUSED: fresh source does not match pinned Fresh V2 (0b7e754c… / a1cda529…, 44,678 tokens, 14,479 targets)"
                .into(),
        );
    }
    Ok(())
}

/// Exact integer `max(min_floor, ceil(2 * sqrt(discordant)))`.
/// `ceil(2 * sqrt(d))` is the smallest integer `k >= 0` with `k * k >= 4 * d`.
pub fn mcnemar_floor(min_floor: u64, discordant: u64) -> u64 {
    let target = discordant.saturating_mul(4);
    let mut k = 0u64;
    while k.saturating_mul(k) < target {
        k += 1;
    }
    min_floor.max(k)
}

/// Exact prefix copy-successor predicate (`evidence::is_copy_successor` on raw
/// token prefixes): true iff `prefix` has length >= 3 and there exists an index
/// `i` with `i + 1 < prefix.len()` such that `prefix[i] == query` and
/// `prefix[i + 1] == target`.
pub fn is_prefix_copy_successor(prefix: &[u32], target: u32) -> bool {
    let Some(&query) = prefix.last() else {
        return false;
    };
    prefix
        .windows(2)
        .any(|pair| pair[0] == query && pair[1] == target)
}

/// Builds the frozen TRAIN count tables (`Unigram` and `T2` context-free
/// successor `Bigram`) strictly from `max_batches` batches of `train_corpus`.
pub fn build_train_count_tables(
    train_corpus: &PreparedCorpus,
    context: u32,
    batch_rows: u32,
    max_batches: usize,
) -> Result<Tables, String> {
    if train_corpus.binding().source.split != Split::Train {
        return Err("count tables must be built strictly from a Split::Train corpus".into());
    }
    let mut train_rows = Vec::with_capacity(max_batches.saturating_mul(batch_rows as usize));
    let mut first = 0u64;
    for _ in 0..max_batches {
        if first >= train_corpus.eligible_len() as u64 {
            break;
        }
        let batch = train_corpus
            .batch(first, context, batch_rows)
            .map_err(|e| e.to_string())?;
        for (window, &target) in batch.windows.iter().zip(&batch.targets) {
            let query = window.last().ok_or("empty TRAIN window")?.token;
            if query as usize >= VOCABULARY || target as usize >= VOCABULARY {
                return Err(format!(
                    "token out of vocabulary range: query={query}, target={target}"
                ));
            }
            train_rows.push(TrainRow {
                code: 0,
                query_token: query as u16,
                target_token: target as u16,
            });
        }
        if batch.next_target <= first {
            return Err("train corpus batch cursor did not advance".into());
        }
        first = batch.next_target;
    }
    build_tables(&train_rows).map_err(|e: CapacityError| e.to_string())
}

/// Per-seed fresh-panel score against the frozen TRAIN `Unigram` and `T2`
/// successor tables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshSeedScore {
    pub seed: u64,
    pub artifact_sha256: String,
    pub rows: u64,
    pub psucc_rows: u64,
    pub refused_rows: u64,
    pub reference_mismatches: u64,
    pub b4_pass: bool,
    pub model_top1: u64,
    pub model_top8: u64,
    pub unigram_top1: u64,
    pub t2_top1: u64,
    pub b_vs_unigram: u64,
    pub c_vs_unigram: u64,
    pub b_vs_t2: u64,
    pub c_vs_t2: u64,
    pub psucc_model_top1: u64,
    pub psucc_unigram_top1: u64,
    pub psucc_t2_top1: u64,
    pub psucc_b_vs_t2: u64,
    pub psucc_c_vs_t2: u64,
}

/// Scores one seed's `RowScore` slice on the fresh panel against the frozen
/// TRAIN `Tables` (`Unigram` and `T2` `Bigram`).
pub fn score_fresh_against_count_tables(
    seed: u64,
    artifact_sha256: &str,
    tables: &Tables,
    rows: &[DevRow],
    scores: &[RowScore],
    reference_mismatches: u64,
    b4_pass: bool,
) -> Result<FreshSeedScore, String> {
    if rows.len() != scores.len() {
        return Err("rows and scores lengths differ".into());
    }
    let uni_ranked = tables.unigram.ranked();
    let uni_top1_tok = uni_ranked.first().copied();
    let mut refused_rows = 0u64;
    let (mut model_top1, mut model_top8) = (0u64, 0u64);
    let (mut unigram_top1, mut t2_top1) = (0u64, 0u64);
    let (mut b_vs_unigram, mut c_vs_unigram) = (0u64, 0u64);
    let (mut b_vs_t2, mut c_vs_t2) = (0u64, 0u64);
    let mut psucc_rows = 0u64;
    let (mut psucc_model_top1, mut psucc_unigram_top1, mut psucc_t2_top1) = (0u64, 0u64, 0u64);
    let (mut psucc_b_vs_t2, mut psucc_c_vs_t2) = (0u64, 0u64);

    for (row, score) in rows.iter().zip(scores) {
        let query = *row.prefix.last().ok_or("fresh row has empty prefix")?;
        if query as usize >= VOCABULARY || row.target_token as usize >= VOCABULARY {
            return Err("fresh row token exceeds vocabulary".into());
        }
        if score.status != 0 {
            refused_rows += 1;
        }
        let target_u16 = row.target_token as u16;
        let m_hit1 = score.status == 0 && score.prediction == row.target_token;
        let m_hit8 = score.top8.contains(&row.target_token);
        let u_hit1 = uni_top1_tok == Some(target_u16);
        let t2_pred = predict(&tables.bigram, query as u16, &tables.unigram);
        let g_hit1 = t2_pred.hit1(target_u16);

        model_top1 += u64::from(m_hit1);
        model_top8 += u64::from(m_hit8);
        unigram_top1 += u64::from(u_hit1);
        t2_top1 += u64::from(g_hit1);

        if m_hit1 && !u_hit1 {
            b_vs_unigram += 1;
        } else if !m_hit1 && u_hit1 {
            c_vs_unigram += 1;
        }
        if m_hit1 && !g_hit1 {
            b_vs_t2 += 1;
        } else if !m_hit1 && g_hit1 {
            c_vs_t2 += 1;
        }

        if is_prefix_copy_successor(&row.prefix, row.target_token) {
            psucc_rows += 1;
            psucc_model_top1 += u64::from(m_hit1);
            psucc_unigram_top1 += u64::from(u_hit1);
            psucc_t2_top1 += u64::from(g_hit1);
            if m_hit1 && !g_hit1 {
                psucc_b_vs_t2 += 1;
            } else if !m_hit1 && g_hit1 {
                psucc_c_vs_t2 += 1;
            }
        }
    }

    Ok(FreshSeedScore {
        seed,
        artifact_sha256: artifact_sha256.to_owned(),
        rows: rows.len() as u64,
        psucc_rows,
        refused_rows,
        reference_mismatches,
        b4_pass,
        model_top1,
        model_top8,
        unigram_top1,
        t2_top1,
        b_vs_unigram,
        c_vs_unigram,
        b_vs_t2,
        c_vs_t2,
        psucc_model_top1,
        psucc_unigram_top1,
        psucc_t2_top1,
        psucc_b_vs_t2,
        psucc_c_vs_t2,
    })
}

/// Evaluates the registered two-seed M1 bar (e) (`m1-fresh-panel-transfer`,
/// `#2841`) from the two seeds' `FreshSeedScore` records.
pub fn evaluate_fresh_transfer_two_seeds(
    seed1: &FreshSeedScore,
    seed2: &FreshSeedScore,
) -> Result<Value, String> {
    if seed1.seed != 1 || seed2.seed != 2 {
        return Err("evaluate_fresh_transfer_two_seeds requires seed 1 and seed 2".into());
    }
    if seed1.rows != seed2.rows
        || seed1.unigram_top1 != seed2.unigram_top1
        || seed1.t2_top1 != seed2.t2_top1
    {
        return Err("seed 1 and seed 2 fresh panel or count-table baselines disagree".into());
    }
    let two_seed_spread = seed1.model_top1.abs_diff(seed2.model_top1);
    let psucc_two_seed_spread = seed1.psucc_model_top1.abs_diff(seed2.psucc_model_top1);

    let eval_seed = |s: &FreshSeedScore| -> Value {
        let floor_u = mcnemar_floor(FULL_PANEL_NOISE_FLOOR, s.b_vs_unigram + s.c_vs_unigram)
            .max(two_seed_spread);
        let floor_t2 =
            mcnemar_floor(FULL_PANEL_NOISE_FLOOR, s.b_vs_t2 + s.c_vs_t2).max(two_seed_spread);
        let floor_psucc = mcnemar_floor(PSUCC_NOISE_FLOOR, s.psucc_b_vs_t2 + s.psucc_c_vs_t2)
            .max(psucc_two_seed_spread);

        let beats_unigram = s.model_top1 >= s.unigram_top1.saturating_add(floor_u);
        let beats_t2 = s.model_top1 >= s.t2_top1.saturating_add(floor_t2);
        let beats_psucc_t2 = s.psucc_model_top1 >= s.psucc_t2_top1.saturating_add(floor_psucc);
        let parity_ok = s.refused_rows == 0 && s.reference_mismatches == 0 && s.b4_pass;
        let seed_pass = beats_unigram && beats_t2 && beats_psucc_t2 && parity_ok;

        json!({
            "seed": s.seed,
            "artifact_sha256": s.artifact_sha256,
            "rows": s.rows,
            "psucc_rows": s.psucc_rows,
            "refused_rows": s.refused_rows,
            "reference_mismatches": s.reference_mismatches,
            "b4_pass": s.b4_pass,
            "model_top1": s.model_top1,
            "model_top8": s.model_top8,
            "unigram_top1": s.unigram_top1,
            "t2_top1": s.t2_top1,
            "b_vs_unigram": s.b_vs_unigram,
            "c_vs_unigram": s.c_vs_unigram,
            "required_floor_vs_unigram": floor_u,
            "b_vs_t2": s.b_vs_t2,
            "c_vs_t2": s.c_vs_t2,
            "required_floor_vs_t2": floor_t2,
            "psucc_model_top1": s.psucc_model_top1,
            "psucc_unigram_top1": s.psucc_unigram_top1,
            "psucc_t2_top1": s.psucc_t2_top1,
            "psucc_b_vs_t2": s.psucc_b_vs_t2,
            "psucc_c_vs_t2": s.psucc_c_vs_t2,
            "required_floor_psucc_vs_t2": floor_psucc,
            "beats_unigram": beats_unigram,
            "beats_t2": beats_t2,
            "beats_psucc_t2": beats_psucc_t2,
            "parity_ok": parity_ok,
            "seed_pass": seed_pass,
        })
    };

    let s1_json = eval_seed(seed1);
    let s2_json = eval_seed(seed2);
    let both_pass = s1_json["seed_pass"].as_bool() == Some(true)
        && s2_json["seed_pass"].as_bool() == Some(true);
    let outcome = if both_pass {
        "PASS_BAR_E_FRESH_TRANSFER"
    } else {
        "FAIL_BAR_E_CLOSE_CONFIGURATION"
    };

    Ok(json!({
        "schema": FRESH_TRANSFER_REPORT_SCHEMA,
        "fresh_tokens_sha256": FRESH_V2_TOKENS_SHA256,
        "fresh_mask_sha256": FRESH_V2_MASK_SHA256,
        "fresh_seal_sha256": FRESH_V2_SEAL_SHA256,
        "fresh_row_binding_sha256": FRESH_V2_ROW_BINDING_SHA256,
        "two_seed_spread_full": two_seed_spread,
        "two_seed_spread_psucc": psucc_two_seed_spread,
        "seed_1": s1_json,
        "seed_2": s2_json,
        "bar_e_pass": both_pass,
        "outcome": outcome,
    }))
}
