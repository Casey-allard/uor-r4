use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_cpu_train::{
    add_turn_memory_uorlut01, audit_self_prefix_generated, audit_self_prefix_target_permutation,
    derive_turn_memory, evaluate_memory_weight_samples, evaluate_self_prefix,
    evaluate_self_prefix_admission, evaluate_self_prefix_prompts, read_gain, replace_final_gain,
    validate_corpus_manifest, CpuConversationSample, CpuMaskedCorpus, CpuQueryAdmissionTrainer,
    MemoryWeightSample, RankMarginConfig, SelfPrefixAdmissionCensus, SelfPrefixConfig,
    SelfPrefixMode, SelfPrefixPromptReading, SelfPrefixReading, SelfPrefixTargets,
};
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

#[derive(Clone)]
struct Config {
    artifact: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    key_count: usize,
    per_key: usize,
    key_tokens: usize,
    reply_turn: usize,
    reply_cap: usize,
    moves: usize,
    seed: u64,
    select_only: bool,
    expected_artifact_sha256: String,
    expected_tokens_sha256: String,
    expected_mask_sha256: String,
    expected_manifest_sha256: String,
}

#[derive(Clone)]
struct KeyGroup {
    key: Vec<u32>,
    training: Vec<CpuConversationSample>,
    evaluation: Vec<CpuConversationSample>,
}

struct ArmReading {
    artifact: Vec<u8>,
    training: SelfPrefixAdmissionCensus,
    evaluation: SelfPrefixAdmissionCensus,
    generated: SelfPrefixReading,
    prompts: Vec<SelfPrefixPromptReading>,
    teacher_top1: u64,
    parity: uor_r4_cpu_train::MemoryWeightParityReading,
    moves: serde_json::Value,
    accepted: u64,
}

