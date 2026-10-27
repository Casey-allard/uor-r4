//! Stage 1 of `m1-d11-artifact-scorer` (#3248, claude-116; ORDER
//! `docs/loop/tasks/m1-d11-artifact-scorer/claude-116/ORDER.md`): the D11
//! exporter CLI and the DEV-panel row builder that M1 step 6 needs.
//!
//! `export` turns a saved discrete checkpoint into a `UORLUT01` file through the
//! crate's own `export_d11`, and checks that the clean-room integer reader
//! accepts it. `rows` rebuilds exactly the (prefix, target) rows that
//! `discrete-stack eval-corpus` scores on a config's DEV split, by the same
//! public batch walk. Both are host data preparation (D24): neither computes a
//! model quantity.
//!
//! Stage 2 adds `score`: the artifact's integer sections ([`artifact`]) scored
//! row by row by an integer CUDA kernel (`src/scorer.cu`, driven by `device`
//! under the `cuda` feature), whose row body (`src/kernel_core.h`) is also
//! built for the host so finite tests can check it against
//! `IntegerDiscreteModel::evaluate_prefix` and `observe_target` ([`kernel`]).
//!
//! Step 6 (`m1-export-gap-and-toks`, #2839, antigravity-96) adds target-hinge
//! and loss-numerator scoring (`token_hinge`, `address_hinge`, `rank_hinge`,
//! `loss_ta`, `loss_tr`), planted-fault control `B4` ([`planted_fault_control`]),
//! and row-by-row export-gap verification against training-side `eval-corpus`
//! receipts ([`compare_with_train_eval`]).
pub mod artifact;
#[cfg(feature = "cuda")]
pub mod device;
pub mod fresh_transfer;
pub mod kernel;
pub mod m1_close;
pub mod toks;

use serde_json::{json, Value};
use uor_r4_discrete::corpus::{sha256, PreparedCorpus};
use uor_r4_discrete::d11_export::export_d11;
use uor_r4_discrete::snapshot::Snapshot;
use uor_r4_discrete::training::{DriverConfig, CHECKPOINT_SCHEMA};
use uor_r4_integer::discrete_d11::IntegerDiscreteModel;

/// A `UORLUT01` artifact and its export report.
pub struct Exported {
    pub bytes: Vec<u8>,
    pub report: Value,
}

/// Exports a snapshot and confirms that the clean-room integer reader accepts
/// the bytes. The bytes are `export_d11`'s, unchanged.
pub fn export_snapshot(snapshot: &Snapshot) -> Result<Exported, String> {
    let bytes =
        export_d11(snapshot).map_err(|e| format!("export_d11 refused the snapshot: {e}"))?;
    let model = IntegerDiscreteModel::from_bytes(&bytes)
        .map_err(|e| format!("the integer reader refused the export: {e}"))?;
    let shape = serde_json::to_value(model.shape()).map_err(|e| e.to_string())?;
    let report = json!({
        "schema": "uor-r4.d11-scorer-export/1",
        "file_sha256": sha256(&bytes),
        "artifact_sha256": model.artifact_sha256(),
        "bytes": bytes.len(),
        "read_mode": format!("{:?}", model.read_mode()),
        "shape": shape,
    });
    Ok(Exported { bytes, report })
}

/// Exports a `DriverCheckpoint` JSON (`training.rs:469`). The checkpoint's
/// schema, its model storage hash (as `training.rs:905` checks it) and its
/// cursor's accepted updates against the snapshot epoch are all checked first.
pub fn export_checkpoint(checkpoint: &Value) -> Result<Exported, String> {
    if checkpoint["schema"] != CHECKPOINT_SCHEMA {
        return Err(format!("not a {CHECKPOINT_SCHEMA} checkpoint"));
    }
    let model = checkpoint.get("model").ok_or("checkpoint lacks model")?;
    let stored = checkpoint["model_sha256"]
        .as_str()
        .ok_or("checkpoint lacks model_sha256")?;
    let model_bytes = serde_json::to_vec(model).map_err(|e| e.to_string())?;
    if sha256(&model_bytes) != stored {
        return Err("checkpoint model storage hash mismatch".into());
    }
    let accepted = checkpoint["cursor"]["accepted_updates"]
        .as_u64()
        .ok_or("checkpoint lacks cursor.accepted_updates")?;
    let snapshot: Snapshot =
        serde_json::from_value(model.clone()).map_err(|e| format!("checkpoint model: {e}"))?;
    if snapshot.epoch != accepted {
        return Err(format!(
            "snapshot epoch {} differs from the cursor's {accepted} accepted updates",
            snapshot.epoch
        ));
    }
    let mut exported = export_snapshot(&snapshot)?;
    exported.report["checkpoint_model_sha256"] = json!(stored);
    exported.report["accepted_updates"] = json!(accepted);
    Ok(exported)
}

/// One scored DEV row: the causal prefix the evaluator reads and its target.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevRow {
    pub ordinal: u64,
    pub target_position: u64,
    pub target_token: u32,
    pub prefix: Vec<u32>,
}

