//! GPU qualification and `m2-fresh256` forced-copy oracle evaluator for
//! `LawfulPointerHead` (`r-lawful-pointer-forced-copy-oracle`, #2880).
//!
//! `controls --out DIR [--device N]` runs the finite CUDA controls across all
//! three `RouteAdmissionMode` variants (`SlotAddressSemiprime`, `SharedAtomGcd`,
//! `OrderedNlet { width: 2 }`) and all four oracle conditions (`N`, `O1`, `O2`, `W`),
//! verifying 100% bit-for-bit device-vs-host parity against `LawfulPointerHead::select_step`.
//!
//! `qualify-m2-fresh256 --repo-root R --panel-dir P --out DIR [--tokenizer T]
//! [--expect-ptx SHA] [--device N]` verifies the sealed `m2-fresh256` panel files
//! and pinned `ByteBpeTokenizer`, verifies `256/256` target and `256/256` distractor
//! span coverage on both raw multi-turn dialogue windows and `inputs.json` packet
//! windows, executes `lawful_pointer_select_batch` on the GPU twice across all four
//! conditions (`N`, `O1`, `O2`, `W`), verifies 100% GPU-vs-CPU parity and two-launch
//! determinism, grades all four conditions through `FrozenAnswers` + `ByteBpeTokenizer`
//! + `pair_id`, and writes `record.json` and `decisions.json`.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::lawful_pointer::device::{ptx_sha256, PointerDevice, PointerStepItem};
use uor_r4_training::lawful_pointer::{
    evaluate_forced_copy_oracle_batch, locate_last_value_span, CanonicalFoldTable, CopySpanOracle,
    LawfulPointerHead, OracleRowInput, PointerDecision, RouteAdmissionMode, SlotAddressKey,
    SourceSpanCandidate,
};
use uor_r4_training::sha256_bytes;

type Result<T> = std::result::Result<T, String>;

const EXPECTED_PLAN_SHA256: &str =
    "0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715";
const EXPECTED_INPUTS_SHA256: &str =
    "1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a";
const EXPECTED_LABELS_SHA256: &str =
    "3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be";
const EXPECTED_MANIFEST_SHA256: &str =
    "7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964";
const PINNED_TOKENIZER_SHA256: &str =
    "d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89";
const EOS_TOKEN_ID: u32 = 1;
const PERIOD_TOKEN_ID: u32 = 16;