#[derive(Clone, Debug, serde::Serialize)]
struct RandomGainMove {
    coordinate: usize,
    before_code: i16,
    after_code: i16,
}

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-query-admission-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> String {
    "usage: cpu-query-admission-driver ARTIFACT TOKENS MASK MANIFEST OUT \
     --expected-artifact-sha256 HEX --expected-tokens-sha256 HEX \
     --expected-mask-sha256 HEX --expected-manifest-sha256 HEX \
     [--keys N] [--per-key N] [--key-tokens N] [--reply-turn N] \
     [--reply-cap N] [--moves N] [--seed N] [--select-only]"
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
        key_count: 16,
        per_key: 8,
        key_tokens: 8,
        reply_turn: 8,
        reply_cap: 16,
        moves: 12,
        seed: 1,
        select_only: false,
        expected_artifact_sha256: String::new(),
        expected_tokens_sha256: String::new(),
        expected_mask_sha256: String::new(),
        expected_manifest_sha256: String::new(),
    };
    while let Some(flag) = values.next() {
        if flag == "--select-only" {
            config.select_only = true;
            continue;
        }
        let value = values
            .next()
            .ok_or_else(|| format!("{flag} needs a value"))?;
        match flag.as_str() {
            "--keys" => config.key_count = number(&value)?,
            "--per-key" => config.per_key = number(&value)?,
            "--key-tokens" => config.key_tokens = number(&value)?,
            "--reply-turn" => config.reply_turn = number(&value)?,
            "--reply-cap" => config.reply_cap = number(&value)?,
            "--moves" => config.moves = number(&value)?,
            "--seed" => config.seed = number(&value)?,
            "--expected-artifact-sha256" => config.expected_artifact_sha256 = hex(&value)?,
            "--expected-tokens-sha256" => config.expected_tokens_sha256 = hex(&value)?,
            "--expected-mask-sha256" => config.expected_mask_sha256 = hex(&value)?,
            "--expected-manifest-sha256" => config.expected_manifest_sha256 = hex(&value)?,
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
    if config.key_count == 0
        || config.per_key == 0
        || !(2..=16).contains(&config.key_tokens)
        || !(1..=16).contains(&config.reply_turn)
        || !(1..=64).contains(&config.reply_cap)
        || !(1..=12).contains(&config.moves)
    {
        return Err("group, reply or move dimensions are out of bounds".to_owned());
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
    let model = IntegerStackModel::parse(&artifact).map_err(|error| error.to_string())?;
    let shape = model.shape();
    if identity.vocab != shape.vocab || identity.bos != 0 {
        return Err("corpus vocabulary or BOS differs from the artifact".to_owned());
    }
    if shape.kept_cells != Some(8) {
        return Err("query-admission source artifact is not K=8".to_owned());
    }
    if read_memory_uorlut01(&artifact)
        .map_err(|error| error.to_string())?
        .is_some()
    {
        return Err("the source artifact already carries memory".to_owned());
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&tokens, &mask, shape.vocab, shape.context, identity.bos)
            .map_err(|error| error.to_string())?;
    let examples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let groups = select_groups(&examples, &config);
    if groups.len() != config.key_count {
        return Err(format!(
            "selected {} keys, expected {}",
            groups.len(),
            config.key_count
        ));
    }
    let training_samples = groups
        .iter()
        .flat_map(|group| group.training.iter().map(memory_sample))
        .collect::<Vec<_>>();
    let evaluation_samples = groups
        .iter()
        .flat_map(|group| group.evaluation.iter().map(memory_sample))
        .collect::<Vec<_>>();
    if training_samples.len() != 98 || evaluation_samples.len() != 103 {
        return Err(format!(
            "selected split differs: {} training, {} evaluation",
            training_samples.len(),
            evaluation_samples.len()
        ));
    }
    let memory = derive_turn_memory(
        &training_samples,
        shape.width,
        config.key_tokens,
        shape.context,
        config.reply_turn,
    )
    .map_err(|error| error.to_string())?;
    let base_artifact =
        add_turn_memory_uorlut01(&artifact, &memory).map_err(|error| error.to_string())?;
    let base = SelfPrefixConfig {
        rank_margin: RankMarginConfig::default(),
        reply_cap: config.reply_cap,
        memory_enabled: true,
        mode: SelfPrefixMode::DynamicGenerated,
        targets: SelfPrefixTargets::TrueLabels,
        shuffle_seed: config.seed,
    };
    let training_preflight =
        CpuQueryAdmissionTrainer::from_artifact(&base_artifact, training_samples.clone(), base, 32)
            .map_err(|error| error.to_string())?;
    if config.select_only {
        return Ok(serde_json::json!({
            "schema": "uor-r4.cpu-query-admission-selection/1",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "selection": selection_json(&groups, &training_samples, &evaluation_samples),
            "objective": training_preflight.reading().map_err(|error| error.to_string())?,
            "base_artifact_sha256": uor_r4_discrete::corpus::sha256(&base_artifact),
        }));
    }
    create_fresh_output(&config.out)?;
    std::fs::write(config.out.join("frozen.uorlut"), &base_artifact)
        .map_err(|error| error.to_string())?;
    let frozen = evaluate_arm(
        &base_artifact,
        &training_samples,
        &evaluation_samples,
        base,
        serde_json::json!([]),
        0,
    )?;
    let mut true_trainer =
        CpuQueryAdmissionTrainer::from_artifact(&base_artifact, training_samples.clone(), base, 32)
            .map_err(|error| error.to_string())?;
    let true_moves = true_trainer
        .train_steps(config.moves)
        .map_err(|error| error.to_string())?;
    let true_moves_json = serde_json::to_value(&true_moves).map_err(|error| error.to_string())?;
    let trained = evaluate_arm(
        true_trainer.artifact(),
        &training_samples,
        &evaluation_samples,
        base,
        true_moves_json,
        true_trainer.accepted(),
    )?;
    std::fs::write(config.out.join("true.uorlut"), &trained.artifact)
        .map_err(|error| error.to_string())?;
    let permuted_config = SelfPrefixConfig {
        targets: SelfPrefixTargets::CrossCellPermuted,
        ..base
    };
    let mut permuted_trainer = CpuQueryAdmissionTrainer::from_artifact(
        &base_artifact,
        training_samples.clone(),
        permuted_config,
        32,
    )
    .map_err(|error| error.to_string())?;
    let permuted_moves = permuted_trainer
        .train_steps(config.moves)
        .map_err(|error| error.to_string())?;
    let permuted_moves_json =
        serde_json::to_value(&permuted_moves).map_err(|error| error.to_string())?;
    let permuted = evaluate_arm(
        permuted_trainer.artifact(),
        &training_samples,
        &evaluation_samples,
        base,
        permuted_moves_json,
        permuted_trainer.accepted(),
    )?;
    std::fs::write(config.out.join("permuted.uorlut"), &permuted.artifact)
        .map_err(|error| error.to_string())?;
    let (random_artifact, random_moves) =
        move_random_gain(&base_artifact, trained.accepted as usize, config.seed, 32)
            .map_err(|error| error.to_string())?;
    let random = evaluate_arm(
        &random_artifact,
        &training_samples,
        &evaluation_samples,
        base,
        serde_json::to_value(&random_moves).map_err(|error| error.to_string())?,
        random_moves.len() as u64,
    )?;
    std::fs::write(config.out.join("random.uorlut"), &random.artifact)
        .map_err(|error| error.to_string())?;
    let true_training = k8(&trained.training)?;
    let frozen_training = k8(&frozen.training)?;
    let true_eval = k8(&trained.evaluation)?;
    let frozen_eval = k8(&frozen.evaluation)?;
    let permuted_eval = k8(&permuted.evaluation)?;
    let random_eval = k8(&random.evaluation)?;
    let gain_only = gain_only_changed(&base_artifact, &trained.artifact);
    let permuted_gain_only = gain_only_changed(&base_artifact, &permuted.artifact);
    let random_gain_only = gain_only_changed(&base_artifact, &random.artifact);
    let permutation_audit = audit_self_prefix_target_permutation(
        &base_artifact,
        &training_samples,
        base,
        permuted_config,
    )
    .map_err(|error| error.to_string())?;
    let random_control_active = trained.accepted > 0 && random.accepted == trained.accepted;
    let bar = serde_json::json!({
        "training_positive_margin_frozen": frozen_training.positive_margin,
        "training_positive_margin_true": true_training.positive_margin,
        "training_positive_margin_pairs_true": true_training.positive_margin_pairs,
        "training_positive_margin_pass": true_training.positive_margin >= 640
            && true_training.positive_margin_pairs >= 64,
        "heldout_admitted_frozen": frozen_eval.target_admitted,
        "heldout_admitted_true": true_eval.target_admitted,
        "heldout_added": true_eval.target_admitted.saturating_sub(frozen_eval.target_admitted),
        "heldout_added_pass": true_eval.target_admitted >= frozen_eval.target_admitted + 128,
        "true_above_permuted": true_eval.target_admitted > permuted_eval.target_admitted,
        "true_above_random": true_eval.target_admitted > random_eval.target_admitted,
        "depth_nonloss": trained.generated.matched_prefix_depth_sum
            >= frozen.generated.matched_prefix_depth_sum,
        "teacher_top1_nonloss": trained.teacher_top1 >= frozen.teacher_top1,
        "gain_only": gain_only,
        "permuted_gain_only": permuted_gain_only,
        "random_gain_only": random_gain_only,
        "permutation_assignments": permutation_audit.assignments,
        "permutation_changed_assignments": permutation_audit.changed_assignments,
        "permutation_cross_cell_assignments": permutation_audit.cross_cell_assignments,
        "permutation_sha256": permutation_audit.permutation_sha256,
        "all_parity_exact": frozen.parity.is_exact()
            && trained.parity.is_exact()
            && permuted.parity.is_exact()
            && random.parity.is_exact(),
        "random_moves": random.accepted,
        "random_control_active": random_control_active,
        "passes": true_training.positive_margin >= 640
            && true_training.positive_margin_pairs >= 64
            && true_eval.target_admitted >= frozen_eval.target_admitted + 128
            && true_eval.target_admitted > permuted_eval.target_admitted
            && true_eval.target_admitted > random_eval.target_admitted
            && trained.generated.matched_prefix_depth_sum
                >= frozen.generated.matched_prefix_depth_sum
            && trained.teacher_top1 >= frozen.teacher_top1
            && gain_only
            && permuted_gain_only
            && random_gain_only
            && permutation_audit.cross_cell_assignments > 0
            && random_control_active
            && frozen.parity.is_exact()
            && trained.parity.is_exact()
            && permuted.parity.is_exact()
            && random.parity.is_exact(),
    });
    let report = serde_json::json!({
        "schema": "uor-r4.cpu-query-admission-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "claim": "generated-query final-gain training toward K8 admission; no serving or language claim",
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
        "selection": selection_json(&groups, &training_samples, &evaluation_samples),
        "arms": {
            "frozen": arm_json(&frozen),
            "true": arm_json(&trained),
            "permuted": arm_json(&permuted),
            "random": arm_json(&random),
        },
        "bar": bar,
        "permutation": permutation_audit,
        "rayon_workers": rayon::current_num_threads(),
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib(),
    });
    std::fs::write(
        config.out.join("report.json"),
        serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok(report)
}

fn evaluate_arm(
    artifact: &[u8],
    training: &[MemoryWeightSample],
    evaluation: &[MemoryWeightSample],
    config: SelfPrefixConfig,
    moves: serde_json::Value,
    accepted: u64,
) -> Result<ArmReading, String> {
    Ok(ArmReading {
        artifact: artifact.to_vec(),
        training: evaluate_self_prefix_admission(artifact, training, config)
            .map_err(|error| error.to_string())?,
        evaluation: evaluate_self_prefix_admission(artifact, evaluation, config)
            .map_err(|error| error.to_string())?,
        generated: evaluate_self_prefix(artifact, evaluation, config)
            .map_err(|error| error.to_string())?,
        prompts: evaluate_self_prefix_prompts(artifact, evaluation, config)
            .map_err(|error| error.to_string())?,
        teacher_top1: evaluate_memory_weight_samples(artifact, evaluation, true)
            .map_err(|error| error.to_string())?
            .top1_correct,
        parity: audit_self_prefix_generated(artifact, evaluation, config)
            .map_err(|error| error.to_string())?,
        moves,
        accepted,
    })
}

fn arm_json(arm: &ArmReading) -> serde_json::Value {
    serde_json::json!({
        "artifact_sha256": uor_r4_discrete::corpus::sha256(&arm.artifact),
        "accepted": arm.accepted,
        "moves": arm.moves,
        "training": arm.training,
        "evaluation": arm.evaluation,
        "generated": arm.generated,
        "prompts": arm.prompts,
        "teacher_top1": arm.teacher_top1,
        "parity": arm.parity,
    })
}

fn k8(census: &SelfPrefixAdmissionCensus) -> Result<uor_r4_cpu_train::CellKReading, String> {
    census
        .readings
        .iter()
        .find(|reading| reading.kept_cells == 8)
        .cloned()
        .ok_or_else(|| "K=8 admission reading is missing".to_owned())
}

fn move_random_gain(
    artifact: &[u8],
    count: usize,
    seed: u64,
    step: i16,
) -> Result<(Vec<u8>, Vec<RandomGainMove>), String> {
    let mut gain = read_gain(artifact).map_err(|error| error.to_string())?;
    let mut used = BTreeSet::new();
    let mut out = artifact.to_vec();
    let mut moves = Vec::with_capacity(count);
    let mut state = seed ^ 0xA076_1D64_78BD_642F;
    for _ in 0..count {
        loop {
            state ^= state >> 12;
            state ^= state << 25;
            state ^= state >> 27;
            let draw = state.wrapping_mul(0x2545_F491_4F6C_DD1D);
            let coordinate = draw as usize % gain.len();
            if !used.insert(coordinate) {
                continue;
            }
            let delta = if (draw >> 63) == 0 { step } else { -step };
            let before_code = gain[coordinate];
            let after_code = before_code.saturating_add(delta);
            if after_code == before_code {
                continue;
            }
            gain[coordinate] = after_code;
            out = replace_final_gain(&out, &gain).map_err(|error| error.to_string())?;
            moves.push(RandomGainMove {
                coordinate,
                before_code,
                after_code,
            });
            break;
        }
    }
    Ok((out, moves))
}

fn gain_only_changed(before: &[u8], after: &[u8]) -> bool {
    let before_span = final_gain_span(before);
    let after_span = final_gain_span(after);
    let (Some((before_start, before_end)), Some((after_start, after_end))) =
        (before_span, after_span)
    else {
        return false;
    };
    before.len() == after.len()
        && before_start == after_start
        && before_end == after_end
        && before_end <= before.len()
        && after_end <= after.len()
        && before[..before_start] == after[..after_start]
        && before[before_end..] == after[after_end..]
        && before[before_start..before_end] != after[after_start..after_end]
}

fn final_gain_span(artifact: &[u8]) -> Option<(usize, usize)> {
    let header_len =
        usize::try_from(u64::from_le_bytes(artifact.get(8..16)?.try_into().ok()?)).ok()?;
    let header_end = 16usize.checked_add(header_len)?;
    let data_start = header_end.div_ceil(64) * 64;
    let header: serde_json::Value = serde_json::from_slice(artifact.get(16..header_end)?).ok()?;
    let entry = header["tables"]
        .as_array()?
        .iter()
        .find(|entry| entry["name"].as_str() == Some("final_norm_gain"))?;
    let offset = usize::try_from(entry["span"]["offset"].as_u64()?).ok()?;
    let bytes = usize::try_from(entry["span"]["bytes"].as_u64()?).ok()?;
    let start = data_start.checked_add(offset)?;
    Some((start, start.checked_add(bytes)?))
}

fn selection_json(
    groups: &[KeyGroup],
    training: &[MemoryWeightSample],
    evaluation: &[MemoryWeightSample],
) -> serde_json::Value {
    serde_json::json!({
        "groups": groups.len(),
        "training_samples": training.len(),
        "evaluation_samples": evaluation.len(),
        "training_content_decisions": training.iter().map(|sample| sample.reply.len().saturating_sub(1)).sum::<usize>(),
        "evaluation_content_decisions": evaluation.iter().map(|sample| sample.reply.len().saturating_sub(1)).sum::<usize>(),
        "keys": groups.iter().map(|group| serde_json::json!({
            "key": group.key,
            "training": group.training.len(),
            "evaluation": group.evaluation.len(),
        })).collect::<Vec<_>>(),
    })
}

fn memory_sample(sample: &CpuConversationSample) -> MemoryWeightSample {
    MemoryWeightSample {
        prompt: sample.prompt.clone(),
        reply: sample.reply.clone(),
    }
}

fn select_groups(examples: &[CpuConversationSample], config: &Config) -> Vec<KeyGroup> {
    let mut groups =
        BTreeMap::<Vec<u32>, (Vec<CpuConversationSample>, Vec<CpuConversationSample>)>::new();
    for sample in examples {
        let key = sample
            .prompt
            .iter()
            .copied()
            .take(config.key_tokens)
            .collect::<Vec<_>>();
        let entry = groups.entry(key).or_default();
        if sample.document_ordinal % 2 == 0 {
            entry.0.push(sample.clone());
        } else {
            entry.1.push(sample.clone());
        }
    }
    let mut groups = groups
        .into_iter()
        .filter_map(|(key, (mut training, mut evaluation))| {
            if training.len() < 4 || evaluation.len() < 4 {
                return None;
            }
            training.truncate(config.per_key);
            evaluation.truncate(config.per_key);
            Some(KeyGroup {
                key,
                training,
                evaluation,
            })
        })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        right
            .training
            .len()
            .min(right.evaluation.len())
            .cmp(&left.training.len().min(left.evaluation.len()))
            .then_with(|| right.evaluation.len().cmp(&left.evaluation.len()))
            .then_with(|| right.training.len().cmp(&left.training.len()))
            .then_with(|| right.key.cmp(&left.key))
    });
    groups.truncate(config.key_count);
    groups
}

fn create_fresh_output(path: &Path) -> Result<(), String> {
    std::fs::create_dir(path).map_err(|error| format!("{}: {error}", path.display()))
}

fn peak_rss_kib() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("VmHWM:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|value| value.parse().ok())
            })
        })
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact_with_gain(gain: [u8; 4]) -> Vec<u8> {
        let header = br#"{"tables":[{"name":"final_norm_gain","span":{"offset":0,"bytes":4}}]}"#;
        let mut out = Vec::new();
        out.extend_from_slice(b"UORLUT01");
        out.extend_from_slice(&(header.len() as u64).to_le_bytes());
        out.extend_from_slice(header);
        out.resize((16 + header.len()).div_ceil(64) * 64, 0);
        out.extend_from_slice(&gain);
        out
    }

    #[test]
    fn gain_only_comparison_requires_changed_gain_and_matching_other_bytes() {
        let before = artifact_with_gain([0; 4]);
        let mut after = before.clone();
        let (start, end) = final_gain_span(&before).unwrap();
        after[start..end].fill(7);
        assert!(gain_only_changed(&before, &after));
        after[0] ^= 1;
        assert!(!gain_only_changed(&before, &after));
        assert!(!gain_only_changed(&before, &before));
    }
}
