use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use uor_r4_core::report_output;
use uor_r4_cpu_train::{
    audit_memory_weight_samples, evaluate_memory_weight_samples,
    evaluate_nearest_rival_gain_samples, random_gain_control, read_gain, validate_corpus_manifest,
    CpuConversationSample, CpuFbsForward, CpuFbsHead, CpuMaskedCorpus, CpuNearestRivalGainTrainer,
    MemoryWeightSample, ServedGainConfig,
};
use uor_r4_integer::stack::{read_memory_uorlut01, IntegerStackModel};

const STEPS: usize = 8;
const KEY_COUNT: usize = 8;
const PER_KEY: usize = 8;
const KEY_TOKENS: usize = 8;
const TRAINING_SAMPLES: usize = 58;
const EVALUATION_SAMPLES: usize = 56;
const EVALUATION_DECISIONS: u64 = 2_431;

#[derive(Clone)]
struct Config {
    source: PathBuf,
    baseline: PathBuf,
    tokens: PathBuf,
    mask: PathBuf,
    manifest: PathBuf,
    out: PathBuf,
    expected_source_sha256: String,
    expected_baseline_sha256: String,
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

fn main() {
    match run() {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("the report serializes")
        ),
        Err(error) => {
            eprintln!("cpu-nearest-rival-gain-driver: {error}");
            std::process::exit(1);
        }
    }
}

fn usage() -> &'static str {
    "usage: cpu-nearest-rival-gain-driver SOURCE BASELINE TOKENS MASK MANIFEST OUT \
     --expected-source-sha256 HEX --expected-baseline-sha256 HEX \
     --expected-tokens-sha256 HEX --expected-mask-sha256 HEX \
     --expected-manifest-sha256 HEX"
}

fn parse_args() -> Result<Config, String> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() < 6 {
        return Err(usage().to_owned());
    }
    let mut named = BTreeMap::new();
    let mut index = 6;
    while index < args.len() {
        let flag = &args[index];
        let value = args
            .get(index + 1)
            .ok_or_else(|| format!("{flag} needs a value"))?;
        let key = match flag.as_str() {
            "--expected-source-sha256" => "source",
            "--expected-baseline-sha256" => "baseline",
            "--expected-tokens-sha256" => "tokens",
            "--expected-mask-sha256" => "mask",
            "--expected-manifest-sha256" => "manifest",
            _ => return Err(format!("unknown argument {flag}\n{}", usage())),
        };
        named.insert(key, hex(value)?);
        index += 2;
    }
    let mut expected = |key: &str| {
        named
            .remove(key)
            .ok_or_else(|| format!("--expected-{key}-sha256 is required"))
    };
    Ok(Config {
        source: args[0].clone().into(),
        baseline: args[1].clone().into(),
        tokens: args[2].clone().into(),
        mask: args[3].clone().into(),
        manifest: args[4].clone().into(),
        out: args[5].clone().into(),
        expected_source_sha256: expected("source")?,
        expected_baseline_sha256: expected("baseline")?,
        expected_tokens_sha256: expected("tokens")?,
        expected_mask_sha256: expected("mask")?,
        expected_manifest_sha256: expected("manifest")?,
    })
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
    report_output::claim(&config.out).map_err(|error| {
        format!(
            "claim exclusive report directory {}: {error}",
            config.out.display()
        )
    })?;
    let result = run_claimed(&config, started);
    if let Err(error) = &result {
        let failure = serde_json::json!({
            "schema": "uor-r4.cpu-nearest-rival-gain-failure/1",
            "status": "INCOMPLETE_OR_UNVERIFIED",
            "substrate": "cpu-development-research",
            "qualification": "gpu-required",
            "error": error,
            "inputs": {
                "source": config.source,
                "source_sha256": config.expected_source_sha256,
                "baseline": config.baseline,
                "baseline_sha256": config.expected_baseline_sha256,
                "tokens": config.tokens,
                "tokens_sha256": config.expected_tokens_sha256,
                "mask": config.mask,
                "mask_sha256": config.expected_mask_sha256,
                "manifest": config.manifest,
                "manifest_sha256": config.expected_manifest_sha256,
            },
            "execution": {
                "threads": rayon::current_num_threads(),
            },
            "scope": "Preserved execution or instrument failure; not a capability verdict.",
            "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
            "peak_rss_kib": peak_rss_kib(),
        });
        let bytes = serde_json::to_vec_pretty(&failure)
            .map_err(|error| format!("serialize failed-attempt report: {error}"))?;
        write_new(&config.out.join("failed-attempt.json"), &bytes)?;
    }
    report_output::seal(&config.out)
        .map_err(|error| format!("seal report directory {}: {error}", config.out.display()))?;
    report_output::verify(&config.out)
        .map_err(|error| format!("verify sealed report directory: {error}"))?;
    result
}

