//! Record assembly for `uor-r4.read-code-capacity/1`: binds the GPU capture
//! (`capture.json`, the compact TRAIN stream and its query sidecar) to the
//! sealed DEV panel, refuses on any instrument disagreement, and scores the
//! exact-code table against the unigram, bigram and writer comparators.
//! Integer arithmetic only.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use crate::w2_capacity::{
    admission_matched, aggregate, build_sym_table, build_tables, decide, dev_panel,
    ordered_row_binding_sha256, parse_uort_tokens, score_admitted, score_dev,
    symmetry_action_non_roots, symmetry_keys, symmetry_root_actions, DevRow, TrainRow,
    ADMITTED_ARMS, BOS_ID, DEV_ROWS, FALLBACK_CAPTURE_MODE, MATCHED_TRAIN_ROWS, TOP_K, VOCABULARY,
};
use crate::w2_capture::{ABSENT_SLOT, CELL_SLOTS, SEALED_SHA256};

pub const SCHEMA: &str = "uor-r4.read-code-capacity/1";
pub const EPOCH: u64 = 407;
pub const CHECKPOINT_SHA256: &str =
    "4477eb5c2480b0b85cc94036a2e15b0d22b654e25deee9c8717aa6e30f8ef7bf";
pub const ORIGINAL_SOURCE_SHA256: &str =
    "e18361495276e491525674fb3fdce308b0ad787c7fd6860a811adb60dacd9e1a";
pub const ORIGINAL_KERNEL_SHA256: &str =
    "931a511bece20641ae00c6b0e943f4f2a380c0f35a824d85fcc0447f1e1ccf5b";
pub const ORIGINAL_OPERATIONAL_DRIVER_SHA256: &str =
    "cb090e85e78805acd2b2347373340168a9e2f87aaf72a1dcb42f091e9ff1f7b4";
pub const ORIGINAL_MODEL_DOMAIN_A_SHA256: &str =
    "49fa448cb3349ac5711966082a9c53918a693d45e11f282221d240cbb463fa73";
pub const BOS_TOKEN: &str = "<|bos|>";

/// SHA-256 of the five producer files, computed by the caller from disk.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FilePins {
    pub dev_tokens_sha256: String,
    pub dev_mask_sha256: String,
    pub train_tokens_sha256: String,
    pub train_mask_sha256: String,
    pub tokenizer_sha256: String,
}

impl FilePins {
    fn named(&self) -> [(&'static str, &str); 5] {
        [
            ("dev_tokens_sha256", &self.dev_tokens_sha256),
            ("dev_mask_sha256", &self.dev_mask_sha256),
            ("train_tokens_sha256", &self.train_tokens_sha256),
            ("train_mask_sha256", &self.train_mask_sha256),
            ("tokenizer_sha256", &self.tokenizer_sha256),
        ]
    }
}

/// The fixed shape a capture must have. Production uses [`PRODUCTION`] only;
/// unit tests build a small synthetic shape.
struct Expected<'a> {
    train_rows: usize,
    dev_rows: usize,
    min_root_rows: u64,
    min_determinism_rows: u64,
    panel_binding: &'a str,
    writer_top1: u64,
    writer_top8: u64,
    writer_admitted: u64,
    /// In [`FilePins::named`] order.
    pins: [&'a str; 5],
}

const PRODUCTION: Expected<'static> = Expected {
    train_rows: MATCHED_TRAIN_ROWS,
    dev_rows: DEV_ROWS,
    min_root_rows: 1_024,
    min_determinism_rows: 1_000,
    panel_binding: "e4ef44fb81d6c3a833212e4857117db71cafb8259b81c286e5b1391129d74446",
    writer_top1: 149,
    writer_top8: 747,
    writer_admitted: 1_920,
    pins: [
        "0919e90f953e988edf05cbb8a69e24bafda0cb8b8ddbbb5d1e9d808c81a6555b",
        "a838bb1a67e96572cb10a86b5a3587dda25f0190eaf85133026e04c281afb078",
        "3dee858f84b750f507ceb3dc3f393711741950954735efaaf61bf704c017a4c8",
        "b5e7352d67d6883a833155a5fd397a03f6d4f1b2649e3d4ec3e2ea7d95b949e3",
        "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89",
    ],
};