fn rows_from_corpus_unchecked(
    corpus: &PreparedCorpus,
    context: u32,
    batch_rows: u32,
) -> Result<Vec<DevRow>, String> {
    let mut rows = Vec::new();
    let mut first = 0u64;
    while first < corpus.eligible_len() as u64 {
        let batch = corpus
            .batch(first, context, batch_rows)
            .map_err(|e| e.to_string())?;
        for ((window, &target), &position) in batch
            .windows
            .iter()
            .zip(&batch.targets)
            .zip(&batch.target_positions)
        {
            rows.push(DevRow {
                ordinal: rows.len() as u64,
                target_position: position,
                target_token: target,
                prefix: window.iter().map(|identity| identity.token).collect(),
            });
        }
        if batch.next_target <= first {
            return Err("the corpus batch cursor did not advance".into());
        }
        first = batch.next_target;
    }
    Ok(rows)
}

/// The rows `eval-corpus` scores on a DEV corpus: its own loop, `first = 0`, then
/// `batch(first, context, batch_rows)` until the eligible targets run out,
/// advancing by `next_target` (`training.rs:3411-3428`). Refuses `Split::Fresh`
/// and the pinned sealed fresh digests (`0b7e754c…` / `a1cda529…`, `82ead8db…` /
/// `8287d20d…`).
pub fn rows_from_corpus(
    corpus: &PreparedCorpus,
    context: u32,
    batch_rows: u32,
) -> Result<Vec<DevRow>, String> {
    fresh_transfer::refuse_sealed_fresh_source(&corpus.binding().source)?;
    rows_from_corpus_unchecked(corpus, context, batch_rows)
}

/// A driver config's DEV rows, loaded and pinned exactly as the driver loads
/// them (`training.rs:499`). Refuses any config whose `dev` or `train` split
/// points to the sealed fresh panel (`0b7e754c…` / `a1cda529…` or `82ead8db…` /
/// `8287d20d…`) before touching disk.
pub fn dev_rows(config: &DriverConfig) -> Result<Vec<DevRow>, String> {
    fresh_transfer::refuse_sealed_fresh_source(&config.dev)?;
    fresh_transfer::refuse_sealed_fresh_source(&config.train)?;
    config.validate().map_err(|e| e.to_string())?;
    let dev = PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    rows_from_corpus(&dev, config.spec.context, config.batch_rows)
}

/// Extracts fresh-panel rows only after `validate_fresh_transfer_gate` confirms
/// a sealed two-seed DEV bar-(a)–(d) `PASS_BARS_A_TO_D_TWO_SEEDS` decision receipt.
pub fn fresh_rows_from_corpus(
    corpus: &PreparedCorpus,
    context: u32,
    batch_rows: u32,
    decision_receipt: &Value,
    artifact_sha256: &str,
) -> Result<Vec<DevRow>, String> {
    fresh_transfer::validate_fresh_transfer_gate(
        decision_receipt,
        artifact_sha256,
        &corpus.binding().source,
    )?;
    rows_from_corpus_unchecked(corpus, context, batch_rows)
}

