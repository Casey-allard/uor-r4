//! One-pass D6 information audit, frozen in
//! `docs/integration/whole-project-synthesis-2026-09-28.md` §4.
//!
//! Evaluation only: the `fit-quaternion-2i-screen-off-8a6d` parent is loaded with fixed
//! weights and the evaluator's development population is read once. The harness first
//! reproduces the retained comparison-tail read NLL anchor, then substitutes only the read
//! score for the frozen O/U/D/G/K arms and reports the fraction of selected read positions
//! whose top-1 read event differs from the dot score's. No training, no model fitting, no
//! parameter change, no serving default and no generation.
#![forbid(unsafe_code)]

use std::cmp::Ordering;
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use safetensors::Dtype as SafeDtype;
use safetensors::SafeTensors;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::native_geometric::learner::embedding::canonical_h4_roots;
use uor_r4_core::report_output;
use uor_r4_core::transformerless::hf_bpe::HfBpeTokenizer;
use uor_r4_training::addressing_arms::{
    fit_lane_kmeans_1024, nearest_lane_centroid, quantize_dyadic_gain, D6_GAIN_LEVELS, D6_LANES,
    D6_LANE_CENTROIDS, D6_LANE_DIM, KEY_WIDTH,
};
use uor_r4_training::baseline_protocol::{
    base_report, load_evaluator, read_tokens, save_json, verify_identity, Evaluator,
};
use uor_r4_training::joint_admission::AdmissionPolicy;
use uor_r4_training::joint_campaign::Campaign;
use uor_r4_training::joint_evaluation::{score_probabilities, story_probes};
use uor_r4_training::joint_model::{JointModel, ReadMode};
use uor_r4_training::{sha256_file, Result, TrainingError};

const CONTEXT: usize = 256;
const READ_WIDTH: usize = 64;
const TUNE_BLOCKS: usize = 64;
const COMPARISON_BLOCKS: usize = 912;
const SMOKE_COMPARISON_BLOCKS: usize = 8;
const SELECTED_READ_THRESHOLD: f64 = 0.5;
const AGE_LEN: usize = CONTEXT - 1;
const KMEANS_FIT_SEED: u64 = 0x0D60_2026_0928_0001;
const GAIN_SPAN_BELOW_MEDIAN: i32 = 3;
const DECLARED_COMPARISON_NLL: f64 = 1.984_752_805;
const DECLARED_COMPARISON_NLL_TOLERANCE: f64 = 1e-3;
const DECLARED_PANEL_COMPLETE: usize = 27;
const MIN_LANE_NORM_SQUARED: f64 = 1e-18;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Arm {
    O,
    U,
    D,
    G,
    K,
}

const ARM_ORDER: [Arm; 5] = [Arm::O, Arm::U, Arm::D, Arm::G, Arm::K];

impl Arm {
    fn label(self) -> &'static str {
        match self {
            Self::O => "O",
            Self::U => "U",
            Self::D => "D",
            Self::G => "G",
            Self::K => "K",
        }
    }

    fn kind(self) -> &'static str {
        match self {
            Self::O => "dot/sqrt(read_width); the parent's own read (anchor)",
            Self::U => "per-lane unit dot, unquantized; 16 x 4-D lanes L2-normalized",
            Self::D => "2I direction only; each unit lane snapped to its nearest of 120 H4 roots",
            Self::G => {
                "2I direction plus 3-bit dyadic gain; 7 direction bits + 3 gain bits = 10 bits/lane"
            }
            Self::K => "per-lane raw k-means with 1024 centroids = 10 bits/lane",
        }
    }

    const fn codebook_bits(self) -> Option<u32> {
        match self {
            Self::G | Self::K => Some(10),
            _ => None,
        }
    }
}