/// The exact bytes the caller must write as the runtime manifest; the record's
/// `original_to_target_bridge.runtime_manifest_sha256` is the SHA-256 of them.
pub fn runtime_manifest_bytes(manifest: &Value) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(manifest).map_err(|e| format!("manifest: {e}"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn at<'v>(v: &'v Value, path: &str) -> Result<&'v Value, String> {
    path.split('.')
        .try_fold(v, |v, k| v.get(k))
        .ok_or_else(|| format!("capture: missing {path}"))
}
fn u64_at(v: &Value, path: &str) -> Result<u64, String> {
    at(v, path)?
        .as_u64()
        .ok_or_else(|| format!("capture: {path} is not an unsigned integer"))
}
fn str_at<'v>(v: &'v Value, path: &str) -> Result<&'v str, String> {
    at(v, path)?
        .as_str()
        .ok_or_else(|| format!("capture: {path} is not a string"))
}
fn bool_at(v: &Value, path: &str) -> Result<bool, String> {
    at(v, path)?
        .as_bool()
        .ok_or_else(|| format!("capture: {path} is not a bool"))
}
fn u16_of(v: &Value, what: &str) -> Result<u16, String> {
    v.as_u64()
        .and_then(|x| u16::try_from(x).ok())
        .ok_or_else(|| format!("capture: {what} is not a u16"))
}
fn token_of(v: &Value, what: &str) -> Result<u16, String> {
    let t = u16_of(v, what)?;
    if usize::from(t) >= VOCABULARY {
        return Err(format!("capture: {what} {t} >= {VOCABULARY}"));
    }
    Ok(t)
}
fn u16_array(v: &Value, path: &str) -> Result<Vec<u16>, String> {
    at(v, path)?
        .as_array()
        .ok_or_else(|| format!("capture: {path} is not an array"))?
        .iter()
        .map(|x| u16_of(x, path))
        .collect()
}

/// `table_top8` padded to `TOP_K` distinct tokens: the table's own ranking,
/// then the unigram ranking, then the lowest unused token ids (the tie rule
/// extended to zero counts), skipping duplicates.
fn padded_top8(own: &[u16], unigram: &[u16]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(TOP_K);
    let pool = own
        .iter()
        .chain(unigram)
        .copied()
        .chain(0..VOCABULARY as u16);
    for t in pool {
        if out.len() == TOP_K {
            break;
        }
        if !out.contains(&t) {
            out.push(t);
        }
    }
    out
}

/// Build the record and the runtime manifest from production inputs. The
/// caller writes the manifest as [`runtime_manifest_bytes`].
pub fn assemble(
    capture: &Value,
    train_stream: &[u8],
    query_sidecar: &[u8],
    dev_tokens: &[u8],
    dev_mask: &[u8],
    pins: &FilePins,
    ratchets_pass: bool,
) -> Result<(Value, Value), String> {
    assemble_with(
        &PRODUCTION,
        capture,
        train_stream,
        query_sidecar,
        dev_tokens,
        dev_mask,
        pins,
        ratchets_pass,
    )
}

