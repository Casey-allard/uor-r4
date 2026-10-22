//! `census <runs-dir> <out.json>`: the five controls, then Q1 and Q3, for both
//! A8 dose arms to Q1. Host only. A placement census is not admission.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use a8_census::{cells, changed_slots, join_mismatches, occupancy, slot_token_block, words_sha256};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const G_INIT: &str = "4122cbab44bd88658743984fc1ab79059bf032c3e2fe526ffcf4ea5f95d9cd74";
const INTERVAL: u64 = 64;
const Q1_BATCH: u64 = 4096;
const CLAIMED_INTERVALS: usize = 66;
const ARMS: [(&str, &str, &str); 2] = [
    ("relation", "a8-dose-relation-seed1", "relation"),
    ("frozen_zero", "a8-dose-frozen-zero-seed1", "frozen-zero"),
];

#[derive(Deserialize)]
struct CheckpointFile {
    checkpoint: Inner,
}
#[derive(Deserialize)]
struct Inner {
    model: Model,
    cursor: Cursor,
}
#[derive(Deserialize)]
struct Model {
    table: Vec<i32>,
}
#[derive(Deserialize)]
struct Cursor {
    completed_batches: u64,
}
#[derive(Deserialize)]
struct Step {
    cursor_before: Cursor,
    coordinate: Coordinate,
    applied: bool,
}
#[derive(Deserialize)]
struct Coordinate {
    kind: String,
    index: u64,
}