fn arm_index(arm: Arm) -> usize {
    match arm {
        Arm::O => 0,
        Arm::U => 1,
        Arm::D => 2,
        Arm::G => 3,
        Arm::K => 4,
    }
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

fn lane_norm_squared(lane: &[f32]) -> f64 {
    let mut total = 0.0;
    for dim in 0..D6_LANE_DIM {
        let value = f64::from(lane[dim]);
        total += value * value;
    }
    total
}

fn lane_unit(lane: &[f32]) -> Option<[f64; 4]> {
    let norm_squared = lane_norm_squared(lane);
    if norm_squared <= MIN_LANE_NORM_SQUARED {
        return None;
    }
    let inverse = 1.0 / norm_squared.sqrt();
    Some([
        f64::from(lane[0]) * inverse,
        f64::from(lane[1]) * inverse,
        f64::from(lane[2]) * inverse,
        f64::from(lane[3]) * inverse,
    ])
}

fn lane_norm(lane: &[f32]) -> f64 {
    lane_norm_squared(lane).sqrt()
}

fn nearest_root(roots: &[[f64; 4]], unit: &[f64; 4]) -> (u16, [f64; 4]) {
    let mut best = 0usize;
    let mut best_dot = f64::NEG_INFINITY;
    for (index, root) in roots.iter().enumerate() {
        let mut dot = 0.0;
        for dim in 0..D6_LANE_DIM {
            dot += unit[dim] * root[dim];
        }
        if dot > best_dot {
            best_dot = dot;
            best = index;
        }
    }
    (best as u16, roots[best])
}

struct Encoded {
    rep: [f32; KEY_WIDTH],
    code: Vec<u16>,
}

struct Encoder {
    roots: Vec<[f64; 4]>,
    k_centroids: Vec<[f64; 4]>,
    gain_min_exponent: i32,
}

impl Encoder {
    fn encode(&self, arm: Arm, vector: &[f32]) -> Encoded {
        let mut rep = [0f32; KEY_WIDTH];
        let mut code = Vec::new();
        match arm {
            Arm::O => {
                rep.copy_from_slice(&vector[..KEY_WIDTH]);
            }
            Arm::U => {
                for lane in 0..D6_LANES {
                    let start = lane * D6_LANE_DIM;
                    if let Some(unit) = lane_unit(&vector[start..start + D6_LANE_DIM]) {
                        for dim in 0..D6_LANE_DIM {
                            rep[start + dim] = unit[dim] as f32;
                        }
                    }
                }
            }
            Arm::D => {
                code.reserve(D6_LANES);
                for lane in 0..D6_LANES {
                    let start = lane * D6_LANE_DIM;
                    match lane_unit(&vector[start..start + D6_LANE_DIM]) {
                        Some(unit) => {
                            let (index, root) = nearest_root(&self.roots, &unit);
                            code.push(index);
                            for dim in 0..D6_LANE_DIM {
                                rep[start + dim] = root[dim] as f32;
                            }
                        }
                        None => code.push(0),
                    }
                }
            }
            Arm::G => {
                code.reserve(D6_LANES);
                for lane in 0..D6_LANES {
                    let start = lane * D6_LANE_DIM;
                    let slice = &vector[start..start + D6_LANE_DIM];
                    let (gain_code, gain, _) =
                        quantize_dyadic_gain(lane_norm(slice), self.gain_min_exponent);
                    match lane_unit(slice) {
                        Some(unit) => {
                            let (index, root) = nearest_root(&self.roots, &unit);
                            code.push(index * D6_GAIN_LEVELS as u16 + u16::from(gain_code));
                            for dim in 0..D6_LANE_DIM {
                                rep[start + dim] = (root[dim] * gain) as f32;
                            }
                        }
                        None => code.push(u16::from(gain_code)),
                    }
                }
            }
            Arm::K => {
                code.reserve(D6_LANES);
                for lane in 0..D6_LANES {
                    let start = lane * D6_LANE_DIM;
                    let lane4 = [
                        f64::from(vector[start]),
                        f64::from(vector[start + 1]),
                        f64::from(vector[start + 2]),
                        f64::from(vector[start + 3]),
                    ];
                    let base = lane * D6_LANE_CENTROIDS;
                    let (index, centroid) = nearest_lane_centroid(
                        &self.k_centroids[base..base + D6_LANE_CENTROIDS],
                        &lane4,
                    );
                    code.push(index);
                    for dim in 0..D6_LANE_DIM {
                        rep[start + dim] = centroid[dim] as f32;
                    }
                }
            }
        }
        Encoded { rep, code }
    }
}

fn dot_rep(a: &[f32], b: &[f32]) -> f64 {
    let mut total = 0.0;
    for index in 0..KEY_WIDTH {
        total += f64::from(a[index]) * f64::from(b[index]);
    }
    total
}

fn score_offset(arm: Arm) -> f64 {
    match arm {
        Arm::O => 1.0 / (READ_WIDTH as f64).sqrt(),
        _ => 1.0,
    }
}

fn arm_top_two(
    arm: Arm,
    query: &[f32],
    keys: &[Encoded],
    age_bias: &[f32],
    previous: usize,
) -> Option<(usize, Option<usize>)> {
    if previous == 0 || keys.len() < previous || age_bias.len() < previous {
        return None;
    }
    let scale = score_offset(arm);
    let mut first: Option<usize> = None;
    let mut second: Option<usize> = None;
    let mut first_score = f64::NEG_INFINITY;
    let mut second_score = f64::NEG_INFINITY;
    for candidate in 0..previous {
        let mut score = dot_rep(query, &keys[candidate].rep) * scale;
        score += f64::from(age_bias[previous - 1 - candidate]);
        if first.is_none() || score > first_score {
            second = first;
            second_score = first_score;
            first = Some(candidate);
            first_score = score;
        } else if second.is_none() || score > second_score {
            second = Some(candidate);
            second_score = score;
        }
    }
    first.map(|first| (first, second))
}

fn argmax(values: &[f32]) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (index, &value) in values.iter().enumerate() {
        if best.is_none_or(|current| value > values[current]) {
            best = Some(index);
        }
    }
    best
}

fn accumulate_lanes(vector: &[f32], lanes: &mut [Vec<[f64; 4]>], norms: &mut Vec<f64>) {
    for lane in 0..D6_LANES {
        let start = lane * D6_LANE_DIM;
        let slice = &vector[start..start + D6_LANE_DIM];
        lanes[lane].push([
            f64::from(slice[0]),
            f64::from(slice[1]),
            f64::from(slice[2]),
            f64::from(slice[3]),
        ]);
        norms.push(lane_norm(slice));
    }
}