fn run_claimed(config: &Config, started: Instant) -> Result<serde_json::Value, String> {
    let source = std::fs::read(&config.source).map_err(|error| error.to_string())?;
    let baseline = std::fs::read(&config.baseline).map_err(|error| error.to_string())?;
    let tokens = std::fs::read(&config.tokens).map_err(|error| error.to_string())?;
    let mask = std::fs::read(&config.mask).map_err(|error| error.to_string())?;
    let manifest = std::fs::read(&config.manifest).map_err(|error| error.to_string())?;
    for (name, bytes, expected) in [
        ("source", source.as_slice(), &config.expected_source_sha256),
        (
            "baseline",
            baseline.as_slice(),
            &config.expected_baseline_sha256,
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
    let model = IntegerStackModel::parse(&baseline).map_err(|error| error.to_string())?;
    let source_model = IntegerStackModel::parse(&source).map_err(|error| error.to_string())?;
    let shape = model.shape();
    let source_shape = source_model.shape();
    if shape.vocab != source_shape.vocab
        || shape.context != source_shape.context
        || shape.width != source_shape.width
        || identity.vocab != shape.vocab
        || identity.bos != 0
    {
        return Err("source, baseline or corpus shape differs".to_owned());
    }
    if read_memory_uorlut01(&baseline)
        .map_err(|error| error.to_string())?
        .is_none()
    {
        return Err("the frozen baseline does not carry turn memory".to_owned());
    }
    let corpus =
        CpuMaskedCorpus::from_bytes(&tokens, &mask, shape.vocab, shape.context, identity.bos)
            .map_err(|error| error.to_string())?;
    let examples = corpus
        .conversation_examples()
        .map_err(|error| error.to_string())?;
    let groups = select_groups(&examples);
    let training_samples = groups
        .iter()
        .flat_map(|group| group.training.iter())
        .map(memory_sample)
        .collect::<Vec<_>>();
    let evaluation_samples = groups
        .iter()
        .flat_map(|group| group.evaluation.iter())
        .map(memory_sample)
        .collect::<Vec<_>>();
    if groups.len() != KEY_COUNT
        || training_samples.len() != TRAINING_SAMPLES
        || evaluation_samples.len() != EVALUATION_SAMPLES
    {
        return Err(format!(
            "registered repeated-key selection differs: {} keys, {} training, {} evaluation",
            groups.len(),
            training_samples.len(),
            evaluation_samples.len()
        ));
    }

    let gain_config = ServedGainConfig { step_code: 1 };
    let mut trained = CpuNearestRivalGainTrainer::from_artifact_cached(
        &baseline,
        training_samples.clone(),
        gain_config,
        0,
    )
    .map_err(|error| error.to_string())?;
    let mut shuffled = CpuNearestRivalGainTrainer::from_artifact_cached(
        &baseline,
        training_samples.clone(),
        gain_config,
        1,
    )
    .map_err(|error| error.to_string())?;
    let shuffled_changes = shuffled.target_changes();
    let mut trained_steps = Vec::new();
    let mut shuffled_steps = Vec::new();
    for _ in 0..STEPS {
        trained_steps.push(trained.train_step().map_err(|error| error.to_string())?);
        shuffled_steps.push(shuffled.train_step().map_err(|error| error.to_string())?);
        check_stop(started, 30 * 60, "registered run")?;
    }
    let trained_artifact = trained.artifact().to_vec();
    let shuffled_artifact = shuffled.artifact().to_vec();
    assert_gain_only(&baseline, &trained_artifact)?;
    assert_gain_only(&baseline, &shuffled_artifact)?;
    write_new(&config.out.join("trained.uorlut"), &trained_artifact)?;
    write_new(&config.out.join("shuffled.uorlut"), &shuffled_artifact)?;

    let accepted_moves = trained_steps.iter().filter(|step| step.updated).count();
    let (random_artifact, random_moves) =
        random_gain_control(&baseline, accepted_moves, 1).map_err(|error| error.to_string())?;
    assert_gain_only(&baseline, &random_artifact)?;
    write_new(&config.out.join("random.uorlut"), &random_artifact)?;

    let trained_reading =
        evaluate_nearest_rival_gain_samples(&trained_artifact, &evaluation_samples)
            .map_err(|error| error.to_string())?;
    let frozen_reading = evaluate_nearest_rival_gain_samples(&baseline, &evaluation_samples)
        .map_err(|error| error.to_string())?;
    let shuffled_reading =
        evaluate_nearest_rival_gain_samples(&shuffled_artifact, &evaluation_samples)
            .map_err(|error| error.to_string())?;
    let random_reading = evaluate_nearest_rival_gain_samples(&random_artifact, &evaluation_samples)
        .map_err(|error| error.to_string())?;
    if [
        trained_reading.decisions,
        frozen_reading.decisions,
        shuffled_reading.decisions,
        random_reading.decisions,
    ]
    .iter()
    .any(|&decisions| decisions != EVALUATION_DECISIONS)
    {
        return Err("held-out decision count differs".to_owned());
    }

    let frozen_rows = decision_correctness(&baseline, &evaluation_samples, true)?;
    let trained_rows = decision_correctness(&trained_artifact, &evaluation_samples, true)?;
    let shuffled_rows = decision_correctness(&shuffled_artifact, &evaluation_samples, true)?;
    let random_rows = decision_correctness(&random_artifact, &evaluation_samples, true)?;
    let frozen_only = frozen_rows
        .iter()
        .zip(&trained_rows)
        .filter(|(frozen, trained)| **frozen && !**trained)
        .count();
    let trained_only = frozen_rows
        .iter()
        .zip(&trained_rows)
        .filter(|(frozen, trained)| !**frozen && **trained)
        .count();
    let paired_delta = trained_only as i64 - frozen_only as i64;
    let three_sigma = 3.0 * ((frozen_only + trained_only) as f64).sqrt();
    let frozen_top1 = frozen_rows.iter().filter(|&&correct| correct).count() as u64;
    let trained_top1 = trained_rows.iter().filter(|&&correct| correct).count() as u64;
    let shuffled_top1 = shuffled_rows.iter().filter(|&&correct| correct).count() as u64;
    let random_top1 = random_rows.iter().filter(|&&correct| correct).count() as u64;

    let memory_off = evaluate_memory_weight_samples(&source, &evaluation_samples, false)
        .map_err(|error| error.to_string())?;
    let trained_memory_off =
        evaluate_memory_weight_samples(&trained_artifact, &evaluation_samples, false)
            .map_err(|error| error.to_string())?;
    let parity = [
        ("frozen", &baseline, true),
        ("trained", &trained_artifact, true),
        ("random", &random_artifact, true),
        ("shuffled", &shuffled_artifact, true),
        ("memory_off", &source, false),
        ("trained_memory_off", &trained_artifact, false),
    ]
    .into_iter()
    .map(|(name, bytes, memory)| {
        audit_memory_weight_samples(bytes, &evaluation_samples, memory)
            .map(|reading| (name, reading))
            .map_err(|error| error.to_string())
    })
    .collect::<Result<BTreeMap<_, _>, _>>()?;
    if parity.values().any(|reading| !reading.is_exact()) {
        return Err("a held-out arm lost CPU/serving parity".to_owned());
    }

    let activity = accepted_moves > 0 && shuffled_changes > 0;
    let shuffle_behavioral = trained_artifact != shuffled_artifact;
    let hinge_better = trained_reading.rival_hinge_sum < frozen_reading.rival_hinge_sum
        && trained_reading.rival_hinge_sum < random_reading.rival_hinge_sum
        && trained_reading.rival_hinge_sum < shuffled_reading.rival_hinge_sum;
    let top1_nonfall = trained_top1 >= shuffled_top1 && trained_top1 >= random_top1;
    let ranking = paired_delta as f64 > three_sigma;
    let verdict = trial_verdict(
        activity,
        shuffle_behavioral,
        hinge_better,
        top1_nonfall,
        ranking,
    );

    let report = serde_json::json!({
        "schema": "uor-r4.cpu-nearest-rival-gain-run/1",
        "substrate": "cpu-development-research",
        "qualification": "gpu-required",
        "inputs": {
            "source": config.source,
            "source_sha256": config.expected_source_sha256,
            "baseline": config.baseline,
            "baseline_sha256": config.expected_baseline_sha256,
            "tokens_sha256": config.expected_tokens_sha256,
            "mask_sha256": config.expected_mask_sha256,
            "manifest_sha256": config.expected_manifest_sha256,
        },
        "selection": {
            "threads": rayon::current_num_threads(),
            "keys": KEY_COUNT,
            "training_samples": training_samples.len(),
            "evaluation_samples": evaluation_samples.len(),
            "key_groups": groups.iter().map(|group| serde_json::json!({
                "key": group.key,
                "training": group.training.len(),
                "evaluation": group.evaluation.len(),
            })).collect::<Vec<_>>(),
        },
        "training": {
            "objective": "mean_hinge_against_eight_nearest_rivals",
            "step_code": gain_config.step_code,
            "steps": STEPS,
            "accepted_updates": accepted_moves,
            "accepted": trained_steps,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&trained_artifact),
            "final_gain": read_gain(&trained_artifact).map_err(|error| error.to_string())?,
        },
        "shuffled_target": {
            "shift": shuffled.target_shift(),
            "changed_targets": shuffled_changes,
            "accepted_updates": shuffled_steps.iter().filter(|step| step.updated).count(),
            "accepted": shuffled_steps,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&shuffled_artifact),
            "final_gain": read_gain(&shuffled_artifact).map_err(|error| error.to_string())?,
        },
        "random_control": {
            "moves": random_moves,
            "artifact_sha256": uor_r4_discrete::corpus::sha256(&random_artifact),
        },
        "readings": {
            "frozen": frozen_reading,
            "trained": trained_reading,
            "random": random_reading,
            "shuffled": shuffled_reading,
            "memory_off": memory_off,
            "trained_memory_off": trained_memory_off,
        },
        "pairing": {
            "frozen_top1": frozen_top1,
            "trained_top1": trained_top1,
            "random_top1": random_top1,
            "shuffled_top1": shuffled_top1,
            "frozen_only": frozen_only,
            "trained_only": trained_only,
            "paired_delta": paired_delta,
            "three_sigma_threshold": three_sigma,
        },
        "parity": parity,
        "bar": {
            "activity": if activity { "pass" } else { "fail" },
            "shuffle_behavioral": if shuffle_behavioral { "pass" } else { "fail" },
            "hinge_below_frozen_random_shuffled": if hinge_better { "pass" } else { "fail" },
            "top1_nonfall": if top1_nonfall { "pass" } else { "fail" },
            "paired_top1_ranking": if ranking { "pass" } else { "fail" },
            "verdict": verdict,
        },
        "wall_elapsed_ms": started.elapsed().as_secs_f64() * 1_000.0,
        "peak_rss_kib": peak_rss_kib(),
    });
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    write_new(&config.out.join("report.json"), &bytes)?;
    Ok(report)
}

