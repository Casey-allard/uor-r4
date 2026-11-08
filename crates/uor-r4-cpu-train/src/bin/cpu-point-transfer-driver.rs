use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Instant;

use rayon::prelude::*;
use serde::Deserialize;
use uor_r4_core::answer_oracle::FrozenAnswers;
use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_memory_weight_samples, derive_turn_memory,
    evaluate_memory_weight_samples, read_uort_token_slice, validate_corpus_manifest,
    CpuConversationSample, CpuFbsForward, CpuFbsHead, CpuMaskedCorpus, MemoryWeightSample,
};
use uor_r4_integer::stack::{
    add_point_key_uorlut01, build_point_key, read_memory_uorlut01, reply_stop, FbsMemorySnapshot,
    IntegerStackModel, PointKeyImage, PointKeyWindow, ReplyEnd, POINT_KEY_BOS,
};
use uor_r4_tokenizer::dialogue::{DialogueEncoder, DialogueProtocol, Message, SCHEMA_V2};
use uor_r4_tokenizer::ByteBpeTokenizer;

const SCHEMA: &str = "uor-r4.cpu-point-transfer-run/1";
const PANEL_SCHEMA: &str = "uor-r4.cpu-opening-panel/1";
const ONE_USER_RENDERING: &str = "one-user-turn/v1";
const PER_TURN_RENDERING: &str = "per-turn/v1";
const PANEL_EOS: u32 = 1;
const PANEL_NGRAM: usize = 6;

#[derive(Clone)]
struct PanelConfig {
    inputs: PathBuf,
    labels: PathBuf,
    plan: PathBuf,
    manifest: PathBuf,
    tokenizer: PathBuf,
    rendering: String,
    train_examples: usize,
    expected_inputs_sha256: String,
    expected_labels_sha256: String,
    expected_plan_sha256: String,
    expected_manifest_sha256: String,
    expected_tokenizer_sha256: String,
}

