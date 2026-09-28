//! One-pass D5 addressing-contest harness (T2 Stage 2c).
//!
//! Reads the frozen `fit-quaternion-6/checkpoint-final` parent and the evaluator's
//! development population once, reproduces the 976 x 256 block layout, fits the
//! ordinary codebooks on the first 64 blocks only, and computes every declared
//! arm's admission metrics on the comparison tail in a single incremental pass.
//! It is read-only: no optimizer step, no parameter change, no serving default.
#![forbid(unsafe_code)]

use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::addressing_arms::{AddressingArm, Codes, KMeansLayout, KEY_WIDTH};
use uor_r4_training::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity, Evaluator,
};
use uor_r4_training::joint_admission::AdmissionPolicy;
use uor_r4_training::joint_campaign::Campaign;
use uor_r4_training::joint_model::{JointModel, ReadMode};
use uor_r4_training::{sha256_file, Result, TrainingError};

const CONTEXT: usize = 256;
const TUNE_BLOCKS: usize = 64;
const COMPARISON_BLOCKS: usize = 912;
const SMOKE_COMPARISON_BLOCKS: usize = 8;
const S_BUDGETS: [usize; 4] = [4, 8, 16, 32];
const RECENT_SHARE: usize = 8;
const NO_READ_CHOICE_THRESHOLD: f64 = 0.5;
const LONG_RANGE_AGE: usize = 32;

const CONDITION_A_ROWS: [PanelRow; 5] = [
    PanelRow {
        probe: "story-source-edit-04|original",
        item: "doll",
        entity_occurrence: 7,
        prompt: &[
            0, 385, 329, 14, 407, 2591, 338, 1068, 719, 265, 914, 16, 341, 572, 311, 356, 265, 393,
            1288, 324, 520, 265, 888, 16, 872, 394, 594, 268, 327, 331, 338, 360, 16, 314, 705,
            269, 1261, 265, 2296, 1278, 2318, 265, 818, 16, 934, 311, 285, 400, 268, 480, 582, 14,
            407, 430, 515, 268, 265, 1288, 269, 906, 368, 265,
        ],
    },
    PanelRow {
        probe: "story-source-edit-04|edited",
        item: "bear",
        entity_occurrence: 7,
        prompt: &[
            0, 385, 329, 14, 407, 2591, 338, 787, 719, 265, 914, 16, 341, 572, 311, 356, 265, 393,
            1288, 324, 520, 265, 888, 16, 872, 394, 594, 268, 327, 331, 338, 360, 16, 314, 705,
            269, 1261, 265, 2296, 1278, 2318, 265, 818, 16, 934, 311, 285, 400, 268, 480, 582, 14,
            407, 430, 515, 268, 265, 1288, 269, 906, 368, 265,
        ],
    },
    PanelRow {
        probe: "story-source-edit-08|original",
        item: "brush",
        entity_occurrence: 7,
        prompt: &[
            0, 385, 329, 14, 407, 2591, 338, 2717, 719, 265, 914, 16, 341, 572, 311, 356, 265, 393,
            1288, 324, 520, 265, 888, 16, 872, 394, 594, 268, 327, 331, 338, 360, 16, 314, 705,
            269, 1261, 265, 2296, 1278, 2318, 265, 818, 16, 934, 311, 285, 400, 268, 480, 582, 14,
            407, 430, 515, 268, 265, 1288, 269, 906, 368, 265,
        ],
    },
    PanelRow {
        probe: "story-source-edit-12|original",
        item: "box",
        entity_occurrence: 7,
        prompt: &[
            0, 385, 329, 14, 407, 2591, 338, 616, 719, 265, 914, 16, 341, 572, 311, 356, 265, 393,
            1288, 324, 520, 265, 888, 16, 872, 394, 594, 268, 327, 331, 338, 360, 16, 314, 705,
            269, 1261, 265, 2296, 1278, 2318, 265, 818, 16, 934, 311, 285, 400, 268, 480, 582, 14,
            407, 430, 515, 268, 265, 1288, 269, 906, 368, 265,
        ],
    },
    PanelRow {
        probe: "story-source-edit-12|edited",
        item: "bag",
        entity_occurrence: 7,
        prompt: &[
            0, 385, 329, 14, 407, 2591, 338, 1216, 719, 265, 914, 16, 341, 572, 311, 356, 265, 393,
            1288, 324, 520, 265, 888, 16, 872, 394, 594, 268, 327, 331, 338, 360, 16, 314, 705,
            269, 1261, 265, 2296, 1278, 2318, 265, 818, 16, 934, 311, 285, 400, 268, 480, 582, 14,
            407, 430, 515, 268, 265, 1288, 269, 906, 368, 265,
        ],
    },
];

struct PanelRow {
    probe: &'static str,
    item: &'static str,
    entity_occurrence: usize,
    prompt: &'static [u32],
}

struct GeometricArm {
    label: String,
    family: &'static str,
    arm: AddressingArm,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ArmRef {
    Recent,
    Oracle,
    Geometric(usize),
    Hybrid(usize),
}

impl ArmRef {
    fn label(self, arms: &[GeometricArm]) -> String {
        match self {
            Self::Recent => "recent-s".to_string(),
            Self::Oracle => "oracle-qk".to_string(),
            Self::Geometric(index) => arms[index].label.clone(),
            Self::Hybrid(index) => format!("{}+recent8", arms[index].label),
        }
    }