fn trial_verdict(
    activity: bool,
    shuffle_behavioral: bool,
    hinge_better: bool,
    top1_nonfall: bool,
    ranking: bool,
) -> &'static str {
    if !shuffle_behavioral {
        "CONTROL_FAULT"
    } else if activity && hinge_better && top1_nonfall && ranking {
        "KEEP"
    } else {
        "REJECT"
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut file = std::fs::File::create_new(path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    file.write_all(bytes)
        .map_err(|error| format!("write {}: {error}", path.display()))
}

fn memory_sample(sample: &CpuConversationSample) -> MemoryWeightSample {
    MemoryWeightSample {
        prompt: sample.prompt.clone(),
        reply: sample.reply.clone(),
    }
}

fn select_groups(examples: &[CpuConversationSample]) -> Vec<KeyGroup> {
    let mut groups =
        BTreeMap::<Vec<u32>, (Vec<CpuConversationSample>, Vec<CpuConversationSample>)>::new();
    for sample in examples {
        let key = sample
            .prompt
            .iter()
            .copied()
            .take(KEY_TOKENS)
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
            training.truncate(PER_KEY);
            evaluation.truncate(PER_KEY);
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
    groups.truncate(KEY_COUNT);
    groups
}

fn decision_correctness(
    artifact: &[u8],
    samples: &[MemoryWeightSample],
    memory_enabled: bool,
) -> Result<Vec<bool>, String> {
    let mut correctness = Vec::new();
    for sample in samples {
        let mut forward =
            CpuFbsForward::from_artifact(artifact).map_err(|error| error.to_string())?;
        let tokens = sample
            .prompt
            .iter()
            .chain(&sample.reply)
            .copied()
            .collect::<Vec<_>>();
        for index in 0..tokens.len() - 1 {
            if memory_enabled && index + 1 == sample.prompt.len() {
                forward.mark_reply_start();
            }
            let step = forward
                .step(tokens[index], CpuFbsHead::Writer)
                .map_err(|error| error.to_string())?;
            correctness.push(step.token == tokens[index + 1]);
        }
    }
    Ok(correctness)
}

fn assert_gain_only(baseline: &[u8], artifact: &[u8]) -> Result<(), String> {
    let gain = read_gain(artifact).map_err(|error| error.to_string())?;
    let rebuilt =
        uor_r4_cpu_train::replace_final_gain(baseline, &gain).map_err(|error| error.to_string())?;
    if rebuilt != artifact {
        return Err("artifact changed bytes outside final_norm_gain".to_owned());
    }
    Ok(())
}

fn check_stop(started: Instant, limit_seconds: u64, name: &str) -> Result<(), String> {
    if started.elapsed().as_secs() > limit_seconds {
        return Err(format!("{name} exceeded its registered stop"));
    }
    if peak_rss_kib() > 2 * 1024 * 1024 {
        return Err(format!("{name} exceeded the 2 GiB RSS stop"));
    }
    Ok(())
}

fn peak_rss_kib() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    status
        .lines()
        .find_map(|line| line.strip_prefix("VmHWM:"))
        .and_then(|value| value.split_whitespace().next())
        .and_then(|value| value.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::trial_verdict;

    #[test]
    fn a_behaviorally_vacuous_shuffle_is_a_control_fault_before_a_verdict() {
        assert_eq!(
            trial_verdict(true, false, true, true, true),
            "CONTROL_FAULT"
        );
    }

    #[test]
    fn a_complete_pass_requires_every_bar() {
        assert_eq!(trial_verdict(true, true, true, true, true), "KEEP");
        assert_eq!(trial_verdict(true, true, true, true, false), "REJECT");
    }
}