fn record_gain(
    norm: f64,
    min_exponent: i32,
    histogram: &mut [u64; 8],
    low: &mut u64,
    high: &mut u64,
) {
    let (code, _, flag) = quantize_dyadic_gain(norm, min_exponent);
    histogram[code as usize] += 1;
    match flag {
        -1 => *low += 1,
        1 => *high += 1,
        _ => {}
    }
}

fn load_age_bias(path: &Path) -> Result<Vec<f32>> {
    let bytes = std::fs::read(path)?;
    let tensors = SafeTensors::deserialize(&bytes)?;
    let view = tensors.tensor("read.age")?;
    if view.dtype() != SafeDtype::F32 || view.shape() != [AGE_LEN] {
        return Err(invalid("read.age tensor dtype/shape mismatch"));
    }
    let values = view
        .data()
        .chunks_exact(4)
        .map(|bytes| f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        .collect::<Vec<_>>();
    if values.len() != AGE_LEN || values.iter().any(|value| !value.is_finite()) {
        return Err(invalid("read.age tensor is not finite and complete"));
    }
    Ok(values)
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
        || model.config.read_width != READ_WIDTH
        || model.quantization().is_some()
        || model.admission_policy() != AdmissionPolicy::Full
    {
        return Err(invalid(
            "D6 requires the continuous Full-admission B16/T256 parent",
        ));
    }
    if campaign.geometric_read.is_some() || embedded.geometric_read.is_some() {
        return Err(invalid("D6 requires the kernel-off dot read parent"));
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
        "read_geometry": "dot",
        "sealed_file_set_verified": true
    });
    Ok((model, provenance))
}

fn locate_item(tokenizer: &HfBpeTokenizer, content: &[u32], noun: &str) -> Result<usize> {
    let target = tokenizer.encode(&format!(" {noun}"));
    let matches: Vec<usize> = if target.is_empty() || target.len() > content.len() {
        Vec::new()
    } else if target.len() == 1 {
        content
            .iter()
            .enumerate()
            .filter(|(_, &id)| id == target[0])
            .map(|(index, _)| index)
            .collect()
    } else {
        content
            .windows(target.len())
            .enumerate()
            .filter(|(_, window)| window == &target.as_slice())
            .map(|(index, _)| index)
            .collect()
    };
    if matches.len() == 1 {
        Ok(matches[0])
    } else {
        Err(invalid(format!(
            "item {noun} has {} prompt token occurrences",
            matches.len()
        )))
    }
}

struct PanelRowResult {
    probe: String,
    variant: &'static str,
    item: String,
    entity_occurrence: usize,
    previous: usize,
    no_read: f64,
    selected: bool,
    top1: Vec<Option<usize>>,
}

fn capture_panel_row(
    model: &JointModel,
    encoder: &Encoder,
    tokenizer: &HfBpeTokenizer,
    age_bias: &[f32],
    probe: &str,
    variant: &'static str,
    noun: &str,
    prompt: &str,
) -> Result<PanelRowResult> {
    let content = tokenizer.encode(prompt);
    let mut inputs = Vec::with_capacity(content.len() + 1);
    inputs.push(0);
    inputs.extend(content);
    if inputs.len() >= CONTEXT {
        return Err(invalid("panel prompt exceeds the model context"));
    }
    let entity_occurrence = locate_item(tokenizer, &inputs[1..], noun)? + 1;
    let mut session = model.new_session(1)?;
    let mut caches: Vec<Vec<Encoded>> = (0..ARM_ORDER.len())
        .map(|_| Vec::with_capacity(inputs.len()))
        .collect();
    let mut query: Option<Vec<f32>> = None;
    let mut no_read = 1.0f64;
    for (position, &token) in inputs.iter().enumerate() {
        let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
        let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
        for arm in ARM_ORDER {
            caches[arm_index(arm)].push(encoder.encode(arm, &written));
        }
        if position + 1 == inputs.len() {
            query = Some(step.read_query.flatten_all()?.to_vec1::<f32>()?);
            no_read = f64::from(step.no_read_mass.to_vec1::<f32>()?[0]);
        }
    }
    let previous = inputs.len() - 1;
    if entity_occurrence >= previous {
        return Err(invalid(
            "panel item occurrence is not a decision-0 read candidate",
        ));
    }
    let query = query.ok_or_else(|| invalid("panel capture missing final query"))?;
    let mut top1 = Vec::with_capacity(ARM_ORDER.len());
    for arm in ARM_ORDER {
        top1.push(
            arm_top_two(arm, &query, &caches[arm_index(arm)], age_bias, previous)
                .map(|(first, _)| first),
        );
    }
    Ok(PanelRowResult {
        probe: probe.to_string(),
        variant,
        item: noun.to_string(),
        entity_occurrence,
        previous,
        no_read,
        selected: no_read < SELECTED_READ_THRESHOLD,
        top1,
    })
}

#[derive(Clone, Copy, Default)]
struct Partition {
    scored_targets: u64,
    correct: u64,
    nll_sum: f64,
}

impl Partition {
    fn add(&mut self, nll: f64, correct: bool) {
        self.scored_targets += 1;
        self.nll_sum += nll;
        if correct {
            self.correct += 1;
        }
    }