    fn family(self, arms: &[GeometricArm]) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Oracle => "oracle",
            Self::Geometric(index) | Self::Hybrid(index) => arms[index].family,
        }
    }
}

#[derive(Clone, Default)]
struct Acc {
    positions: u64,
    recall_top1: u64,
    recall_top3_intersection: u64,
    captured_mass: f64,
    dense_mass: f64,
}

#[derive(Clone, Default)]
struct SlotMetrics {
    full: Acc,
    long_range: Acc,
    selected_read: Acc,
}

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}

fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&std::fs::read(path)?)?)
}

fn identity(path: &Path) -> Result<Value> {
    Ok(json!({"path":path,"bytes":std::fs::metadata(path)?.len(),"sha256":sha256_file(path)?}))
}

fn current_rss_kib() -> Option<u64> {
    let pid = std::process::id().to_string();
    let output = std::process::Command::new("ps")
        .args(["-o", "rss=", "-p", &pid])
        .output()
        .ok()?;
    String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse::<u64>()
        .ok()
}

fn load_parent(
    campaign: &Campaign,
    checkpoint: &Path,
    evaluator_sha: &str,
) -> Result<(JointModel, Value)> {
    report_output::verify(checkpoint)?;
    let embedded = Campaign::load(&checkpoint.join("campaign.json"))?;
    let binding = read_json(&checkpoint.join("checkpoint.json"))?;
    let step = binding["optimizer_step"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid("parent optimizer step"))?;
    if binding["schema"] != "uor-r4.joint-recurrent-checkpoint/1"
        || binding["status"] != "TARGET_COMPLETE"
        || binding["evaluator_sha256"] != evaluator_sha
        || binding["model_sha256"] != sha256_file(&checkpoint.join("model.safetensors"))?
        || binding["model_config_sha256"] != sha256_file(&checkpoint.join("config.json"))?
        || binding["next_data_step"] != json!(step)
        || !binding["source_commit"].as_str().is_some_and(|commit| {
            commit.len() == 40 && commit.chars().all(|c| c.is_ascii_hexdigit())
        })
    {
        return Err(invalid("parent checkpoint/evaluator binding mismatch"));
    }
    let model = JointModel::load(checkpoint, &Device::Cpu)?;
    if model.config != campaign.model
        || model.config != embedded.model
        || model.config.context != CONTEXT
        || model.config.read_width != KEY_WIDTH
        || model.quantization().is_some()
        || model.admission_policy() != AdmissionPolicy::Full
    {
        return Err(invalid(
            "contest requires the continuous Full-admission B16/T256 parent",
        ));
    }
    let provenance = json!({
        "path": checkpoint,
        "optimizer_step": step,
        "source_commit": binding["source_commit"],
        "checkpoint_sha256": sha256_file(&checkpoint.join("checkpoint.json"))?,
        "campaign_sha256": sha256_file(&checkpoint.join("campaign.json"))?,
        "config_sha256": sha256_file(&checkpoint.join("config.json"))?,
        "model_sha256": sha256_file(&checkpoint.join("model.safetensors"))?,
        "model_config": campaign.model,
        "admission": "full",
        "quantization": model.quantization(),
        "sealed_file_set_verified": true
    });
    Ok((model, provenance))
}