#[derive(Clone, Debug, Deserialize)]
struct PlanFile {
    histories: Vec<PlanHistory>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlanHistory {
    id: String,
    stratum: String,
    turns: Vec<PlanWire>,
    queries: Vec<PlanWire>,
}

#[derive(Clone, Debug, Deserialize)]
struct PlanWire {
    text: String,
    relation: String,
    template: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeInputs {
    schema: String,
    cases: Vec<ProbePacket>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbePacket {
    id: String,
    segments: Vec<ProbeSegment>,
    query_ids: Vec<u32>,
    #[allow(dead_code)]
    actual_prefix_ids: Vec<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum ProbeSegment {
    Source {
        #[allow(dead_code)]
        event: u64,
        #[allow(dead_code)]
        record: u64,
        #[allow(dead_code)]
        commit: u64,
        #[allow(dead_code)]
        scope: String,
        #[allow(dead_code)]
        entity: Vec<u32>,
        relation: u32,
        #[allow(dead_code)]
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        #[allow(dead_code)]
        event: u64,
        #[allow(dead_code)]
        role: u32,
        token_ids: Vec<u32>,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeLabels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<ProbeLabel>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProbeLabel {
    id: String,
    pair_id: String,
    answers: FrozenAnswers,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ConditionDecisionRecord {
    condition: String,
    row_id: String,
    step_offset: usize,
    emitted_token: u32,
    base_top_token: u32,
    base_top_margin: i64,
    copied: bool,
    selected_span: Option<(usize, usize)>,
    candidate_copy_token: Option<u32>,
    route_score: Option<i64>,
    effective_gate_margin: i64,
}

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("lawful-pointer-oracle: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<()> {
    let (mode, rest) = args
        .split_first()
        .ok_or("usage: lawful-pointer-oracle controls|qualify-m2-fresh256 ...")?;
    let mut flags = BTreeMap::new();
    for pair in rest.chunks(2) {
        match pair {
            [key, value] if key.starts_with("--") => {
                flags.insert(key.trim_start_matches("--").to_string(), value.clone());
            }
            _ => return Err(format!("expected --flag value pairs, got {pair:?}")),
        }
    }
    let get = |key: &str| flags.get(key).cloned();
    let number = |key: &str, default: u64| -> Result<u64> {
        get(key).map_or(Ok(default), |v| {
            v.parse().map_err(|e| format!("--{key}: {e}"))
        })
    };
    let out = PathBuf::from(get("out").ok_or("--out is required")?);
    fs::create_dir_all(&out).map_err(|e| format!("{}: {e}", out.display()))?;
    let device = PointerDevice::new(number("device", 0)? as usize).map_err(|e| e.to_string())?;

    let (name, record) = match mode.as_str() {
        "controls" => ("controls.json", controls(&device)?),
        "qualify-m2-fresh256" => {
            let repo_root = PathBuf::from(get("repo-root").ok_or("--repo-root is required")?);
            let panel_dir = PathBuf::from(get("panel-dir").ok_or("--panel-dir is required")?);
            let tok_override = get("tokenizer").map(PathBuf::from);
            let expect_ptx = get("expect-ptx");
            let (record, decisions) = qualify_m2_fresh256(
                &device,
                &repo_root,
                &panel_dir,
                tok_override.as_deref(),
                expect_ptx.as_deref(),
            )?;
            fs::write(out.join("decisions.json"), decisions)
                .map_err(|e| format!("decisions.json: {e}"))?;
            ("record.json", record)
        }
        other => return Err(format!("unknown mode {other:?}")),
    };

    let bytes = serde_json::to_vec_pretty(&record).map_err(|e| e.to_string())?;
    fs::write(out.join(name), &bytes).map_err(|e| format!("{name}: {e}"))?;
    println!("{}", String::from_utf8_lossy(&bytes));
    Ok(())
}

fn sample_pieces() -> Vec<Vec<u8>> {
    vec![
        b"<pad>".to_vec(),   // 0
        b"Ana".to_vec(),     // 1
        b" ana".to_vec(),    // 2
        b"Bob".to_vec(),     // 3
        b"locker".to_vec(),  // 4
        b" locker".to_vec(), // 5
        b"pin".to_vec(),     // 6
        b" is".to_vec(),     // 7
        b" 4".to_vec(),      // 8
        b"7".to_vec(),       // 9
        b" 9".to_vec(),      // 10
        b"2".to_vec(),       // 11
        b".".to_vec(),       // 12
    ]
}

fn check_host_parity(
    head: &LawfulPointerHead,
    items: &[PointerStepItem],
    gpu_decisions: &[PointerDecision],
) -> Result<usize> {
    if items.len() != gpu_decisions.len() {
        return Err(format!(
            "GPU decision count {} != item count {}",
            gpu_decisions.len(),
            items.len()
        ));
    }
    let mut h = head.clone();
    for (idx, (item, gpu_dec)) in items.iter().zip(gpu_decisions).enumerate() {
        h.set_copy_span_oracle(item.oracle, item.window_tokens.len())
            .map_err(|e| format!("item {idx} set_copy_span_oracle: {e}"))?;
        let cpu_dec = h
            .select_step(
                &item.window_tokens,
                &item.base_margins,
                item.learned_gate_margin,
                item.step_offset,
                item.query_slot,
                item.query_product,
                &item.query_preceding,
                &item.candidates,
            )
            .map_err(|e| format!("item {idx} cpu select_step: {e}"))?;
        if &cpu_dec != gpu_dec {
            return Err(format!(
                "GPU vs CPU decision mismatch at item {idx}: gpu={gpu_dec:?} vs cpu={cpu_dec:?}"
            ));
        }
    }
    Ok(items.len())
}

fn controls(device: &PointerDevice) -> Result<Value> {
    let pieces = sample_pieces();
    let fold = CanonicalFoldTable::from_pieces(&pieces).map_err(|e| e.to_string())?;
    let window = vec![1, 5, 7, 8, 9, 12, 3, 5, 7, 10, 11, 12];
    let ana_locker =
        SlotAddressKey::from_entity_attribute_primes(5, 11).map_err(|e| e.to_string())?;
    let bob_locker =
        SlotAddressKey::from_entity_attribute_primes(7, 11).map_err(|e| e.to_string())?;
    let bob_pin = SlotAddressKey::from_entity_attribute_primes(7, 13).map_err(|e| e.to_string())?;

    let mut base_low = vec![0i64; 13];
    base_low[7] = 200;
    let mut base_high = vec![0i64; 13];
    base_high[7] = 1500;

    let candidates_semiprime = vec![
        SourceSpanCandidate {
            span: (3, 5),
            slot_product: 55,
            preceding_tokens: vec![1, 5, 7],
            learned_rank_bonus: 74,
        },
        SourceSpanCandidate {
            span: (9, 11),
            slot_product: 77,
            preceding_tokens: vec![3, 5, 7],
            learned_rank_bonus: 500,
        },
    ];

    // 1. SlotAddressSemiprime controls (10 items):
    //    - step 0 and step 1 admitted copy for Ana's locker (55)
    //    - gate closed (40 < 100) falls back to base_top_token (7)
    //    - unadmitted query (Bob's pin = 91) falls back to base_top_token (7)
    //    - base-margin block in N (base 1500 >= copy 1474)
    //    - base-margin block in O1 (base 1500 >= copy 1474)
    //    - O1 blocked by learned gate (25 < 100) while selecting span (9, 11)
    //    - O2 forced gate (250 >= 100) emitting Bob's locker token 10 (" 9") and token 11 ("2") (2 steps)
    //    - W wrong-span distractor control under O2ForcedGate
    let head_semi =
        LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold.clone(), 100);
    let semi_items = vec![
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 150,
            step_offset: 0,
            query_slot: Some(ana_locker),
            query_product: 55,
            query_preceding: vec![],
            candidates: candidates_semiprime.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 150,
            step_offset: 1,
            query_slot: Some(ana_locker),
            query_product: 55,
            query_preceding: vec![],
            candidates: candidates_semiprime.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 40,
            step_offset: 0,
            query_slot: Some(ana_locker),
            query_product: 55,
            query_preceding: vec![],
            candidates: candidates_semiprime.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 500,
            step_offset: 0,
            query_slot: Some(bob_pin),
            query_product: 91,
            query_preceding: vec![],
            candidates: candidates_semiprime.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_high.clone(),
            learned_gate_margin: 120,
            step_offset: 0,
            query_slot: Some(ana_locker),
            query_product: 55,
            query_preceding: vec![],
            candidates: vec![candidates_semiprime[0].clone()],
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_high,
            learned_gate_margin: 120,
            step_offset: 0,
            query_slot: Some(ana_locker),
            query_product: 55,
            query_preceding: vec![],
            candidates: vec![candidates_semiprime[0].clone()],
            oracle: CopySpanOracle::O1SpanLearnedGate { start: 3, end: 5 },
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 25,
            step_offset: 0,
            query_slot: Some(bob_locker),
            query_product: 77,
            query_preceding: vec![],
            candidates: vec![],
            oracle: CopySpanOracle::O1SpanLearnedGate { start: 9, end: 11 },
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 25,
            step_offset: 0,
            query_slot: Some(bob_locker),
            query_product: 77,
            query_preceding: vec![],
            candidates: vec![],
            oracle: CopySpanOracle::O2SpanForcedGate {
                start: 9,
                end: 11,
                forced_gate_margin: 250,
            },
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 25,
            step_offset: 1,
            query_slot: Some(bob_locker),
            query_product: 77,
            query_preceding: vec![],
            candidates: vec![],
            oracle: CopySpanOracle::O2SpanForcedGate {
                start: 9,
                end: 11,
                forced_gate_margin: 250,
            },
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 25,
            step_offset: 0,
            query_slot: Some(bob_locker),
            query_product: 77,
            query_preceding: vec![],
            candidates: vec![],
            oracle: CopySpanOracle::O2SpanForcedGate {
                start: 3,
                end: 5,
                forced_gate_margin: 250,
            },
        },
    ];
    let (semi_gpu, semi_us) = device
        .select_batch(&head_semi, &semi_items)
        .map_err(|e| e.to_string())?;
    let semi_checked = check_host_parity(&head_semi, &semi_items, &semi_gpu)?;

    // 2. SharedAtomGcd controls (3 items):
    //    - rare prime 13 beats common prime 3
    //    - equal route_score tie-break selects latest span.0 ((9, 11) over (3, 5))
    //    - 128-bit high-bit prime (> 2^64) exercises full u128 binary GCD on device
    let head_gcd = LawfulPointerHead::new(RouteAdmissionMode::SharedAtomGcd, fold.clone(), 100);
    let high_prime: u128 = (1u128 << 70) + 9;
    let gcd_items = vec![
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: vec![0i64; 13],
            learned_gate_margin: 150,
            step_offset: 0,
            query_slot: None,
            query_product: 3u128 * 13u128,
            query_preceding: vec![],
            candidates: vec![
                SourceSpanCandidate {
                    span: (3, 5),
                    slot_product: 3,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
                SourceSpanCandidate {
                    span: (9, 11),
                    slot_product: 13,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
            ],
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window.clone(),
            base_margins: vec![0i64; 13],
            learned_gate_margin: 150,
            step_offset: 0,
            query_slot: None,
            query_product: 3u128 * 13u128,
            query_preceding: vec![],
            candidates: vec![
                SourceSpanCandidate {
                    span: (3, 5),
                    slot_product: 13,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
                SourceSpanCandidate {
                    span: (9, 11),
                    slot_product: 13,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
            ],
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: window,
            base_margins: vec![0i64; 13],
            learned_gate_margin: 150,
            step_offset: 0,
            query_slot: None,
            query_product: high_prime * 5u128,
            query_preceding: vec![],
            candidates: vec![
                SourceSpanCandidate {
                    span: (3, 5),
                    slot_product: 5,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
                SourceSpanCandidate {
                    span: (9, 11),
                    slot_product: high_prime * 7u128,
                    preceding_tokens: vec![],
                    learned_rank_bonus: 0,
                },
            ],
            oracle: CopySpanOracle::Off,
        },
    ];
    let (gcd_gpu, gcd_us) = device
        .select_batch(&head_gcd, &gcd_items)
        .map_err(|e| e.to_string())?;
    let gcd_checked = check_host_parity(&head_gcd, &gcd_items, &gcd_gpu)?;

    // 3. OrderedNlet { width: 2 } controls (3 items):
    //    - canonical fold matches (" ana", " locker") against ("Ana", "locker") and emits raw token 10
    //    - mismatched n-let is rejected and falls back to base_top_token (7)
    //    - O2 overrides mismatched n-let and emits raw token 10
    let head_nlet = LawfulPointerHead::new(RouteAdmissionMode::OrderedNlet { width: 2 }, fold, 100);
    let nlet_window = vec![2, 5, 7, 10, 12];
    let nlet_cand = vec![SourceSpanCandidate {
        span: (3, 4),
        slot_product: 1,
        preceding_tokens: vec![2, 5],
        learned_rank_bonus: 50,
    }];
    let nlet_items = vec![
        PointerStepItem {
            window_tokens: nlet_window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 120,
            step_offset: 0,
            query_slot: None,
            query_product: 1,
            query_preceding: vec![1, 4],
            candidates: nlet_cand.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: nlet_window.clone(),
            base_margins: base_low.clone(),
            learned_gate_margin: 120,
            step_offset: 0,
            query_slot: None,
            query_product: 1,
            query_preceding: vec![3, 4],
            candidates: nlet_cand.clone(),
            oracle: CopySpanOracle::Off,
        },
        PointerStepItem {
            window_tokens: nlet_window,
            base_margins: base_low,
            learned_gate_margin: 20,
            step_offset: 0,
            query_slot: None,
            query_product: 1,
            query_preceding: vec![3, 4],
            candidates: nlet_cand,
            oracle: CopySpanOracle::O2SpanForcedGate {
                start: 3,
                end: 4,
                forced_gate_margin: 250,
            },
        },
    ];
    let (nlet_gpu, nlet_us) = device
        .select_batch(&head_nlet, &nlet_items)
        .map_err(|e| e.to_string())?;
    let nlet_checked = check_host_parity(&head_nlet, &nlet_items, &nlet_gpu)?;

    let total_checked = semi_checked + gcd_checked + nlet_checked;
    Ok(json!({
        "schema": "uor-r4.lawful_pointer_cuda_controls.v1",
        "status": "PASS",
        "ptx_sha256": ptx_sha256(),
        "total_controls": total_checked,
        "passed_controls": total_checked,
        "gpu_vs_cpu_mismatches": 0,
        "modes": {
            "slot_address_semiprime": {
                "checked": semi_checked,
                "elapsed_microseconds": semi_us,
            },
            "shared_atom_gcd": {
                "checked": gcd_checked,
                "elapsed_microseconds": gcd_us,
            },
            "ordered_nlet_width_2": {
                "checked": nlet_checked,
                "elapsed_microseconds": nlet_us,
            }
        }
    }))
}

fn extract_literal(w: &PlanWire) -> Result<&str> {
    let template = w.template.as_deref().ok_or("write template absent")?;
    let (prefix, suffix) = template.split_once("{v}").ok_or("write slot absent")?;
    w.text
        .strip_prefix(prefix)
        .and_then(|s| s.strip_suffix(suffix))
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "write text/template mismatch".to_string())
}

fn load_pinned_tokenizer(
    repo_root: &Path,
    override_path: Option<&Path>,
) -> Result<(ByteBpeTokenizer, String)> {
    let candidates = [
        override_path.map(Path::to_path_buf),
        Some(repo_root.join(".uor-models/research/issue-1019/tokenizer/tokenizer.json")),
        Some(repo_root.join("../../.uor-models/research/issue-1019/tokenizer/tokenizer.json")),
        Some(
            repo_root
                .join(".uor-models/research/issue-973-group-retention/tokenizer/tokenizer.json"),
        ),
        Some(
            repo_root.join(
                "../../.uor-models/research/issue-973-group-retention/tokenizer/tokenizer.json",
            ),
        ),
    ];
    let tok_path = candidates
        .into_iter()
        .flatten()
        .find(|p| p.is_file())
        .ok_or_else(|| {
            format!(
                "pinned tokenizer.json ({PINNED_TOKENIZER_SHA256}) not found under {}",
                repo_root.display()
            )
        })?;
    let bytes = fs::read(&tok_path).map_err(|e| format!("{}: {e}", tok_path.display()))?;
    let actual_sha = sha256_bytes(&bytes);
    if actual_sha != PINNED_TOKENIZER_SHA256 {
        return Err(format!(
            "tokenizer.json SHA-256 mismatch: expected {PINNED_TOKENIZER_SHA256}, got {actual_sha}"
        ));
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| "ByteBpeTokenizer::from_tokenizer_json_bytes failed".to_string())?;
    Ok((tok, actual_sha))
}

fn grade_with_evaluator_oracle(
    tok: &ByteBpeTokenizer,
    inputs: &ProbeInputs,
    labels: &ProbeLabels,
    generated_by_case: &[Vec<u32>],
) -> Result<(usize, usize)> {
    if inputs.cases.len() != labels.cases.len() || inputs.cases.len() != generated_by_case.len() {
        return Err("case count mismatch in evaluator oracle grading".to_string());
    }
    let mut seen = BTreeSet::new();
    let mut complete = 0usize;
    let mut pair_groups: BTreeMap<String, Vec<(String, bool, Vec<u32>)>> = BTreeMap::new();
    for ((p, l), generated_ids) in inputs
        .cases
        .iter()
        .zip(&labels.cases)
        .zip(generated_by_case)
    {
        if p.id.is_empty() || p.id != l.id || !seen.insert(p.id.clone()) {
            return Err("input/label IDs mismatched or duplicate".to_string());
        }
        l.answers.validate().map_err(|e| e.to_string())?;
        let eos = generated_ids.last().copied() == Some(EOS_TOKEN_ID);
        let output_ids = if eos {
            &generated_ids[..generated_ids.len() - 1]
        } else {
            &generated_ids[..]
        };
        let decoded_bytes = tok.decode_bytes(output_ids);
        let valid_text = std::str::from_utf8(&decoded_bytes).ok();
        let accepted = eos && valid_text.is_some_and(|text| l.answers.accepts(text));
        complete += usize::from(accepted);
        pair_groups.entry(l.pair_id.clone()).or_default().push((
            p.id.clone(),
            accepted,
            generated_ids.clone(),
        ));
    }
    let mut pairs_both_complete_distinct = 0usize;
    for arms in pair_groups.values() {
        if arms.len() != 2 {
            return Err("each evaluator pair must have two cases".to_string());
        }
        let both = arms.iter().all(|v| v.1);
        let distinct = arms[0].2 != arms[1].2;
        pairs_both_complete_distinct += usize::from(both && distinct);
    }
    Ok((complete, pairs_both_complete_distinct))
}

fn qualify_m2_fresh256(
    device: &PointerDevice,
    repo_root: &Path,
    panel_dir: &Path,
    tok_override: Option<&Path>,
    expect_ptx: Option<&str>,
) -> Result<(Value, Vec<u8>)> {
    let actual_ptx = ptx_sha256();
    if let Some(expected) = expect_ptx {
        if actual_ptx != expected {
            return Err(format!(
                "PTX SHA-256 mismatch: expected {expected}, got {actual_ptx}"
            ));
        }
    }

    let read_verified = |name: &str, expected_sha: &str| -> Result<Vec<u8>> {
        let path = panel_dir.join(name);
        let bytes = fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let actual = sha256_bytes(&bytes);
        if actual != expected_sha {
            return Err(format!(
                "{name} SHA-256 mismatch: expected {expected_sha}, got {actual}"
            ));
        }
        Ok(bytes)
    };

    let plan_bytes = read_verified("plan.json", EXPECTED_PLAN_SHA256)?;
    let inputs_bytes = read_verified("inputs.json", EXPECTED_INPUTS_SHA256)?;
    let labels_bytes = read_verified("labels.json", EXPECTED_LABELS_SHA256)?;
    let _manifest_bytes = read_verified("manifest.json", EXPECTED_MANIFEST_SHA256)?;

    let plan: PlanFile =
        serde_json::from_slice(&plan_bytes).map_err(|e| format!("plan.json: {e}"))?;
    let inputs: ProbeInputs =
        serde_json::from_slice(&inputs_bytes).map_err(|e| format!("inputs.json: {e}"))?;
    let labels: ProbeLabels =
        serde_json::from_slice(&labels_bytes).map_err(|e| format!("labels.json: {e}"))?;

    if plan.histories.len() != 128
        || inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || inputs.cases.len() != 256
        || labels.cases.len() != 256
    {
        return Err("m2-fresh256 header or case count mismatch".to_string());
    }

    let (tok, tok_sha) = load_pinned_tokenizer(repo_root, tok_override)?;
    let pieces: Vec<Vec<u8>> = (0..tok.vocab_size() as u32)
        .map(|id| tok.decode_bytes(&[id]))
        .collect();
    let fold = CanonicalFoldTable::from_pieces(&pieces).map_err(|e| e.to_string())?;
    let head = LawfulPointerHead::new(RouteAdmissionMode::SlotAddressSemiprime, fold.clone(), 100);

    let user_job =
        SlotAddressKey::from_entity_attribute_primes(5, 11).map_err(|e| e.to_string())?;
    let user_home =
        SlotAddressKey::from_entity_attribute_primes(5, 13).map_err(|e| e.to_string())?;

    let mut dialogue_target_spans_found = 0usize;
    let mut dialogue_distractor_spans_found = 0usize;
    let mut dialogue_canonical_fold_matches = 0usize;
    let mut update_rows_selecting_latest = 0usize;
    let mut reassert_rows_selecting_latest = 0usize;

    let mut oracle_rows = Vec::with_capacity(256);

    for (h_idx, h) in plan.histories.iter().enumerate() {
        let block_index = h_idx / 8;
        let stratum = h
            .stratum
            .split("/q")
            .next()
            .ok_or_else(|| "stratum absent".to_string())?;

        let mut current_lit: BTreeMap<&str, String> = BTreeMap::new();
        let mut turn_token_ranges = Vec::new();
        let mut dialogue_turns_tokens = Vec::new();
        for w in &h.turns {
            let start = dialogue_turns_tokens.len();
            let t_ids = tok.encode(&w.text);
            dialogue_turns_tokens.extend_from_slice(&t_ids);
            turn_token_ranges.push((w.relation.as_str(), start, dialogue_turns_tokens.len()));
            current_lit.insert(w.relation.as_str(), extract_literal(w)?.to_string());
        }

        let mut sorted_queries: Vec<&PlanWire> = h.queries.iter().collect();
        sorted_queries.sort_by_key(|q| if q.relation == "job" { 0usize } else { 1usize });

        for (q_pos, q) in sorted_queries.into_iter().enumerate() {
            let row_idx = h_idx * 2 + q_pos;
            let packet = &inputs.cases[row_idx];
            let label = &labels.cases[row_idx];
            let expected_id = format!("fresh-{}-{}", h.id, q.relation);
            if packet.id != expected_id || label.id != expected_id {
                return Err(format!(
                    "row {row_idx} id mismatch: expected {expected_id}, got {}",
                    packet.id
                ));
            }

            let expected_lit = current_lit
                .get(q.relation.as_str())
                .ok_or("missing expected role literal")?;
            let other_role = if q.relation == "job" { "home" } else { "job" };
            let distractor_lit = current_lit
                .get(other_role)
                .ok_or("missing distractor role literal")?;

            // 1. Check `locate_last_value_span` on raw multi-turn dialogue window (`h.turns` + `q.text`).
            let mut raw_dialogue_window = dialogue_turns_tokens.clone();
            raw_dialogue_window.extend_from_slice(&packet.query_ids);
            let raw_t_span =
                locate_last_value_span(&raw_dialogue_window, &pieces, expected_lit.as_bytes())
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| {
                        format!("missing target span in raw dialogue window at {expected_id}")
                    })?;
            let _raw_d_span =
                locate_last_value_span(&raw_dialogue_window, &pieces, distractor_lit.as_bytes())
                    .map_err(|e| e.to_string())?
                    .ok_or_else(|| {
                        format!("missing distractor span in raw dialogue window at {expected_id}")
                    })?;
            dialogue_target_spans_found += 1;
            dialogue_distractor_spans_found += 1;

            let unprefixed_target_ids = tok.encode(expected_lit);
            let raw_span_ids = &raw_dialogue_window[raw_t_span.0..raw_t_span.1];
            if raw_span_ids.len() == unprefixed_target_ids.len()
                && raw_span_ids
                    .iter()
                    .zip(&unprefixed_target_ids)
                    .all(|(&a, &b)| fold.fold(a).ok() == fold.fold(b).ok())
            {
                dialogue_canonical_fold_matches += 1;
            }

            let last_role_turn = turn_token_ranges
                .iter()
                .rev()
                .find(|&&(rel, _, _)| rel == q.relation.as_str())
                .ok_or("missing role turn range")?;
            if raw_t_span.0 >= last_role_turn.1 && raw_t_span.1 <= last_role_turn.2 {
                if stratum == "update" {
                    update_rows_selecting_latest += 1;
                } else if stratum == "reassert" {
                    reassert_rows_selecting_latest += 1;
                }
            }

            // 2. Build the `inputs.json` packet window (`Context` + `Source` segments + `query_ids`).
            let mut packet_window = Vec::new();
            let mut candidates = Vec::new();
            for seg in &packet.segments {
                match seg {
                    ProbeSegment::Context { token_ids, .. } => {
                        packet_window.extend_from_slice(token_ids);
                    }
                    ProbeSegment::Source {
                        relation,
                        original_source_ids,
                        ..
                    } => {
                        let start = packet_window.len();
                        packet_window.extend_from_slice(original_source_ids);
                        let end = packet_window.len();
                        packet_window.push(PERIOD_TOKEN_ID);
                        let slot_prod = if *relation == 1 {
                            user_job.semiprime()
                        } else {
                            user_home.semiprime()
                        };
                        candidates.push(SourceSpanCandidate {
                            span: (start, end),
                            slot_product: slot_prod,
                            preceding_tokens: vec![],
                            learned_rank_bonus: 0,
                        });
                    }
                }
            }
            packet_window.extend_from_slice(&packet.query_ids);

            let accepted_str = &label.answers.accepted[0];
            let accepted_tokens = tok.encode(accepted_str);
            // The value literal occupies `0..accepted_tokens.len() - 1`, followed by PERIOD_TOKEN_ID (16).
            if accepted_tokens.last().copied() != Some(PERIOD_TOKEN_ID) {
                return Err(format!(
                    "row {expected_id} accepted answer does not end with PERIOD_TOKEN_ID"
                ));
            }
            let value_step_count = accepted_tokens.len() - 1;
            let distractor_tokens = tok.encode(distractor_lit);
            let step_count = value_step_count.max(distractor_tokens.len());

            // Base scorer alone (gate closed: learned_gate_margin = 25 < threshold 100) emits
            // PERIOD_TOKEN_ID (16), representing a memoryless base scorer before Stage T / M2 trunk training.
            let mut base_margins = vec![0i64; tok.vocab_size()];
            base_margins[PERIOD_TOKEN_ID as usize] = 300;
            let step_base_margins = vec![base_margins; step_count];
            let step_learned_gate_margins = vec![25i64; step_count];
            let recorded_baseline_tokens = vec![PERIOD_TOKEN_ID; step_count];

            let (query_slot, query_product) = if q.relation == "job" {
                (Some(user_job), user_job.semiprime())
            } else {
                (Some(user_home), user_home.semiprime())
            };

            let mut accepted_step_tokens = unprefixed_target_ids;
            accepted_step_tokens.resize(step_count, PERIOD_TOKEN_ID);

            oracle_rows.push(OracleRowInput {
                row_id: packet.id.clone(),
                pair_id: label.pair_id.clone(),
                block_index,
                window_tokens: packet_window,
                query_slot,
                query_product,
                query_preceding: vec![],
                candidates,
                expected_value_bytes: expected_lit.as_bytes().to_vec(),
                distractor_value_bytes: distractor_lit.as_bytes().to_vec(),
                accepted_reply_tokens: vec![accepted_step_tokens],
                step_base_margins,
                step_learned_gate_margins,
                recorded_baseline_tokens,
                forced_gate_margin: 500,
            });
        }
    }

    if dialogue_target_spans_found != 256
        || dialogue_distractor_spans_found != 256
        || update_rows_selecting_latest != 32
        || reassert_rows_selecting_latest != 32
    {
        return Err(format!(
            "raw dialogue span coverage failed: target={dialogue_target_spans_found}/256 distractor={dialogue_distractor_spans_found}/256 fold={dialogue_canonical_fold_matches}/256 update={update_rows_selecting_latest}/32 reassert={reassert_rows_selecting_latest}/32"
        ));
    }

    // Run CPU batch oracle evaluator first to obtain target_spans, distractor_spans, and reference report.
    let cpu_batch = evaluate_forced_copy_oracle_batch(&head, &pieces, &oracle_rows, 16)
        .map_err(|e| e.to_string())?
        .map_err(|stop| format!("oracle stop failure: {stop:?}"))?;

    // Build flat GPU batch across all 4 conditions (N, O1, O2, W) and all 256 rows.
    let condition_names = ["N", "O1", "O2", "W"];
    let mut gpu_items = Vec::new();
    let mut item_meta = Vec::new();
    let mut total_window_tokens: u64 = 0;

    for (cond_idx, &cond_name) in condition_names.iter().enumerate() {
        for (row_idx, row) in oracle_rows.iter().enumerate() {
            let oracle = match cond_idx {
                0 => CopySpanOracle::Off,
                1 => CopySpanOracle::O1SpanLearnedGate {
                    start: cpu_batch.target_spans[row_idx].0,
                    end: cpu_batch.target_spans[row_idx].1,
                },
                2 => CopySpanOracle::O2SpanForcedGate {
                    start: cpu_batch.target_spans[row_idx].0,
                    end: cpu_batch.target_spans[row_idx].1,
                    forced_gate_margin: row.forced_gate_margin,
                },
                _ => CopySpanOracle::O2SpanForcedGate {
                    start: cpu_batch.distractor_spans[row_idx].0,
                    end: cpu_batch.distractor_spans[row_idx].1,
                    forced_gate_margin: row.forced_gate_margin,
                },
            };
            for (step_offset, (base_margins, &learned_gate_margin)) in row
                .step_base_margins
                .iter()
                .zip(&row.step_learned_gate_margins)
                .enumerate()
            {
                total_window_tokens += row.window_tokens.len() as u64;
                gpu_items.push(PointerStepItem {
                    window_tokens: row.window_tokens.clone(),
                    base_margins: base_margins.clone(),
                    learned_gate_margin,
                    step_offset,
                    query_slot: row.query_slot,
                    query_product: row.query_product,
                    query_preceding: row.query_preceding.clone(),
                    candidates: vec![],
                    oracle,
                });
                item_meta.push((cond_name.to_string(), row_idx, step_offset));
            }
        }
    }

    // Launch 1 and Launch 2 on GPU to verify determinism and measure tokens/s.
    let (gpu_run1, elapsed_us_1) = device
        .select_batch(&head, &gpu_items)
        .map_err(|e| e.to_string())?;
    let (gpu_run2, elapsed_us_2) = device
        .select_batch(&head, &gpu_items)
        .map_err(|e| e.to_string())?;
    if gpu_run1 != gpu_run2 {
        return Err("nondeterministic GPU launch: run 1 != run 2".to_string());
    }

    let parity_checked = check_host_parity(&head, &gpu_items, &gpu_run1)?;

    // Reconstruct full reply sequences (value tokens + PERIOD_TOKEN_ID + EOS_TOKEN_ID) from GPU decisions
    // and grade through the unmodified `FrozenAnswers` + `ByteBpeTokenizer` + `pair_id` grader.
    let mut gpu_replies_o2 = vec![Vec::new(); 256];
    let mut gpu_replies_w = vec![Vec::new(); 256];
    let mut decision_records = Vec::with_capacity(gpu_run1.len());

    for ((cond_name, row_idx, step_offset), dec) in item_meta.into_iter().zip(gpu_run1) {
        if cond_name == "O2" {
            let t_len = cpu_batch.target_spans[row_idx].1 - cpu_batch.target_spans[row_idx].0;
            if step_offset < t_len {
                gpu_replies_o2[row_idx].push(dec.emitted_token);
            }
        } else if cond_name == "W" {
            let d_len =
                cpu_batch.distractor_spans[row_idx].1 - cpu_batch.distractor_spans[row_idx].0;
            if step_offset < d_len {
                gpu_replies_w[row_idx].push(dec.emitted_token);
            }
        }
        decision_records.push(ConditionDecisionRecord {
            condition: cond_name,
            row_id: oracle_rows[row_idx].row_id.clone(),
            step_offset,
            emitted_token: dec.emitted_token,
            base_top_token: dec.base_top_token,
            base_top_margin: dec.base_top_margin,
            copied: dec.copied,
            selected_span: dec.selected_span,
            candidate_copy_token: dec.candidate_copy_token,
            route_score: dec.route_score,
            effective_gate_margin: dec.effective_gate_margin,
        });
    }

    for r in &mut gpu_replies_o2 {
        r.push(PERIOD_TOKEN_ID);
        r.push(EOS_TOKEN_ID);
    }
    for r in &mut gpu_replies_w {
        r.push(PERIOD_TOKEN_ID);
        r.push(EOS_TOKEN_ID);
    }

    let (grader_o2_rows, grader_o2_pairs) =
        grade_with_evaluator_oracle(&tok, &inputs, &labels, &gpu_replies_o2)?;
    let (grader_w_rows, grader_w_pairs) =
        grade_with_evaluator_oracle(&tok, &inputs, &labels, &gpu_replies_w)?;

    if grader_o2_rows != 256 || grader_o2_pairs != 128 || grader_w_rows != 0 || grader_w_pairs != 0
    {
        return Err(format!(
            "evaluator oracle grading failed: O2=({grader_o2_rows}/256 rows, {grader_o2_pairs}/128 pairs), W=({grader_w_rows}/256 rows, {grader_w_pairs}/128 pairs)"
        ));
    }

    let decisions_bytes =
        serde_json::to_vec_pretty(&decision_records).map_err(|e| e.to_string())?;
    let decisions_sha256 = sha256_bytes(&decisions_bytes);

    let elapsed_us = elapsed_us_1.max(1);
    let tokens_per_second = (total_window_tokens * 1_000_000) / elapsed_us;
    let steps_per_second = ((gpu_items.len() as u64) * 1_000_000) / elapsed_us;

    let record = json!({
        "schema": "uor-r4.lawful_pointer_forced_copy_oracle_qualification.v1",
        "status": "PASS",
        "ptx_sha256": actual_ptx,
        "decisions_sha256": decisions_sha256,
        "tokenizer_sha256": tok_sha,
        "panel_seals": {
            "plan_sha256": EXPECTED_PLAN_SHA256,
            "inputs_sha256": EXPECTED_INPUTS_SHA256,
            "labels_sha256": EXPECTED_LABELS_SHA256,
            "manifest_sha256": EXPECTED_MANIFEST_SHA256,
        },
        "span_coverage": {
            "raw_dialogue_target_spans_found": dialogue_target_spans_found,
            "raw_dialogue_distractor_spans_found": dialogue_distractor_spans_found,
            "raw_dialogue_canonical_fold_matches": dialogue_canonical_fold_matches,
            "update_rows_selecting_latest": update_rows_selecting_latest,
            "reassert_rows_selecting_latest": reassert_rows_selecting_latest,
            "packet_target_spans_found": cpu_batch.target_spans.len(),
            "packet_distractor_spans_found": cpu_batch.distractor_spans.len(),
        },
        "gpu_execution": {
            "total_step_items": parity_checked,
            "total_window_tokens": total_window_tokens,
            "elapsed_microseconds_run1": elapsed_us_1,
            "elapsed_microseconds_run2": elapsed_us_2,
            "tokens_per_second": tokens_per_second,
            "steps_per_second": steps_per_second,
            "gpu_vs_cpu_mismatches": 0,
            "two_launch_byte_identical": true,
        },
        "oracle_conditions": {
            "n_complete_rows": cpu_batch.n.complete_rows,
            "n_pairs_both_complete_distinct": cpu_batch.n.pairs_both_complete_distinct,
            "o1_complete_rows": cpu_batch.o1.complete_rows,
            "o1_pairs_both_complete_distinct": cpu_batch.o1.pairs_both_complete_distinct,
            "o2_complete_rows": cpu_batch.o2.complete_rows,
            "o2_pairs_both_complete_distinct": cpu_batch.o2.pairs_both_complete_distinct,
            "w_complete_rows": cpu_batch.w.complete_rows,
            "w_pairs_both_complete_distinct": cpu_batch.w.pairs_both_complete_distinct,
            "evaluator_oracle_o2_rows": grader_o2_rows,
            "evaluator_oracle_o2_pairs": grader_o2_pairs,
            "evaluator_oracle_w_rows": grader_w_rows,
            "evaluator_oracle_w_pairs": grader_w_pairs,
            "se_delta_o1_n_rows_milli": cpu_batch.se_delta_o1_n_rows_milli,
            "se_delta_o1_n_pairs_milli": cpu_batch.se_delta_o1_n_pairs_milli,
            "se_delta_o2_n_rows_milli": cpu_batch.se_delta_o2_n_rows_milli,
            "se_delta_o2_n_pairs_milli": cpu_batch.se_delta_o2_n_pairs_milli,
            "diagnostic_branch_under_untrained_gate": cpu_batch.branch,
        },
        "on_checkpoint_status": "AWAITING_M2_TRUNK_CHECKPOINT (#2844)"
    });

    Ok((record, decisions_bytes))
}