    fn mean_nll(&self) -> Option<f64> {
        (self.scored_targets != 0).then(|| self.nll_sum / self.scored_targets as f64)
    }

    fn accuracy(&self) -> Option<f64> {
        (self.scored_targets != 0).then(|| self.correct as f64 / self.scored_targets as f64)
    }
}

struct Config {
    campaign: PathBuf,
    checkpoint: PathBuf,
    out: PathBuf,
    smoke: bool,
    anchor_only: bool,
}

fn parse_args() -> Result<Config> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 4 {
        return Err(invalid(
            "usage: joint-d6-information-audit CAMPAIGN_JSON PARENT_CHECKPOINT NEW_ROOT cpu [--smoke] [--anchor-only]",
        ));
    }
    if args[3] != "cpu" {
        return Err(invalid("D6 information audit is a CPU-only light job"));
    }
    let mut smoke = false;
    let mut anchor_only = false;
    for argument in &args[4..] {
        match argument.as_str() {
            "--smoke" => smoke = true,
            "--anchor-only" => anchor_only = true,
            other => return Err(invalid(format!("unknown argument {other}"))),
        }
    }
    Ok(Config {
        campaign: PathBuf::from(&args[0]),
        checkpoint: PathBuf::from(&args[1]),
        out: PathBuf::from(&args[2]),
        smoke,
        anchor_only,
    })
}