fn build_geometric_arms(tune_keys: &[[f32; KEY_WIDTH]]) -> Result<Vec<GeometricArm>> {
    let fixed: [(&str, &'static str, AddressingArm); 8] = [
        ("h4", "h4", AddressingArm::contest_h4()),
        ("h4-rht", "h4", AddressingArm::contest_h4_rht()),
        ("e8", "e8", AddressingArm::contest_e8()),
        ("e8-rht", "e8", AddressingArm::contest_e8_rht()),
        ("sign4", "sign", AddressingArm::contest_sign4()),
        ("sign4-rht", "sign", AddressingArm::contest_sign4_rht()),
        ("sign8", "sign", AddressingArm::contest_sign8()),
        ("sign8-rht", "sign", AddressingArm::contest_sign8_rht()),
    ];
    let mut arms: Vec<GeometricArm> = fixed
        .into_iter()
        .map(|(label, family, arm)| GeometricArm {
            label: label.to_string(),
            family,
            arm,
        })
        .collect();
    for (seed_index, arm) in
        AddressingArm::fit_kmeans_three_seeds(tune_keys, KMeansLayout::h4_120())?
            .into_iter()
            .enumerate()
    {
        arms.push(GeometricArm {
            label: format!("kmeans120-seed{}", seed_index + 1),
            family: "kmeans",
            arm,
        });
    }
    for (seed_index, arm) in
        AddressingArm::fit_kmeans_three_seeds(tune_keys, KMeansLayout::e8_240())?
            .into_iter()
            .enumerate()
    {
        arms.push(GeometricArm {
            label: format!("kmeans240-seed{}", seed_index + 1),
            family: "kmeans",
            arm,
        });
    }
    Ok(arms)
}

fn ranking_by_score(scores: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..scores.len()).collect();
    order.sort_by(|&a, &b| {
        scores[b]
            .partial_cmp(&scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    order
}

fn select_top(ranking: &[usize], s: usize) -> Vec<usize> {
    ranking[..s.min(ranking.len())].to_vec()
}

fn select_recent(previous: usize, s: usize) -> Vec<usize> {
    let count = s.min(previous);
    (previous - count..previous).collect()
}

fn select_hybrid(ranking: &[usize], previous: usize, s: usize) -> Vec<usize> {
    let share = RECENT_SHARE.min(s).min(previous);
    let mut selected: Vec<usize> = (previous - share..previous).collect();
    let boundary = previous - share;
    for &occurrence in ranking {
        if selected.len() >= s {
            break;
        }
        if occurrence < boundary {
            selected.push(occurrence);
        }
    }
    selected
}

fn dense_ranking(masses: &[f64]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..masses.len()).collect();
    order.sort_by(|&a, &b| {
        masses[b]
            .partial_cmp(&masses[a])
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.cmp(&b))
    });
    order
}

fn accumulate(
    acc: &mut Acc,
    selected: &[usize],
    masses: &[f64],
    dense_top1: usize,
    dense_top3: &[usize],
) {
    acc.positions += 1;
    if selected.contains(&dense_top1) {
        acc.recall_top1 += 1;
    }
    acc.recall_top3_intersection += selected
        .iter()
        .filter(|occurrence| dense_top3.contains(occurrence))
        .count() as u64;
    acc.captured_mass += selected.iter().map(|&index| masses[index]).sum::<f64>();
    acc.dense_mass += masses.iter().sum::<f64>();
}

fn arm_codes(arm: &AddressingArm, key: &[f32]) -> Result<Codes> {
    Ok(arm.encode(key)?)
}

fn arm_ranking(arm: &AddressingArm, codes: &[Codes], query: &[f32]) -> Result<Vec<usize>> {
    let lut = arm.query_lut(query)?;
    let scores: Vec<f64> = codes
        .iter()
        .map(|code| Ok(f64::from(arm.score(code, &lut)?)))
        .collect::<Result<_>>()?;
    Ok(ranking_by_score(&scores))
}

fn metric_record(label: &str, family: &str, s: usize, acc: &Acc, spec: &Value) -> Value {
    let recall_top1 = (acc.positions != 0).then(|| acc.recall_top1 as f64 / acc.positions as f64);
    let recall_top3 = (acc.positions != 0)
        .then(|| acc.recall_top3_intersection as f64 / (3.0 * acc.positions as f64));
    let captured_mass = (acc.dense_mass > 0.0).then(|| acc.captured_mass / acc.dense_mass);
    json!({
        "arm": label,
        "family": family,
        "s": s,
        "positions": acc.positions,
        "recall_top1": recall_top1,
        "recall_top3": recall_top3,
        "captured_mass": captured_mass,
        "captured_dense_mass": acc.captured_mass,
        "total_dense_mass": acc.dense_mass,
        "decode_work_per_query": spec,
    })
}

fn decode_spec(arm_ref: ArmRef, arms: &[GeometricArm], s: usize) -> Result<Value> {
    match arm_ref {
        ArmRef::Recent => Ok(json!({
            "kind":"address-only recency control; no content index or LUT",
            "table_reads":0,"adds":0,"multiplies":0,"index_bytes_per_event":0
        })),
        ArmRef::Oracle => Ok(json!({
            "kind":"dense q.k ceiling; not a deployable multiplier-free index",
            "table_reads":0,
            "adds":(KEY_WIDTH as u64 - 1) * s as u64,
            "multiplies":KEY_WIDTH as u64 * s as u64,
            "index_bytes_per_event":KEY_WIDTH as u64 * 4
        })),
        ArmRef::Geometric(index) | ArmRef::Hybrid(index) => {
            let arm = &arms[index].arm;
            let per_event = arm.decode_work_per_event();
            let lut = arm.lut_build_work();
            Ok(json!({
                "kind":arm.decode_note(),
                "table_reads":per_event.table_reads * s as u64 + lut.table_reads,
                "adds":per_event.adds * s as u64 + lut.adds,
                "multiplies":0,
                "index_bytes_per_event":arm.bytes_per_event(),
                "per_event_table_reads":per_event.table_reads,
                "per_event_adds":per_event.adds,
                "lut_build_table_reads":lut.table_reads,
                "lut_build_adds":lut.adds,
                "bits_per_block":arm.bits_per_block(),
                "information_bits_per_block":arm.information_bits_per_block()
            }))
        }
    }
}

/// One decision-0 read state plus its per-arm admission result.
struct PanelCapture {
    previous: usize,
    query: Vec<f32>,
    keys: Vec<f32>,
    masses: Vec<f64>,
    no_read: f64,
    codes: Vec<Vec<Codes>>,
}

fn capture_panel_row(
    model: &JointModel,
    arms: &[GeometricArm],
    prompt: &[u32],
) -> Result<PanelCapture> {
    let mut session = model.new_session(1)?;
    let mut codes: Vec<Vec<Codes>> = (0..arms.len()).map(|_| Vec::new()).collect();
    let mut capture: Option<PanelCapture> = None;
    for (position, &token) in prompt.iter().enumerate() {
        let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
        let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
        for (index, arm) in arms.iter().enumerate() {
            codes[index].push(arm_codes(&arm.arm, &written)?);
        }
        if position + 1 == prompt.len() {
            let keys = step
                .read_keys
                .as_ref()
                .ok_or_else(|| invalid("panel decision-0 read has no dense keys"))?;
            capture = Some(PanelCapture {
                previous: position,
                query: step.read_query.flatten_all()?.to_vec1::<f32>()?,
                keys: keys.flatten_all()?.to_vec1::<f32>()?,
                masses: step
                    .read_masses
                    .flatten_all()?
                    .to_vec1::<f32>()?
                    .into_iter()
                    .map(f64::from)
                    .collect(),
                no_read: f64::from(step.no_read_mass.to_vec1::<f32>()?[0]),
                codes: std::mem::take(&mut codes),
            });
        }
    }
    capture.ok_or_else(|| invalid("panel prompt is empty"))
}

fn panel_row_report(
    row_id: usize,
    row: &PanelRow,
    arms: &[GeometricArm],
    capture: &PanelCapture,
) -> Result<Value> {
    let previous = capture.previous;
    let masses = &capture.masses;
    let dense = dense_ranking(masses);
    let entity = row.entity_occurrence;
    if entity >= previous {
        return Err(invalid(
            "panel entity occurrence is not a decision-0 candidate",
        ));
    }
    let dense_rank = 1 + dense.iter().position(|&index| index == entity).unwrap_or(0);
    let dense_mass = masses[entity];
    let query = &capture.query;
    let oracle = ranking_by_score(
        &(0..previous)
            .map(|i| {
                query
                    .iter()
                    .zip(&capture.keys[i * KEY_WIDTH..(i + 1) * KEY_WIDTH])
                    .map(|(&q, &k)| f64::from(q) * f64::from(k))
                    .sum::<f64>()
            })
            .collect::<Vec<_>>(),
    );
    let dense_qk_rank = 1 + oracle
        .iter()
        .position(|&index| index == entity)
        .unwrap_or(0);
    let mut arm_rows = Vec::new();
    let mut admitted_top16 = 0usize;
    let mut admitted_top32 = 0usize;
    let mut emit =
        |label: String, family: &'static str, ranking: Option<Vec<usize>>| -> Result<()> {
            let ranking = ranking.unwrap_or_else(|| (0..previous).rev().collect());
            let arm_rank = 1 + ranking
                .iter()
                .position(|&index| index == entity)
                .unwrap_or(0);
            let top16 = ranking[..16.min(previous)].contains(&entity);
            let top32 = ranking[..32.min(previous)].contains(&entity);
            admitted_top16 += usize::from(top16);
            admitted_top32 += usize::from(top32);
            arm_rows.push(json!({
                "arm": label,
                "family": family,
                "dense_rank": dense_rank,
                "dense_mass": dense_mass,
                "dense_qk_rank": dense_qk_rank,
                "arm_rank": arm_rank,
                "admitted_top16": top16,
                "admitted_top32": top32
            }));
            Ok(())
        };
    emit("recent-s".to_string(), "recent", None)?;
    emit("oracle-qk".to_string(), "oracle", Some(oracle))?;
    for (index, arm) in arms.iter().enumerate() {
        emit(
            arm.label.clone(),
            arm.family,
            Some(arm_ranking(
                &arm.arm,
                &capture.codes[index][..previous],
                query,
            )?),
        )?;
    }
    Ok(json!({
        "row_id": row_id,
        "probe": row.probe,
        "item": row.item,
        "entity_occurrence": entity,
        "decision_zero_previous": previous,
        "no_read_mass": capture.no_read,
        "dense_rank": dense_rank,
        "dense_mass": dense_mass,
        "dense_qk_rank": dense_qk_rank,
        "admitted_top16_count": admitted_top16,
        "admitted_top32_count": admitted_top32,
        "base_arm_count": arm_rows.len(),
        "arms": arm_rows
    }))
}