struct PanelTrainSource<'a> {
    token_bytes: &'a [u8],
    mask: &'a [u8],
    tokenizer_sha256: &'a str,
    examples: &'a [CpuConversationSample],
}

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    train_keys: usize,
    evaluation_keys: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    random_seed: u64,
    neighbor_vote: usize,
    target_prototypes: usize,
    reply_cap: usize,
    select_only: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
    panel: Option<PanelConfig>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyGroup {
    key: Vec<u32>,
    samples: Vec<CpuConversationSample>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct OpeningReading {
    decisions: u64,
    top1_correct: u64,
    target_distance_sum: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OpeningOutcome {
    top1: bool,
    target_distance: u64,
    token: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct NeighborReading {
    decisions: u64,
    nearest_top1: u64,
    vote_top1: u64,
    random_vote_top1: u64,
    nearest_matches_served: u64,
    voted_matches_served: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct ReplyReading {
    replies: u64,
    bos_complete: u64,
    cap_stopped: u64,
    context_stopped: u64,
    exact_reference: u64,
    reference_reachable: u64,
    opening_correct: u64,
    non_empty: u64,
    distinct_non_empty: u64,
    repeated_4gram: u64,
    generated_tokens: u64,
    reference_tokens: u64,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
struct TimedReplyReading {
    reading: ReplyReading,
    reply_sha256: String,
    wall_elapsed_us: u64,
    generated_tokens_per_second: f64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct GeneratedReply {
    tokens: Vec<u32>,
    stop: &'static str,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-point-transfer-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-point-transfer-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--train-keys N] [--evaluation-keys N] [--per-key N] \
     [--key-tokens N] [--reply-turn N] [--random-seed N] \
     [--neighbor-vote N] [--target-prototypes N] [--reply-cap N] [--select-only] \
     [--train-examples N --rendering one-user-turn/v1|per-turn/v1 \
      --panel-inputs PATH --panel-labels PATH --panel-plan PATH \
      --panel-manifest PATH --panel-tokenizer PATH \
      --expected-panel-inputs-sha256 HEX --expected-panel-labels-sha256 HEX \
      --expected-panel-plan-sha256 HEX --expected-panel-manifest-sha256 HEX \
      --expected-panel-tokenizer-sha256 HEX]"
        .to_owned()
}

fn parse_args() -> Result<Config, String> {
    let mut values = std::env::args().skip(1);
    let artifact = PathBuf::from(values.next().ok_or_else(usage)?);
    let tokens = PathBuf::from(values.next().ok_or_else(usage)?);
    let mask = PathBuf::from(values.next().ok_or_else(usage)?);
    let manifest = PathBuf::from(values.next().ok_or_else(usage)?);
    let out = PathBuf::from(values.next().ok_or_else(usage)?);
    let mut config = Config {
        artifact,
        tokens,
        mask,
        manifest,
        out,
        train_keys: 8,
        evaluation_keys: 8,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        random_seed: 1,
        neighbor_vote: 0,
        target_prototypes: 0,
        reply_cap: 0,
        select_only: false,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
        panel: None,
    };
    let mut panel_paths = [None, None, None, None, None];
    let mut panel_hashes = [None, None, None, None, None];
    let mut panel_rendering = ONE_USER_RENDERING.to_owned();
    let mut panel_train_examples = 0usize;
    let mut panel_requested = false;
    while let Some(flag) = values.next() {
        if flag == "--select-only" {
            config.select_only = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--train-keys" => config.train_keys = number(&value)?,
            "--evaluation-keys" => config.evaluation_keys = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--random-seed" => config.random_seed = number(&value)?,
            "--neighbor-vote" => config.neighbor_vote = number(&value)?,
            "--target-prototypes" => config.target_prototypes = number(&value)?,
            "--reply-cap" => config.reply_cap = number(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
            "--panel-inputs" => {
                panel_requested = true;
                panel_paths[0] = Some(PathBuf::from(value));
            }
            "--panel-labels" => {
                panel_requested = true;
                panel_paths[1] = Some(PathBuf::from(value));
            }
            "--panel-plan" => {
                panel_requested = true;
                panel_paths[2] = Some(PathBuf::from(value));
            }
            "--panel-manifest" => {
                panel_requested = true;
                panel_paths[3] = Some(PathBuf::from(value));
            }
            "--panel-tokenizer" => {
                panel_requested = true;
                panel_paths[4] = Some(PathBuf::from(value));
            }
            "--expected-panel-inputs-sha256" => {
                panel_requested = true;
                panel_hashes[0] = Some(hex(&value)?);
            }
            "--expected-panel-labels-sha256" => {
                panel_requested = true;
                panel_hashes[1] = Some(hex(&value)?);
            }
            "--expected-panel-plan-sha256" => {
                panel_requested = true;
                panel_hashes[2] = Some(hex(&value)?);
            }
            "--expected-panel-manifest-sha256" => {
                panel_requested = true;
                panel_hashes[3] = Some(hex(&value)?);
            }
            "--expected-panel-tokenizer-sha256" => {
                panel_requested = true;
                panel_hashes[4] = Some(hex(&value)?);
            }
            "--rendering" => {
                panel_requested = true;
                panel_rendering = value;
            }
            "--train-examples" => {
                panel_requested = true;
                panel_train_examples = number(&value)?;
            }
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        }
    }
    for (name, value) in [
        ("artifact", &config.expected_artifact_sha256),
        ("tokens", &config.expected_tokens_sha256),
        ("mask", &config.expected_mask_sha256),
        ("manifest", &config.expected_manifest_sha256),
    ] {
        if value.is_empty() {
            return Err(format!("--expected-{name}-sha256 is required"));
        }
    }
    if config.train_keys == 0
        || config.evaluation_keys == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
        || !(config.target_prototypes == 0 || (2..=16).contains(&config.target_prototypes))
        || config.reply_cap > 64
    {
        return Err("group dimensions must be positive and bounded".to_owned());
    }
    if panel_requested {
        if panel_rendering != ONE_USER_RENDERING && panel_rendering != PER_TURN_RENDERING {
            return Err(format!(
                "--rendering wants {ONE_USER_RENDERING} or {PER_TURN_RENDERING}"
            ));
        }
        let path_names = [
            "panel-inputs",
            "panel-labels",
            "panel-plan",
            "panel-manifest",
            "panel-tokenizer",
        ];
        let hash_names = [
            "expected-panel-inputs-sha256",
            "expected-panel-labels-sha256",
            "expected-panel-plan-sha256",
            "expected-panel-manifest-sha256",
            "expected-panel-tokenizer-sha256",
        ];
        for (index, name) in path_names.iter().enumerate() {
            if panel_paths[index].is_none() {
                return Err(format!("--{name} is required in panel mode"));
            }
            if panel_hashes[index].is_none() {
                return Err(format!("--{} is required in panel mode", hash_names[index]));
            }
        }
        let [Some(inputs), Some(labels), Some(plan), Some(manifest), Some(tokenizer)] = panel_paths
        else {
            unreachable!("all panel paths were checked");
        };
        let [Some(expected_inputs_sha256), Some(expected_labels_sha256), Some(expected_plan_sha256), Some(expected_manifest_sha256), Some(expected_tokenizer_sha256)] =
            panel_hashes
        else {
            unreachable!("all panel hashes were checked");
        };
        config.panel = Some(PanelConfig {
            inputs,
            labels,
            plan,
            manifest,
            tokenizer,
            rendering: panel_rendering,
            train_examples: panel_train_examples,
            expected_inputs_sha256,
            expected_labels_sha256,
            expected_plan_sha256,
            expected_manifest_sha256,
            expected_tokenizer_sha256,
        });
    } else if panel_train_examples != 0 || panel_rendering != ONE_USER_RENDERING {
        return Err("panel flags were parsed without panel mode".to_owned());
    }
    Ok(config)
}

fn number<T: std::str::FromStr>(value: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("not a number: {value}"))
}

fn hex(value: &str) -> Result<String, String> {
    let value = value.to_ascii_lowercase();
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(format!("not a SHA-256: {value}"))
    }
}

fn run() -> Result<serde_json::Value, String> {
    let config = parse_args()?;
    if config.out.exists() {
        return Err(format!(
            "output root already exists: {}",
            config.out.display()
        ));
    }
    let started = Instant::now();
    let artifact = std::fs::read(&config.artifact).map_err(|error| error.to_string())?;
    let tokens = std::fs::read(&config.tokens).map_err(|error| error.to_string())?;
    let mask = std::fs::read(&config.mask).map_err(|error| error.to_string())?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| error.to_string())?;
    for (name, bytes, expected) in [
        (
            "artifact",
            artifact.as_slice(),
            &config.expected_artifact_sha256,
        ),
        ("tokens", tokens.as_slice(), &config.expected_tokens_sha256),
        ("mask", mask.as_slice(), &config.expected_mask_sha256),
        (
            "manifest",
            manifest.as_slice(),
            &config.expected_manifest_sha256,
        ),
    ] {
        let actual = uor_r4_discrete::corpus::sha256(bytes);
        if &actual != expected {
            return Err(format!("{name} SHA-256 differs: got {actual}"));
        }
    }
    let identity =
        validate_corpus_manifest(&manifest, &tokens, &mask).map_err(|error| error.to_string())?;
    if config.panel.is_some() && identity.split != "train" {
        return Err(format!(
            "panel mode requires a TRAIN corpus, found split {:?}",
            identity.split
        ));
    }
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    let shape = model.shape();
    if identity.vocab != shape.vocab || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the artifact".to_owned());
    }
    if read_memory_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("the public source artifact already carries memory".to_owned());
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&tokens, &mask, shape.vocab, shape.context, identity.bos)
            .map_err(|error| error.to_string())?;
    let examples = match config.panel.as_ref() {
        Some(panel) if panel.train_examples > 0 => corpus
            .conversation_examples_capped(panel.train_examples)
            .map_err(|error| error.to_string())?,
        _ => corpus
            .conversation_examples()
            .map_err(|error| error.to_string())?,
    };
    if let Some(panel) = &config.panel {
        let train_source = PanelTrainSource {
            token_bytes: &tokens,
            mask: &mask,
            tokenizer_sha256: &identity.tokenizer_sha256,
            examples: &examples,
        };
        return run_panel_probe(
            &config,
            panel,
            &artifact,
            &train_source,
            shape.width,
            shape.vocab,
            shape.context,
        );
    }
    let (training, evaluation) = select_transfer_groups(&examples, &config)?;
    let training_samples = training
        .iter()
        .flat_map(|group| group.samples.iter())
        .cloned()
        .collect::<Vec<_>>();
    let evaluation_samples = evaluation
        .iter()
        .flat_map(|group| group.samples.iter())
        .cloned()
        .collect::<Vec<_>>();
    let training_keys = training
        .iter()
        .map(|group| group.key.clone())
        .collect::<BTreeSet<_>>();
    if evaluation
        .iter()
        .any(|group| training_keys.contains(&group.key))
    {
        return Err("a held-out opening key overlaps training".to_owned());
    }
    if training_samples.len() < config.train_keys * 4
        || evaluation_samples.len() < config.evaluation_keys * 4
    {
        return Err("selected key groups have too few examples".to_owned());
    }
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-point-transfer-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "training_keys": training_keys,
            "evaluation_keys": evaluation.iter().map(|group| group.key.clone()).collect::<Vec<_>>(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "training_groups": training.iter().map(group_json).collect::<Vec<_>>(),
            "evaluation_groups": evaluation.iter().map(group_json).collect::<Vec<_>>(),
        }));
    }

    let weight_samples = training_samples
        .iter()
        .map(memory_sample)
        .collect::<Vec<_>>();
    let held_out = evaluation_samples
        .iter()
        .map(memory_sample)
        .collect::<Vec<_>>();
    let memory = derive_turn_memory(
        &weight_samples,
        shape.width,
        config.key_tokens,
        shape.context,
        config.reply_turn,
    )
    .map_err(|error| error.to_string())?;
    let memory_artifact =
        add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    assert_held_out_exact_keys_miss(&memory_artifact, &held_out)?;

    let point = derive_point_table(
        &memory_artifact,
        &training_samples,
        shape.width,
        shape.vocab,
    )?;
    let point_artifact =
        add_point_key_uorlut01(&memory_artifact, &point).map_err(|error| error.to_string())?;
    let medoid = derive_target_medoids(&point)?;
    let medoid_artifact =
        add_point_key_uorlut01(&memory_artifact, &medoid).map_err(|error| error.to_string())?;
    let prototypes = if config.target_prototypes == 0 {
        None
    } else {
        Some(derive_target_prototypes(&point, config.target_prototypes)?)
    };
    if let Some(prototypes) = &prototypes {
        if prototypes.rows() >= point.rows()
            || (prototypes.points == point.points && prototypes.targets == point.targets)
        {
            return Err("target prototypes do not reduce the raw point neighborhood".to_owned());
        }
    }
    let prototype_artifact = prototypes
        .as_ref()
        .map(|image| add_point_key_uorlut01(&memory_artifact, image))
        .transpose()
        .map_err(|error| error.to_string())?;
    let voted_point = (config.neighbor_vote > 1).then(|| PointKeyImage {
        width: point.width,
        points: point.points.clone(),
        targets: point.targets.clone(),
        vote_k: config.neighbor_vote,
    });
    let voted_artifact = voted_point
        .as_ref()
        .map(|image| add_point_key_uorlut01(&memory_artifact, image))
        .transpose()
        .map_err(|error| error.to_string())?;
    let random_point = PointKeyImage {
        width: point.width,
        points: point.points.clone(),
        targets: shuffled_targets(&point.targets, config.random_seed)?,
        vote_k: 1,
    };
    let random_artifact = add_point_key_uorlut01(&memory_artifact, &random_point)
        .map_err(|error| error.to_string())?;
    let random_medoid = PointKeyImage {
        width: medoid.width,
        points: medoid.points.clone(),
        targets: shuffled_targets(&medoid.targets, config.random_seed ^ 0x4D45_444F_4944)?,
        vote_k: 1,
    };
    let random_medoid_artifact = add_point_key_uorlut01(&memory_artifact, &random_medoid)
        .map_err(|error| error.to_string())?;
    let random_prototypes = prototypes
        .as_ref()
        .map(|prototypes| {
            shuffled_targets(&prototypes.targets, config.random_seed ^ 0x5052_4F54_4F54).map(
                |targets| PointKeyImage {
                    width: prototypes.width,
                    points: prototypes.points.clone(),
                    targets,
                    vote_k: 1,
                },
            )
        })
        .transpose()?;
    let random_prototype_artifact = match &random_prototypes {
        Some(image) => Some(
            add_point_key_uorlut01(&memory_artifact, image).map_err(|error| error.to_string())?,
        ),
        None => None,
    };

    std::fs::create_dir_all(&config.out).map_err(|error| error.to_string())?;
    let mut outputs = vec![
        ("memory-only.uorlut", memory_artifact.as_slice()),
        ("point.uorlut", point_artifact.as_slice()),
        ("point-medoid.uorlut", medoid_artifact.as_slice()),
        ("random-point.uorlut", random_artifact.as_slice()),
        (
            "random-point-medoid.uorlut",
            random_medoid_artifact.as_slice(),
        ),
    ];
    if let Some(bytes) = &prototype_artifact {
        outputs.push(("point-prototypes.uorlut", bytes.as_slice()));
    }
    if let Some(bytes) = &random_prototype_artifact {
        outputs.push(("random-point-prototypes.uorlut", bytes.as_slice()));
    }
    if let Some(bytes) = &voted_artifact {
        outputs.push(("point-vote.uorlut", bytes.as_slice()));
    }
    for (name, bytes) in outputs {
        std::fs::write(config.out.join(name), bytes).map_err(|error| error.to_string())?;
    }

    let mut arms = vec![
        ("point_off", memory_artifact.as_slice()),
        ("point", point_artifact.as_slice()),
        ("random", random_artifact.as_slice()),
        ("point_medoid", medoid_artifact.as_slice()),
        ("random_medoid", random_medoid_artifact.as_slice()),
    ];
    if let Some(bytes) = &prototype_artifact {
        arms.push(("point_prototypes", bytes.as_slice()));
    }
    if let Some(bytes) = &random_prototype_artifact {
        arms.push(("random_prototypes", bytes.as_slice()));
    }
    if let Some(bytes) = &voted_artifact {
        arms.push(("point_vote", bytes.as_slice()));
    }
    let opening_outcomes = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            evaluate_opening_outcomes(bytes, &evaluation_samples, shape.width)
                .map(|outcomes| (name, outcomes))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let opening = opening_outcomes
        .iter()
        .map(|(name, outcomes)| {
            let reading = summarize_openings(outcomes)?;
            Ok((*name, reading))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    let baseline_outcomes = &opening_outcomes["point_off"];
    if opening_outcomes
        .values()
        .any(|outcomes| !opening_distances_equal(outcomes, baseline_outcomes))
    {
        return Err("point fallback changed an opening query distance".to_owned());
    }
    let point_fallback_exposure = arms
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| *name != "point_off")
        .map(|name| {
            (
                name,
                changed_openings(&opening_outcomes[name], &opening_outcomes["point_off"]),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if point_fallback_exposure
        .values()
        .any(|&changed| changed == 0)
    {
        return Err("a point-key arm never changed the served opening".to_owned());
    }
    if prototypes.is_some() {
        if point_fallback_exposure["point_prototypes"] != evaluation_samples.len() {
            return Err(
                "the target-prototype arm did not change every held-out opening".to_owned(),
            );
        }
        if changed_openings(
            &opening_outcomes["random_prototypes"],
            &opening_outcomes["point_prototypes"],
        ) == 0
        {
            return Err(
                "the shuffled-target prototype control named the treatment's opening every time"
                    .to_owned(),
            );
        }
    }
    let neighbor = if config.neighbor_vote == 0 {
        None
    } else {
        if config.neighbor_vote > point.rows() {
            return Err("neighbor vote exceeds the point-table row count".to_owned());
        }
        Some(
            evaluate_neighbor_vote(
                &memory_artifact,
                &point_artifact,
                voted_artifact.as_deref(),
                &point,
                &random_point,
                &evaluation_samples,
                config.neighbor_vote,
            )
            .map_err(|error| error.to_string())?,
        )
    };
    if neighbor.is_some_and(|reading| {
        reading.nearest_matches_served != reading.decisions
            || reading.voted_matches_served != reading.decisions
    }) {
        return Err("point extraction differs from the served point key".to_owned());
    }
    let complete_replies = if config.reply_cap == 0 {
        BTreeMap::new()
    } else {
        arms.iter()
            .copied()
            .map(|(name, bytes)| {
                evaluate_replies(bytes, &evaluation_samples, config.reply_cap, shape.context)
                    .map(|reading| (name, reading))
                    .map_err(|error| error.to_string())
            })
            .collect::<Result<BTreeMap<_, _>, String>>()?
    };
    let full = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            evaluate_memory_weight_samples(bytes, &held_out, true)
                .map(|reading| (name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let parity = arms
        .iter()
        .copied()
        .map(|(name, bytes)| {
            audit_memory_weight_samples(bytes, &held_out, true)
                .map(|reading| (name, reading))
                .map_err(|error| error.to_string())
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    for (name, reading) in &parity {
        if !reading.is_exact() {
            return Err(format!("{name} lost CPU/serving parity"));
        }
    }
    let baseline_hits = opening["point_off"].top1_correct;
    let n = opening["point_off"].decisions;
    let p = baseline_hits as f64 / n as f64;
    let sigma = 3.0 * (n as f64 * p * (1.0 - p)).sqrt();
    let threshold = sigma.ceil() as u64;
    let neighbor_three_sigma = neighbor.map(|reading| {
        let p = reading.nearest_top1 as f64 / reading.decisions as f64;
        let threshold = (3.0 * (reading.decisions as f64 * p * (1.0 - p)).sqrt()).ceil() as u64;
        serde_json::json!({
            "baseline_nearest": reading.nearest_top1,
            "decisions": reading.decisions,
            "vote_delta": reading.vote_top1 as i64 - reading.nearest_top1 as i64,
            "threshold": threshold,
        })
    });
    let medoid_vs_point_off = paired_opening_comparison(
        &opening_outcomes["point_medoid"],
        &opening_outcomes["point_off"],
    )?;
    let medoid_vs_random = paired_opening_comparison(
        &opening_outcomes["point_medoid"],
        &opening_outcomes["random_medoid"],
    )?;
    let medoid_full_not_lower = full["point_medoid"].top1_correct >= full["point_off"].top1_correct
        && full["point_medoid"].top1_correct >= full["random_medoid"].top1_correct;
    let prototype_comparison = match (
        opening_outcomes.get("point_prototypes"),
        opening_outcomes.get("random_prototypes"),
    ) {
        (Some(prototypes), Some(random_prototypes)) => {
            let vs_point_off =
                paired_opening_comparison(prototypes, &opening_outcomes["point_off"])?;
            let vs_random = paired_opening_comparison(prototypes, random_prototypes)?;
            let full_top1_not_lower = full["point_prototypes"].top1_correct
                >= full["point_off"].top1_correct
                && full["point_prototypes"].top1_correct >= full["random_prototypes"].top1_correct;
            Some(serde_json::json!({
                "point_off": vs_point_off,
                "random_prototypes": vs_random,
                "full_top1_not_lower": full_top1_not_lower,
                "passes": vs_point_off["passes"].as_bool() == Some(true)
                    && vs_random["passes"].as_bool() == Some(true)
                    && full_top1_not_lower,
            }))
        }
        (None, None) => None,
        _ => return Err("target-prototype arms were built incompletely".to_owned()),
    };
    let report = serde_json::json!({
        "schema": SCHEMA,
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": {
            "artifact": config.artifact,
            "artifact_sha256": config.expected_artifact_sha256,
            "tokens": config.tokens,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask": config.mask,
            "mask_sha256": config.expected_mask_sha256,
            "manifest": config.manifest,
            "manifest_sha256": config.expected_manifest_sha256,
        },
        "selection": {
            "key_tokens": config.key_tokens,
            "per_key": config.per_key,
            "training_keys": training_keys,
            "evaluation_keys": evaluation.iter().map(|group| group.key.clone()).collect::<Vec<_>>(),
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "training_documents": training_samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>().len(),
            "evaluation_documents": evaluation_samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>().len(),
        },
        "memory": {
            "k": memory.k,
            "key_window": memory.key_window,
            "opening_rows": memory.opening.len(),
            "reply_rows": memory.reply.as_ref().map_or(0, |reply| reply.entries.len()),
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "held_out_exact_key_hits": 0,
        },
        "point": {
            "rows": point.rows(),
            "width": point.width,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&point_artifact),
            "voted_artifact_sha256": voted_artifact
                .as_ref()
                .map(|bytes| uor_r4_discrete::corpus::sha256(bytes)),
            "random_artifact_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
            "random_seed": config.random_seed,
        },
        "target_medoid": {
            "rows": medoid.rows(),
            "targets": medoid.targets,
            "width": medoid.width,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&medoid_artifact),
            "random_artifact_sha256": uor_r4_discrete::corpus::sha256(&random_medoid_artifact),
            "source_rows": point.rows(),
        },
        "target_prototypes": {
            "k": config.target_prototypes,
            "rows": prototypes.as_ref().map(PointKeyImage::rows),
            "targets": prototypes.as_ref().map(|image| image.targets.clone()),
            "width": prototypes.as_ref().map(|image| image.width),
            "artifact_sha256": prototype_artifact
                .as_ref()
                .map(|bytes| uor_r4_discrete::corpus::sha256(bytes)),
            "random_artifact_sha256": random_prototype_artifact
                .as_ref()
                .map(|bytes| uor_r4_discrete::corpus::sha256(bytes)),
            "source_rows": point.rows(),
            "comparison": prototype_comparison,
        },
        "medoid_comparison": {
            "point_off": medoid_vs_point_off,
            "random_medoid": medoid_vs_random,
            "full_top1_not_lower": medoid_full_not_lower,
            "passes": medoid_vs_point_off["passes"].as_bool() == Some(true)
                && medoid_vs_random["passes"].as_bool() == Some(true)
                && medoid_full_not_lower,
        },
        "point_fallback_exposure": point_fallback_exposure,
        "neighbor_vote": {
            "k": config.neighbor_vote,
            "reading": neighbor,
        },
        "complete_replies": {
            "cap": config.reply_cap,
            "arms": complete_replies,
        },
        "opening": opening,
        "full_teacher_forced": full,
        "parity": parity,
        "three_sigma": {
            "baseline_top1": baseline_hits,
            "decisions": n,
            "point_delta": opening["point"].top1_correct as i64 - baseline_hits as i64,
            "random_delta": opening["random"].top1_correct as i64 - baseline_hits as i64,
            "threshold": threshold,
        },
        "neighbor_three_sigma": neighbor_three_sigma,
        "threads": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib()?,
        "claim": "CPU development-research transfer test; not GPU qualification or language capability",
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelInputs {
    schema: String,
    cases: Vec<PanelInputCase>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelInputCase {
    #[serde(default)]
    #[serde(rename = "actual_prefix_ids")]
    _actual_prefix_ids: Vec<u32>,
    id: String,
    query_ids: Vec<u32>,
    segments: Vec<PanelSegment>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelSegment {
    kind: String,
    event: u64,
    #[serde(default)]
    token_ids: Vec<u32>,
    #[serde(default)]
    #[serde(rename = "role")]
    _role: Option<u64>,
    #[serde(default)]
    #[serde(rename = "commit")]
    _commit: Option<u64>,
    #[serde(default)]
    #[serde(rename = "entity")]
    _entity: Vec<u64>,
    #[serde(default)]
    #[serde(rename = "original_source_ids")]
    _original_source_ids: Vec<u32>,
    #[serde(default)]
    #[serde(rename = "record")]
    _record: Option<u64>,
    #[serde(default)]
    #[serde(rename = "relation")]
    _relation: Option<u64>,
    #[serde(default)]
    #[serde(rename = "scope")]
    _scope: Option<String>,
    #[serde(default)]
    #[serde(rename = "view")]
    _view: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelLabels {
    cases: Vec<PanelLabelCase>,
    membership_only: bool,
    protocol: String,
    schema: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelLabelCase {
    answers: FrozenAnswers,
    id: String,
    pair_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelPlan {
    schema: String,
    source_policy: String,
    split: String,
    histories: Vec<PanelHistory>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelHistory {
    id: String,
    stratum: String,
    turns: Vec<PanelTextRow>,
    queries: Vec<PanelTextRow>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelTextRow {
    #[serde(rename = "act")]
    _act: String,
    #[serde(rename = "relation")]
    _relation: String,
    #[serde(rename = "template")]
    _template: String,
    text: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelManifest {
    excluded_historical_exposures: PanelExcludedExposures,
    files: BTreeMap<String, PanelManifestFile>,
    histories: usize,
    issue: String,
    milestone: String,
    milestone_issue: String,
    model_predictions: String,
    opposite_query_pairs: usize,
    ordered_role_value_banks: usize,
    panel_id: String,
    rows: usize,
    schema: String,
    target_bar_metric: String,
    target_bar_of: usize,
    target_bar_pairs: usize,
    target_bar_pairs_of: usize,
    target_bar_rows: usize,
    tokenizer_sha256: String,
    strata_rows: BTreeMap<String, usize>,
    unordered_bank_blocks: usize,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelManifestFile {
    bytes: usize,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PanelExcludedExposures {
    dev512_inputs_sha256: String,
    historical_exposed_literals: usize,
    opened_fresh128_inputs_sha256: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize)]
struct PanelOverlapCounts {
    documents: usize,
    assistant_replies: usize,
    exact_panel_in_document: usize,
    exact_accepted_assistant_reply: usize,
    documents_with_panel_6gram: usize,
    assistant_replies_with_panel_6gram: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PanelOverlapAudit {
    counts: PanelOverlapCounts,
    pass: bool,
}

#[derive(Clone, Copy, Debug, Default, serde::Serialize)]
struct PanelOpeningTally {
    rows: u64,
    raw_top1: u64,
    point_changed_raw: u64,
    point_top1: u64,
    point_top8: u64,
    point_top32: u64,
    shuffled_top1: u64,
    shuffled_top8: u64,
    shuffled_changed_raw: u64,
    constant_top1: u64,
}

#[derive(serde::Serialize)]
struct PanelOpeningRow {
    id: String,
    stratum: String,
    prompt_tokens: usize,
    target_token: u32,
    raw_token: u32,
    point_token: u32,
    shuffled_token: u32,
    constant_token: u32,
    point_top8: bool,
    point_top32: bool,
    shuffled_top8: bool,
}

fn run_panel_probe(
    config: &Config,
    panel: &PanelConfig,
    artifact: &[u8],
    train: &PanelTrainSource<'_>,
    width: usize,
    vocab: usize,
    context: usize,
) -> Result<serde_json::Value, String> {
    let started = Instant::now();
    let training = training_openings(train.examples);
    if training.is_empty() {
        return Err("panel mode has no non-EOS reply openings in TRAIN".to_owned());
    }
    let empty_memory = FbsMemorySnapshot {
        k: 8,
        key_window: context,
        opening: Vec::new(),
        opening_weight: vec![0.0; width],
        reply: None,
    };
    let memory_artifact =
        add_turn_memory_uorlut01(artifact, &empty_memory).map_err(|error| error.to_string())?;
    let point = derive_point_table(&memory_artifact, &training, width, vocab)?;
    let random_point = PointKeyImage {
        width: point.width,
        points: point.points.clone(),
        targets: shuffled_targets(&point.targets, config.random_seed)?,
        vote_k: 1,
    };
    let point_artifact =
        add_point_key_uorlut01(&memory_artifact, &point).map_err(|error| error.to_string())?;
    let random_artifact = add_point_key_uorlut01(&memory_artifact, &random_point)
        .map_err(|error| error.to_string())?;
    let constant_token = constant_opening(&training)?;

    let read = |path: &PathBuf, expected: &str| -> Result<(Vec<u8>, String), String> {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let sha = uor_r4_discrete::corpus::sha256(&bytes);
        if sha != expected {
            return Err(format!("{} SHA-256 differs: got {sha}", path.display()));
        }
        Ok((bytes, sha))
    };
    let (inputs_bytes, inputs_sha) = read(&panel.inputs, &panel.expected_inputs_sha256)?;
    let (labels_bytes, labels_sha) = read(&panel.labels, &panel.expected_labels_sha256)?;
    let (plan_bytes, plan_sha) = read(&panel.plan, &panel.expected_plan_sha256)?;
    let (manifest_bytes, manifest_sha) = read(&panel.manifest, &panel.expected_manifest_sha256)?;
    let (tokenizer_bytes, tokenizer_sha) =
        read(&panel.tokenizer, &panel.expected_tokenizer_sha256)?;
    let manifest: PanelManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|error| format!("panel manifest: {error}"))?;
    let expected_strata = BTreeMap::from([
        ("length2".to_owned(), 64),
        ("length4".to_owned(), 64),
        ("length8".to_owned(), 64),
        ("reassert".to_owned(), 32),
        ("update".to_owned(), 32),
    ]);
    if manifest.schema != "uor-r4.m2-fresh256-panel-manifest/1"
        || manifest.panel_id != "m2-fresh256"
        || manifest.milestone != "m2"
        || manifest.milestone_issue != "#2030"
        || manifest.issue != "#2849"
        || manifest.rows != 256
        || manifest.histories != 128
        || manifest.opposite_query_pairs != 128
        || manifest.unordered_bank_blocks != 16
        || manifest.ordered_role_value_banks != 32
        || manifest.model_predictions != "NOT_RUN"
        || manifest.target_bar_metric != "pairs_both_complete_distinct"
        || manifest.target_bar_rows != 104
        || manifest.target_bar_of != 256
        || manifest.target_bar_pairs != 52
        || manifest.target_bar_pairs_of != 128
        || manifest
            .excluded_historical_exposures
            .historical_exposed_literals
            != 74
        || manifest.strata_rows != expected_strata
    {
        return Err("panel manifest identity, scope or row inventory differs".to_owned());
    }
    for hash in [
        manifest
            .excluded_historical_exposures
            .dev512_inputs_sha256
            .as_str(),
        manifest
            .excluded_historical_exposures
            .opened_fresh128_inputs_sha256
            .as_str(),
    ] {
        uor_r4_discrete::corpus::validate_sha256(hash)
            .map_err(|error| format!("panel exclusion identity: {error}"))?;
    }
    if manifest.files.len() != 3 {
        return Err("panel manifest file inventory differs".to_owned());
    }
    for (name, actual_sha, actual_bytes) in [
        ("inputs.json", inputs_sha.as_str(), inputs_bytes.len()),
        ("labels.json", labels_sha.as_str(), labels_bytes.len()),
        ("plan.json", plan_sha.as_str(), plan_bytes.len()),
    ] {
        if manifest
            .files
            .get(name)
            .is_none_or(|entry| entry.sha256.as_str() != actual_sha || entry.bytes != actual_bytes)
        {
            return Err(format!(
                "panel manifest files.{name} differs from the file read"
            ));
        }
    }
    if manifest.tokenizer_sha256 != tokenizer_sha {
        return Err("panel manifest tokenizer_sha256 differs".to_owned());
    }
    if train.tokenizer_sha256 != tokenizer_sha {
        return Err("TRAIN corpus and panel tokenizer identities differ".to_owned());
    }
    let inputs: PanelInputs =
        serde_json::from_slice(&inputs_bytes).map_err(|error| format!("panel inputs: {error}"))?;
    let labels: PanelLabels =
        serde_json::from_slice(&labels_bytes).map_err(|error| format!("panel labels: {error}"))?;
    let plan: PanelPlan =
        serde_json::from_slice(&plan_bytes).map_err(|error| format!("panel plan: {error}"))?;
    validate_panel_labels(&labels)?;
    if inputs.schema != "uor-r4.native-source-bank-probe-input/1"
        || plan.schema != "uor-r4.raw-natural-reader-construction-plan/1"
        || plan.split != "fresh"
        || plan.source_policy
            != "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2"
        || inputs.cases.len() != 256
        || labels.cases.len() != 256
        || plan.histories.len() != 128
        || inputs.cases.iter().any(|case| {
            !case._actual_prefix_ids.is_empty()
                || case.query_ids.is_empty()
                || case.segments.iter().any(|segment| {
                    !matches!(segment.kind.as_str(), "Context" | "Source")
                        || (segment.kind == "Context" && segment.token_ids.is_empty())
                })
        })
    {
        return Err("m2-fresh256 case or history count differs".to_owned());
    }
    let by_id = inputs
        .cases
        .iter()
        .map(|case| (case.id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let label_ids = labels
        .cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<BTreeSet<_>>();
    let pair_rows =
        labels
            .cases
            .iter()
            .fold(BTreeMap::<&str, usize>::new(), |mut counts, label| {
                *counts.entry(label.pair_id.as_str()).or_default() += 1;
                counts
            });
    if by_id.len() != inputs.cases.len()
        || label_ids.len() != labels.cases.len()
        || pair_rows.len() != 128
        || pair_rows.values().any(|&rows| rows != 2)
    {
        return Err("panel case, label or pair identities are not unique".to_owned());
    }
    let strata = plan
        .histories
        .iter()
        .map(|history| {
            (
                format!("fresh-{}", history.id),
                history.stratum.split('/').next().unwrap_or("").to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    if strata.len() != plan.histories.len() {
        return Err("panel history identities are not unique".to_owned());
    }
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| "panel tokenizer is not a byte-level BPE tokenizer".to_owned())?;
    let overlap =
        audit_panel_training_overlap(train.token_bytes, train.mask, &tokenizer, &plan, &labels)?;
    if !overlap.pass {
        return Err(format!(
            "panel/training overlap found: {:?}",
            overlap.counts
        ));
    }
    let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).map_err(|e| e.to_string())?;
    let encoder = protocol.bind(&tokenizer).map_err(|e| e.to_string())?;
    let mut totals = PanelOpeningTally::default();
    let mut per_stratum = BTreeMap::<String, PanelOpeningTally>::new();
    let mut observed_strata = BTreeMap::<String, usize>::new();
    let mut rows = Vec::with_capacity(labels.cases.len());
    let mut raw_query_mismatches = 0u64;
    let mut point_served_mismatches = 0u64;
    let mut shuffled_served_mismatches = 0u64;
    let mut point_vs_shuffled = Vec::with_capacity(labels.cases.len());
    let mut point_vs_raw = Vec::with_capacity(labels.cases.len());
    let mut point_vs_constant = Vec::with_capacity(labels.cases.len());
    let mut stratified_pairs =
        BTreeMap::<String, (Vec<(bool, bool)>, Vec<(bool, bool)>, Vec<(bool, bool)>)>::new();
    for label in &labels.cases {
        let case = by_id
            .get(label.id.as_str())
            .ok_or_else(|| format!("{}: no input case", label.id))?;
        let stratum = strata
            .get(&label.pair_id)
            .ok_or_else(|| format!("{}: no plan history", label.pair_id))?
            .clone();
        let (_, prompt) = render_panel_case(&tokenizer, &encoder, case, &panel.rendering)?;
        let (_, reference_ids) = panel_reference_ids(&encoder, &label.answers)?;
        let target = *reference_ids
            .first()
            .ok_or_else(|| format!("{}: empty reference", label.id))?;
        if target == POINT_KEY_BOS || target == PANEL_EOS {
            return Err(format!(
                "{}: reference starts at boundary token {target}",
                label.id
            ));
        }
        let (raw_query, raw_token) =
            opening_query_and_token_with_head(&memory_artifact, &prompt, CpuFbsHead::Nearest)?;
        let (point_query, point_served) =
            opening_query_and_token_with_head(&point_artifact, &prompt, CpuFbsHead::Nearest)?;
        let (shuffle_query, shuffled_served) =
            opening_query_and_token_with_head(&random_artifact, &prompt, CpuFbsHead::Nearest)?;
        if point_query != raw_query || shuffle_query != raw_query {
            raw_query_mismatches += 1;
        }
        let point_top = point_candidates(&point, &raw_query, 32)?;
        let shuffled_top = point_candidates(&random_point, &raw_query, 8)?;
        let point_token = *point_top
            .first()
            .ok_or_else(|| "point table produced no candidate".to_owned())?;
        let shuffled_token = *shuffled_top
            .first()
            .ok_or_else(|| "shuffled point table produced no candidate".to_owned())?;
        point_served_mismatches += u64::from(point_served != point_token);
        shuffled_served_mismatches += u64::from(shuffled_served != shuffled_token);
        let point_top8 = point_top.iter().take(8).any(|token| *token == target);
        let point_top32 = point_top.iter().take(32).any(|token| *token == target);
        let shuffled_top8 = shuffled_top.iter().take(8).any(|token| *token == target);
        let row = PanelOpeningRow {
            id: label.id.clone(),
            stratum: stratum.clone(),
            prompt_tokens: prompt.len(),
            target_token: target,
            raw_token,
            point_token,
            shuffled_token,
            constant_token,
            point_top8,
            point_top32,
            shuffled_top8,
        };
        add_panel_tally(&mut totals, &row);
        add_panel_tally(per_stratum.entry(stratum.clone()).or_default(), &row);
        *observed_strata.entry(stratum.clone()).or_default() += 1;
        let point_win = point_token == target;
        let shuffled_win = shuffled_token == target;
        let raw_win = raw_token == target;
        let constant_win = constant_token == target;
        point_vs_shuffled.push((point_win, shuffled_win));
        point_vs_raw.push((point_win, raw_win));
        point_vs_constant.push((point_win, constant_win));
        let (shuffled_pairs, raw_pairs, constant_pairs) =
            stratified_pairs.entry(stratum).or_default();
        shuffled_pairs.push((point_win, shuffled_win));
        raw_pairs.push((point_win, raw_win));
        constant_pairs.push((point_win, constant_win));
        rows.push(row);
    }
    if observed_strata != manifest.strata_rows {
        return Err(format!(
            "panel strata rows {observed_strata:?} differ from manifest {:?}",
            manifest.strata_rows
        ));
    }
    let panel_targets = rows
        .iter()
        .map(|row| row.target_token)
        .collect::<BTreeSet<_>>();
    let point_targets = point.targets.iter().copied().collect::<BTreeSet<_>>();
    let absent_targets = panel_targets
        .difference(&point_targets)
        .copied()
        .collect::<Vec<_>>();
    if raw_query_mismatches != 0 || point_served_mismatches != 0 || shuffled_served_mismatches != 0
    {
        return Err(format!(
            "panel query/served parity failed: queries={raw_query_mismatches}, point={point_served_mismatches}, shuffled={shuffled_served_mismatches}"
        ));
    }
    let per_stratum_paired = stratified_pairs
        .into_iter()
        .map(|(stratum, (shuffled, raw, constant))| {
            Ok((
                stratum,
                serde_json::json!({
                    "point_vs_shuffled": paired_binary(&shuffled)?,
                    "point_vs_raw": paired_binary(&raw)?,
                    "point_vs_constant": paired_binary(&constant)?,
                }),
            ))
        })
        .collect::<Result<BTreeMap<_, _>, String>>()?;
    std::fs::create_dir_all(&config.out).map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("memory-scaffold.uorlut"), &memory_artifact)
        .map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("point.uorlut"), &point_artifact)
        .map_err(|error| error.to_string())?;
    std::fs::write(config.out.join("shuffled-point.uorlut"), &random_artifact)
        .map_err(|error| error.to_string())?;
    let report = serde_json::json!({
        "schema": PANEL_SCHEMA,
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": {
            "artifact": config.artifact,
            "artifact_sha256": config.expected_artifact_sha256,
            "train_tokens": config.tokens,
            "train_tokens_sha256": config.expected_tokens_sha256,
            "train_mask": config.mask,
            "train_mask_sha256": config.expected_mask_sha256,
            "train_manifest": config.manifest,
            "train_manifest_sha256": config.expected_manifest_sha256,
            "panel_inputs": panel.inputs,
            "panel_inputs_sha256": inputs_sha,
            "panel_labels": panel.labels,
            "panel_labels_sha256": labels_sha,
            "panel_plan": panel.plan,
            "panel_plan_sha256": plan_sha,
            "panel_manifest": panel.manifest,
            "panel_manifest_sha256": manifest_sha,
            "panel_tokenizer": panel.tokenizer,
            "panel_tokenizer_sha256": tokenizer_sha,
        },
        "settings": {
            "rendering": panel.rendering,
            "protocol": SCHEMA_V2,
            "raw_head": "nearest",
            "train_examples_requested": panel.train_examples,
            "train_openings": training.len(),
            "random_seed": config.random_seed,
            "point_width": point.width,
            "point_rows": point.rows(),
        },
        "artifacts": {
            "memory_scaffold_sha256": uor_r4_discrete::corpus::sha256(&memory_artifact),
            "point_sha256": uor_r4_discrete::corpus::sha256(&point_artifact),
            "shuffled_point_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
        },
        "opening_vocabulary": {
            "panel_targets": panel_targets.len(),
            "point_targets": point_targets.len(),
            "panel_targets_absent_from_point_table": absent_targets.len(),
            "absent_target_tokens": absent_targets,
        },
        "overlap_control": {
            "pass": overlap.pass,
            "counts": overlap.counts,
        },
        "totals": totals,
        "per_stratum": per_stratum,
        "per_stratum_paired": per_stratum_paired,
        "paired": {
            "point_vs_shuffled": paired_binary(&point_vs_shuffled)?,
            "point_vs_raw": paired_binary(&point_vs_raw)?,
            "point_vs_constant": paired_binary(&point_vs_constant)?,
        },
        "parity": {
            "query_mismatches": raw_query_mismatches,
            "point_served_mismatches": point_served_mismatches,
            "shuffled_served_mismatches": shuffled_served_mismatches,
        },
        "rows": rows,
        "threads": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib()?,
        "claim": "CPU development-research opening screen on the sealed m2-fresh256 panel; not a fresh capability bar or chat result",
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

fn training_openings(examples: &[CpuConversationSample]) -> Vec<CpuConversationSample> {
    examples
        .iter()
        .filter(|sample| {
            sample
                .reply
                .first()
                .is_some_and(|token| *token != POINT_KEY_BOS && *token != PANEL_EOS)
        })
        .cloned()
        .collect()
}

fn validate_panel_labels(labels: &PanelLabels) -> Result<(), String> {
    if labels.protocol != SCHEMA_V2 {
        return Err(format!("panel labels protocol is not {SCHEMA_V2}"));
    }
    if labels.schema != "uor-r4.native-source-bank-labels/1" || !labels.membership_only {
        return Err("panel labels schema or membership_only differs".to_owned());
    }
    Ok(())
}

fn panel_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        if character.is_alphanumeric() {
            current.extend(character.to_lowercase());
        } else if !current.is_empty() {
            words.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn panel_grams(words: &[String]) -> BTreeSet<String> {
    words
        .windows(PANEL_NGRAM)
        .map(|window| window.join(" "))
        .collect()
}

fn contains_word_sequence(haystack: &[String], needle: &[String]) -> bool {
    !needle.is_empty()
        && haystack
            .windows(needle.len())
            .any(|window| window == needle)
}

fn audit_panel_training_overlap(
    train_token_bytes: &[u8],
    train_mask: &[u8],
    tokenizer: &ByteBpeTokenizer,
    plan: &PanelPlan,
    labels: &PanelLabels,
) -> Result<PanelOverlapAudit, String> {
    let token_slice = read_uort_token_slice(train_token_bytes, 0, train_mask.len())
        .map_err(|error| format!("panel overlap token source: {error}"))?;
    let prompt_words = plan
        .histories
        .iter()
        .flat_map(|history| history.turns.iter().chain(&history.queries))
        .map(|row| panel_words(&row.text))
        .filter(|words| !words.is_empty())
        .collect::<Vec<_>>();
    let answer_words = labels
        .cases
        .iter()
        .flat_map(|case| case.answers.accepted.iter())
        .map(|answer| panel_words(answer))
        .filter(|words| !words.is_empty())
        .collect::<Vec<_>>();
    let panel_gram_set = prompt_words
        .iter()
        .chain(&answer_words)
        .flat_map(|words| panel_grams(words))
        .collect::<BTreeSet<_>>();
    let mut documents = Vec::new();
    let mut start = 0usize;
    for end in (0..=token_slice.tokens.len()).filter(|&index| {
        index == token_slice.tokens.len() || token_slice.tokens[index] == POINT_KEY_BOS
    }) {
        if end <= start {
            start = end + 1;
            continue;
        }
        documents.push(start..end);
        start = end + 1;
    }
    let counts = documents
        .par_iter()
        .map(|document| {
            let mut counts = PanelOverlapCounts {
                documents: 1,
                ..PanelOverlapCounts::default()
            };
            let text = tokenizer.decode(&token_slice.tokens[document.clone()]);
            let document_words = panel_words(&text);
            counts.exact_panel_in_document = prompt_words
                .iter()
                .chain(&answer_words)
                .filter(|needle| contains_word_sequence(&document_words, needle))
                .count();
            if panel_grams(&document_words)
                .intersection(&panel_gram_set)
                .next()
                .is_some()
            {
                counts.documents_with_panel_6gram = 1;
            }
            let mut reply_start = None;
            for (offset, masked) in (document.start..=document.end)
                .map(|offset| (offset, train_mask.get(offset).copied().unwrap_or(0)))
            {
                let inside = offset < document.end && masked == 1;
                match (inside, reply_start) {
                    (true, None) => reply_start = Some(offset),
                    (false, Some(from)) => {
                        counts.assistant_replies += 1;
                        let reply =
                            panel_words(&tokenizer.decode(&token_slice.tokens[from..offset]));
                        if answer_words.contains(&reply) {
                            counts.exact_accepted_assistant_reply += 1;
                        }
                        if panel_grams(&reply)
                            .intersection(&panel_gram_set)
                            .next()
                            .is_some()
                        {
                            counts.assistant_replies_with_panel_6gram += 1;
                        }
                        reply_start = None;
                    }
                    _ => {}
                }
            }
            counts
        })
        .reduce(PanelOverlapCounts::default, |mut total, counts| {
            total.documents += counts.documents;
            total.assistant_replies += counts.assistant_replies;
            total.exact_panel_in_document += counts.exact_panel_in_document;
            total.exact_accepted_assistant_reply += counts.exact_accepted_assistant_reply;
            total.documents_with_panel_6gram += counts.documents_with_panel_6gram;
            total.assistant_replies_with_panel_6gram += counts.assistant_replies_with_panel_6gram;
            total
        });
    let pass = counts.exact_panel_in_document == 0
        && counts.exact_accepted_assistant_reply == 0
        && counts.documents_with_panel_6gram == 0
        && counts.assistant_replies_with_panel_6gram == 0;
    Ok(PanelOverlapAudit { counts, pass })
}

fn constant_opening(training: &[CpuConversationSample]) -> Result<u32, String> {
    let mut counts = BTreeMap::<u32, u64>::new();
    for sample in training {
        let token = *sample
            .reply
            .first()
            .ok_or_else(|| "training opening is empty".to_owned())?;
        *counts.entry(token).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(token, _)| token)
        .ok_or_else(|| "training opening census is empty".to_owned())
}

fn add_panel_tally(tally: &mut PanelOpeningTally, row: &PanelOpeningRow) {
    tally.rows += 1;
    tally.raw_top1 += u64::from(row.raw_token == row.target_token);
    tally.point_changed_raw += u64::from(row.point_token != row.raw_token);
    tally.point_top1 += u64::from(row.point_token == row.target_token);
    tally.point_top8 += u64::from(row.point_top8);
    tally.point_top32 += u64::from(row.point_top32);
    tally.shuffled_top1 += u64::from(row.shuffled_token == row.target_token);
    tally.shuffled_top8 += u64::from(row.shuffled_top8);
    tally.shuffled_changed_raw += u64::from(row.shuffled_token != row.raw_token);
    tally.constant_top1 += u64::from(row.constant_token == row.target_token);
}

fn paired_binary(rows: &[(bool, bool)]) -> Result<serde_json::Value, String> {
    if rows.is_empty() {
        return Err("paired binary comparison requires rows".to_owned());
    }
    let candidate_top1 = rows.iter().filter(|(candidate, _)| *candidate).count();
    let control_top1 = rows.iter().filter(|(_, control)| *control).count();
    let candidate_only = rows
        .iter()
        .filter(|(candidate, control)| *candidate && !*control)
        .count();
    let control_only = rows
        .iter()
        .filter(|(candidate, control)| !*candidate && *control)
        .count();
    let discordant = candidate_only + control_only;
    let threshold = std::cmp::max(2, (2.0 * (discordant as f64).sqrt()).ceil() as u64);
    let gain = candidate_only as i64 - control_only as i64;
    Ok(serde_json::json!({
        "rows": rows.len(),
        "candidate_top1": candidate_top1,
        "control_top1": control_top1,
        "candidate_only": candidate_only,
        "control_only": control_only,
        "discordant": discordant,
        "gain": gain,
        "threshold": threshold,
        "passes": gain >= i64::try_from(threshold).unwrap_or(i64::MAX),
    }))
}

fn point_candidates(
    image: &PointKeyImage,
    query: &[i32],
    maximum: usize,
) -> Result<Vec<u32>, String> {
    if query.len() != image.width
        || image.rows() == 0
        || maximum == 0
        || image.points.len() != image.rows() * image.width
    {
        return Err("invalid point-candidate query or table".to_owned());
    }
    let mut distances = image
        .points
        .chunks_exact(image.width)
        .enumerate()
        .map(|(row, point)| {
            let distance = query
                .iter()
                .zip(point)
                .try_fold(0u64, |sum, (&query, &point)| {
                    sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                        .ok_or_else(|| "point-candidate distance overflows".to_owned())
                })?;
            Ok((distance, row))
        })
        .collect::<Result<Vec<_>, String>>()?;
    distances.sort_unstable();
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    for (_, row) in distances {
        let target = image.targets[row];
        if seen.insert(target) {
            candidates.push(target);
            if candidates.len() == maximum {
                break;
            }
        }
    }
    Ok(candidates)
}

fn render_panel_case(
    tokenizer: &ByteBpeTokenizer,
    encoder: &DialogueEncoder<'_>,
    case: &PanelInputCase,
    rendering: &str,
) -> Result<(String, Vec<u32>), String> {
    let mut contexts = case
        .segments
        .iter()
        .filter(|segment| segment.kind == "Context")
        .collect::<Vec<_>>();
    contexts.sort_by_key(|segment| segment.event);
    let mut parts = contexts
        .iter()
        .map(|segment| tokenizer.decode(&segment.token_ids).trim().to_owned())
        .collect::<Vec<_>>();
    parts.push(tokenizer.decode(&case.query_ids).trim().to_owned());
    let (content, ids) = match rendering {
        ONE_USER_RENDERING => {
            let content = parts.join(" ");
            let ids = encoder
                .encode_assistant_prefix(&[Message {
                    role: "user",
                    content: &content,
                }])
                .tokens;
            (content, ids)
        }
        PER_TURN_RENDERING => {
            if parts.iter().any(|part| part.is_empty()) {
                return Err(format!("{}: an empty turn cannot be rendered", case.id));
            }
            let messages = parts
                .iter()
                .map(|content| Message {
                    role: "user",
                    content,
                })
                .collect::<Vec<_>>();
            let content = parts.join("\n");
            let ids = encoder.encode_assistant_prefix(&messages).tokens;
            (content, ids)
        }
        _ => return Err("unknown panel rendering".to_owned()),
    };
    Ok((content, ids))
}

fn panel_reference_ids(
    encoder: &DialogueEncoder<'_>,
    answers: &FrozenAnswers,
) -> Result<(String, Vec<u32>), String> {
    let answer = answers
        .accepted
        .iter()
        .min_by_key(|answer| answer.len())
        .ok_or_else(|| "frozen answer set is empty".to_owned())?;
    let encoded = encoder.encode_assistant_suffix(
        &[Message {
            role: "assistant",
            content: answer,
        }],
        true,
    );
    let ids = encoded
        .tokens
        .iter()
        .zip(&encoded.response_mask)
        .filter_map(|(&token, &mask)| (mask == 1).then_some(token))
        .collect::<Vec<_>>();
    if ids.last().copied() != Some(PANEL_EOS) {
        return Err(format!("reference answer {answer:?} does not end in EOS"));
    }
    Ok((answer.clone(), ids))
}

fn group_json(group: &KeyGroup) -> serde_json::Value {
    serde_json::json!({
        "key": group.key,
        "samples": group.samples.len(),
        "documents": group.samples.iter().map(|sample| sample.document_ordinal).collect::<BTreeSet<_>>(),
    })
}

fn memory_sample(sample: &CpuConversationSample) -> MemoryWeightSample {
    MemoryWeightSample {
        prompt: sample.prompt.clone(),
        reply: sample.reply.clone(),
    }
}

fn opening_key(sample: &CpuConversationSample, key_tokens: usize) -> Vec<u32> {
    sample.prompt.iter().copied().take(key_tokens).collect()
}

fn select_transfer_groups(
    examples: &[CpuConversationSample],
    config: &Config,
) -> Result<(Vec<KeyGroup>, Vec<KeyGroup>), String> {
    let mut training = BTreeMap::<Vec<u32>, Vec<CpuConversationSample>>::new();
    let mut evaluation = BTreeMap::<Vec<u32>, Vec<CpuConversationSample>>::new();
    for sample in examples {
        let key = opening_key(sample, config.key_tokens);
        if sample.document_ordinal % 2 == 0 {
            training.entry(key).or_default().push(sample.clone());
        } else {
            evaluation.entry(key).or_default().push(sample.clone());
        }
    }
    let mut training = select_groups(training, config.train_keys, config.per_key);
    let training_keys = training
        .iter()
        .map(|group| group.key.clone())
        .collect::<BTreeSet<_>>();
    let evaluation = evaluation
        .into_iter()
        .filter(|(key, samples)| !training_keys.contains(key) && samples.len() >= 4)
        .collect::<BTreeMap<_, _>>();
    let evaluation = select_groups(evaluation, config.evaluation_keys, config.per_key);
    if training.is_empty() || evaluation.is_empty() {
        return Err("no disjoint training/evaluation opening-key groups were found".to_owned());
    }
    training.sort_by(|left, right| left.key.cmp(&right.key));
    Ok((training, evaluation))
}

fn select_groups(
    groups: BTreeMap<Vec<u32>, Vec<CpuConversationSample>>,
    count: usize,
    per_key: usize,
) -> Vec<KeyGroup> {
    let mut groups = groups
        .into_iter()
        .filter_map(|(key, mut samples)| {
            if samples.len() < 4 {
                return None;
            }
            samples.truncate(per_key);
            Some(KeyGroup { key, samples })
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .samples
            .len()
            .cmp(&left.samples.len())
            .then_with(|| right.key.cmp(&left.key))
    });
    groups.truncate(count);
    groups
}

fn derive_point_table(
    memory_artifact: &[u8],
    training: &[CpuConversationSample],
    width: usize,
    vocab: usize,
) -> Result<PointKeyImage, String> {
    let model = IntegerStackModel::parse(memory_artifact).map_err(|error| error.to_string())?;
    let windows = training
        .iter()
        .map(|sample| {
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "point training sample has no reply".to_owned())?;
            let at = sample
                .prompt
                .len()
                .checked_sub(1)
                .ok_or_else(|| "point training prompt is empty".to_owned())?;
            Ok(PointKeyWindow {
                input: sample.prompt.clone(),
                openings: vec![(at, target)],
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let built = windows
        .par_iter()
        .map(|window| {
            build_point_key(&model, std::slice::from_ref(window)).map_err(|error| error.to_string())
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut targets = BTreeMap::<Vec<i16>, BTreeMap<u32, usize>>::new();
    for image in built {
        for (point, &target) in image.points.chunks_exact(width).zip(&image.targets) {
            let point = point.to_vec();
            *targets.entry(point).or_default().entry(target).or_default() += 1;
        }
    }
    let rows = targets
        .into_iter()
        .map(|(point, counts)| {
            let target = counts
                .into_iter()
                .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
                .map(|(target, _)| target)
                .expect("a point row has a target");
            (point, target)
        })
        .collect::<Vec<_>>();
    let image = PointKeyImage {
        width,
        points: rows
            .iter()
            .flat_map(|(point, _)| point.iter().copied())
            .collect(),
        targets: rows.into_iter().map(|(_, target)| target).collect(),
        vote_k: 1,
    };
    image.check(vocab).map_err(|error| error.to_string())?;
    Ok(image)
}

fn derive_target_medoids(point: &PointKeyImage) -> Result<PointKeyImage, String> {
    derive_target_prototypes(point, 1)
}

fn derive_target_prototypes(point: &PointKeyImage, k: usize) -> Result<PointKeyImage, String> {
    if point.width == 0
        || point.rows() == 0
        || point.points.len() != point.rows() * point.width
        || k == 0
    {
        return Err(
            "target prototypes require a non-empty rectangular point table and k > 0".to_owned(),
        );
    }
    let mut by_target = BTreeMap::<u32, Vec<Vec<i16>>>::new();
    for (point, &target) in point.points.chunks_exact(point.width).zip(&point.targets) {
        by_target.entry(target).or_default().push(point.to_vec());
    }
    if by_target.len() < 2 {
        return Err("target prototypes require at least two targets".to_owned());
    }
    let mut rows = Vec::with_capacity(by_target.len() * k);
    for (target, mut members) in by_target {
        members.sort();
        members.dedup();
        let mut selected = vec![target_medoid(&members)?];
        while selected.len() < k.min(members.len()) {
            let mut best: Option<(u64, &Vec<i16>)> = None;
            for candidate in &members {
                if selected.contains(candidate) {
                    continue;
                }
                let nearest = selected
                    .iter()
                    .map(|selected| point_distance(candidate, selected))
                    .collect::<Option<Vec<_>>>()
                    .ok_or_else(|| "target-prototype distance overflows".to_owned())?
                    .into_iter()
                    .min()
                    .ok_or_else(|| "target prototypes have no selected point".to_owned())?;
                if best.as_ref().is_none_or(|(best_distance, best_point)| {
                    nearest > *best_distance
                        || (nearest == *best_distance && candidate < best_point)
                }) {
                    best = Some((nearest, candidate));
                }
            }
            selected.push(
                best.ok_or_else(|| "target-prototype residual group is empty".to_owned())?
                    .1
                    .clone(),
            );
        }
        for point in selected {
            rows.push((point, target));
        }
    }
    rows.sort();
    Ok(PointKeyImage {
        width: point.width,
        points: rows
            .iter()
            .flat_map(|(point, _)| point.iter().copied())
            .collect(),
        targets: rows.into_iter().map(|(_, target)| target).collect(),
        vote_k: 1,
    })
}

fn target_medoid(members: &[Vec<i16>]) -> Result<Vec<i16>, String> {
    let mut best: Option<(u64, &Vec<i16>)> = None;
    for candidate in members {
        let total = members.iter().try_fold(0u64, |total, member| {
            let distance = point_distance(candidate, member)
                .ok_or_else(|| "target-medoid distance overflows".to_owned())?;
            total
                .checked_add(distance)
                .ok_or_else(|| "target-medoid distance overflows".to_owned())
        })?;
        if best.as_ref().is_none_or(|(best_total, best_point)| {
            total < *best_total || (total == *best_total && candidate < best_point)
        }) {
            best = Some((total, candidate));
        }
    }
    Ok(best
        .ok_or_else(|| "a target-medoid group is empty".to_owned())?
        .1
        .clone())
}

fn point_distance(left: &[i16], right: &[i16]) -> Option<u64> {
    left.iter()
        .zip(right)
        .try_fold(0u64, |distance, (&left, &right)| {
            distance.checked_add(u64::from(left.abs_diff(right)))
        })
}

fn shuffled_targets(targets: &[u32], seed: u64) -> Result<Vec<u32>, String> {
    let mut out = targets.to_vec();
    let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    for index in (1..out.len()).rev() {
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
        out.swap(index, (draw as usize) % (index + 1));
    }
    if out == targets {
        out.rotate_left(1);
    }
    if out == targets {
        return Err("random target control is identical to the point table".to_owned());
    }
    Ok(out)
}

fn assert_held_out_exact_keys_miss(
    memory_artifact: &[u8],
    held_out: &[MemoryWeightSample],
) -> Result<(), String> {
    let memory = read_memory_uorlut01(memory_artifact)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "attached memory disappeared".to_owned())?;
    let keys = memory
        .opening
        .keys
        .iter()
        .map(Vec::as_slice)
        .collect::<BTreeSet<_>>();
    for sample in held_out {
        let key = &sample.prompt[..memory.record.k.min(sample.prompt.len())];
        if keys.contains(key) {
            return Err(format!(
                "held-out opening key {:?} remains in the exact memory table",
                key
            ));
        }
    }
    Ok(())
}

fn evaluate_neighbor_vote(
    memory_artifact: &[u8],
    point_artifact: &[u8],
    voted_artifact: Option<&[u8]>,
    point: &PointKeyImage,
    random_point: &PointKeyImage,
    samples: &[CpuConversationSample],
    neighbors: usize,
) -> Result<NeighborReading, String> {
    let readings = samples
        .par_iter()
        .map(|sample| {
            let (query, _) = opening_query_and_token(memory_artifact, &sample.prompt)?;
            let (_, served) = opening_query_and_token(point_artifact, &sample.prompt)?;
            let nearest = point_vote(point, &query, 1)?;
            let vote = point_vote(point, &query, neighbors)?;
            let random_vote = point_vote(random_point, &query, neighbors)?;
            let voted_served = voted_artifact
                .map(|bytes| opening_query_and_token(bytes, &sample.prompt).map(|(_, token)| token))
                .transpose()?
                .unwrap_or(nearest);
            if nearest != served {
                return Err(format!(
                    "nearest extraction named {nearest}, while the served point key named {served}"
                ));
            }
            if vote != voted_served {
                return Err(format!(
                    "eight-row extraction named {vote}, while the served vote_k={neighbors} artifact named {voted_served}"
                ));
            }
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "evaluation reply is empty".to_owned())?;
            Ok((
                u64::from(nearest == target),
                u64::from(vote == target),
                u64::from(random_vote == target),
                u64::from(nearest == served),
                u64::from(vote == voted_served),
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut reading = NeighborReading {
        decisions: readings.len() as u64,
        ..NeighborReading::default()
    };
    for (nearest, vote, random_vote, nearest_matches_served, voted_matches_served) in readings {
        reading.nearest_top1 += nearest;
        reading.vote_top1 += vote;
        reading.random_vote_top1 += random_vote;
        reading.nearest_matches_served += nearest_matches_served;
        reading.voted_matches_served += voted_matches_served;
    }
    Ok(reading)
}

fn evaluate_replies(
    artifact: &[u8],
    samples: &[CpuConversationSample],
    cap: usize,
    context: usize,
) -> Result<TimedReplyReading, String> {
    let started = Instant::now();
    let rows = samples
        .par_iter()
        .map(|sample| {
            let generated = generate_reply(artifact, &sample.prompt, cap, context)?;
            let reference = normalized_reference(&sample.reply).to_vec();
            let reachable = reference_reachable(sample.prompt.len(), reference.len(), cap, context);
            Ok((generated, reference, reachable))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let reply_sha256 = reply_sequence_sha256(&rows);
    let reading = summarize_replies(rows)?;
    let elapsed = started.elapsed();
    Ok(TimedReplyReading {
        reading,
        reply_sha256,
        wall_elapsed_us: elapsed.as_micros().try_into().unwrap_or(u64::MAX),
        generated_tokens_per_second: reading.generated_tokens as f64
            / elapsed.as_secs_f64().max(1e-12),
    })
}

fn summarize_replies(rows: Vec<(GeneratedReply, Vec<u32>, bool)>) -> Result<ReplyReading, String> {
    let mut reading = ReplyReading {
        replies: rows.len() as u64,
        ..ReplyReading::default()
    };
    let mut distinct = BTreeSet::new();
    for (generated, reference, reference_reachable) in rows {
        let reference = normalized_reference(&reference);
        match generated.stop {
            "bos" => reading.bos_complete = add_count(reading.bos_complete, 1)?,
            "cap" => reading.cap_stopped = add_count(reading.cap_stopped, 1)?,
            "context" => reading.context_stopped = add_count(reading.context_stopped, 1)?,
            other => return Err(format!("unknown reply stop {other}")),
        }
        reading.exact_reference = add_count(
            reading.exact_reference,
            u64::from(generated.tokens == reference),
        )?;
        reading.reference_reachable =
            add_count(reading.reference_reachable, u64::from(reference_reachable))?;
        reading.opening_correct = add_count(
            reading.opening_correct,
            u64::from(matches!(
                (generated.tokens.first(), reference.first()),
                (Some(generated), Some(reference)) if generated == reference
            )),
        )?;
        let non_empty = !generated.tokens.is_empty();
        reading.non_empty = add_count(reading.non_empty, u64::from(non_empty))?;
        if non_empty {
            distinct.insert(generated.tokens.clone());
        }
        reading.repeated_4gram = add_count(
            reading.repeated_4gram,
            u64::from(has_repeated_4gram(&generated.tokens)),
        )?;
        reading.generated_tokens =
            add_count(reading.generated_tokens, generated.tokens.len() as u64)?;
        reading.reference_tokens = add_count(reading.reference_tokens, reference.len() as u64)?;
    }
    reading.distinct_non_empty = distinct.len() as u64;
    Ok(reading)
}

fn generate_reply(
    artifact: &[u8],
    prompt: &[u32],
    cap: usize,
    context: usize,
) -> Result<GeneratedReply, String> {
    if cap == 0 || prompt.is_empty() || prompt.len() > context {
        return Err("invalid bounded reply prompt, cap or context".to_owned());
    }
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
    let mut first = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, CpuFbsHead::Writer)
            .map_err(|error| error.to_string())?;
        if index + 1 == prompt.len() {
            first = Some(step.token);
        }
    }
    bounded_reply(
        first.ok_or_else(|| "evaluation prompt is empty".to_owned())?,
        prompt.len(),
        cap,
        context,
        |token| {
            forward
                .step(token, CpuFbsHead::Writer)
                .map(|step| step.token)
                .map_err(|error| error.to_string())
        },
    )
}

fn bounded_reply(
    first: u32,
    prompt_len: usize,
    cap: usize,
    context: usize,
    mut step: impl FnMut(u32) -> Result<u32, String>,
) -> Result<GeneratedReply, String> {
    let mut tokens = Vec::with_capacity(cap);
    let mut next = first;
    loop {
        let stop = reply_stop(next, tokens.len(), prompt_len, cap, context);
        if stop != Some(ReplyEnd::Bos) {
            tokens.push(next);
        }
        if let Some(stop) = stop {
            return Ok(GeneratedReply {
                tokens,
                stop: match stop {
                    ReplyEnd::Bos => "bos",
                    ReplyEnd::Length => "cap",
                    ReplyEnd::Context => "context",
                },
            });
        }
        next = step(next)?;
    }
}

fn has_repeated_4gram(tokens: &[u32]) -> bool {
    let mut seen = BTreeSet::new();
    tokens.windows(4).any(|window| !seen.insert(window))
}

fn reply_sequence_sha256(rows: &[(GeneratedReply, Vec<u32>, bool)]) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(rows.len() as u64).to_le_bytes());
    for (reply, _, _) in rows {
        let stop = match reply.stop {
            "bos" => 0u8,
            "cap" => 1,
            "context" => 2,
            _ => 3,
        };
        bytes.push(stop);
        bytes.extend_from_slice(&(reply.tokens.len() as u64).to_le_bytes());
        for token in &reply.tokens {
            bytes.extend_from_slice(&token.to_le_bytes());
        }
    }
    uor_r4_discrete::corpus::sha256(&bytes)
}

fn normalized_reference(reference: &[u32]) -> &[u32] {
    reference
        .strip_suffix(&[POINT_KEY_BOS])
        .unwrap_or(reference)
}

fn reference_reachable(
    prompt_len: usize,
    reference_len: usize,
    cap: usize,
    context: usize,
) -> bool {
    reference_len <= cap
        && prompt_len
            .checked_add(reference_len)
            .is_some_and(|length| length <= context)
}

fn add_count(left: u64, right: u64) -> Result<u64, String> {
    left.checked_add(right)
        .ok_or_else(|| "complete-reply counter overflow".to_owned())
}

fn opening_query_and_token(artifact: &[u8], prompt: &[u32]) -> Result<(Vec<i32>, u32), String> {
    opening_query_and_token_with_head(artifact, prompt, CpuFbsHead::Writer)
}

fn opening_query_and_token_with_head(
    artifact: &[u8],
    prompt: &[u32],
    head: CpuFbsHead,
) -> Result<(Vec<i32>, u32), String> {
    let mut forward = CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
    let mut result = None;
    for (index, &token) in prompt.iter().enumerate() {
        if index + 1 == prompt.len() {
            forward.mark_reply_start();
        }
        let step = forward
            .step(token, head)
            .map_err(|error| error.to_string())?;
        if index + 1 == prompt.len() {
            result = Some((step.query_q12, step.token));
        }
    }
    result.ok_or_else(|| "evaluation prompt is empty".to_owned())
}

fn point_vote(image: &PointKeyImage, query: &[i32], neighbors: usize) -> Result<u32, String> {
    if query.len() != image.width || image.rows() == 0 || neighbors == 0 || neighbors > image.rows()
    {
        return Err("invalid point-neighborhood query or size".to_owned());
    }
    let mut distances = image
        .points
        .chunks_exact(image.width)
        .enumerate()
        .map(|(row, point)| {
            let distance = query
                .iter()
                .zip(point)
                .try_fold(0u64, |sum, (&query, &point)| {
                    sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                        .ok_or_else(|| "point-neighborhood distance overflows".to_owned())
                })?;
            Ok((distance, row))
        })
        .collect::<Result<Vec<_>, String>>()?;
    distances.sort_unstable();
    let mut counts = BTreeMap::<u32, usize>::new();
    for &(_, row) in distances.iter().take(neighbors) {
        *counts.entry(image.targets[row]).or_default() += 1;
    }
    counts
        .into_iter()
        .max_by(|left, right| left.1.cmp(&right.1).then_with(|| right.0.cmp(&left.0)))
        .map(|(target, _)| target)
        .ok_or_else(|| "point neighborhood names no target".to_owned())
}

fn paired_opening_comparison(
    candidate: &[OpeningOutcome],
    control: &[OpeningOutcome],
) -> Result<serde_json::Value, String> {
    if candidate.len() != control.len() || candidate.is_empty() {
        return Err("paired opening comparison requires equal non-empty rows".to_owned());
    }
    let mut candidate_only = 0u64;
    let mut control_only = 0u64;
    for (candidate, control) in candidate.iter().zip(control) {
        candidate_only += u64::from(candidate.top1 && !control.top1);
        control_only += u64::from(control.top1 && !candidate.top1);
    }
    let discordant = candidate_only
        .checked_add(control_only)
        .ok_or_else(|| "paired opening disagreement overflows".to_owned())?;
    let threshold = std::cmp::max(2, (2.0 * (discordant as f64).sqrt()).ceil() as u64);
    let gain = i64::try_from(candidate_only)
        .map_err(|_| "candidate-only count overflows i64".to_owned())?
        - i64::try_from(control_only).map_err(|_| "control-only count overflows i64".to_owned())?;
    Ok(serde_json::json!({
        "rows": candidate.len(),
        "candidate_top1": candidate.iter().filter(|row| row.top1).count(),
        "control_top1": control.iter().filter(|row| row.top1).count(),
        "candidate_only": candidate_only,
        "control_only": control_only,
        "discordant": discordant,
        "gain": gain,
        "threshold": threshold,
        "passes": gain >= i64::try_from(threshold).unwrap_or(i64::MAX),
    }))
}

fn summarize_openings(outcomes: &[OpeningOutcome]) -> Result<OpeningReading, String> {
    let mut reading = OpeningReading {
        decisions: u64::try_from(outcomes.len())
            .map_err(|_| "opening decision count overflows".to_owned())?,
        ..OpeningReading::default()
    };
    for outcome in outcomes {
        reading.top1_correct = reading
            .top1_correct
            .checked_add(u64::from(outcome.top1))
            .ok_or_else(|| "opening top-1 count overflows".to_owned())?;
        reading.target_distance_sum = reading
            .target_distance_sum
            .checked_add(outcome.target_distance)
            .ok_or_else(|| "opening total overflows".to_owned())?;
    }
    Ok(reading)
}

fn changed_openings(candidate: &[OpeningOutcome], control: &[OpeningOutcome]) -> usize {
    candidate
        .iter()
        .zip(control)
        .filter(|(candidate, control)| candidate.token != control.token)
        .count()
}

fn opening_distances_equal(candidate: &[OpeningOutcome], baseline: &[OpeningOutcome]) -> bool {
    candidate.len() == baseline.len()
        && candidate
            .iter()
            .zip(baseline)
            .all(|(candidate, baseline)| candidate.target_distance == baseline.target_distance)
}

fn evaluate_opening_outcomes(
    artifact: &[u8],
    samples: &[CpuConversationSample],
    width: usize,
) -> Result<Vec<OpeningOutcome>, String> {
    let model = IntegerStackModel::parse(artifact).map_err(|error| error.to_string())?;
    let table = model
        .fbs_writer_table()
        .ok_or_else(|| "the artifact has no FBS writer table".to_owned())?
        .to_vec();
    if table.len() != model.shape().vocab * width {
        return Err("writer table shape differs".to_owned());
    }
    samples
        .par_iter()
        .map(|sample| {
            let mut forward =
                CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
            let mut step = None;
            for (index, &token) in sample.prompt.iter().enumerate() {
                if index + 1 == sample.prompt.len() {
                    forward.mark_reply_start();
                }
                step = Some(
                    forward
                        .step(token, CpuFbsHead::Writer)
                        .map_err(|error| error.to_string())?,
                );
            }
            let step = step.ok_or_else(|| "evaluation prompt is empty".to_owned())?;
            let target = *sample
                .reply
                .first()
                .ok_or_else(|| "evaluation reply is empty".to_owned())?;
            let row = &table[target as usize * width..][..width];
            let distance =
                step.query_q12
                    .iter()
                    .zip(row)
                    .try_fold(0u64, |sum, (&query, &point)| {
                        sum.checked_add(u64::from(query.abs_diff(i32::from(point))))
                            .ok_or_else(|| "opening distance overflows".to_owned())
                    })?;
            Ok(OpeningOutcome {
                top1: step.token == target,
                target_distance: distance,
                token: step.token,
            })
        })
        .collect::<Result<Vec<_>, String>>()
}

fn peak_rss_kib() -> Result<u64, String> {
    let status = std::fs::read_to_string("/proc/self/status")
        .map_err(|error| format!("cannot measure peak RSS: {error}"))?;
    parse_peak_rss_kib(&status)
}

fn parse_peak_rss_kib(status: &str) -> Result<u64, String> {
    let peak = status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmHWM:")
                .and_then(|value| value.split_whitespace().next())
                .and_then(|value| value.parse().ok())
        })
        .ok_or_else(|| "peak RSS measurement is unavailable".to_owned())?;
    if peak == 0 {
        return Err("peak RSS measurement is zero".to_owned());
    }
    Ok(peak)
}

#[cfg(test)]
mod tests {
    use super::*;
    use uor_r4_integer::stack::synthetic::{synthetic_fbs_trunk_with, FbsSynth};
    use uor_r4_integer::stack::{export_fbs_trunk_uorlut01, FbsMemoryEntry, FbsMemorySnapshot};

    fn byte_level_tokenizer() -> ByteBpeTokenizer {
        let direct = (b'!'..=b'~')
            .chain(0xA1..=0xAC)
            .chain(0xAE..=0xFF)
            .collect::<BTreeSet<_>>();
        let mut codepoints = Vec::new();
        for byte in 0u16..=255 {
            let byte = byte as u8;
            codepoints.push(u32::from(byte));
        }
        let mut extra = 256u32;
        for byte in 0u16..=255 {
            let byte = byte as u8;
            if !direct.contains(&byte) {
                codepoints.push(extra);
                extra += 1;
            }
        }
        let mut vocab = BTreeMap::<String, u32>::from([
            ("<|bos|>".to_owned(), 0),
            ("<|eos|>".to_owned(), 1),
            ("<|unk|>".to_owned(), 2),
        ]);
        for (index, codepoint) in codepoints.into_iter().enumerate() {
            vocab.insert(
                char::from_u32(codepoint).expect("byte mapping").to_string(),
                3 + index as u32,
            );
        }
        let added = (0..3)
            .map(|id| {
                let content = ["<|bos|>", "<|eos|>", "<|unk|>"][id as usize];
                serde_json::json!({
                    "id": id,
                    "content": content,
                    "single_word": false,
                    "lstrip": false,
                    "rstrip": false,
                    "normalized": false,
                    "special": true
                })
            })
            .collect::<Vec<_>>();
        let json = serde_json::json!({
            "version": "1.0",
            "truncation": null,
            "padding": null,
            "added_tokens": added,
            "normalizer": null,
            "pre_tokenizer": {
                "type": "ByteLevel",
                "add_prefix_space": false,
                "trim_offsets": true,
                "use_regex": true
            },
            "post_processor": null,
            "decoder": {
                "type": "ByteLevel",
                "add_prefix_space": true,
                "trim_offsets": true,
                "use_regex": true
            },
            "model": {
                "type": "BPE",
                "dropout": null,
                "unk_token": "<|unk|>",
                "continuing_subword_prefix": null,
                "end_of_word_suffix": null,
                "fuse_unk": false,
                "byte_fallback": false,
                "ignore_merges": false,
                "vocab": vocab,
                "merges": []
            }
        });
        ByteBpeTokenizer::from_tokenizer_json_bytes(
            &serde_json::to_vec(&json).expect("serialize tokenizer"),
        )
        .expect("byte tokenizer")
    }

    fn panel_labels(protocol: &str, schema: &str, membership_only: bool) -> PanelLabels {
        PanelLabels {
            cases: Vec::new(),
            membership_only,
            protocol: protocol.to_owned(),
            schema: schema.to_owned(),
        }
    }

    fn sample(document: usize, key: u32, reply: u32) -> CpuConversationSample {
        CpuConversationSample {
            document_ordinal: document,
            prompt: vec![
                key,
                key + 1,
                key + 2,
                key + 3,
                key + 4,
                key + 5,
                key + 6,
                key + 7,
            ],
            reply: vec![reply, 1],
        }
    }

    fn config() -> Config {
        Config {
            artifact: PathBuf::new(),
            tokens: PathBuf::new(),
            mask: PathBuf::new(),
            manifest: PathBuf::new(),
            out: PathBuf::new(),
            train_keys: 2,
            evaluation_keys: 2,
            per_key: 4,
            key_tokens: 8,
            reply_turn: 8,
            random_seed: 1,
            neighbor_vote: 0,
            target_prototypes: 0,
            reply_cap: 0,
            select_only: true,
            expected_artifact_sha256: String::new(),
            expected_tokens_sha256: String::new(),
            expected_mask_sha256: String::new(),
            expected_manifest_sha256: String::new(),
            panel: None,
        }
    }

    #[test]
    fn panel_point_candidates_are_unique_ordered_and_bounded() {
        let image = PointKeyImage {
            width: 1,
            points: vec![3, 2, 1, 0],
            targets: vec![7, 7, 8, 9],
            vote_k: 1,
        };
        assert_eq!(point_candidates(&image, &[0], 1).unwrap(), vec![9]);
        assert_eq!(point_candidates(&image, &[0], 3).unwrap(), vec![9, 8, 7]);
        assert_eq!(point_candidates(&image, &[0], 32).unwrap(), vec![9, 8, 7]);
        assert!(point_candidates(&image, &[], 1).is_err());
        assert!(point_candidates(&image, &[0], 0).is_err());
    }

    #[test]
    fn constant_opening_uses_mode_then_smaller_token() {
        let rows = vec![
            sample(0, 10, 7),
            sample(2, 20, 7),
            sample(4, 30, 6),
            sample(6, 40, 6),
        ];
        assert_eq!(constant_opening(&rows).unwrap(), 6);
        let tie = vec![sample(0, 10, 7), sample(2, 20, 6)];
        assert_eq!(constant_opening(&tie).unwrap(), 6);
    }

    #[test]
    fn panel_training_skips_boundary_and_eos_openings() {
        let mut boundary = sample(0, 10, 7);
        boundary.reply[0] = POINT_KEY_BOS;
        let mut eos = sample(1, 20, 7);
        eos.reply[0] = PANEL_EOS;
        let mut empty = sample(2, 30, 7);
        empty.reply.clear();
        let kept = sample(3, 40, 7);
        let rows = training_openings(&[boundary, eos, empty, kept]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reply.first().copied(), Some(7));
    }

    #[test]
    fn panel_label_contract_fails_closed() {
        let valid = panel_labels(SCHEMA_V2, "uor-r4.native-source-bank-labels/1", true);
        validate_panel_labels(&valid).unwrap();
        assert!(validate_panel_labels(&panel_labels(
            "wrong",
            "uor-r4.native-source-bank-labels/1",
            true
        ))
        .is_err());
        assert!(validate_panel_labels(&panel_labels(
            SCHEMA_V2,
            "uor-r4.native-source-bank-labels/1",
            false
        ))
        .is_err());
        let extra = format!(
            r#"{{"cases":[],"membership_only":true,"protocol":"{SCHEMA_V2}","schema":"uor-r4.native-source-bank-labels/1","extra":1}}"#
        );
        assert!(serde_json::from_str::<PanelLabels>(&extra).is_err());
    }

    #[test]
    fn panel_input_and_plan_contracts_refuse_unknown_metadata() {
        let inputs = r#"{"schema":"uor-r4.native-source-bank-probe-input/1","cases":[],"extra":1}"#;
        assert!(serde_json::from_str::<PanelInputs>(inputs).is_err());
        let plan = format!(
            r#"{{"schema":"uor-r4.raw-natural-reader-construction-plan/1","source_policy":"explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2","split":"fresh","histories":[],"extra":1}}"#
        );
        assert!(serde_json::from_str::<PanelPlan>(&plan).is_err());
    }

    fn overlap_plan(prompt: &str) -> PanelPlan {
        PanelPlan {
            schema: "uor-r4.raw-natural-reader-construction-plan/1".to_owned(),
            source_policy:
                "explicit-current-role-assertions/raw-current-role-queries/all-source-candidates/2"
                    .to_owned(),
            split: "fresh".to_owned(),
            histories: vec![PanelHistory {
                id: "history".to_owned(),
                stratum: "length2/q0".to_owned(),
                turns: Vec::new(),
                queries: vec![PanelTextRow {
                    _act: "query".to_owned(),
                    _relation: "job".to_owned(),
                    _template: prompt.to_owned(),
                    text: prompt.to_owned(),
                }],
            }],
        }
    }

    fn overlap_labels() -> PanelLabels {
        PanelLabels {
            cases: vec![PanelLabelCase {
                answers: FrozenAnswers {
                    intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
                    accepted: vec!["eight nine ten eleven twelve thirteen".to_owned()],
                },
                id: "case".to_owned(),
                pair_id: "history".to_owned(),
            }],
            membership_only: true,
            protocol: SCHEMA_V2.to_owned(),
            schema: "uor-r4.native-source-bank-labels/1".to_owned(),
        }
    }

    fn overlap_uort(tokens: &[u32]) -> Vec<u8> {
        let mut bytes = vec![0u8; 64];
        bytes[..4].copy_from_slice(b"UORT");
        bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
        bytes[8..16].copy_from_slice(&(tokens.len() as u64).to_le_bytes());
        bytes[16..20].copy_from_slice(&512u32.to_le_bytes());
        for token in tokens {
            bytes.extend_from_slice(&u16::try_from(*token).expect("test token").to_le_bytes());
        }
        bytes
    }

    #[test]
    fn panel_training_overlap_checks_exact_text_and_six_grams() {
        let tokenizer = byte_level_tokenizer();
        let training_text = "alpha beta gamma delta epsilon zeta eta";
        let mut train_tokens = vec![POINT_KEY_BOS];
        train_tokens.extend(tokenizer.encode(training_text));
        let train_bytes = overlap_uort(&train_tokens);
        let mask = vec![0; train_tokens.len()];
        let labels = overlap_labels();
        let clear = audit_panel_training_overlap(
            &train_bytes,
            &mask,
            &tokenizer,
            &overlap_plan("one two three four five six seven"),
            &labels,
        )
        .unwrap();
        assert!(clear.pass);
        assert_eq!(clear.counts.documents, 1);
        let reply = tokenizer.encode("reply text at end");
        let mut eof_tokens = vec![POINT_KEY_BOS];
        eof_tokens.extend(&reply);
        let eof_bytes = overlap_uort(&eof_tokens);
        let mut eof_mask = vec![0; eof_tokens.len()];
        eof_mask[1..].fill(1);
        let eof = audit_panel_training_overlap(
            &eof_bytes,
            &eof_mask,
            &tokenizer,
            &overlap_plan("one two three four five six seven"),
            &labels,
        )
        .unwrap();
        assert!(eof.pass);
        assert_eq!(eof.counts.assistant_replies, 1);
        let overlap = audit_panel_training_overlap(
            &train_bytes,
            &mask,
            &tokenizer,
            &overlap_plan("alpha beta gamma delta epsilon zeta"),
            &labels,
        )
        .unwrap();
        assert!(!overlap.pass);
        assert_eq!(overlap.counts.exact_panel_in_document, 1);
        assert_eq!(overlap.counts.documents_with_panel_6gram, 1);
    }

    #[test]
    fn panel_reference_encoding_names_the_first_scored_token() {
        let tokenizer = byte_level_tokenizer();
        let protocol = DialogueProtocol::literal_roles_v2(&tokenizer).expect("dialogue protocol");
        let encoder = protocol.bind(&tokenizer).expect("bound encoder");
        let answers = FrozenAnswers {
            intent: uor_r4_core::answer_oracle::RecordedValueIntent::Current,
            accepted: vec!["a.".to_owned()],
        };
        let (answer, ids) = panel_reference_ids(&encoder, &answers).unwrap();
        assert_eq!(answer, "a.");
        assert!(!ids.is_empty());
        assert_eq!(ids.last().copied(), Some(PANEL_EOS));
        assert!(ids[0..ids.len() - 1]
            .iter()
            .all(|token| *token != PANEL_EOS));
    }

    #[test]
    fn paired_binary_reports_the_registered_discordant_margin() {
        let rows = [(true, false), (true, false), (false, true), (false, false)];
        let report = paired_binary(&rows).unwrap();
        assert_eq!(report["candidate_top1"], 2);
        assert_eq!(report["control_top1"], 1);
        assert_eq!(report["candidate_only"], 2);
        assert_eq!(report["control_only"], 1);
        assert_eq!(report["gain"], 1);
        assert_eq!(report["threshold"], 4);
        assert_eq!(report["passes"], false);
        assert!(paired_binary(&[]).is_err());
    }

    #[test]
    fn transfer_selection_is_deterministic_disjoint_and_bounded() {
        let mut examples = Vec::new();
        for key in [10, 20] {
            for document in [0, 2, 4, 6] {
                examples.push(sample(document, key, 100 + key));
            }
        }
        for key in [30, 40] {
            for document in [1, 3, 5, 7] {
                examples.push(sample(document, key, 200 + key));
            }
        }
        let first = select_transfer_groups(&examples, &config()).unwrap();
        let second = select_transfer_groups(&examples, &config()).unwrap();
        assert_eq!(first, second);
        let keys = first
            .0
            .iter()
            .map(|group| group.key.clone())
            .collect::<BTreeSet<_>>();
        assert!(first.1.iter().all(|group| !keys.contains(&group.key)));
        assert_eq!(first.0.len(), 2);
        assert_eq!(first.1.len(), 2);
    }

    #[test]
    fn random_control_permutes_targets_without_changing_points() {
        let points = PointKeyImage {
            width: 2,
            points: vec![1, 2, 3, 4, 5, 6],
            targets: vec![7, 8, 9],
            vote_k: 1,
        };
        let targets = shuffled_targets(&points.targets, 17).unwrap();
        let mut sorted = targets.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, points.targets);
        assert_ne!(targets, points.targets);
        assert_eq!(points.points.len(), points.rows() * points.width);
        assert!(shuffled_targets(&[7], 17).is_err());
    }

    #[test]
    fn target_medoids_choose_observed_minimum_l1_members_deterministically() {
        let point = PointKeyImage {
            width: 2,
            points: vec![0, 0, 1, 0, 4, 0, 2, 0, 2, 2, 2, 0],
            targets: vec![7, 7, 7, 8, 8, 9],
            vote_k: 1,
        };
        let medoid = derive_target_medoids(&point).unwrap();
        assert_eq!(medoid.points, vec![1, 0, 2, 0, 2, 0]);
        assert_eq!(medoid.targets, vec![7, 8, 9]);
        assert_eq!(medoid.vote_k, 1);
        assert_eq!(medoid, derive_target_medoids(&point).unwrap());

        let shuffled = shuffled_targets(&medoid.targets, 17).unwrap();
        let mut before = medoid.targets.clone();
        let mut after = shuffled.clone();
        before.sort_unstable();
        after.sort_unstable();
        assert_eq!(after, before);
        assert_ne!(shuffled, medoid.targets);
    }

    #[test]
    fn target_medoids_refuse_malformed_tables_and_one_target() {
        assert!(derive_target_medoids(&PointKeyImage {
            width: 2,
            points: vec![1, 2, 3],
            targets: vec![4, 5],
            vote_k: 1,
        })
        .is_err());
        assert!(derive_target_medoids(&PointKeyImage {
            width: 1,
            points: vec![1, 2],
            targets: vec![4, 4],
            vote_k: 1,
        })
        .is_err());
    }

    #[test]
    fn target_prototypes_keep_a_medoid_and_farthest_local_residual() {
        let point = PointKeyImage {
            width: 2,
            points: vec![0, 0, 1, 0, 4, 0, 2, 0, 2, 2, 3, 0, 5, 0],
            targets: vec![7, 7, 7, 8, 8, 9, 9],
            vote_k: 1,
        };
        let prototypes = derive_target_prototypes(&point, 2).unwrap();
        assert_eq!(prototypes.points, vec![1, 0, 2, 0, 2, 2, 3, 0, 4, 0, 5, 0]);
        assert_eq!(prototypes.targets, vec![7, 8, 8, 9, 7, 9]);
        assert!(prototypes.rows() < point.rows());
        assert_eq!(prototypes, derive_target_prototypes(&point, 2).unwrap());
        assert_eq!(
            derive_target_prototypes(&point, 16).unwrap().rows(),
            point.rows()
        );
    }

    #[test]
    fn target_prototypes_break_residual_ties_with_signed_lexicographic_points() {
        let point = PointKeyImage {
            width: 2,
            // The target-7 medoid is (0, 0); four residual points are all at L1
            // distance 2. Signed component-wise lexicographic order chooses
            // (-2, 0), including across a negative coordinate.
            points: vec![0, 0, 2, 0, -2, 0, 0, 2, 0, -2, 9, 9],
            targets: vec![7, 7, 7, 7, 7, 8],
            vote_k: 1,
        };
        let prototypes = derive_target_prototypes(&point, 2).unwrap();
        assert_eq!(prototypes.points, vec![-2, 0, 0, 0, 9, 9]);
        assert_eq!(prototypes.targets, vec![7, 7, 8]);
    }

    #[test]
    fn target_prototypes_refuse_zero_malformed_and_one_target_tables() {
        let malformed = PointKeyImage {
            width: 2,
            points: vec![1, 2, 3],
            targets: vec![4, 5],
            vote_k: 1,
        };
        assert!(derive_target_prototypes(&malformed, 2).is_err());
        assert!(derive_target_prototypes(
            &PointKeyImage {
                width: 1,
                points: vec![1, 2],
                targets: vec![4, 5],
                vote_k: 1,
            },
            0,
        )
        .is_err());
        assert!(derive_target_prototypes(
            &PointKeyImage {
                width: 1,
                points: vec![1, 2],
                targets: vec![4, 4],
                vote_k: 1,
            },
            2,
        )
        .is_err());
    }

    #[test]
    fn paired_opening_comparison_uses_the_registered_discordant_margin() {
        let outcome = |top1| OpeningOutcome {
            top1,
            target_distance: 0,
            token: 0,
        };
        let candidate = [
            outcome(true),
            outcome(true),
            outcome(true),
            outcome(true),
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
        ];
        let control = [
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
            outcome(false),
        ];
        let reading = paired_opening_comparison(&candidate, &control).unwrap();
        assert_eq!(reading["candidate_only"], 4);
        assert_eq!(reading["control_only"], 0);
        assert_eq!(reading["threshold"], 4);
        assert_eq!(reading["passes"], true);
    }

    #[test]
    fn opening_instruments_require_every_row_and_per_row_distance_equality() {
        let mut baseline = (0..60)
            .map(|index| OpeningOutcome {
                top1: false,
                target_distance: index,
                token: 7,
            })
            .collect::<Vec<_>>();
        let mut candidate = baseline.clone();
        for row in &mut candidate {
            row.token = 8;
        }
        assert_eq!(changed_openings(&candidate, &baseline), 60);
        assert!(opening_distances_equal(&candidate, &baseline));

        candidate[0].target_distance = 1;
        baseline[1].target_distance = 2;
        assert!(
            candidate.iter().map(|row| row.target_distance).sum::<u64>()
                == baseline.iter().map(|row| row.target_distance).sum::<u64>()
        );
        assert!(!opening_distances_equal(&candidate, &baseline));
    }

    #[test]
    fn peak_rss_measurement_fails_closed() {
        assert_eq!(parse_peak_rss_kib("VmHWM:\t1234 kB\n").unwrap(), 1234);
        assert!(parse_peak_rss_kib("VmRSS:\t1234 kB\n").is_err());
        assert!(parse_peak_rss_kib("VmHWM:\t0 kB\n").is_err());
        assert!(parse_peak_rss_kib("VmHWM:\tnot-a-number kB\n").is_err());
    }

    #[test]
    fn eight_row_vote_can_override_a_wrong_nearest_row() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            targets: vec![9, 7, 7, 7, 7, 7, 7, 7, 8],
            vote_k: 1,
        };
        assert_eq!(point_vote(&points, &[0], 1).unwrap(), 9);
        assert_eq!(point_vote(&points, &[0], 8).unwrap(), 7);
    }

    #[test]
    fn point_vote_breaks_equal_counts_to_the_smaller_target() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 1, 2, 2],
            targets: vec![9, 5, 7, 8],
            vote_k: 1,
        };
        assert_eq!(point_vote(&points, &[0], 2).unwrap(), 5);
    }

    #[test]
    fn point_vote_refuses_an_empty_or_oversized_neighborhood() {
        let points = PointKeyImage {
            width: 1,
            points: vec![1, 2],
            targets: vec![3, 4],
            vote_k: 1,
        };
        assert!(point_vote(&points, &[0], 0).is_err());
        assert!(point_vote(&points, &[0], 3).is_err());
    }

    #[test]
    fn bounded_reply_feeds_only_generated_tokens_and_stops_at_bos() {
        let mut fed = Vec::new();
        let reply = bounded_reply(5, 3, 8, 20, |token| {
            fed.push(token);
            Ok(if fed.len() == 2 {
                POINT_KEY_BOS
            } else {
                token + 1
            })
        })
        .unwrap();
        assert_eq!(reply.tokens, vec![5, 6]);
        assert_eq!(reply.stop, "bos");
        assert_eq!(fed, vec![5, 6], "the prompt is not fed twice");
    }

    #[test]
    fn bounded_reply_stops_at_its_cap_or_context_deterministically() {
        let capped = bounded_reply(5, 3, 3, 20, |token| Ok(token + 1)).unwrap();
        assert_eq!((capped.tokens, capped.stop), (vec![5, 6, 7], "cap"));
        let context = bounded_reply(5, 15, 8, 17, |token| Ok(token + 1)).unwrap();
        assert_eq!((context.tokens, context.stop), (vec![5, 6], "context"));
    }

    #[test]
    fn reply_summary_counts_exact_opening_distinct_and_repeated_replies() {
        let rows = vec![
            (
                GeneratedReply {
                    tokens: vec![5, 6, 7, 8, 5, 6, 7, 8],
                    stop: "bos",
                },
                vec![5, 6, 7, 8, 5, 6, 7, 8, POINT_KEY_BOS],
                true,
            ),
            (
                GeneratedReply {
                    tokens: vec![5, 9],
                    stop: "bos",
                },
                vec![5, 6, POINT_KEY_BOS],
                true,
            ),
            (
                GeneratedReply {
                    tokens: vec![5],
                    stop: "cap",
                },
                vec![3, 6, POINT_KEY_BOS],
                false,
            ),
        ];
        let reading = summarize_replies(rows).unwrap();
        assert_eq!(reading.replies, 3);
        assert_eq!(reading.bos_complete, 2);
        assert_eq!(reading.cap_stopped, 1);
        assert_eq!(reading.exact_reference, 1);
        assert_eq!(reading.reference_reachable, 2);
        assert_eq!(reading.opening_correct, 2);
        assert_eq!(reading.non_empty, 3);
        assert_eq!(reading.distinct_non_empty, 3);
        assert_eq!(reading.repeated_4gram, 1);
        assert_eq!(reading.generated_tokens, 11);
        assert_eq!(reading.reference_tokens, 12);
    }

    #[test]
    fn reference_reachability_and_stop_normalization_have_exact_boundaries() {
        assert!(reference_reachable(10, 6, 6, 16), "exactly cap and context");
        assert!(!reference_reachable(10, 7, 6, 20), "cap plus one");
        assert!(!reference_reachable(10, 7, 8, 16), "context plus one");
        assert_eq!(normalized_reference(&[5, 6, POINT_KEY_BOS]), &[5, 6]);
        assert_eq!(
            normalized_reference(&[5, POINT_KEY_BOS, POINT_KEY_BOS]),
            &[5, POINT_KEY_BOS],
            "exactly one stop token is removed"
        );
        let empty = summarize_replies(vec![(
            GeneratedReply {
                tokens: Vec::new(),
                stop: "bos",
            },
            vec![POINT_KEY_BOS],
            true,
        )])
        .unwrap();
        assert_eq!(empty.exact_reference, 1);
        assert_eq!(empty.opening_correct, 0, "two absent openings do not match");
        assert_eq!(empty.non_empty, 0);
        assert!(add_count(u64::MAX, 1).is_err(), "counter overflow");
    }

    #[test]
    fn reply_sequence_digest_tracks_tokens_and_stop_markers() {
        let rows = vec![(
            GeneratedReply {
                tokens: vec![5, 6, 7],
                stop: "bos",
            },
            vec![5, 6, 7],
            true,
        )];
        let first = reply_sequence_sha256(&rows);
        assert_eq!(first, reply_sequence_sha256(&rows));
        let mut changed_token = rows.clone();
        changed_token[0].0.tokens[1] = 8;
        assert_ne!(first, reply_sequence_sha256(&changed_token));
        let mut changed_stop = rows;
        changed_stop[0].0.stop = "cap";
        assert_ne!(first, reply_sequence_sha256(&changed_stop));
    }

    fn opening_token(artifact: &[u8], prompt: &[u32]) -> u32 {
        let mut forward = CpuFbsForward::from_artifact(artifact).unwrap();
        let mut token = None;
        for (index, &value) in prompt.iter().enumerate() {
            if index + 1 == prompt.len() {
                forward.mark_reply_start();
            }
            token = Some(forward.step(value, CpuFbsHead::Writer).unwrap().token);
        }
        token.unwrap()
    }

    #[test]
    fn unseen_exact_key_uses_the_training_derived_point_target() {
        let mut snapshot = synthetic_fbs_trunk_with(
            104,
            FbsSynth {
                vocab: 512,
                d: 64,
                mlp: 96,
                layers: 1,
                context: 24,
                kept: 8,
                gain_lo: 0.75,
                gain_span: 1.5,
                table: 0.5,
                decay_step: 0.25,
            },
        );
        snapshot.readout_nearest = false;
        snapshot.memory = Some(FbsMemorySnapshot {
            k: 8,
            key_window: 24,
            opening: vec![FbsMemoryEntry {
                key: vec![1, 2, 3],
                value: 7,
            }],
            opening_weight: vec![0.0; snapshot.width],
            reply: None,
        });
        let memory_artifact = export_fbs_trunk_uorlut01(&snapshot).unwrap();
        let prompt = vec![30, 31, 32, 33];
        let base = opening_token(&memory_artifact, &prompt);
        let target = (base + 1) % 512;
        let training = CpuConversationSample {
            document_ordinal: 0,
            prompt: vec![10, 11, 12, 13],
            reply: vec![target, 1],
        };
        let second_target = (target + 1) % 512;
        let second_training = CpuConversationSample {
            document_ordinal: 2,
            prompt: vec![20, 21, 22, 23],
            reply: vec![second_target, 1],
        };
        let third_training = CpuConversationSample {
            document_ordinal: 4,
            prompt: vec![40, 41, 42, 43],
            reply: vec![target, 1],
        };
        let fourth_training = CpuConversationSample {
            document_ordinal: 6,
            prompt: vec![50, 51, 52, 53],
            reply: vec![target, 1],
        };
        let held_out = CpuConversationSample {
            document_ordinal: 1,
            prompt,
            reply: vec![target, 1],
        };
        let point = derive_point_table(
            &memory_artifact,
            &[training, second_training, third_training, fourth_training],
            snapshot.width,
            snapshot.vocab,
        )
        .unwrap();
        let point_artifact = add_point_key_uorlut01(&memory_artifact, &point).unwrap();
        assert_ne!(opening_token(&memory_artifact, &held_out.prompt), target);
        let point_token = opening_token(&point_artifact, &held_out.prompt);
        assert!([target, second_target].contains(&point_token));
        let (nearest_query, nearest_point_token) = opening_query_and_token_with_head(
            &point_artifact,
            &held_out.prompt,
            CpuFbsHead::Nearest,
        )
        .unwrap();
        let (writer_query, writer_point_token) = opening_query_and_token_with_head(
            &point_artifact,
            &held_out.prompt,
            CpuFbsHead::Writer,
        )
        .unwrap();
        assert_eq!(nearest_query, writer_query);
        assert_eq!(nearest_point_token, writer_point_token);
        let medoid = derive_target_medoids(&point).unwrap();
        let medoid_artifact = add_point_key_uorlut01(&memory_artifact, &medoid).unwrap();
        let (medoid_query, medoid_token) =
            opening_query_and_token(&medoid_artifact, &held_out.prompt).unwrap();
        assert_eq!(point_vote(&medoid, &medoid_query, 1).unwrap(), medoid_token);
        let random_medoid = PointKeyImage {
            width: medoid.width,
            points: medoid.points.clone(),
            targets: shuffled_targets(&medoid.targets, 17).unwrap(),
            vote_k: 1,
        };
        let random_medoid_artifact =
            add_point_key_uorlut01(&memory_artifact, &random_medoid).unwrap();
        assert_ne!(
            opening_token(&random_medoid_artifact, &held_out.prompt),
            medoid_token
        );
        let random_point = PointKeyImage {
            width: point.width,
            points: point.points.clone(),
            targets: shuffled_targets(&point.targets, 17).unwrap(),
            vote_k: 1,
        };
        let random_artifact = add_point_key_uorlut01(&memory_artifact, &random_point).unwrap();
        assert_ne!(
            opening_token(&random_artifact, &held_out.prompt),
            point_token
        );
        let prototypes = derive_target_prototypes(&point, 2).unwrap();
        assert!(prototypes.rows() < point.rows());
        let prototype_artifact = add_point_key_uorlut01(&memory_artifact, &prototypes).unwrap();
        let (prototype_query, prototype_token) =
            opening_query_and_token(&prototype_artifact, &held_out.prompt).unwrap();
        assert_eq!(
            point_vote(&prototypes, &prototype_query, 1).unwrap(),
            prototype_token
        );
        let (random_prototypes, random_prototype_artifact) = (2..100)
            .find_map(|seed| {
                let image = PointKeyImage {
                    width: prototypes.width,
                    points: prototypes.points.clone(),
                    targets: shuffled_targets(&prototypes.targets, seed).ok()?,
                    vote_k: 1,
                };
                let artifact = add_point_key_uorlut01(&memory_artifact, &image).ok()?;
                (opening_token(&artifact, &held_out.prompt) != prototype_token)
                    .then_some((image, artifact))
            })
            .unwrap();
        assert_eq!(random_prototypes.points, prototypes.points);
        let mut treatment_targets = prototypes.targets.clone();
        let mut control_targets = random_prototypes.targets.clone();
        treatment_targets.sort_unstable();
        control_targets.sort_unstable();
        assert_eq!(control_targets, treatment_targets);
        let baseline_opening = evaluate_opening_outcomes(
            &memory_artifact,
            std::slice::from_ref(&held_out),
            snapshot.width,
        )
        .unwrap();
        for artifact in [
            point_artifact.as_slice(),
            medoid_artifact.as_slice(),
            prototype_artifact.as_slice(),
            random_prototype_artifact.as_slice(),
        ] {
            let opening = evaluate_opening_outcomes(
                artifact,
                std::slice::from_ref(&held_out),
                snapshot.width,
            )
            .unwrap();
            assert_eq!(
                opening[0].target_distance,
                baseline_opening[0].target_distance
            );
        }
        let audit =
            audit_memory_weight_samples(&prototype_artifact, &[memory_sample(&held_out)], true)
                .unwrap();
        assert!(audit.is_exact());
        let neighbors = evaluate_neighbor_vote(
            &memory_artifact,
            &point_artifact,
            None,
            &point,
            &random_point,
            std::slice::from_ref(&held_out),
            1,
        )
        .unwrap();
        assert_eq!(neighbors.decisions, 1);
        assert_eq!(neighbors.nearest_matches_served, 1);
        assert_eq!(neighbors.voted_matches_served, 1);
        assert_held_out_exact_keys_miss(&memory_artifact, &[memory_sample(&held_out)]).unwrap();
        let hit = CpuConversationSample {
            document_ordinal: 1,
            prompt: vec![1, 2, 3],
            reply: vec![target, 1],
        };
        assert!(
            assert_held_out_exact_keys_miss(&memory_artifact, &[memory_sample(&hit)],).is_err()
        );
    }

    #[test]
    fn overlapping_transfer_keys_are_refused() {
        let mut examples = Vec::new();
        for document in [0, 2, 4, 6, 1, 3, 5, 7] {
            examples.push(sample(document, 10, 100));
        }
        assert!(select_transfer_groups(&examples, &config()).is_err());
    }

    #[test]
    fn malformed_point_rows_are_refused() {
        let malformed = PointKeyImage {
            width: 2,
            points: vec![1, 2, 3],
            targets: vec![4, 5],
            vote_k: 1,
        };
        assert!(malformed.check(512).is_err());
        let empty = PointKeyImage {
            width: 2,
            points: Vec::new(),
            targets: Vec::new(),
            vote_k: 1,
        };
        assert!(empty.check(512).is_err());
    }
}