fn run(config: &Config) -> Result<()> {
    let started = Instant::now();
    let campaign = Campaign::load(&config.campaign)?;
    let evaluator: Evaluator = load_evaluator(&campaign.evaluator_path)?;
    let mut report = base_report(&evaluator, "joint-d6-information-audit")?;
    report["schema"] = json!("uor-r4.d6-information-audit/1");
    report["scope"] = json!("Offline, fixed weights, evaluation-only D6 information audit (whole-project synthesis 2026-09-28 §4): each arm substitutes only the read score and the fraction of selected read positions whose top-1 read event differs from the dot score's is reported. No training, no model fitting, no serving change, no generation and no codebook sweep.");
    report["compiled_source_sha256"] = json!({
        "example": hex::encode(Sha256::digest(include_bytes!("joint-d6-information-audit.rs"))),
        "joint_model": hex::encode(Sha256::digest(include_bytes!("../src/joint_model.rs"))),
        "addressing_arms": hex::encode(Sha256::digest(include_bytes!("../src/addressing_arms.rs"))),
        "joint_evaluation": hex::encode(Sha256::digest(include_bytes!("../src/joint_evaluation.rs")))
    });
    report["device"] = json!("cpu");
    report["cpu_accelerate_compiled"] = json!(cfg!(feature = "cpu-accelerate"));
    report["thread_environment"] = json!({
        "RAYON_NUM_THREADS": std::env::var("RAYON_NUM_THREADS").ok(),
        "VECLIB_MAXIMUM_THREADS": std::env::var("VECLIB_MAXIMUM_THREADS").ok(),
        "OMP_NUM_THREADS": std::env::var("OMP_NUM_THREADS").ok()
    });
    report["gradients_tracked"] = json!(false);
    report["neural_optimizer_steps"] = json!(0);
    report["rss_kib_after_setup"] = json!(current_rss_kib());
    save_json(&config.out.join("run-provenance.json"), &report)?;
    save_json(&config.out.join("evaluator.json"), &evaluator.document)?;

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

    let (model, parent) = load_parent(&campaign, &config.checkpoint, &evaluator.sha256)?;
    report["parent"] = parent;
    report["tokenizer"] = json!({
        "identity": identity(&tokenizer_path)?,
        "cid": tokenizer.address(),
        "vocabulary": tokenizer.vocab_size()
    });
    report["dev_source"] = evaluator.document["dev_source"].clone();

    let checkpoint_age = load_age_bias(&config.checkpoint.join("model.safetensors"))?;
    let model_age = model
        .variables()
        .get("read.age")
        .ok_or_else(|| invalid("model has no read.age variable"))?
        .as_tensor()
        .flatten_all()?
        .to_vec1::<f32>()?;
    let age_matches = model_age.len() == checkpoint_age.len()
        && model_age
            .iter()
            .zip(&checkpoint_age)
            .all(|(a, b)| a.to_bits() == b.to_bits());
    if !age_matches {
        return Err(invalid("read.age tensor differs from the loaded model"));
    }
    report["age_bias"] = json!({
        "source": "model.safetensors read.age read directly with the safetensors crate",
        "length": checkpoint_age.len(),
        "first": checkpoint_age.first(),
        "last": checkpoint_age.last(),
        "matches_loaded_model_bitwise": age_matches,
        "initialization": "-(occurrence age)/64; age starts 1 for the immediately previous write"
    });

    let comparison_blocks = if config.smoke {
        SMOKE_COMPARISON_BLOCKS
    } else {
        COMPARISON_BLOCKS
    };

    let tune_started = Instant::now();
    let mut tune_lanes: Vec<Vec<[f64; 4]>> = (0..D6_LANES).map(|_| Vec::new()).collect();
    let mut tune_norms: Vec<f64> = Vec::new();
    let mut tune = Partition::default();
    let mut tune_model_seconds = 0.0f64;
    for block in 0..TUNE_BLOCKS {
        let mut session = model.new_session(1)?;
        for position in 0..CONTEXT {
            let token = u32::from(tokens[block * CONTEXT + position]);
            let step_started = Instant::now();
            let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
            tune_model_seconds += step_started.elapsed().as_secs_f64();
            let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
            accumulate_lanes(&written, &mut tune_lanes, &mut tune_norms);
            if position >= 1 {
                let query = step.read_query.flatten_all()?.to_vec1::<f32>()?;
                accumulate_lanes(&query, &mut tune_lanes, &mut tune_norms);
            }
            let probabilities = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
            let score = score_probabilities(
                &probabilities,
                u32::from(tokens[block * CONTEXT + position + 1]),
            )?;
            tune.add(score.nll_nats, score.correct);
        }
    }
    report["tune"] = json!({
        "blocks": TUNE_BLOCKS, "context": CONTEXT, "scored_targets": tune.scored_targets,
        "mean_nll_nats": tune.mean_nll(), "top1_accuracy": tune.accuracy(),
        "model_seconds": tune_model_seconds,
        "elapsed_seconds": tune_started.elapsed().as_secs_f64(),
        "partition": "first 64 development blocks; they fit the K arm and never contribute to the comparison Δ population"
    });
    report["rss_kib_after_tune"] = json!(current_rss_kib());

    tune_norms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let tune_median_norm = tune_norms[tune_norms.len() / 2];
    let gain_min_exponent = (tune_median_norm.log2().floor() as i32) - GAIN_SPAN_BELOW_MEDIAN;
    report["gain_scheme"] = json!({
        "norm_definition": "L2 norm of a 4-D lane of the raw (unquantized) read query/key",
        "quantizer": "r_hat = 2^e, e = clamp(round(log2(norm)), e_min, e_min + 7); dyadic and multiplier-free at serving (shifts)",
        "gain_bits_per_lane": 3,
        "direction_bits_per_lane": 7,
        "bits_per_lane": 10,
        "e_min_rule": format!("floor(log2(median tune-split lane norm)) - {GAIN_SPAN_BELOW_MEDIAN}; chosen once from the tune split before any comparison scoring"),
        "e_min": gain_min_exponent,
        "gain_levels": D6_GAIN_LEVELS,
        "low_gain": 2f64.powi(gain_min_exponent),
        "high_gain": 2f64.powi(gain_min_exponent + D6_GAIN_LEVELS as i32 - 1),
        "tune_median_lane_norm": tune_median_norm,
        "tune_lane_count": tune_norms.len()
    });
    let encoder = if config.anchor_only {
        Encoder {
            roots: Vec::new(),
            k_centroids: Vec::new(),
            gain_min_exponent,
        }
    } else {
        let fit_started = Instant::now();
        let mut k_centroids: Vec<[f64; 4]> = Vec::with_capacity(D6_LANES * D6_LANE_CENTROIDS);
        for (block, lanes) in tune_lanes.iter().enumerate() {
            let centroids = fit_lane_kmeans_1024(lanes, KMEANS_FIT_SEED)?;
            if centroids.len() != D6_LANE_CENTROIDS {
                return Err(invalid(format!(
                    "k-means block {block} returned wrong size"
                )));
            }
            k_centroids.extend_from_slice(&centroids);
        }
        report["kmeans_fit"] = json!({
            "blocks": D6_LANES, "centroids_per_lane": D6_LANE_CENTROIDS, "stored_bits_per_lane": 10,
            "seed": KMEANS_FIT_SEED, "seed_count": 1,
            "partition": "tune-split query and key lane vectors (union); one fit, one seed, no sweep",
            "points_per_block": tune_lanes.first().map(Vec::len),
            "fit_seconds": fit_started.elapsed().as_secs_f64()
        });
        let roots = canonical_h4_roots()
            .iter()
            .map(|root| root.to_array())
            .collect::<Vec<_>>();
        report["rss_kib_after_fit"] = json!(current_rss_kib());
        Encoder {
            roots,
            k_centroids,
            gain_min_exponent,
        }
    };
    drop(tune_lanes);
    report["arms"] = json!(ARM_ORDER
        .iter()
        .map(|arm| json!({
            "arm": arm.label(),
            "kind": arm.kind(),
            "codebook_bits_per_lane": arm.codebook_bits()
        }))
        .collect::<Vec<_>>());

    let mut comparison = Partition::default();
    let mut positions_total = 0u64;
    let mut selected_total = 0u64;
    let mut no_read_sum = 0.0f64;
    let mut no_read_max = 0.0f64;
    let mut delta_differ = [0u64; ARM_ORDER.len()];
    let mut collision_top2 = [0u64; ARM_ORDER.len()];
    let mut collision_eligible = [0u64; ARM_ORDER.len()];
    let mut o_mismatch = 0u64;
    let mut gain_histogram = [0u64; 8];
    let mut gain_low_clip = 0u64;
    let mut gain_high_clip = 0u64;
    let compare_started = Instant::now();
    let mut comparison_model_seconds = 0.0f64;
    let mut arm_seconds = 0.0f64;
    for local_block in 0..comparison_blocks {
        let block = TUNE_BLOCKS + local_block;
        let mut session = model.new_session(1)?;
        let mut caches: Vec<Vec<Encoded>> = (0..ARM_ORDER.len())
            .map(|_| Vec::with_capacity(CONTEXT))
            .collect();
        for position in 0..CONTEXT {
            let token = u32::from(tokens[block * CONTEXT + position]);
            let step_started = Instant::now();
            let step = model.step(&mut session, &[token], ReadMode::Enabled)?;
            comparison_model_seconds += step_started.elapsed().as_secs_f64();
            let probabilities = step.probabilities.flatten_all()?.to_vec1::<f32>()?;
            let score = score_probabilities(
                &probabilities,
                u32::from(tokens[block * CONTEXT + position + 1]),
            )?;
            comparison.add(score.nll_nats, score.correct);

            let arm_started = Instant::now();
            let written = step.written_key.flatten_all()?.to_vec1::<f32>()?;
            if !config.anchor_only {
                for arm in ARM_ORDER {
                    caches[arm_index(arm)].push(encoder.encode(arm, &written));
                }
                for lane in 0..D6_LANES {
                    let start = lane * D6_LANE_DIM;
                    record_gain(
                        lane_norm(&written[start..start + D6_LANE_DIM]),
                        gain_min_exponent,
                        &mut gain_histogram,
                        &mut gain_low_clip,
                        &mut gain_high_clip,
                    );
                }
            }
            arm_seconds += arm_started.elapsed().as_secs_f64();

            if position == 0 {
                continue;
            }
            positions_total += 1;
            let previous = position;
            let no_read = f64::from(step.no_read_mass.to_vec1::<f32>()?[0]);
            no_read_sum += no_read;
            no_read_max = no_read_max.max(no_read);
            if no_read >= SELECTED_READ_THRESHOLD || config.anchor_only {
                continue;
            }
            selected_total += 1;
            let query = step.read_query.flatten_all()?.to_vec1::<f32>()?;
            for lane in 0..D6_LANES {
                let start = lane * D6_LANE_DIM;
                record_gain(
                    lane_norm(&query[start..start + D6_LANE_DIM]),
                    gain_min_exponent,
                    &mut gain_histogram,
                    &mut gain_low_clip,
                    &mut gain_high_clip,
                );
            }
            let arm_started = Instant::now();
            let mut tops: Vec<Option<(usize, Option<usize>)>> = Vec::with_capacity(ARM_ORDER.len());
            for arm in ARM_ORDER {
                let query_rep = encoder.encode(arm, &query);
                tops.push(arm_top_two(
                    arm,
                    &query_rep.rep,
                    &caches[arm_index(arm)],
                    &checkpoint_age,
                    previous,
                ));
            }
            arm_seconds += arm_started.elapsed().as_secs_f64();
            let o_top = tops[arm_index(Arm::O)].map(|(first, _)| first);
            let masses = step.read_masses.flatten_all()?.to_vec1::<f32>()?;
            if let (Some(model_top), Some(o_top)) = (argmax(&masses), o_top) {
                if model_top != o_top {
                    o_mismatch += 1;
                }
            }
            if let Some(o_top) = o_top {
                for arm in ARM_ORDER.iter().skip(1) {
                    let index = arm_index(*arm);
                    if tops[index].map(|(first, _)| first) != Some(o_top) {
                        delta_differ[index] += 1;
                    }
                }
                for arm in [Arm::D, Arm::G, Arm::K] {
                    let index = arm_index(arm);
                    collision_eligible[index] += 1;
                    if let Some((first, Some(second))) = tops[index] {
                        if caches[index][first].code == caches[index][second].code {
                            collision_top2[index] += 1;
                        }
                    }
                }
            }
        }
    }
    report["rss_kib_after_comparison"] = json!(current_rss_kib());

    let comparison_nll = comparison.mean_nll();
    let anchor_reproduced = if config.smoke {
        None
    } else {
        Some(
            comparison_nll
                .map(|nll| {
                    (nll - DECLARED_COMPARISON_NLL).abs() <= DECLARED_COMPARISON_NLL_TOLERANCE
                })
                .unwrap_or(false),
        )
    };
    report["anchor"] = json!({
        "comparison_blocks": comparison_blocks,
        "comparison_scored_targets": comparison.scored_targets,
        "comparison_mean_nll_nats": comparison_nll,
        "comparison_top1_accuracy": comparison.accuracy(),
        "tune_mean_nll_nats": tune.mean_nll(),
        "declared_comparison_nll_nats": DECLARED_COMPARISON_NLL,
        "tolerance": DECLARED_COMPARISON_NLL_TOLERANCE,
        "reproduced": anchor_reproduced,
        "smoke": config.smoke,
        "retained_root": "/Users/casey.allard/uor-r4/.uor-models/investigations/language-continuation-20260925/evaluate-quaternion-2i-off-read-8a6d",
        "declared_panel_complete": DECLARED_PANEL_COMPLETE,
        "panel_complete_source": "retained evaluate-quaternion-2i-off-read-8a6d story-probes.json (32/32 rows, 16 source-edit pairs)"
    });
    if anchor_reproduced == Some(false) {
        save_json(
            &config.out.join("anchor-failure.json"),
            &json!({"status":"ANCHOR_NOT_REPRODUCED","comparison_mean_nll_nats":comparison_nll,"declared":DECLARED_COMPARISON_NLL}),
        )?;
        report["status"] = json!("ANCHOR_NOT_REPRODUCED");
        report["outcome"] = json!(null);
        save_json(&config.out.join("d6-information-audit.json"), &report)?;
        return Err(invalid(
            "comparison-tail read NLL did not reproduce the anchor",
        ));
    }

    let delta: Vec<Option<f64>> = ARM_ORDER
        .iter()
        .map(|arm| {
            let index = arm_index(*arm);
            (selected_total != 0).then(|| delta_differ[index] as f64 / selected_total as f64)
        })
        .collect();
    let delta_d = delta[arm_index(Arm::D)];
    let delta_u = delta[arm_index(Arm::U)];
    let delta_g = delta[arm_index(Arm::G)];
    let delta_k = delta[arm_index(Arm::K)];
    let magnitude_load_bearing = matches!(
        (delta_d, delta_u),
        (Some(d), Some(u)) if d >= 0.05 && u >= d / 2.0
    );
    let gain_restores = matches!(
        (delta_d, delta_g, delta_k),
        (Some(d), Some(g), Some(k)) if g <= d / 2.0 && g <= k + 0.01
    );
    let outcome = match (delta_d, delta_u, delta_g, delta_k) {
        (Some(d), Some(u), Some(g), Some(k)) => {
            if d < 0.05 {
                "REPRESENTATION-ADEQUATE"
            } else if u < d / 2.0 {
                "RESOLUTION"
            } else if g <= d / 2.0 && g <= k + 0.01 {
                "GAIN"
            } else {
                "ORDINARY-BETTER"
            }
        }
        _ => "UNAVAILABLE",
    };
    report["population"] = json!({
        "selected_read_definition": format!("comparison-tail positions with previous >= 1 and NoRead mass < {SELECTED_READ_THRESHOLD}; NoRead mass = 1 - sum(read masses)"),
        "comparison_blocks": comparison_blocks,
        "positions_total": positions_total,
        "selected_read_positions": selected_total,
        "excluded_by_no_read": positions_total - selected_total,
        "no_read_mass_mean": (positions_total != 0).then(|| no_read_sum / positions_total as f64),
        "no_read_mass_maximum": no_read_max,
        "top1_definition": "argmax over candidate events of (arm score + that event's learned age bias from read.age)"
    });
    report["delta"] = json!(ARM_ORDER
        .iter()
        .map(|arm| {
            let index = arm_index(*arm);
            json!({
                "arm": arm.label(),
                "kind": arm.kind(),
                "positions": selected_total,
                "top1_differs_from_O": delta_differ[index],
                "delta": delta[index]
            })
        })
        .collect::<Vec<_>>());
    report["outcome_logic"] = json!({
        "magnitude_load_bearing": magnitude_load_bearing,
        "gain_restores": gain_restores,
        "criteria": {
            "magnitude_load_bearing": "delta(D) >= 0.05 and delta(U) >= delta(D)/2",
            "gain_restores": "delta(G) <= delta(D)/2 and delta(G) <= delta(K) + 0.01"
        },
        "table": {
            "GAIN": "magnitude load-bearing and the gain restores it",
            "RESOLUTION": "delta(D) >= 0.05 and delta(U) < delta(D)/2",
            "ORDINARY-BETTER": "magnitude load-bearing and G fails either restoration criterion",
            "REPRESENTATION-ADEQUATE": "delta(D) < 0.05"
        }
    });
    report["outcome"] = json!(outcome);
    report["reported_not_gated"] = json!({
        "top2_code_collisions": {
            "definition": "fraction of selected-read positions where the arm's top-1 and top-2 ranked events share an identical lane-code tuple; undefined for O and U (no lane code)",
            "D": (collision_eligible[arm_index(Arm::D)] != 0).then(|| collision_top2[arm_index(Arm::D)] as f64 / collision_eligible[arm_index(Arm::D)] as f64),
            "G": (collision_eligible[arm_index(Arm::G)] != 0).then(|| collision_top2[arm_index(Arm::G)] as f64 / collision_eligible[arm_index(Arm::G)] as f64),
            "K": (collision_eligible[arm_index(Arm::K)] != 0).then(|| collision_top2[arm_index(Arm::K)] as f64 / collision_eligible[arm_index(Arm::K)] as f64),
            "eligible_positions": collision_eligible[arm_index(Arm::D)]
        },
        "o_top1_vs_model_read_consistency": {
            "definition": "selected-read positions where the computed dot-score argmax differs from the model's own read-mass argmax",
            "mismatches": o_mismatch,
            "positions": selected_total
        },
        "gain_distribution": {
            "histogram_code_0_to_7": gain_histogram,
            "low_clip": gain_low_clip,
            "high_clip": gain_high_clip,
            "low_clip_fraction": (gain_histogram.iter().sum::<u64>() != 0).then(|| gain_low_clip as f64 / gain_histogram.iter().sum::<u64>() as f64),
            "high_clip_fraction": (gain_histogram.iter().sum::<u64>() != 0).then(|| gain_high_clip as f64 / gain_histogram.iter().sum::<u64>() as f64),
            "total": gain_histogram.iter().sum::<u64>(),
            "definition": "raw query (selected-read positions) and key (all comparison positions) lane norms encoded during the comparison pass; clipping is the fraction of lanes outside the eight declared dyadic exponents"
        }
    });

    let panel_started = Instant::now();
    let panel = if config.anchor_only {
        json!({"status": "SKIPPED_ANCHOR_ONLY"})
    } else {
        let probes = story_probes();
        let mut rows: Vec<PanelRowResult> = Vec::with_capacity(32);
        for probe in &probes {
            for (variant, selection) in [("original", &probe.original), ("edited", &probe.edited)] {
                rows.push(capture_panel_row(
                    &model,
                    &encoder,
                    &tokenizer,
                    &checkpoint_age,
                    &probe.id,
                    variant,
                    selection.item.noun(),
                    &selection.prompt,
                )?);
            }
        }
        let mut correct = [0u64; ARM_ORDER.len()];
        let mut selected_rows = 0u64;
        let mut records = Vec::with_capacity(rows.len());
        for row in &rows {
            if row.selected {
                selected_rows += 1;
            }
            for arm in ARM_ORDER {
                if row.top1[arm_index(arm)] == Some(row.entity_occurrence) {
                    correct[arm_index(arm)] += 1;
                }
            }
            records.push(json!({
                "probe": row.probe, "variant": row.variant, "item": row.item,
                "entity_occurrence": row.entity_occurrence, "previous": row.previous,
                "no_read_mass": row.no_read, "selected_read": row.selected,
                "top1": ARM_ORDER.iter().map(|arm| row.top1[arm_index(*arm)]).collect::<Vec<_>>()
            }));
        }
        let total = rows.len() as u64;
        json!({
            "definition": "correct-source top-1 = the arm's decision-0 top-1 read event equals the prompt occurrence of the item noun; offline from the same capture, no generation",
            "rows": total,
            "selected_read_rows": selected_rows,
            "correct_source_top1": ARM_ORDER.iter().map(|arm| json!({
                "arm": arm.label(),
                "correct": correct[arm_index(*arm)],
                "fraction": (total != 0).then(|| correct[arm_index(*arm)] as f64 / total as f64)
            })).collect::<Vec<_>>(),
            "per_row": records
        })
    };
    report["panel"] = panel;
    report["panel_seconds"] = json!(panel_started.elapsed().as_secs_f64());
    report["generation_complete_answers"] = json!({
        "status": "NOT_RUN",
        "reason": "generating complete answers with a substituted arm score needs a scorer-substitution model hook that does not exist; D6 intentionally builds no such hook. The retained root evaluate-quaternion-2i-off-read-8a6d records the parent's 27/32 complete answers."
    });
    report["timing"] = json!({
        "tune_model_seconds": tune_model_seconds,
        "comparison_model_seconds": comparison_model_seconds,
        "comparison_arm_seconds": arm_seconds,
        "comparison_seconds": compare_started.elapsed().as_secs_f64(),
        "total_seconds": started.elapsed().as_secs_f64(),
        "note": "model seconds bound only joint_model::step calls; arm seconds bound encoding, scoring and accumulation; the K fit and JSON work are harness overhead"
    });
    report["elapsed_seconds"] = json!(started.elapsed().as_secs_f64());
    report["status"] = json!("COMPLETE");
    save_json(&config.out.join("d6-information-audit.json"), &report)?;
    Ok(())
}