fn parse_args() -> Result<(PathBuf, PathBuf, PathBuf, bool, Option<usize>)> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 4 {
        return Err(invalid(
            "usage: joint-addressing-contest CAMPAIGN_JSON PARENT_CHECKPOINT NEW_ROOT cpu [--smoke] [--positions N]",
        ));
    }
    let campaign = PathBuf::from(&args[0]);
    let checkpoint = PathBuf::from(&args[1]);
    let out = PathBuf::from(&args[2]);
    if args[3] != "cpu" {
        return Err(invalid("addressing contest is a CPU-only light job"));
    }
    let mut smoke = false;
    let mut positions = None;
    let mut index = 4;
    while index < args.len() {
        match args[index].as_str() {
            "--smoke" => smoke = true,
            "--positions" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| invalid("--positions requires a count"))?
                    .parse::<usize>()
                    .map_err(|_| invalid("--positions must be a positive integer"))?;
                if value == 0 || value > CONTEXT {
                    return Err(invalid("--positions must be within 1..=256"));
                }
                positions = Some(value);
            }
            other => return Err(invalid(format!("unknown argument {other}"))),
        }
        index += 1;
    }
    Ok((campaign, checkpoint, out, smoke, positions))
}

fn run(
    campaign_path: &Path,
    checkpoint: &Path,
    out: &Path,
    smoke: bool,
    positions: Option<usize>,
) -> Result<()> {
    let started = Instant::now();
    let campaign = Campaign::load(campaign_path)?;
    let evaluator: Evaluator = load_evaluator(&campaign.evaluator_path)?;
    let mut report = base_report(&evaluator, "joint-addressing-contest")?;
    report["schema"] = json!("uor-r4.addressing-contest/1");
    report["scope"] = json!("Offline, fixed weights, read-only D5 index question: at equal bits and bytes touched per query, does a fixed geometric codebook admit the dense-read events as well as the best ordinary index? One model, T=256, these bit rates. No training, admission change, serving path or language claim.");
    report["compiled_source_sha256"] = json!({
        "example":hex::encode(Sha256::digest(include_bytes!("joint-addressing-contest.rs"))),
        "joint_model":hex::encode(Sha256::digest(include_bytes!("../src/joint_model.rs"))),
        "addressing_arms":hex::encode(Sha256::digest(include_bytes!("../src/addressing_arms.rs")))
    });
    report["device"] = json!("cpu");
    report["cpu_accelerate_compiled"] = json!(cfg!(feature = "cpu-accelerate"));
    report["thread_environment"] = json!({
        "RAYON_NUM_THREADS":std::env::var("RAYON_NUM_THREADS").ok(),
        "VECLIB_MAXIMUM_THREADS":std::env::var("VECLIB_MAXIMUM_THREADS").ok(),
        "OMP_NUM_THREADS":std::env::var("OMP_NUM_THREADS").ok()
    });
    report["gradients_tracked"] = json!(false);
    report["neural_optimizer_steps"] = json!(0);
    report["rss_kib_after_setup"] = json!(current_rss_kib());
    save_json(&out.join("run-provenance.json"), &report)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    let tokens = read_tokens(&evaluator.document["dev_source"])?;
    if tokens.len() < (TUNE_BLOCKS + COMPARISON_BLOCKS) * CONTEXT {
        return Err(invalid(
            "development population is too short for 976 blocks",
        ));
    }
    let tokenizer_identities: Vec<&Value> = evaluator.document["reference_inputs"]
        .as_array()
        .ok_or_else(|| invalid("reference input inventory"))?
        .iter()
        .filter(|item| {
            item["path"]
                .as_str()
                .is_some_and(|path| path.ends_with("/tokenizer.json"))
        })
        .collect();
    if tokenizer_identities.len() != 1 {
        return Err(invalid("exactly one tokenizer identity required"));
    }
    let tokenizer_path = verify_identity(tokenizer_identities[0])?;
    let tokenizer = HfBpeTokenizer::from_dir(
        tokenizer_path
            .parent()
            .ok_or_else(|| invalid("tokenizer parent"))?,
    )
    .map_err(|error| invalid(format!("joint tokenizer: {error}")))?;
    let (model, parent) = load_parent(&campaign, checkpoint, &evaluator.sha256)?;
    report["parent"] = parent;
    report["tokenizer"] = json!({
        "identity": identity(&tokenizer_path)?,
        "cid": tokenizer.address(),
        "vocabulary": tokenizer.vocab_size()
    });
    report["dev_source"] = evaluator.document["dev_source"].clone();

    let comparison_blocks = if smoke {
        SMOKE_COMPARISON_BLOCKS
    } else {
        COMPARISON_BLOCKS
    };
    let stride = positions.map_or(1, |count| CONTEXT.div_ceil(count));
    let position_evaluated = |position: usize| position % stride == 0;

    let tune_started = Instant::now();
    let mut tune_keys: Vec<[f32; KEY_WIDTH]> = Vec::with_capacity(TUNE_BLOCKS * CONTEXT);
    let mut tune_model_seconds = 0.0f64;
    for block in 0..TUNE_BLOCKS {
        let mut session = model.new_session(1)?;
        for position in 0..CONTEXT {
            let token = u32::from(tokens[block * CONTEXT + position]);
            let step_started = Instant::now();
            let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
            tune_model_seconds += step_started.elapsed().as_secs_f64();
            let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
            let mut key = [0f32; KEY_WIDTH];
            key.copy_from_slice(&written);
            tune_keys.push(key);
        }
    }
    report["tune"] = json!({
        "blocks":TUNE_BLOCKS,"context":CONTEXT,"tokens":TUNE_BLOCKS*CONTEXT,
        "keys_collected":tune_keys.len(),"model_seconds":tune_model_seconds,
        "elapsed_seconds":tune_started.elapsed().as_secs_f64(),
        "partition":"first 64 development blocks only; the comparison tail never contributes to the codebooks"
    });
    report["rss_kib_after_tune"] = json!(current_rss_kib());

    let fit_started = Instant::now();
    let arms = build_geometric_arms(&tune_keys)?;
    drop(tune_keys);
    report["kmeans_fit_seconds"] = json!(fit_started.elapsed().as_secs_f64());
    report["arms"] = json!(arms
        .iter()
        .map(|arm| json!({
            "label":arm.label,
            "family":arm.family,
            "codes_per_block":arm.arm.codes_per_block(),
            "blocks":arm.arm.blocks(),
            "block_dim":arm.arm.block_dim(),
            "bits_per_block":arm.arm.bits_per_block(),
            "information_bits_per_block":arm.arm.information_bits_per_block(),
            "index_bytes_per_event":arm.arm.bytes_per_event(),
            "pre_rotation":arm.arm.pre_rotation_enabled(),
            "fit_seed":arm.arm.fit_seed()
        }))
        .collect::<Vec<_>>());

    let codebook_dir = out.join("codebooks");
    std::fs::create_dir(&codebook_dir)?;
    let mut codebook_records = Vec::with_capacity(arms.len());
    for arm in &arms {
        let file = codebook_dir.join(format!("{}.json", arm.label));
        arm.arm.save(&file)?;
        codebook_records.push(json!({
            "label":arm.label,
            "family":arm.family,
            "file":file.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
            "bytes":std::fs::metadata(&file)?.len(),
            "sha256":sha256_file(&file)?
        }));
    }
    report["codebooks"] = json!({
        "directory":"codebooks",
        "records":codebook_records,
        "scope":"The exact fitted serde arms scored above, written for the restricted-read evaluation. Fixed root/sign arms carry no fitted file; the k-means arms carry their centroids."
    });

    let slot_refs: Vec<ArmRef> = std::iter::once(ArmRef::Recent)
        .chain(std::iter::once(ArmRef::Oracle))
        .chain((0..arms.len()).map(ArmRef::Geometric))
        .chain((0..arms.len()).map(ArmRef::Hybrid))
        .collect();
    let mut slot_specs: Vec<(ArmRef, usize)> = Vec::new();
    for arm_ref in &slot_refs {
        for &s in &S_BUDGETS {
            slot_specs.push((*arm_ref, s));
        }
    }
    let mut metrics = vec![SlotMetrics::default(); slot_specs.len()];

    report["definitions"] = json!({
        "admitted_set":format!("top-s events of the arm's ranking over the dense-read candidate set; recent-s is the s most recent occurrences; oracle-qk is top-s by the exact dense query dot key; hybrids reserve min(8,s) slots for the most recent occurrences and fill the rest from the arm ranking"),
        "recall_top1":"fraction of dense-read positions whose dense top-1 event (lowest occurrence attaining the maximum dense read mass) is in the admitted set",
        "recall_top3":"mean over positions of |admitted set intersect dense top-3| / 3",
        "captured_mass":"sum over positions of admitted dense masses divided by sum of all dense read masses",
        "decode_work_per_query":"arm per-event decode work times s plus the arm query-LUT build; every geometric family declares zero multiplies; oracle-qk is the dense multiplier ceiling and recent-s is an address-only control",
        "index_bytes_per_event":"ceil(stored index bits per event / 8)",
        "dense_read_position":format!("previous >= 1 and NoRead mass < {NO_READ_CHOICE_THRESHOLD}; the dense read is the model's chosen path"),
        "populations":format!("metrics = the full comparison tail (every position with previous >= 1); long_range = its subset whose dense top-1 event age (tokens back) >= {LONG_RANGE_AGE}; selected_read_metrics = the subset with NoRead mass < {NO_READ_CHOICE_THRESHOLD}. The director's NoRead-exclusion rule is not adjudicated; all three denominators are reported."),
        "long_range_position":format!("dense-read position whose dense top-1 event age (tokens back) >= {LONG_RANGE_AGE}"),
        "candidate_ages":"age of occurrence o at position p is p - o, so the immediately previous write has age 1",
        "assignment":"fixed root/sign arms use maximum inner product; k-means arms use L2 nearest centroid, both as declared by the module",
        "tie_breaks":"ranking is score descending with ties to the lower occurrence index; dense top-k are mass descending with ties to the lower occurrence index"
    });

    let compare_started = Instant::now();
    let mut model_seconds = 0.0f64;
    let mut arm_seconds = 0.0f64;
    let mut positions_total = 0u64;
    let mut positions_dense_read = 0u64;
    let mut positions_excluded_no_read = 0u64;
    let mut max_no_read = 0.0f64;
    let mut no_read_sum = 0.0f64;
    let mut no_read_histogram = [0u64; 10];
    let mut min_mass_sum = f64::INFINITY;
    let mut max_mass_sum = 0.0f64;
    let mut max_mass_error = 0.0f64;
    for local_block in 0..comparison_blocks {
        let block = TUNE_BLOCKS + local_block;
        let mut session = model.new_session(1)?;
        let mut code_cache: Vec<Vec<Codes>> = (0..arms.len())
            .map(|_| Vec::with_capacity(CONTEXT))
            .collect();
        for position in 0..CONTEXT {
            let token = u32::from(tokens[block * CONTEXT + position]);
            let step_started = Instant::now();
            let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
            model_seconds += step_started.elapsed().as_secs_f64();
            let arm_started = Instant::now();
            let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
            for (index, arm) in arms.iter().enumerate() {
                code_cache[index].push(arm_codes(&arm.arm, &written)?);
            }
            arm_seconds += arm_started.elapsed().as_secs_f64();
            if position == 0 {
                continue;
            }
            positions_total += 1;
            let previous = position;
            let no_read = f64::from(step.no_read_mass.to_vec1::<f32>()?[0]);
            let masses: Vec<f64> = step
                .read_masses
                .flatten_all()?
                .to_vec1::<f32>()?
                .into_iter()
                .map(f64::from)
                .collect();
            if masses.len() != previous {
                return Err(invalid(
                    "dense read mass row does not match the candidate count",
                ));
            }
            let mass_sum = masses.iter().sum::<f64>() + no_read;
            min_mass_sum = min_mass_sum.min(mass_sum);
            max_mass_sum = max_mass_sum.max(mass_sum);
            max_mass_error = max_mass_error.max((mass_sum - 1.0).abs());
            let selected_read = no_read < NO_READ_CHOICE_THRESHOLD;
            if selected_read {
                positions_dense_read += 1;
            } else {
                positions_excluded_no_read += 1;
            }
            max_no_read = max_no_read.max(no_read);
            no_read_sum += no_read;
            let bucket = ((no_read * 10.0).floor() as usize).min(9);
            no_read_histogram[bucket] += 1;
            if !position_evaluated(position) {
                continue;
            }
            let dense = dense_ranking(&masses);
            let dense_top1 = dense[0];
            let dense_top3 = &dense[..3.min(previous)];
            let long_range = previous - dense_top1 >= LONG_RANGE_AGE;
            let arm_started = Instant::now();
            let query = step.read_query.flatten_all()?.to_vec1::<f32>()?;
            let keys = step
                .read_keys
                .as_ref()
                .ok_or_else(|| invalid("dense read did not expose its key history"))?
                .flatten_all()?
                .to_vec1::<f32>()?;
            if keys.len() != previous * KEY_WIDTH {
                return Err(invalid(
                    "dense read key row does not match the candidate count",
                ));
            }
            let oracle_scores: Vec<f64> = (0..previous)
                .map(|index| {
                    query
                        .iter()
                        .zip(&keys[index * KEY_WIDTH..(index + 1) * KEY_WIDTH])
                        .map(|(&q, &k)| f64::from(q) * f64::from(k))
                        .sum()
                })
                .collect();
            let oracle_ranking = ranking_by_score(&oracle_scores);
            let geo_rankings: Vec<Vec<usize>> = (0..arms.len())
                .map(|index| arm_ranking(&arms[index].arm, &code_cache[index][..previous], &query))
                .collect::<Result<_>>()?;
            arm_seconds += arm_started.elapsed().as_secs_f64();
            for (slot, (arm_ref, s)) in slot_specs.iter().enumerate() {
                let selected = match arm_ref {
                    ArmRef::Recent => select_recent(previous, *s),
                    ArmRef::Oracle => select_top(&oracle_ranking, *s),
                    ArmRef::Geometric(index) => select_top(&geo_rankings[*index], *s),
                    ArmRef::Hybrid(index) => select_hybrid(&geo_rankings[*index], previous, *s),
                };
                let target = &mut metrics[slot];
                accumulate(&mut target.full, &selected, &masses, dense_top1, dense_top3);
                if long_range {
                    accumulate(
                        &mut target.long_range,
                        &selected,
                        &masses,
                        dense_top1,
                        dense_top3,
                    );
                }
                if selected_read {
                    accumulate(
                        &mut target.selected_read,
                        &selected,
                        &masses,
                        dense_top1,
                        dense_top3,
                    );
                }
            }
        }
    }
    report["rss_kib_after_comparison"] = json!(current_rss_kib());

    let mut records = Vec::with_capacity(slot_specs.len());
    let mut long_records = Vec::with_capacity(slot_specs.len());
    let mut selected_records = Vec::with_capacity(slot_specs.len());
    for (slot, (arm_ref, s)) in slot_specs.iter().enumerate() {
        let spec = decode_spec(*arm_ref, &arms, *s)?;
        let label = arm_ref.label(&arms);
        let family = arm_ref.family(&arms);
        records.push(metric_record(
            &label,
            family,
            *s,
            &metrics[slot].full,
            &spec,
        ));
        long_records.push(metric_record(
            &label,
            family,
            *s,
            &metrics[slot].long_range,
            &spec,
        ));
        selected_records.push(metric_record(
            &label,
            family,
            *s,
            &metrics[slot].selected_read,
            &spec,
        ));
    }
    let families = ["h4", "e8", "kmeans", "sign"];
    let mut best_per_family = Vec::new();
    for family in families {
        for (slot, (arm_ref, s)) in slot_specs.iter().enumerate() {
            if let ArmRef::Geometric(index) = arm_ref {
                if arms[*index].family != family {
                    continue;
                }
                let hybrid_slot = slot_specs
                    .iter()
                    .position(|(candidate, budget)| {
                        matches!(candidate, ArmRef::Hybrid(other) if *other == *index)
                            && budget == s
                    })
                    .ok_or_else(|| invalid("missing hybrid slot"))?;
                let base = &metrics[slot].full;
                let hybrid = &metrics[hybrid_slot].full;
                let base_captured =
                    (base.dense_mass > 0.0).then(|| base.captured_mass / base.dense_mass);
                let hybrid_captured =
                    (hybrid.dense_mass > 0.0).then(|| hybrid.captured_mass / hybrid.dense_mass);
                best_per_family.push(json!({
                    "family":family,
                    "s":s,
                    "candidate":arm_ref.label(&arms),
                    "candidate_captured_mass":base_captured,
                    "candidate_recall_top1":(base.positions!=0).then(|| base.recall_top1 as f64/base.positions as f64),
                    "hybrid_captured_mass":hybrid_captured,
                    "hybrid_recall_top1":(hybrid.positions!=0).then(|| hybrid.recall_top1 as f64/hybrid.positions as f64),
                    "selection_rule":"per family and s, highest full-population captured_mass over the base arms, then recall_top1, then label"
                }));
            }
        }
    }
    report["comparison"] = json!({
        "blocks":comparison_blocks,
        "context":CONTEXT,
        "positions_total":positions_total,
        "positions_dense_read":positions_dense_read,
        "positions_excluded_no_read":positions_excluded_no_read,
        "long_range_positions":metrics.iter().map(|slot|slot.long_range.positions).max().unwrap_or(0),
        "mass_normalization":{"minimum_read_plus_no_read":min_mass_sum,"maximum_read_plus_no_read":max_mass_sum,"maximum_absolute_error":max_mass_error},
        "no_read_mass":{"mean":(positions_total!=0).then(||no_read_sum/positions_total as f64),"maximum":max_no_read,"histogram_bucket_tenths":no_read_histogram},
        "positions_subsample_stride":stride,
        "metrics":records,
        "long_range":long_records,
        "selected_read_metrics":selected_records
    });
    report["best_per_family_hybrids"] = json!(best_per_family);

    let panel_started = Instant::now();
    let mut panel_rows = Vec::new();
    for (row_id, row) in CONDITION_A_ROWS.iter().enumerate() {
        let capture = capture_panel_row(&model, &arms, row.prompt)?;
        panel_rows.push(panel_row_report(row_id, row, &arms, &capture)?);
    }
    report["condition_a"] = json!({
        "rows":panel_rows,
        "source":"Five condition-A distractor rows transcribed verbatim (prompt token ids, entity occurrence, item) from the retained read-localization oracle-rerank report. The contest re-feeds each prompt through its own session and reports admission only; it does not re-derive the fixture.",
        "admission":"Per base arm: entity-occurrence membership in the arm's top-16 and top-32 decision-0 admitted set, with the entity's dense read-mass rank, dense q.k rank, dense read mass and arm rank. Recent-s is the recency control; oracle-qk is the dense q.k ceiling; the rest are the fitted or fixed codebook arms."
    });
    report["condition_a_seconds"] = json!(panel_started.elapsed().as_secs_f64());
    let kmeans_fit_seconds = report["kmeans_fit_seconds"].clone();
    let rss_after_setup = report["rss_kib_after_setup"].clone();
    let rss_after_tune = report["rss_kib_after_tune"].clone();
    let rss_after_comparison = report["rss_kib_after_comparison"].clone();
    report["timing"] = json!({
        "tune_model_seconds":tune_model_seconds,
        "kmeans_fit_seconds":kmeans_fit_seconds,
        "comparison_model_seconds":model_seconds,
        "comparison_arm_seconds":arm_seconds,
        "comparison_seconds":compare_started.elapsed().as_secs_f64(),
        "total_seconds":started.elapsed().as_secs_f64(),
        "note":"model seconds bound only joint_model::step calls; arm seconds bound encoding, LUT build, scoring, ranking and accumulation; k-means fit and JSON work are harness overhead"
    });
    report["resources"] = json!({
        "rss_kib_after_setup":rss_after_setup,
        "rss_kib_after_tune":rss_after_tune,
        "rss_kib_after_comparison":rss_after_comparison,
        "peak_rss_note":"in-process samples via ps; the true maximum resident set size is captured by running the sealed binary under /usr/bin/time -l"
    });
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!("COMPLETE");
    save_json(&out.join("addressing-contest.json"), &report)?;
    Ok(())
}

fn main() -> Result<()> {
    let (campaign_path, checkpoint, out, smoke, positions) = parse_args()?;
    let checkpoint = checkpoint.canonicalize()?;
    let parent = out
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
        .canonicalize()?;
    if parent.starts_with(&checkpoint) || out.file_name().is_none() {
        return Err(invalid(
            "new report must be outside the sealed checkpoint root",
        ));
    }
    report_output::claim(&out)?;
    let result = run(&campaign_path, &checkpoint, &out, smoke, positions);
    if let Err(error) = &result {
        save_json(
            &out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    println!("COMPLETE: {}; sealed and verified", out.display());
    Ok(())
}