/// Loads and extracts the sealed fresh panel (`config.dev`) only after
/// `validate_fresh_transfer_gate` confirms a sealed two-seed DEV bar-(a)–(d)
/// `PASS_BARS_A_TO_D_TWO_SEEDS` decision receipt before touching disk.
pub fn fresh_rows(
    config: &DriverConfig,
    decision_receipt: &Value,
    artifact_sha256: &str,
) -> Result<Vec<DevRow>, String> {
    fresh_transfer::validate_fresh_transfer_gate(decision_receipt, artifact_sha256, &config.dev)?;
    let mut validated_cfg = config.clone();
    validated_cfg.dev.split = uor_r4_discrete::corpus::Split::Dev;
    validated_cfg.validate().map_err(|e| e.to_string())?;
    let fresh = PreparedCorpus::load(&config.dev, config.spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    rows_from_corpus_unchecked(&fresh, config.spec.context, config.batch_rows)
}

/// sha256 over every row as little-endian `ordinal u64, target_position u64,
/// target_token u32, prefix_len u32, prefix u32 x prefix_len`.
pub fn rows_sha256(rows: &[DevRow]) -> String {
    let mut bytes = Vec::new();
    for row in rows {
        bytes.extend_from_slice(&row.ordinal.to_le_bytes());
        bytes.extend_from_slice(&row.target_position.to_le_bytes());
        bytes.extend_from_slice(&row.target_token.to_le_bytes());
        bytes.extend_from_slice(&(row.prefix.len() as u32).to_le_bytes());
        for token in &row.prefix {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
    }
    sha256(&bytes)
}

/// A scored panel's integer totals and its per-row records (one JSON line per
/// row: prediction, top-8, admitted cells, and target loss numerators).
pub fn score_summary(
    tables: &artifact::ArtifactTables,
    rows: &[DevRow],
    scores: &[kernel::RowScore],
) -> Result<(Value, Vec<u8>), String> {
    if rows.len() != scores.len() {
        return Err("scores do not cover the rows".into());
    }
    let (mut top1, mut top8, mut admitted, mut refused) = (0u64, 0u64, 0u64, 0u64);
    let (mut psucc_rows, mut psucc_top1, mut psucc_top8, mut psucc_admitted) =
        (0u64, 0u64, 0u64, 0u64);
    let (mut token_hinge_sum, mut address_hinge_sum, mut rank_hinge_sum) = (0u64, 0u64, 0u64);
    let (mut loss_ta_sum, mut loss_tr_sum) = (0u64, 0u64);
    let mut lines = Vec::new();
    let mut digest = Vec::new();
    for (row, score) in rows.iter().zip(scores) {
        let cell = *tables
            .cells
            .get(row.target_token as usize)
            .ok_or("a target is outside the artifact vocabulary")?;
        if score.status != 0 {
            refused += 1;
        }
        let hit1 = score.status == 0 && score.prediction == row.target_token;
        let hit8 = score.top8.contains(&row.target_token);
        let adm = score.admitted_cells.contains(&cell);
        top1 += u64::from(hit1);
        top8 += u64::from(hit8);
        admitted += u64::from(adm);
        if fresh_transfer::is_prefix_copy_successor(&row.prefix, row.target_token) {
            psucc_rows += 1;
            psucc_top1 += u64::from(hit1);
            psucc_top8 += u64::from(hit8);
            psucc_admitted += u64::from(adm);
        }
        token_hinge_sum += score.token_hinge;
        address_hinge_sum += score.address_hinge;
        rank_hinge_sum += score.rank_hinge;
        loss_ta_sum += score.loss_ta;
        loss_tr_sum += score.loss_tr;
        let line = json!({
            "ordinal": row.ordinal,
            "target": row.target_token,
            "status": score.status,
            "prediction": score.prediction,
            "top8": score.top8,
            "admitted": score.admitted_cells,
            "token_hinge": score.token_hinge,
            "address_hinge": score.address_hinge,
            "cell_rank_hinge": score.rank_hinge,
            "loss_ta_numerator": score.loss_ta,
            "loss_tr_numerator": score.loss_tr,
        });
        lines.extend(serde_json::to_vec(&line).map_err(|e| e.to_string())?);
        lines.push(b'\n');
        digest.extend_from_slice(&row.ordinal.to_le_bytes());
        digest.extend_from_slice(&score.status.to_le_bytes());
        digest.extend_from_slice(&score.prediction.to_le_bytes());
        digest.extend_from_slice(&score.token_hinge.to_le_bytes());
        digest.extend_from_slice(&score.address_hinge.to_le_bytes());
        digest.extend_from_slice(&score.rank_hinge.to_le_bytes());
        digest.extend_from_slice(&score.loss_ta.to_le_bytes());
        digest.extend_from_slice(&score.loss_tr.to_le_bytes());
        for list in [&score.top8, &score.admitted_cells] {
            digest.extend_from_slice(&(list.len() as u32).to_le_bytes());
            for value in list {
                digest.extend_from_slice(&value.to_le_bytes());
            }
        }
    }
    let summary = json!({
        "schema": "uor-r4.d11-scorer-score/1",
        "artifact_sha256": tables.artifact_sha256,
        "rows": rows.len(),
        "rows_sha256": rows_sha256(rows),
        "refused": refused,
        "top1": top1,
        "top8": top8,
        "admitted": admitted,
        "psucc_rows": psucc_rows,
        "psucc_top1": psucc_top1,
        "psucc_top8": psucc_top8,
        "psucc_admitted": psucc_admitted,
        "token_hinge_sum": token_hinge_sum,
        "address_hinge_sum": address_hinge_sum,
        "cell_rank_hinge_sum": rank_hinge_sum,
        "loss_ta_numerator_sum": loss_ta_sum,
        "loss_tr_numerator_sum": loss_tr_sum,
        "scores_sha256": sha256(&digest),
        "row_records_sha256": sha256(&lines),
    });
    Ok((summary, lines))
}

/// Checks every scored row against `IntegerDiscreteModel::observe_target`.
pub fn reference_check(
    model: &IntegerDiscreteModel,
    rows: &[DevRow],
    scores: &[kernel::RowScore],
) -> Value {
    let mut agreed = 0u64;
    let mut first_mismatch = Value::Null;
    for (row, score) in rows.iter().zip(scores) {
        match kernel::compare_row_with_target(model, &row.prefix, row.target_token, score) {
            Ok(()) => agreed += 1,
            Err(e) if first_mismatch.is_null() => {
                first_mismatch = json!({"ordinal": row.ordinal, "reason": e});
            }
            Err(_) => {}
        }
    }
    json!({"checked": rows.len().min(scores.len()), "agreed": agreed, "first_mismatch": first_mismatch})
}

/// Flips `mask` into the first byte of the named table section of a `UORLUT01` container.
fn flip_container_table_byte(bytes: &[u8], table_name: &str, mask: u8) -> Result<Vec<u8>, String> {
    if bytes.len() < 16 {
        return Err("short container".into());
    }
    let header_len =
        u64::from_le_bytes(bytes[8..16].try_into().map_err(|_| "short header")?) as usize;
    let header: Value = serde_json::from_slice(&bytes[16..16 + header_len])
        .map_err(|e| format!("artifact header: {e}"))?;
    let unaligned = 16 + header_len;
    let data_start = unaligned + (64 - unaligned % 64) % 64;
    let entry = header["tables"]
        .as_array()
        .and_then(|tables| tables.iter().find(|t| t["name"] == table_name))
        .ok_or_else(|| format!("artifact lacks table {table_name}"))?;
    let offset = entry["span"]["offset"]
        .as_u64()
        .ok_or_else(|| format!("table {table_name} lacks offset"))? as usize;
    let pos = data_start + offset;
    let mut tampered = bytes.to_vec();
    let slot = tampered
        .get_mut(pos)
        .ok_or_else(|| format!("table {table_name} offset out of bounds"))?;
    *slot ^= mask;
    Ok(tampered)
}

/// Runs the Stage 4 planted-fault control `B4` (`m1-export-gap-and-toks` sheet
/// line `:27` and `:82`):
/// 1. Flipping `cells[0] ^= 1` in the `UORLUT01` container fails
///    `IntegerDiscreteModel::from_bytes` (endpoint-root invariant check) and
///    changes `artifact_sha256`; flipping `routes[0] ^= 1` changes
///    `artifact_sha256` (failing the container hash pin), and `routes[0] ^= 0x40`
///    fails `IntegerDiscreteModel::from_bytes` (action bound check).
/// 2. Flipping a single writer cell (`cells[tok] ^= 1`) or read route/gate
///    parameter in an in-memory copy of `ArtifactTables` and scoring with
///    `score_fn` (on the GPU when called from `d11-scorer score`) must produce
///    `> 0` row mismatches against `baseline_scores`.
pub fn planted_fault_control(
    artifact_bytes: &[u8],
    tables: &artifact::ArtifactTables,
    rows: &[DevRow],
    baseline_scores: &[kernel::RowScore],
    score_fn: impl Fn(&artifact::ArtifactTables) -> Result<Vec<kernel::RowScore>, String>,
) -> Result<Value, String> {
    if rows.is_empty() || rows.len() != baseline_scores.len() {
        return Err("planted_fault_control requires non-empty scored rows".into());
    }
    let tampered_cells_bytes = flip_container_table_byte(artifact_bytes, "discrete.cells", 1)?;
    let cells_tamper_rejected = IntegerDiscreteModel::from_bytes(&tampered_cells_bytes).is_err();
    let tampered_routes_lsb = flip_container_table_byte(artifact_bytes, "discrete.routes", 1)?;
    let routes_lsb_hash_changed = sha256(&tampered_routes_lsb) != tables.artifact_sha256;
    let tampered_routes_oob = flip_container_table_byte(artifact_bytes, "discrete.routes", 0x40)?;
    let routes_oob_rejected = IntegerDiscreteModel::from_bytes(&tampered_routes_oob).is_err();
    if !cells_tamper_rejected || !routes_lsb_hash_changed || !routes_oob_rejected {
        return Err(
            "planted-fault B4 failed: container reader or hash check missed tampered section bytes"
                .into(),
        );
    }

    // In-memory fault 1: flip a writer cell (`cells[tok] ^= 1`) on the first row's
    // target token (or predicted token if target is 0), and score on the device/host.
    let mut cells_fault_tables = tables.clone();
    let fault_tok = baseline_scores[0].prediction as usize;
    let mutated_tok = if fault_tok < cells_fault_tables.cells.len() {
        fault_tok
    } else {
        0
    };
    cells_fault_tables.cells[mutated_tok] = (cells_fault_tables.cells[mutated_tok] ^ 1) & 255;
    let cells_fault_scores = score_fn(&cells_fault_tables)?;
    let cells_mismatched_rows = baseline_scores
        .iter()
        .zip(&cells_fault_scores)
        .filter(|(a, b)| a != b)
        .count();

    // In-memory fault 2: flip `routes[0] ^= 1` (for Route read) or `gates[0]` +
    // `routes[0] ^= 1` (for Flock/Relation read, where routes are unused unless
    // route read is active and gates[0] controls level-0 combination).
    let mut route_or_gate_fault_tables = tables.clone();
    if !route_or_gate_fault_tables.routes.is_empty() {
        route_or_gate_fault_tables.routes[0] = (route_or_gate_fault_tables.routes[0] ^ 1) & 63;
    }
    if tables.shape[5] != artifact::READ_ROUTE && !route_or_gate_fault_tables.gates.is_empty() {
        route_or_gate_fault_tables.gates[0] = if route_or_gate_fault_tables.gates[0] == 0 {
            1
        } else {
            0
        };
    }
    let route_or_gate_scores = score_fn(&route_or_gate_fault_tables)?;
    let route_or_gate_mismatched_rows = baseline_scores
        .iter()
        .zip(&route_or_gate_scores)
        .filter(|(a, b)| a != b)
        .count();

    if cells_mismatched_rows == 0 || route_or_gate_mismatched_rows == 0 {
        return Err(format!(
            "planted-fault B4 failed: cells_mismatched_rows={cells_mismatched_rows}, route_or_gate_mismatched_rows={route_or_gate_mismatched_rows}"
        ));
    }

    Ok(json!({
        "b4_pass": true,
        "container_cells_tamper_rejected": cells_tamper_rejected,
        "container_routes_lsb_hash_changed": routes_lsb_hash_changed,
        "container_routes_oob_rejected": routes_oob_rejected,
        "in_memory_cells_fault": {
            "mutated_token": mutated_tok,
            "mismatched_rows": cells_mismatched_rows,
            "total_rows": rows.len(),
        },
        "in_memory_route_or_gate_fault": {
            "read_mode_code": tables.shape[5],
            "mismatched_rows": route_or_gate_mismatched_rows,
            "total_rows": rows.len(),
        },
    }))
}

/// Compares the GPU `d11-scorer score` output against a training-side
/// `discrete-stack eval-corpus` JSON receipt (`final-dev.json`), checking
/// aggregate and row-by-row agreement over all `14,287` DEV rows (`B2` and `B4`).
pub fn compare_with_train_eval(
    score_summary: &Value,
    rows: &[DevRow],
    scores: &[kernel::RowScore],
    train_eval: &Value,
    b4_control: &Value,
) -> Result<Value, String> {
    if rows.len() != scores.len() {
        return Err("rows and scores lengths differ".into());
    }
    let agg = train_eval
        .get("aggregate")
        .ok_or("train_eval lacks aggregate")?;
    let train_u64 = |key: &str| -> Result<u64, String> {
        agg[key]
            .as_u64()
            .ok_or_else(|| format!("train_eval.aggregate lacks {key}"))
    };
    let d11_u64 = |key: &str| -> Result<u64, String> {
        score_summary[key]
            .as_u64()
            .ok_or_else(|| format!("score_summary lacks {key}"))
    };

    let scored_rows_train = train_u64("scored_rows")?;
    if scored_rows_train != rows.len() as u64 {
        return Err(format!(
            "train_eval scored_rows ({scored_rows_train}) != D11 rows ({})",
            rows.len()
        ));
    }
    let top1_train = train_u64("top1_correct")?;
    let top8_train = train_u64("top8_correct")?;
    let adm_train = train_u64("target_admitted_rows")?;
    let loss_ta_train = train_u64("loss_ta_numerator_sum")?;
    let loss_tr_train = train_u64("loss_tr_numerator_sum")?;
    let token_hinge_train = train_u64("token_hinge_sum")?;
    let address_hinge_train = train_u64("address_hinge_sum")?;
    let rank_hinge_train = train_u64("cell_rank_hinge_sum")?;

    let top1_d11 = d11_u64("top1")?;
    let top8_d11 = d11_u64("top8")?;
    let adm_d11 = d11_u64("admitted")?;
    let loss_ta_d11 = d11_u64("loss_ta_numerator_sum")?;
    let loss_tr_d11 = d11_u64("loss_tr_numerator_sum")?;
    let token_hinge_d11 = d11_u64("token_hinge_sum")?;
    let address_hinge_d11 = d11_u64("address_hinge_sum")?;
    let rank_hinge_d11 = d11_u64("cell_rank_hinge_sum")?;

    let batches = train_eval["batches"]
        .as_array()
        .ok_or("train_eval lacks batches array")?;
    let mut train_rows = Vec::with_capacity(rows.len());
    for batch in batches {
        let batch_rows = batch["rows"]
            .as_array()
            .ok_or("train_eval batch lacks rows array")?;
        train_rows.extend(batch_rows.iter());
    }
    if train_rows.len() != rows.len() {
        return Err(format!(
            "train_eval batch rows ({}) != D11 rows ({})",
            train_rows.len(),
            rows.len()
        ));
    }

    let mut row_mismatches = 0u64;
    let mut first_row_mismatch = Value::Null;
    for (idx, ((row, score), tr)) in rows.iter().zip(scores).zip(train_rows).enumerate() {
        let tr_pos = tr["target_position"].as_u64().unwrap_or(u64::MAX);
        let tr_tok = tr["target_token"].as_u64().unwrap_or(u64::MAX);
        let tr_top8: Vec<u32> = tr["top8"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64().map(|x| x as u32))
                    .collect()
            })
            .unwrap_or_default();
        let tr_top1_hit = tr["top1_correct"].as_u64().unwrap_or(u64::MAX);
        let tr_top8_hit = tr["top8_correct"].as_u64().unwrap_or(u64::MAX);
        let tr_tok_hinge = tr["token_hinge"].as_u64().unwrap_or(u64::MAX);
        let tr_addr_hinge = tr["address_hinge"].as_u64().unwrap_or(u64::MAX);
        let tr_rank_hinge = tr["cell_rank_hinge"].as_u64().unwrap_or(u64::MAX);
        let tr_loss_ta = tr["loss_ta_numerator"].as_u64().unwrap_or(u64::MAX);
        let tr_loss_tr = tr["loss_tr_numerator"].as_u64().unwrap_or(u64::MAX);

        let d11_top1_hit = u64::from(score.status == 0 && score.prediction == row.target_token);
        let d11_top8_hit = u64::from(score.top8.contains(&row.target_token));

        let same = score.status == 0
            && tr_pos == row.target_position
            && tr_tok == u64::from(row.target_token)
            && tr_top8 == score.top8
            && tr_top1_hit == d11_top1_hit
            && tr_top8_hit == d11_top8_hit
            && tr_tok_hinge == score.token_hinge
            && tr_addr_hinge == score.address_hinge
            && tr_rank_hinge == score.rank_hinge
            && tr_loss_ta == score.loss_ta
            && tr_loss_tr == score.loss_tr;
        if !same {
            row_mismatches += 1;
            if first_row_mismatch.is_null() {
                first_row_mismatch = json!({
                    "ordinal": idx,
                    "target_position": row.target_position,
                    "target_token": row.target_token,
                    "d11_top8": score.top8,
                    "train_top8": tr_top8,
                    "d11_loss_ta": score.loss_ta,
                    "train_loss_ta": tr_loss_ta,
                    "d11_loss_tr": score.loss_tr,
                    "train_loss_tr": tr_loss_tr,
                });
            }
        }
    }

    let delta_top1 = top1_train.abs_diff(top1_d11);
    let delta_top8 = top8_train.abs_diff(top8_d11);
    let delta_adm = adm_train.abs_diff(adm_d11);
    let delta_loss_ta = loss_ta_train.abs_diff(loss_ta_d11);
    let delta_loss_tr = loss_tr_train.abs_diff(loss_tr_d11);
    let delta_token_hinge = token_hinge_train.abs_diff(token_hinge_d11);
    let delta_address_hinge = address_hinge_train.abs_diff(address_hinge_d11);
    let delta_rank_hinge = rank_hinge_train.abs_diff(rank_hinge_d11);

    let exact_zero_gap = row_mismatches == 0
        && delta_top1 == 0
        && delta_top8 == 0
        && delta_adm == 0
        && delta_loss_ta == 0
        && delta_loss_tr == 0
        && delta_token_hinge == 0
        && delta_address_hinge == 0
        && delta_rank_hinge == 0;
    let b4_pass = b4_control["b4_pass"].as_bool() == Some(true);

    Ok(json!({
        "schema": "uor-r4.d11-scorer-gap/1",
        "artifact_sha256": score_summary["artifact_sha256"],
        "rows": rows.len(),
        "rows_sha256": score_summary["rows_sha256"],
        "train_eval": {
            "accepted_updates_in_model": train_eval["accepted_updates_in_model"],
            "top1": top1_train,
            "top8": top8_train,
            "admitted": adm_train,
            "token_hinge_sum": token_hinge_train,
            "address_hinge_sum": address_hinge_train,
            "cell_rank_hinge_sum": rank_hinge_train,
            "loss_ta_numerator_sum": loss_ta_train,
            "loss_tr_numerator_sum": loss_tr_train,
        },
        "d11_eval": {
            "top1": top1_d11,
            "top8": top8_d11,
            "admitted": adm_d11,
            "token_hinge_sum": token_hinge_d11,
            "address_hinge_sum": address_hinge_d11,
            "cell_rank_hinge_sum": rank_hinge_d11,
            "loss_ta_numerator_sum": loss_ta_d11,
            "loss_tr_numerator_sum": loss_tr_d11,
        },
        "deltas": {
            "delta_top1": delta_top1,
            "delta_top8": delta_top8,
            "delta_admitted": delta_adm,
            "delta_token_hinge": delta_token_hinge,
            "delta_address_hinge": delta_address_hinge,
            "delta_cell_rank_hinge": delta_rank_hinge,
            "delta_loss_ta": delta_loss_ta,
            "delta_loss_tr": delta_loss_tr,
        },
        "registered_two_seed_noise_band": {
            "delta_noise_top1": 398,
            "delta_noise_top8": 748,
            "delta_noise_admitted": 930,
        },
        "row_agreement": {
            "checked_rows": rows.len(),
            "agreed_rows": rows.len() as u64 - row_mismatches,
            "mismatched_rows": row_mismatches,
            "first_mismatch": first_row_mismatch,
        },
        "planted_fault_b4": b4_control,
        "exact_zero_gap": exact_zero_gap,
        "within_noise_band": delta_top1 <= 398 && delta_top8 <= 748 && delta_adm <= 930,
        "b2_pass": exact_zero_gap,
        "b4_pass": b4_pass,
    }))
}