fn main() -> Result<()> {
    let config = parse_args()?;
    let runtime = Config {
        campaign: config.campaign.clone(),
        checkpoint: config.checkpoint.canonicalize()?,
        out: config.out.clone(),
        smoke: config.smoke,
        anchor_only: config.anchor_only,
    };
    let out = if runtime.out.is_absolute() {
        runtime.out.clone()
    } else {
        std::env::current_dir()?.join(&runtime.out)
    };
    if out.file_name().is_none() || out.starts_with(&runtime.checkpoint) {
        return Err(invalid(
            "new report must be outside the sealed checkpoint root",
        ));
    }
    let parent = out
        .parent()
        .ok_or_else(|| invalid("new report has no parent directory"))?;
    std::fs::create_dir_all(parent)?;
    if parent.canonicalize()?.starts_with(&runtime.checkpoint) {
        return Err(invalid(
            "new report must be outside the sealed checkpoint root",
        ));
    }
    let runtime = Config { out, ..runtime };
    report_output::claim(&runtime.out)?;
    let result = run(&runtime);
    if let Err(error) = &result {
        save_json(
            &runtime.out.join("failed-attempt.json"),
            &json!({"status":"FAILED_ATTEMPT","error":error.to_string()}),
        )?;
    }
    report_output::seal(&runtime.out)?;
    report_output::verify(&runtime.out)?;
    result?;
    println!("COMPLETE: {}; sealed and verified", runtime.out.display());
    Ok(())
}