#[allow(clippy::too_many_arguments)]
fn assemble_with(
    ex: &Expected<'_>,
    capture: &Value,
    train_stream: &[u8],
    query_sidecar: &[u8],
    dev_tokens: &[u8],
    dev_mask: &[u8],
    pins: &FilePins,
    ratchets_pass: bool,
) -> Result<(Value, Value), String> {
    // Producer file pins.
    for ((name, got), want) in pins.named().into_iter().zip(ex.pins) {
        if got != want {
            return Err(format!("file pin {name}: {got} != {want}"));
        }
    }

    // Capture mode, Gate R, root round-trip, determinism.
    let mode = str_at(capture, "capture_mode")?;
    if mode != FALLBACK_CAPTURE_MODE {
        return Err(format!("capture_mode {mode} != {FALLBACK_CAPTURE_MODE}"));
    }
    if !bool_at(capture, "gate_r.pass")? {
        return Err("gate_r did not pass".into());
    }
    let gate_r_mismatches = u64_at(capture, "gate_r.mismatch_count")?;
    if gate_r_mismatches != 0 {
        return Err(format!("gate_r.mismatch_count {gate_r_mismatches} != 0"));
    }
    let (rt_rows, rt_equal) = (
        u64_at(capture, "root_round_trip.rows")?,
        u64_at(capture, "root_round_trip.equal")?,
    );
    if rt_rows < ex.min_root_rows || rt_equal != rt_rows {
        return Err(format!(
            "root round-trip {rt_equal}/{rt_rows} (need all equal, rows >= {})",
            ex.min_root_rows
        ));
    }
    let (det_rows, det_equal) = (
        u64_at(capture, "determinism.rows")?,
        u64_at(capture, "determinism.equal")?,
    );
    if det_rows < ex.min_determinism_rows || det_equal != det_rows {
        return Err(format!(
            "determinism {det_equal}/{det_rows} (need all equal, rows >= {})",
            ex.min_determinism_rows
        ));
    }

    // Compact TRAIN stream (code, target) and the query sidecar.
    if train_stream.len() != 4 * ex.train_rows {
        return Err(format!(
            "train stream {} bytes != {}",
            train_stream.len(),
            4 * ex.train_rows
        ));
    }
    if query_sidecar.len() != 2 * ex.train_rows {
        return Err(format!(
            "query sidecar {} bytes != {}",
            query_sidecar.len(),
            2 * ex.train_rows
        ));
    }
    let stream_sha = sha256_hex(train_stream);
    for (name, bytes, sha) in [
        ("train_stream", train_stream, stream_sha.clone()),
        ("query_sidecar", query_sidecar, sha256_hex(query_sidecar)),
    ] {
        let rows = u64_at(capture, &format!("{name}.rows"))?;
        let recorded = str_at(capture, &format!("{name}.sha256"))?;
        if rows != ex.train_rows as u64 || recorded != sha {
            return Err(format!(
                "{name}: capture records {rows} rows sha {recorded}, file has {} bytes sha {sha}",
                bytes.len()
            ));
        }
    }
    let word = |b: &[u8], i: usize| u16::from_le_bytes([b[2 * i], b[2 * i + 1]]);
    let train: Vec<TrainRow> = (0..ex.train_rows)
        .map(|i| TrainRow {
            code: word(train_stream, 2 * i),
            query_token: word(query_sidecar, i),
            target_token: word(train_stream, 2 * i + 1),
        })
        .collect();
    let invalid_tokens = train
        .iter()
        .flat_map(|r| [r.query_token, r.target_token])
        .filter(|&t| usize::from(t) >= VOCABULARY)
        .count() as u64;
    if invalid_tokens != 0 {
        return Err(format!(
            "train stream: {invalid_tokens} tokens >= {VOCABULARY}"
        ));
    }

    // Sealed DEV panel and its binding.
    let tokens = parse_uort_tokens(dev_tokens)?;
    let panel = dev_panel(&tokens, dev_mask)?;
    if panel.len() != ex.dev_rows {
        return Err(format!("DEV panel {} rows != {}", panel.len(), ex.dev_rows));
    }
    let panel_binding = ordered_row_binding_sha256(&panel);
    if panel_binding != ex.panel_binding {
        return Err(format!(
            "panel binding {panel_binding} != {}",
            ex.panel_binding
        ));
    }

    let token_cells = u16_array(capture, "token_cells")?;
    if token_cells.len() != VOCABULARY {
        return Err(format!("token_cells {} != {VOCABULARY}", token_cells.len()));
    }
    let cap_rows = at(capture, "dev_rows")?
        .as_array()
        .ok_or("capture: dev_rows is not an array")?;
    if cap_rows.len() != ex.dev_rows {
        return Err(format!(
            "capture dev_rows {} != {}",
            cap_rows.len(),
            ex.dev_rows
        ));
    }

    // Per-row binding to the panel and the admission checks.
    struct Captured {
        code: u16,
        admitted_cells: u64,
        address_rivals: u64,
        target_admitted: u64,
        slots: Vec<u16>,
        top8: Value,
        top1_correct: u64,
        top8_correct: u64,
    }
    let mut captured = Vec::with_capacity(ex.dev_rows);
    let mut dev = Vec::with_capacity(ex.dev_rows);
    let (mut sum_top1, mut sum_top8, mut sum_admitted) = (0u64, 0u64, 0u64);
    for (i, (row, p)) in cap_rows.iter().zip(&panel).enumerate() {
        let get = |k: &str| u64_at(row, k).map_err(|e| format!("dev row {i}: {e}"));
        let query = token_of(at(row, "query_token")?, "query_token")?;
        let target = token_of(at(row, "target_token")?, "target_token")?;
        let (ordinal, position) = (get("row_ordinal")?, get("target_position")?);
        if (ordinal, position, query, target)
            != (
                p.ordinal as u64,
                p.position as u64,
                p.query_token,
                p.target_token,
            )
        {
            return Err(format!(
                "dev row {i}: capture ({ordinal},{position},{query},{target}) != panel ({},{},{},{})",
                p.ordinal, p.position, p.query_token, p.target_token
            ));
        }
        let (admitted_cells, address_rivals) = (get("admitted_cells")?, get("address_rivals")?);
        let target_admitted = admitted_cells
            .checked_sub(address_rivals)
            .filter(|&d| d <= 1)
            .ok_or_else(|| {
                format!(
                    "dev row {i}: target_admitted = {admitted_cells} - {address_rivals} not in {{0,1}}"
                )
            })?;
        let slots = u16_array(row, "admitted_cell_slots")?;
        if slots.len() != CELL_SLOTS {
            return Err(format!("dev row {i}: {} cell slots", slots.len()));
        }
        let cell = token_cells[usize::from(target)];
        let in_slots = cell != ABSENT_SLOT && slots.iter().any(|&s| s != ABSENT_SLOT && s == cell);
        if in_slots != (target_admitted == 1) {
            return Err(format!(
                "dev row {i}: token_cells[{target}]={cell} in admitted slots is {in_slots} but target_admitted is {target_admitted}"
            ));
        }
        let (top1_correct, top8_correct) = (get("top1_correct")?, get("top8_correct")?);
        if top1_correct > 1 || top8_correct > 1 {
            return Err(format!("dev row {i}: top1/top8_correct not in {{0,1}}"));
        }
        let top8 = u16_array(row, "top8")?;
        if top8.len() != TOP_K {
            return Err(format!(
                "dev row {i}: top8 has {} entries, need {TOP_K}",
                top8.len()
            ));
        }
        for (j, &t) in top8.iter().enumerate() {
            if usize::from(t) >= VOCABULARY {
                return Err(format!("dev row {i}: top8[{j}] {t} >= {VOCABULARY}"));
            }
            if top8[..j].contains(&t) {
                return Err(format!("dev row {i}: top8[{j}] {t} repeats"));
            }
        }
        if (top1_correct == 1) != (top8[0] == target)
            || (top8_correct == 1) != top8.contains(&target)
        {
            return Err(format!(
                "dev row {i}: top1/top8_correct contradict top8 and target {target}"
            ));
        }
        sum_top1 += top1_correct;
        sum_top8 += top8_correct;
        sum_admitted += target_admitted;
        let code = u16_of(at(row, "code")?, "code")?;
        dev.push(DevRow {
            code,
            query_token: query,
            target_token: target,
            writer_hit1: top1_correct == 1,
            writer_hit8: top8_correct == 1,
        });
        captured.push(Captured {
            code,
            admitted_cells,
            address_rivals,
            target_admitted,
            slots,
            top8: at(row, "top8")?.clone(),
            top1_correct,
            top8_correct,
        });
    }
    if (sum_top1, sum_top8, sum_admitted) != (ex.writer_top1, ex.writer_top8, ex.writer_admitted) {
        return Err(format!(
            "writer sums top1 {sum_top1} top8 {sum_top8} admitted {sum_admitted} != sealed {} {} {}",
            ex.writer_top1, ex.writer_top8, ex.writer_admitted
        ));
    }

    // Tables, scores, aggregates, verdict.
    let tables = build_tables(&train).map_err(|e| e.to_string())?;
    let scores = score_dev(&tables, &dev).map_err(|e| e.to_string())?;
    let a = aggregate(&tables, &scores);
    if a.matched_train_total != ex.train_rows as u64 || a.bigram_train_total != ex.train_rows as u64
    {
        return Err(format!(
            "count tables not conserved: matched {} bigram {} rows {}",
            a.matched_train_total, a.bigram_train_total, ex.train_rows
        ));
    }
    for (name, d, comparator) in [
        ("U", a.d_u, a.u_top1),
        ("G", a.d_g, a.g_top1),
        ("W", a.d_w, a.w_top1),
    ] {
        if d.c as i64 - d.b as i64 != a.t_top1 as i64 - comparator as i64 {
            return Err(format!("{name}: c - b != T - comparator"));
        }
    }
    let decision = decide(&a);
    let verdict = if ratchets_pass {
        decision.band.label()
    } else {
        "INSTRUMENT_FAULT"
    };

    // Descriptive arms (sheet §4.3 items 4 and 6), never inputs to the verdict.
    let sym_keys = symmetry_keys(&symmetry_root_actions()?);
    let table_sym = build_sym_table(&train, &sym_keys);
    let slots: Vec<Vec<u16>> = captured.iter().map(|c| c.slots.clone()).collect();
    let admitted = score_admitted(&tables, &table_sym, &sym_keys, &dev, &token_cells, &slots);
    let am = admission_matched(&admitted, &dev);
    let arms = |counts: &[crate::w2_capacity::ArmCounts; 5]| {
        let mut m = serde_json::Map::new();
        for (name, k) in ADMITTED_ARMS.iter().zip(counts) {
            m.insert(name.to_string(), json!({ "hit1": k.hit1, "hit8": k.hit8 }));
        }
        Value::Object(m)
    };

    let unigram = tables.unigram.ranked();
    let unigram_top1 = *unigram.first().ok_or("unigram is empty")?;
    let mut rows = Vec::with_capacity(ex.dev_rows);
    for (((p, c), s), r) in panel.iter().zip(&captured).zip(&scores).zip(&admitted) {
        let target = p.target_token;
        let table_top8 = padded_top8(&s.table.top8, &unigram);
        let table_top1 = table_top8[0];
        let bigram_top1 = s.bigram.top1().ok_or("bigram prediction is empty")?;
        rows.push(json!({
            "row_ordinal": p.ordinal,
            "document_id": p.document_id,
            "record": (1u64 << 62) | p.document_id as u64,
            "target_position": p.position,
            "query_occurrence": p.position - 1,
            "query_token": p.query_token,
            "target_token": target,
            "code": c.code,
            "root_first": c.code & 0xF,
            "root_last": (c.code >> 12) & 0xF,
            "admitted_cells": c.admitted_cells,
            "address_rivals": c.address_rivals,
            "target_admitted": c.target_admitted,
            "admitted_cell_slots": c.slots,
            "top8": c.top8,
            "top1_correct": c.top1_correct,
            "top8_correct": c.top8_correct,
            "table_top1": table_top1,
            "table_top8": table_top8,
            "unigram_1049856_top1": unigram_top1,
            "bigram_top1": bigram_top1,
            "table_hit1": table_top1 == target,
            "table_hit8": table_top8.contains(&target),
            "unigram_1049856_hit1": unigram_top1 == target,
            "bigram_hit1": bigram_top1 == target,
            "seen_code": s.seen_code,
            "table_prediction_admitted": r.table.top1(),
            "table_hit1_admitted": r.table.hit1(target),
            "table_hit8_admitted": r.table.hit8(target),
            "table_sym_top1": r.table_sym_full.top1(),
            "table_sym_hit1": r.table_sym_full.hit1(target),
        }));
    }

    let identity = |k: &str| str_at(capture, &format!("target_identity.{k}"));
    let target_source = identity("source_sha256")?;
    let target_kernel = identity("kernel_sha256")?;
    let target_binary = str_at(capture, "target_binary_sha256")?;
    let manifest = json!({
        "original_source_sha256": ORIGINAL_SOURCE_SHA256,
        "original_kernel_sha256": ORIGINAL_KERNEL_SHA256,
        "original_operational_driver_sha256": ORIGINAL_OPERATIONAL_DRIVER_SHA256,
        "original_model_domain_a_sha256": ORIGINAL_MODEL_DOMAIN_A_SHA256,
        "target_compiled_source_sha256": target_source,
        "target_kernel_sha256": target_kernel,
        "target_binary_sha256": target_binary,
    });
    let manifest_sha = sha256_hex(&runtime_manifest_bytes(&manifest)?);
    let admission = json!({
        "rows": am.rows,
        "target_admitted_rows": am.target_admitted_rows,
        "all_rows": arms(&am.all),
        "target_admitted": arms(&am.target_admitted),
        "table_sym_top1": am.table_sym_top1_full,
    });
    let disc = |d: crate::w2_capacity::Discordance| json!({ "b": d.b, "c": d.c });

    let record = json!({
        "schema": SCHEMA,
        "zero_apply": true,
        "applied_updates": 0,
        "updates_applied": 0,
        "snapshot_unchanged": true,
        "model_compute_executed": true,
        "source_sha256": target_source,
        "epoch": EPOCH,
        "checkpoint_sha256": CHECKPOINT_SHA256,
        "panel_binding_sha256": panel_binding,
        "verdict": verdict,
        "original_to_target_bridge": {
            "original_source_sha256": ORIGINAL_SOURCE_SHA256,
            "original_kernel_sha256": ORIGINAL_KERNEL_SHA256,
            "target_compiled_source_sha256": target_source,
            "target_kernel_sha256": target_kernel,
            "target_binary_sha256": target_binary,
            "runtime_manifest_sha256": manifest_sha,
        },
        "instrument_validity": {
            "root_round_trip": {
                "sample_rows": rt_rows,
                "mismatches": rt_rows - rt_equal,
                // Holds by construction, not by measurement here: a capture with an ABSENT
                // block writes fault.json and no capture.json.
                "absent_roots": 0,
            },
            "train_capture": { "mode": mode, "smaller_table_bias_disclosed": true },
            "sealed_receipt_agreement": {
                "dev_rows": ex.dev_rows,
                "row_disagreements": gate_r_mismatches,
                "evaluation_sha256": SEALED_SHA256,
                "writer_top1": ex.writer_top1,
                "writer_top8": ex.writer_top8,
                "writer_admitted": ex.writer_admitted,
            },
            "count_table_conservation": {
                "matched_train_total": a.matched_train_total,
                "bigram_train_total": a.bigram_train_total,
                "invalid_tokens": invalid_tokens,
                // Holds by construction: counts are unsigned.
                "negative_counts": 0,
            },
            "determinism": { "repeat_rows": det_rows, "byte_identical": det_equal == det_rows },
            "symmetry_action_non_roots": symmetry_action_non_roots(),
            // Holds by construction, not by measurement here: a CUDA or status error
            // writes fault.json and no capture.json.
            "cuda_or_status_fault": false,
            "d23_d25_d32_ratchets_pass": ratchets_pass,
        },
        "producer_file_pins": {
            "dev_tokens_sha256": pins.dev_tokens_sha256,
            "dev_mask_sha256": pins.dev_mask_sha256,
            "train_tokens_sha256": pins.train_tokens_sha256,
            "train_mask_sha256": pins.train_mask_sha256,
            "tokenizer_sha256": pins.tokenizer_sha256,
            "bos_token": BOS_TOKEN,
            "bos_id": BOS_ID,
            "compact_train_stream_bytes": train_stream.len(),
            "compact_train_stream_sha256": stream_sha,
        },
        "evaluability": {
            "dev_rows": a.dev_rows,
            "coverage_rows": a.coverage_rows,
            "codes_with_count_ge_2": a.codes_with_count_ge_2,
        },
        "aggregates": {
            "T_table_matched_top1": a.t_top1,
            "U_unigram_matched_top1": a.u_top1,
            "G_bigram_matched_top1": a.g_top1,
            "W_writer_top1": a.w_top1,
            "paired_discordance": { "U": disc(a.d_u), "G": disc(a.d_g), "W": disc(a.d_w) },
            "descriptive": {
                "t_top8_unpadded": a.t_top8,
                "u_top8": a.u_top8,
                "g_top8": a.g_top8,
                "w_top8": a.w_top8,
                "address_only_top1": a.address_only_top1,
                "interior_only_top1": a.interior_only_top1,
                "dev_rows_code_count_ge_2": a.dev_rows_code_count_ge_2,
                "fallback_rows": a.fallback_rows,
                "distinct_codes": a.distinct_codes,
                "m_top1": a.m_top1(),
                "decision_band": decision.band.label(),
                "decision_precedence": decision.precedence,
                "pathological_ordering": decision.pathological_ordering,
                "underpowered_reasons": decision.underpowered_reasons,
                "admission_matched": admission,
            },
        },
        "rows": rows,
    });
    Ok((record, manifest))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::w2_capacity::PanelRow;

    // doc 0: BOS 5 6 5 6 7 5 6 — seven eligible rows.
    const DEV: [u16; 8] = [0, 5, 6, 5, 6, 7, 5, 6];
    /// Row 6's code is never trained, so the table falls back and misses it.
    const UNSEEN_CODE: u16 = 99;
    /// Rows the writer hits at top-1 / admits.
    const WRITER_HITS: [usize; 3] = [0, 1, 6];
    const ADMITTED: [usize; 4] = [0, 2, 3, 6];
    const TEST_PINS: [&str; 5] = ["dt", "dm", "tt", "tm", "tk"];

    fn uort(tokens: &[u16]) -> Vec<u8> {
        let mut b = b"UORT".to_vec();
        b.extend(1u32.to_le_bytes());
        b.extend((tokens.len() as u64).to_le_bytes());
        b.extend((VOCABULARY as u32).to_le_bytes());
        b.resize(64, 0);
        tokens.iter().for_each(|t| b.extend(t.to_le_bytes()));
        b
    }

    fn panel() -> Vec<PanelRow> {
        dev_panel(&DEV, &[1; 8]).unwrap()
    }

    fn code_of(i: usize) -> u16 {
        if i == 6 {
            UNSEEN_CODE
        } else {
            0x1001 + i as u16
        }
    }

    struct Fixture {
        capture: Value,
        stream: Vec<u8>,
        sidecar: Vec<u8>,
        tokens: Vec<u8>,
        mask: Vec<u8>,
        pins: FilePins,
        binding: String,
    }

    fn fixture() -> Fixture {
        let panel = panel();
        let (mut stream, mut sidecar) = (Vec::new(), Vec::new());
        // Each seen row's code is trained twice on its own (query, target).
        for p in panel.iter().filter(|p| p.ordinal != 6) {
            for _ in 0..2 {
                stream.extend(code_of(p.ordinal).to_le_bytes());
                stream.extend(p.target_token.to_le_bytes());
                sidecar.extend(p.query_token.to_le_bytes());
            }
        }
        let rows: Vec<Value> = panel
            .iter()
            .map(|p| {
                let i = p.ordinal;
                let admitted = ADMITTED.contains(&i);
                let hit1 = WRITER_HITS.contains(&i);
                let others: Vec<u16> = [1u16, 2, 3, 4, 8, 9, 10, 11]
                    .into_iter()
                    .filter(|&t| t != p.target_token)
                    .take(7)
                    .collect();
                let mut top8 = others.clone();
                top8.insert(if hit1 { 0 } else { 7 }, p.target_token);
                let mut slots = vec![ABSENT_SLOT; CELL_SLOTS];
                slots[0] = 3000;
                if admitted {
                    slots[1] = p.target_token;
                }
                json!({
                    "row_ordinal": i,
                    "target_position": p.position,
                    "query_token": p.query_token,
                    "target_token": p.target_token,
                    "code": code_of(i),
                    "admitted_cell_slots": slots,
                    "top8": top8,
                    "top1_correct": u64::from(hit1),
                    "top8_correct": 1,
                    "admitted_cells": 1 + u64::from(admitted),
                    "address_rivals": 1,
                })
            })
            .collect();
        let token_cells: Vec<u16> = (0..VOCABULARY as u16).collect();
        let train_rows = (stream.len() / 4) as u64;
        let capture = json!({
            "capture_mode": FALLBACK_CAPTURE_MODE,
            "target_identity": { "source_sha256": "src", "kernel_sha256": "ker" },
            "target_binary_sha256": "bin",
            "gate_r": { "pass": true, "mismatch_count": 0 },
            "dev_rows": rows,
            "root_round_trip": { "rows": 2000, "equal": 2000 },
            "determinism": { "rows": 1000, "equal": 1000 },
            "token_cells": token_cells,
            "train_stream": { "rows": train_rows, "sha256": sha256_hex(&stream) },
            "query_sidecar": { "rows": train_rows, "sha256": sha256_hex(&sidecar) },
        });
        Fixture {
            capture,
            stream,
            sidecar,
            tokens: uort(&DEV),
            mask: vec![1; 8],
            pins: FilePins {
                dev_tokens_sha256: TEST_PINS[0].into(),
                dev_mask_sha256: TEST_PINS[1].into(),
                train_tokens_sha256: TEST_PINS[2].into(),
                train_mask_sha256: TEST_PINS[3].into(),
                tokenizer_sha256: TEST_PINS[4].into(),
            },
            binding: ordered_row_binding_sha256(&panel),
        }
    }

    fn run(f: &Fixture) -> Result<(Value, Value), String> {
        let ex = Expected {
            train_rows: f.stream.len() / 4,
            dev_rows: DEV.len() - 1,
            min_root_rows: 1_024,
            min_determinism_rows: 1_000,
            panel_binding: &f.binding,
            writer_top1: WRITER_HITS.len() as u64,
            writer_top8: (DEV.len() - 1) as u64,
            writer_admitted: ADMITTED.len() as u64,
            pins: TEST_PINS,
        };
        assemble_with(
            &ex, &f.capture, &f.stream, &f.sidecar, &f.tokens, &f.mask, &f.pins, true,
        )
    }

    #[test]
    fn w2_record_assembles_with_paired_discordance_identity() {
        let f = fixture();
        let (record, manifest) = run(&f).unwrap();
        let ag = &record["aggregates"];
        let t = ag["T_table_matched_top1"].as_i64().unwrap();
        assert_eq!(t, 6, "the unseen-code row is the table's only miss");
        for (name, key) in [
            ("U", "U_unigram_matched_top1"),
            ("G", "G_bigram_matched_top1"),
            ("W", "W_writer_top1"),
        ] {
            let d = &ag["paired_discordance"][name];
            let (b, c) = (d["b"].as_i64().unwrap(), d["c"].as_i64().unwrap());
            assert_eq!(c - b, t - ag[key].as_i64().unwrap(), "{name}");
        }
        assert_eq!(ag["paired_discordance"]["G"]["b"], 1);
        assert_eq!(ag["paired_discordance"]["W"]["b"], 1);
        assert_eq!(record["schema"], SCHEMA);
        assert_eq!(record["source_sha256"], "src");
        let rows = record["rows"].as_array().unwrap();
        assert_eq!(rows.len(), DEV.len() - 1);
        for r in rows {
            let target = &r["target_token"];
            let top8 = r["table_top8"].as_array().unwrap();
            assert_eq!(top8.len(), TOP_K);
            assert_eq!(top8[0], r["table_top1"]);
            assert_eq!(r["table_hit1"], json!(&r["table_top1"] == target));
            assert_eq!(r["table_hit8"], json!(top8.contains(target)));
            assert_eq!(r["bigram_hit1"], json!(&r["bigram_top1"] == target));
            assert_eq!(
                r["record"].as_u64().unwrap(),
                (1u64 << 62) | r["document_id"].as_u64().unwrap()
            );
        }
        assert_eq!(rows[6]["seen_code"], false);
        assert_eq!(rows[6]["table_hit1"], false);
        let bytes = runtime_manifest_bytes(&manifest).unwrap();
        assert_eq!(
            record["original_to_target_bridge"]["runtime_manifest_sha256"],
            sha256_hex(&bytes)
        );
    }

    #[test]
    fn w2_record_refuses_target_admitted_two() {
        let mut f = fixture();
        f.capture["dev_rows"][0]["admitted_cells"] = json!(3);
        let err = run(&f).unwrap_err();
        assert!(err.contains("target_admitted = 3 - 1 not in"), "{err}");
    }

    #[test]
    fn w2_record_refuses_token_cells_disagreement() {
        let mut f = fixture();
        let target = DEV[1] as usize; // row 0 is admitted
        f.capture["token_cells"][target] = json!(3001);
        let err = run(&f).unwrap_err();
        assert!(err.contains("dev row 0: token_cells"), "{err}");
    }

    #[test]
    fn w2_record_refuses_gate_r_mismatch() {
        let mut f = fixture();
        f.capture["gate_r"]["mismatch_count"] = json!(1);
        let err = run(&f).unwrap_err();
        assert!(err.contains("gate_r.mismatch_count 1 != 0"), "{err}");
    }

    #[test]
    fn w2_record_refuses_top8_token_out_of_vocabulary() {
        let mut f = fixture();
        f.capture["dev_rows"][0]["top8"][3] = json!(VOCABULARY);
        let err = run(&f).unwrap_err();
        assert!(err.contains("dev row 0: top8[3] 4096 >= 4096"), "{err}");
    }

    #[test]
    fn w2_record_refuses_top1_contradicting_top8() {
        let mut f = fixture();
        let hit = u64_at(&f.capture["dev_rows"][0], "top1_correct").unwrap();
        f.capture["dev_rows"][0]["top1_correct"] = json!(1 - hit);
        let err = run(&f).unwrap_err();
        assert!(
            err.contains("dev row 0: top1/top8_correct contradict"),
            "{err}"
        );
    }

    #[test]
    fn w2_record_refuses_top8_correct_contradicting_top8() {
        let mut f = fixture();
        let hit = u64_at(&f.capture["dev_rows"][0], "top8_correct").unwrap();
        f.capture["dev_rows"][0]["top8_correct"] = json!(1 - hit);
        let err = run(&f).unwrap_err();
        assert!(
            err.contains("dev row 0: top1/top8_correct contradict"),
            "{err}"
        );
    }

    #[test]
    fn w2_record_refuses_a_repeated_top8_token() {
        let mut f = fixture();
        let first = f.capture["dev_rows"][0]["top8"][0].clone();
        f.capture["dev_rows"][0]["top8"][1] = first;
        let err = run(&f).unwrap_err();
        assert!(err.contains("dev row 0: top8[1]"), "{err}");
        assert!(err.contains("repeats"), "{err}");
    }

    #[test]
    fn w2_record_refuses_a_changed_file_pin() {
        let mut f = fixture();
        f.pins.train_mask_sha256 = "changed".into();
        let err = run(&f).unwrap_err();
        assert!(err.contains("file pin train_mask_sha256"), "{err}");
    }

    #[test]
    fn w2_record_production_pins_refuse_test_pins() {
        let f = fixture();
        let err = assemble(
            &f.capture, &f.stream, &f.sidecar, &f.tokens, &f.mask, &f.pins, true,
        )
        .unwrap_err();
        assert!(err.contains("file pin dev_tokens_sha256"), "{err}");
    }
}