/// The row report: the count, the hash, and per-row metadata (not prefixes).
pub fn rows_report(rows: &[DevRow]) -> Value {
    let listed: Vec<Value> = rows
        .iter()
        .map(|row| {
            json!({"ordinal": row.ordinal, "target_position": row.target_position,
                   "target_token": row.target_token, "prefix_len": row.prefix.len()})
        })
        .collect();
    json!({
        "schema": "uor-r4.d11-scorer-rows/1",
        "rows": rows.len(),
        "rows_sha256": rows_sha256(rows),
        "row_metadata": listed,
    })
}

/// Exports a 19-table `uor-r4.lut-discrete-trunk/1` container (`Exported`) from a
/// `DriverCheckpoint` JSON and `DriverConfig` by scoring `config.dev` (for Gate R)
/// and up to `max_train_batches` batches of `config.train` through `score_fn`
/// (which runs `DeviceScorer` on the GPU when called from `d11-scorer export-trunk`),
/// computing the seed-bound `DiscreteQuaternionRecurrenceV1` (`ActiveHolonomyV1`,
/// `ShuffledOrderControlV1`, `UnitSnapControlV1`) states across worker threads,
/// fitting `TrunkTables`, and round-trip verifying via `export_and_verify_trunk_artifact`.
pub fn export_trunk_with_scorer(
    checkpoint: &Value,
    config: &DriverConfig,
    max_train_batches: usize,
    score_fn: impl Fn(
        &artifact::ArtifactTables,
        &kernel::PackedRows,
    ) -> Result<Vec<kernel::RowScore>, String>,
) -> Result<Exported, String> {
    let base_exported = export_checkpoint(checkpoint)?;
    let (base_tables, base_model) = artifact::ArtifactTables::from_bytes(&base_exported.bytes)?;
    let model_val = checkpoint.get("model").ok_or("checkpoint lacks model")?;
    let mut snapshot: Snapshot =
        serde_json::from_value(model_val.clone()).map_err(|e| format!("checkpoint model: {e}"))?;
    snapshot.position = None;
    config.validate().map_err(|e| e.to_string())?;
    if config.spec != snapshot.spec {
        return Err("config.spec does not match checkpoint snapshot.spec".into());
    }

    // 1. Gate R check on config.dev
    let dev_rows_vec = dev_rows(config)?;
    let dev_views: Vec<&[u32]> = dev_rows_vec.iter().map(|r| r.prefix.as_slice()).collect();
    let dev_targets: Vec<u32> = dev_rows_vec.iter().map(|r| r.target_token).collect();
    let dev_packed = kernel::pack_rows_with_targets(&dev_views, &dev_targets)?;
    let dev_scores = score_fn(&base_tables, &dev_packed)?;
    let dev_ref = reference_check(&base_model, &dev_rows_vec, &dev_scores);
    let dev_ref_mismatches = dev_ref["checked"]
        .as_u64()
        .unwrap_or(0)
        .saturating_sub(dev_ref["agreed"].as_u64().unwrap_or(0));
    if dev_ref_mismatches != 0 {
        return Err(format!(
            "Gate R reference mismatch on dev before trunk export: {}",
            dev_ref["first_mismatch"]
        ));
    }
    let (dev_summary, _) = score_summary(&base_tables, &dev_rows_vec, &dev_scores)?;
    let mut dev_read_codes = std::collections::BTreeSet::new();
    for sc in &dev_scores {
        let q_read: [i32; 32] = sc
            .read_vector
            .as_slice()
            .try_into()
            .map_err(|_| "dev read_vector length != 32".to_string())?;
        let rc =
            uor_r4_discrete::w2_capacity::decode_read_vector(&q_read).map_err(|e| e.to_string())?;
        dev_read_codes.insert(rc);
    }

    // 2. Capture up to `max_train_batches` batches of `config.train`
    let train_corpus = PreparedCorpus::load(&config.train, config.spec.vocabulary, &config.bos)
        .map_err(|e| e.to_string())?;
    let (codebook_roots, _) = uor_r4_discrete::trunk::decode_codebook_table(&snapshot.table)
        .map_err(|e| e.to_string())?;
    let seed_offset = (snapshot.seed.saturating_sub(1)) as usize;
    let step_table =
        uor_r4_discrete::trunk::build_trunk_step_table(snapshot.spec.blocks, seed_offset)
            .map_err(|e| e.to_string())?;
    let norm_frames =
        uor_r4_discrete::trunk::build_trunk_norm_frames(snapshot.spec.blocks, seed_offset)
            .map_err(|e| e.to_string())?;

    let mut raw_prefixes: Vec<Vec<u32>> = Vec::new();
    let mut raw_targets: Vec<u32> = Vec::new();
    let mut first = 0u64;
    for _ in 0..max_train_batches {
        if first >= train_corpus.eligible_len() as u64 {
            break;
        }
        let batch = train_corpus
            .batch(first, config.spec.context, config.batch_rows)
            .map_err(|e| e.to_string())?;
        for (window, &target) in batch.windows.iter().zip(&batch.targets) {
            raw_prefixes.push(window.iter().map(|id| id.token).collect());
            raw_targets.push(target);
        }
        if batch.next_target <= first {
            return Err("train corpus batch cursor did not advance".into());
        }
        first = batch.next_target;
    }

    let total_train = raw_prefixes.len();
    let mut read_codes = Vec::with_capacity(total_train);
    const SCORE_CHUNK: usize = 262_144;
    let mut chunk_start = 0usize;
    while chunk_start < total_train {
        let chunk_end = (chunk_start + SCORE_CHUNK).min(total_train);
        let views: Vec<&[u32]> = raw_prefixes[chunk_start..chunk_end]
            .iter()
            .map(|p| p.as_slice())
            .collect();
        let packed = kernel::pack_rows_with_targets(&views, &raw_targets[chunk_start..chunk_end])?;
        let scores = score_fn(&base_tables, &packed)?;
        for sc in scores {
            if sc.status != 0 {
                return Err(format!("train row refused with status {}", sc.status));
            }
            let q_read: [i32; 32] = sc
                .read_vector
                .as_slice()
                .try_into()
                .map_err(|_| "train read_vector length != 32".to_string())?;
            let rc = uor_r4_discrete::w2_capacity::decode_read_vector(&q_read)
                .map_err(|e| e.to_string())?;
            read_codes.push(rc);
        }
        chunk_start = chunk_end;
    }

    // Compute recurrence states across worker threads in deterministic order
    let workers = std::thread::available_parallelism()
        .map(|n| n.get().min(16))
        .unwrap_or(4)
        .max(1);
    let slice_len = total_train.div_ceil(workers).max(1);
    let mut train_rows = Vec::with_capacity(total_train);
    std::thread::scope(|s| -> Result<(), String> {
        let mut handles = Vec::new();
        for w in 0..workers {
            let start = w * slice_len;
            if start >= total_train {
                break;
            }
            let end = ((w + 1) * slice_len).min(total_train);
            let pfx_slice = &raw_prefixes[start..end];
            let tgt_slice = &raw_targets[start..end];
            let rc_slice = &read_codes[start..end];
            let roots_ref = &codebook_roots;
            let step_ref = &step_table;
            let norm_ref = &norm_frames;
            handles.push(s.spawn(move || -> Result<Vec<uor_r4_discrete::trunk::TrunkTrainRow>, String> {
                let mut out = Vec::with_capacity(end - start);
                for i in 0..(end - start) {
                    let global_idx = start + i;
                    let win_u16: Vec<u16> = pfx_slice[i].iter().map(|&t| t as u16).collect();
                    let query_token = *win_u16.last().ok_or("empty train prefix")?;
                    let target_token = u16::try_from(tgt_slice[i]).map_err(|_| "target > u16")?;
                    let row_seed = (global_idx as u64).wrapping_add(1);
                    let (_, rec_code) = uor_r4_discrete::trunk::compute_recurrence_state(
                        &win_u16,
                        roots_ref,
                        step_ref,
                        norm_ref,
                        uor_r4_discrete::trunk::RecurrenceMode::ActiveHolonomyV1,
                        uor_r4_discrete::normalization_mechanism::NormalizationMode::ScaleMatchedSnapV1,
                        row_seed,
                    )
                    .map_err(|e| e.to_string())?;
                    let (_, rec_shuf_code) = uor_r4_discrete::trunk::compute_recurrence_state(
                        &win_u16,
                        roots_ref,
                        step_ref,
                        norm_ref,
                        uor_r4_discrete::trunk::RecurrenceMode::ShuffledOrderControlV1,
                        uor_r4_discrete::normalization_mechanism::NormalizationMode::ScaleMatchedSnapV1,
                        row_seed,
                    )
                    .map_err(|e| e.to_string())?;
                    let (_, rec_unit_code) = uor_r4_discrete::trunk::compute_recurrence_state(
                        &win_u16,
                        roots_ref,
                        step_ref,
                        norm_ref,
                        uor_r4_discrete::trunk::RecurrenceMode::ActiveHolonomyV1,
                        uor_r4_discrete::normalization_mechanism::NormalizationMode::UnitSnapControlV1,
                        row_seed,
                    )
                    .map_err(|e| e.to_string())?;
                    out.push(uor_r4_discrete::trunk::TrunkTrainRow {
                        read_code: rc_slice[i],
                        rec_code,
                        rec_shuf_code,
                        rec_unit_code,
                        query_token,
                        target_token,
                    });
                }
                Ok(out)
            }));
        }
        for h in handles {
            let chunk_rows = h
                .join()
                .map_err(|_| "recurrence worker thread panicked".to_string())??;
            train_rows.extend(chunk_rows);
        }
        Ok(())
    })?;

    let trunk_tables =
        uor_r4_discrete::trunk::TrunkTables::build(&train_rows).map_err(|e| e.to_string())?;
    let token_cells: Vec<u16> = base_tables.cells.iter().map(|&c| c as u16).collect();
    let (manifest, bytes) = uor_r4_discrete::trunk::export_and_verify_trunk_artifact(
        &snapshot,
        &token_cells,
        &step_table,
        &norm_frames,
        &trunk_tables,
    )
    .map_err(|e| e.to_string())?;

    // Verify clean-room IntegerDiscreteModel and ArtifactTables + score_fn agreement on dev sample
    let (trunk_artifact_tables, trunk_model) = artifact::ArtifactTables::from_bytes(&bytes)?;
    let sample_n = dev_rows_vec.len().min(64);
    let sample_views: Vec<&[u32]> = dev_rows_vec[..sample_n]
        .iter()
        .map(|r| r.prefix.as_slice())
        .collect();
    let sample_targets: Vec<u32> = dev_rows_vec[..sample_n]
        .iter()
        .map(|r| r.target_token)
        .collect();
    let sample_packed = kernel::pack_rows_with_targets(&sample_views, &sample_targets)?;
    let sample_scores = score_fn(&trunk_artifact_tables, &sample_packed)?;
    let sample_ref = reference_check(&trunk_model, &dev_rows_vec[..sample_n], &sample_scores);
    let sample_mismatches = sample_ref["checked"]
        .as_u64()
        .unwrap_or(0)
        .saturating_sub(sample_ref["agreed"].as_u64().unwrap_or(0));
    if sample_mismatches != 0 {
        return Err(format!(
            "trunk D11 sample reference mismatch after export: {}",
            sample_ref["first_mismatch"]
        ));
    }

    let shape_json = serde_json::to_value(trunk_model.shape()).map_err(|e| e.to_string())?;
    let report = json!({
        "schema": "uor-r4.d11-scorer-trunk-export/1",
        "file_sha256": manifest.d11_binary_sha256,
        "artifact_sha256": trunk_model.artifact_sha256(),
        "bytes": manifest.d11_binary_bytes,
        "base_artifact_sha256": base_exported.report["file_sha256"],
        "checkpoint_model_sha256": base_exported.report["checkpoint_model_sha256"],
        "accepted_updates": base_exported.report["accepted_updates"],
        "seed": snapshot.seed,
        "read_mode": format!("{:?}", trunk_model.read_mode()),
        "shape": shape_json,
        "gate_r_dev": {
            "rows": dev_rows_vec.len(),
            "top1": dev_summary["top1"],
            "top8": dev_summary["top8"],
            "admitted": dev_summary["admitted"],
            "distinct_read_codes": dev_read_codes.len(),
            "reference_mismatches": dev_ref_mismatches,
        },
        "train_rows_captured": train_rows.len(),
        "distinct_train_read_codes": manifest.distinct_train_read_codes,
        "distinct_train_recurrence_codes": manifest.distinct_train_recurrence_codes,
        "distinct_train_read_rec_pairs": manifest.distinct_train_read_rec_pairs,
        "distinct_train_query_tokens": manifest.distinct_train_query_tokens,
        "round_trip_verified": manifest.round_trip_verified,
        "trunk_dev_sample_checked": sample_n,
        "trunk_dev_sample_reference_mismatches": sample_mismatches,
    });
    Ok(Exported { bytes, report })
}