type Res<T> = Result<T, String>;

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn read(path: &Path) -> Res<Vec<u8>> {
    fs::read(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The run's `FILES.sha256`: `./name` -> digest.
fn manifest(dir: &Path) -> Res<BTreeMap<String, String>> {
    let text = String::from_utf8(read(&dir.join("FILES.sha256"))?).map_err(|e| e.to_string())?;
    Ok(text
        .lines()
        .filter_map(|l| l.split_once("  "))
        .map(|(h, n)| (n.trim_start_matches("./").to_string(), h.to_string()))
        .collect())
}

fn verified(dir: &Path, name: &str, pins: &BTreeMap<String, String>) -> Res<Vec<u8>> {
    let bytes = read(&dir.join(name))?;
    let want = pins
        .get(name)
        .ok_or_else(|| format!("{name} not in FILES.sha256"))?;
    if &sha256_hex(&bytes) != want {
        return Err(format!(
            "{}/{name}: sha256 differs from FILES.sha256",
            dir.display()
        ));
    }
    Ok(bytes)
}

struct Arm {
    name: &'static str,
    batches: Vec<u64>,
    tables: Vec<Vec<i32>>,
    steps: Vec<Step>,
    gpu_cells: Vec<u16>,
    gpu_report: Value,
    files_verified: usize,
    steps_sha256: String,
    gpu_cells_sha256: String,
}

fn load(root: &Path, (name, run, gates): (&'static str, &str, &str)) -> Res<Arm> {
    let dir = root.join(run);
    let pins = manifest(&dir)?;
    let mut batches: Vec<u64> = fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter_map(|n| {
            n.strip_prefix("checkpoint-")?
                .strip_suffix(".json")?
                .parse()
                .ok()
        })
        .filter(|&b| b <= Q1_BATCH)
        .collect();
    batches.sort_unstable();
    let mut tables = Vec::with_capacity(batches.len());
    for &b in &batches {
        let bytes = verified(&dir, &format!("checkpoint-{b:05}.json"), &pins)?;
        let ck: CheckpointFile = serde_json::from_slice(&bytes).map_err(|e| format!("{b}: {e}"))?;
        if ck.checkpoint.cursor.completed_batches != b {
            return Err(format!(
                "{run} checkpoint {b}: cursor says {}",
                ck.checkpoint.cursor.completed_batches
            ));
        }
        tables.push(ck.checkpoint.model.table);
    }
    let steps_bytes = verified(&dir, "steps.jsonl", &pins)?;
    let steps = steps_bytes
        .split(|&c| c == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice::<Step>(l).map_err(|e| e.to_string()))
        .collect::<Res<Vec<_>>>()?;
    let cells_dir = root
        .join("a8-dose-gates-20261019")
        .join(gates)
        .join("cells");
    let raw = read(&cells_dir.join("token_cells.u16"))?;
    let gpu_report: Value = serde_json::from_slice(&read(&cells_dir.join("report.json"))?)
        .map_err(|e| e.to_string())?;
    Ok(Arm {
        name,
        files_verified: batches.len() + 1,
        batches,
        tables,
        steps,
        gpu_cells_sha256: sha256_hex(&raw),
        gpu_cells: raw
            .chunks_exact(2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
            .collect(),
        gpu_report,
        steps_sha256: sha256_hex(&steps_bytes),
    })
}

fn pass(ok: bool) -> &'static str {
    if ok {
        "PASS"
    } else {
        "FAIL"
    }
}

fn main() -> Res<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        return Err("usage: census <runs-dir> <out.json>".into());
    }
    let root = Path::new(&args[1]);
    let arms = ARMS
        .iter()
        .map(|&a| load(root, a))
        .collect::<Res<Vec<_>>>()?;

    // Control 1: byte-equal checkpoint 0.
    let (t0a, t0b) = (&arms[0].tables[0], &arms[1].tables[0]);
    let differing_words = if t0a.len() == t0b.len() {
        t0a.iter().zip(t0b).filter(|(x, y)| x != y).count()
    } else {
        usize::MAX
    };
    let c1 = differing_words == 0;
    // Control 2: G-Init digest.
    let digests: Vec<String> = arms.iter().map(|a| words_sha256(&a.tables[0])).collect();
    let c2 = digests.iter().all(|d| d == G_INIT);
    // Control 3: placement at checkpoint 0 equals the GPU receipt.
    let mut c3 = true;
    let mut c3_arms = serde_json::Map::new();
    for a in &arms {
        let c = cells(&a.tables[0]);
        let occ = occupancy(&c);
        let mismatches = if a.gpu_cells.len() == c.len() {
            c.iter()
                .zip(&a.gpu_cells)
                .filter(|(h, g)| h.map(u16::from) != Some(**g))
                .count()
        } else {
            usize::MAX
        };
        let ok = occ.occupied == 256 && occ.max == 16 && occ.absent == 0 && mismatches == 0;
        c3 &= ok;
        c3_arms.insert(
            a.name.into(),
            json!({
                "result": pass(ok), "tokens": c.len(), "occupied_cells": occ.occupied,
                "max_occupancy": occ.max, "absent": occ.absent,
                "gpu_entries": a.gpu_cells.len(), "host_vs_gpu_mismatches": mismatches,
                "gpu_cells_sha256": a.gpu_cells_sha256,
                "gpu_report_completed_batches": a.gpu_report["completed_batches"],
                "gpu_report_driver_checkpoint_sha256": a.gpu_report["driver_checkpoint_sha256"],
            }),
        );
    }
    // Controls 4 and 5, and Q1/Q3 per arm.
    let gate_open = c1 && c2 && c3;
    let mut c4_arms = serde_json::Map::new();
    let mut c5_arms = serde_json::Map::new();
    let mut q1 = serde_json::Map::new();
    let mut q3 = serde_json::Map::new();
    let mut c4 = true;
    let mut c5 = true;
    for a in &arms {
        let intervals = a.batches.len().saturating_sub(1);
        let contiguous = a
            .batches
            .iter()
            .enumerate()
            .all(|(i, &b)| b == i as u64 * INTERVAL);
        c5 &= contiguous && a.batches.last() == Some(&Q1_BATCH);
        c5_arms.insert(a.name.into(), json!({
            "checkpoints": a.batches.len(), "first": a.batches.first(), "last": a.batches.last(),
            "contiguous_every_64": contiguous, "intervals_to_q1": intervals,
            "claim_66_holds": intervals == CLAIMED_INTERVALS,
            "files_verified_against_FILES_sha256": a.files_verified, "steps_sha256": a.steps_sha256,
            "receipts_total": a.steps.len(),
            "receipts_before_q1": a.steps.iter().filter(|s| s.cursor_before.completed_batches < Q1_BATCH).count(),
        }));
        let all_cells: Vec<Vec<Option<u8>>> = a.tables.iter().map(|t| cells(t)).collect();
        let mut rows4 = Vec::new();
        let mut rows3 = Vec::new();
        let mut total_mismatch = 0usize;
        let mut tot = [0u64; 4];
        let (mut tot_cc, mut tot_gate, mut tot_route, mut tot_refused_gate) =
            (0u64, 0u64, 0u64, 0u64);
        let mut cc_tokens = BTreeSet::new();
        for i in 0..intervals {
            let (lo, hi) = (a.batches[i], a.batches[i + 1]);
            let in_iv: Vec<&Step> = a
                .steps
                .iter()
                .filter(|s| (lo..hi).contains(&s.cursor_before.completed_batches))
                .collect();
            let vec_writes: Vec<u32> = in_iv
                .iter()
                .filter(|s| s.applied && s.coordinate.kind == "vector_block")
                .map(|s| s.coordinate.index as u32)
                .collect();
            let accepted: BTreeSet<u32> = vec_writes.iter().copied().collect();
            let changed = changed_slots(&a.tables[i], &a.tables[i + 1]);
            let mism = join_mismatches(&changed, &accepted);
            total_mismatch += mism;
            let gate = in_iv
                .iter()
                .filter(|s| s.applied && s.coordinate.kind == "gate")
                .count() as u64;
            let route = in_iv
                .iter()
                .filter(|s| s.applied && s.coordinate.kind == "route")
                .count() as u64;
            rows4.push(json!({
                "interval": [lo, hi], "changed_slots": changed.len(),
                "accepted_vector_slots": accepted.len(), "accepted_vector_writes": vec_writes.len(),
                "applied_gate_writes": gate, "applied_route_writes": route, "mismatches": mism,
            }));
            // Q3: a write changes its token's cell when the token's cell at the
            // interval end differs from its cell at the interval start.
            let mut by_block = [0u64; 4];
            let mut cc = 0u64;
            let mut cc_tok_iv = BTreeSet::new();
            for &slot in &vec_writes {
                let (t, b) = slot_token_block(slot);
                by_block[b as usize] += 1;
                if all_cells[i][t as usize] != all_cells[i + 1][t as usize] {
                    cc += 1;
                    cc_tok_iv.insert(t);
                }
            }
            let repeated = vec_writes.len() - accepted.len();
            let refused = in_iv
                .iter()
                .filter(|s| s.coordinate.kind == "gate" && !s.applied)
                .count() as u64;
            for b in 0..4 {
                tot[b] += by_block[b];
            }
            tot_cc += cc;
            tot_gate += gate;
            tot_route += route;
            tot_refused_gate += refused;
            cc_tokens.extend(cc_tok_iv.iter().copied());
            rows3.push(json!({
                "interval": [lo, hi], "vector_writes_by_block": by_block,
                "cell_changing_writes": cc, "cell_changing_tokens": cc_tok_iv.len(),
                "slots_written_more_than_once_extra_writes": repeated,
                "applied_gate_writes": gate, "applied_route_writes": route,
            }));
        }
        c4 &= total_mismatch == 0;
        c4_arms.insert(a.name.into(), json!({
            "result": pass(total_mismatch == 0), "total_mismatches": total_mismatch, "intervals": rows4,
        }));
        if gate_open {
            let rows1: Vec<Value> = all_cells.iter().enumerate().map(|(k, c)| {
                let occ = occupancy(c);
                let diff = |o: &Vec<Option<u8>>| c.iter().zip(o).filter(|(x, y)| x != y).count();
                json!({
                    "batch": a.batches[k], "occupied_cells": occ.occupied,
                    "occupancy_histogram": occ.histogram.iter().map(|(n, m)| (n.to_string(), *m)).collect::<BTreeMap<_, _>>(),
                    "max_occupancy": occ.max, "absent": occ.absent,
                    "tokens_moved_vs_checkpoint0": diff(&all_cells[0]),
                    "tokens_moved_vs_previous": if k == 0 { 0 } else { diff(&all_cells[k - 1]) },
                })
            }).collect();
            q1.insert(a.name.into(), Value::Array(rows1));
            q3.insert(
                a.name.into(),
                json!({
                    "intervals": rows3,
                    "totals_to_q1": {
                        "vector_writes_by_block": tot, "vector_writes": tot.iter().sum::<u64>(),
                        "cell_changing_writes": tot_cc, "cell_changing_tokens": cc_tokens.len(),
                        "applied_gate_writes": tot_gate, "applied_route_writes": tot_route,
                        "unapplied_gate_receipts": tot_refused_gate,
                    },
                }),
            );
        }
    }
    let out = json!({
        "schema": "uor-r4.a8-placement-census/1",
        "scope": "A placement census is not admission: no reach or admission value is recomputed here.",
        "controls": {
            "c1_byte_equal_checkpoint0": {"result": pass(c1), "differing_words": differing_words, "words": t0a.len()},
            "c2_g_init_digest": {"result": pass(c2), "expected": G_INIT,
                "relation": digests[0], "frozen_zero": digests[1]},
            "c3_checkpoint0_placement": {"result": pass(c3), "arms": c3_arms},
            "c4_receipt_join": {"result": pass(c4), "arms": c4_arms},
            "c5_interval_count": {"result": pass(c5), "claimed": CLAIMED_INTERVALS, "arms": c5_arms},
        },
        "stopped_before_q1_q3": !gate_open,
        "q1": if gate_open { Value::Object(q1) } else { Value::Null },
        "q3": if gate_open { Value::Object(q3) } else { Value::Null },
    });
    let text = serde_json::to_string_pretty(&out).map_err(|e| e.to_string())? + "\n";
    fs::write(&args[2], &text).map_err(|e| e.to_string())?;
    println!(
        "controls: c1 {} c2 {} c3 {} c4 {} c5 {}; wrote {} sha256 {}",
        pass(c1),
        pass(c2),
        pass(c3),
        pass(c4),
        pass(c5),
        args[2],
        sha256_hex(text.as_bytes())
    );
    Ok(())
}
