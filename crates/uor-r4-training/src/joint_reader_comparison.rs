//! Fixed three-reader adaptation comparison over existing sealed evaluations.
//! No model loading, generation, training, new score threshold or promotion.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;

use crate::baseline_protocol::{read_tokens, save_json, verify_identity};
use crate::joint_campaign::{finish_attempt, Campaign, ReadInitialization};
use crate::joint_comparison::{joint_row, new_output_path, read_input, read_json, validate_joint};
use crate::joint_evaluation::{
    story_probes, GREEDY_POLICY, SEEDED_POLICY, STORY_PROBE_SCOPE, STORY_STOP_POLICY,
};
use crate::joint_model::{JointConfig, ReadGeometry, ReadMode, Transport};
use crate::joint_optimizer::AdamConfig;
use crate::joint_transfer::TransferReceipt;
use crate::{invalid, sha256_file, Result};

const EVALUATOR: &[u8] = include_bytes!("../../../docs/integration/reference-evaluator-v2.json");
const EVALUATOR_SHA: &str = "d2432fbba0e24ba51d7568700d6718c4e85d01ccc08e4fc3cc3fa2a77e928a62";
const PARENT_MODEL: &str = "6defec21fc2be395a9505b9f10301c03aeb3c23529c6e1be4e2e1639c7ce6d79";
const PARENT_CHECKPOINT: &str = "490922decefee3e6036331dd0bc112746f24662d4bddfbe540b59c1d00820571";
const PARENT_CAMPAIGN: &str = "6b34a04d49fa965dc20335002a1948a8253ef74e70890ba1e9484f63d2677f14";
const PARENT_SOURCE: &str = "b5b5fe757b9513877911719f014e73165f704b13";
const SHARED_MANIFEST: &str = "dd13ad77bd54b3117d90e5e85814a2b9f3c5cb0392e2834c14fae7f5eef5f3a5";
const CONTEXT: usize = 256;
const BLOCKS: usize = 976;
const TARGETS: usize = BLOCKS * CONTEXT;
const NAMES: [&str; 7] = [
    "parent_read",
    "dot_read",
    "dot_no_read",
    "lorentz_read",
    "lorentz_no_read",
    "affine_read",
    "affine_no_read",
];
const PAIRS: [(usize, usize); 9] = [
    (3, 5),
    (3, 1),
    (5, 1),
    (1, 0),
    (3, 0),
    (5, 0),
    (2, 1),
    (4, 3),
    (6, 5),
];
const SCOPE: &str = "Fixed1024-update development comparison. Parent is historical context; adapted Dot is the matched reset control. No automatic prose grade, new acceptance threshold, promotion, curvature, integer-serving, held-out generalization or efficiency claim.";

/// PARENT_READ DOT_READ DOT_NOREAD LORENTZ_READ LORENTZ_NOREAD AFFINE_READ
/// AFFINE_NOREAD NEW_ROOT. Every input is an already sealed joint-evaluate root.
pub fn run_cli(args: &[String]) -> Result<()> {
    if args.len() != 9
        || args[0] != "joint-reader-compare"
        || args[1..].iter().any(String::is_empty)
    {
        return Err(invalid("usage: joint-reader-compare PARENT_READ DOT_READ DOT_NOREAD LORENTZ_READ LORENTZ_NOREAD AFFINE_READ AFFINE_NOREAD NEW_ROOT"));
    }
    let roots = args[1..8]
        .iter()
        .map(fs::canonicalize)
        .collect::<std::io::Result<Vec<_>>>()?;
    let out = new_output_path(Path::new(&args[8]))?;
    if roots.iter().collect::<BTreeSet<_>>().len() != 7
        || roots.iter().any(|p| !p.is_dir() || out.starts_with(p))
    {
        return Err(invalid(
            "reader comparison needs seven distinct sealed roots and a separate new output",
        ));
    }
    report_output::claim(&out)?;
    let result = compare(&roots, &out);
    finish_attempt(&out, result)
}

fn mode(index: usize) -> ReadMode {
    if index != 0 && index % 2 == 0 {
        ReadMode::NoRead
    } else {
        ReadMode::Enabled
    }
}

fn compare(roots: &[PathBuf], out: &Path) -> Result<()> {
    let source = option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNBOUND");
    if source == "UNBOUND" || hex::encode(Sha256::digest(EVALUATOR)) != EVALUATOR_SHA {
        return Err(invalid(
            "reader comparison requires a source-bound build and canonical evaluator",
        ));
    }
    let evaluator: Value = serde_json::from_slice(EVALUATOR)?;
    save_json(&out.join("evaluator.json"), &evaluator)?;
    let mut bindings = Vec::new();
    let mut reports = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let report = read_input(
            root,
            NAMES[index],
            "evaluation-report.json",
            "targets.bin",
            &mut bindings,
        )?;
        validate_joint(&report, Transport::Quaternion, mode(index))?;
        // read_input binds the recorded evaluator digest to our compiled
        // manifest, including its ordered training stores. Evaluation roots
        // historically do not copy evaluator.json; do not require a new replay.
        if report["mode"] != "joint-evaluate"
            || report["admission_policy"] != "full"
            || !report["executed_quantization"].is_null()
            || !report["checkpoint_binding"]["quantization"].is_null()
            || report["evaluation_quantization_strength"] != 0.0
        {
            return Err(invalid(format!(
                "{} is not the canonical continuous Full evaluation",
                NAMES[index]
            )));
        }
        reports.push(report);
    }
    let receipts = validate_study(&reports, &evaluator)?;
    save_json(
        &out.join("input-bindings.json"),
        &json!({"inputs":bindings,
        "scope":"Sealed evaluation and copied checkpoint metadata; external model/optimizer checkpoints are not reopened."}),
    )?;
    let population = compare_population(roots, &reports, &evaluator, out)?;
    let probes = join_source_rows(roots, &evaluator)?;
    save_json(&out.join("source-comparison.json"), &probes)?;
    let prose = join_prose(roots, &reports, &evaluator)?;
    save_json(&out.join("prose-comparison.json"), &prose)?;
    save_json(
        &out.join("comparison-report.json"),
        &json!({
            "schema":"uor-r4.joint-reader-comparison/1","status":"COMPLETE","scope":SCOPE,
            "source_commit":source,"executable_sha256":sha256_file(&std::env::current_exe()?)?,
            "evaluator_sha256":EVALUATOR_SHA,"population":population,"input_roots":roots,
            "local_updates_per_arm":1024,"local_target_visits_per_arm":4194304,
            "historical_parent_updates":15672,"historical_parent_target_visits":64192512,
            "transfer_receipts":receipts,"training_source_commit":reports[1]["checkpoint_binding"]["source_commit"],
            "learning_objective":"Unweighted next-token NLL under the common bound training source and fixed campaign configuration; no termination weighting or quantization.",
            "stream_binding":{"sampler":reports[1]["checkpoint_binding"]["data_sampler"],"data_seed":240927,
                "batch":16,"context":256,"local_steps":[0,1023],"train_sources":evaluator["train_sources"],
                "scope":"Same deterministic counter schedule by source/configuration; no independent per-update executed-token hash is claimed."},
            "training_execution_limit":"Checkpoint bindings establish training source, shards, local clocks and lineage. Evaluation executable/features describe evaluation, not fitting. A separately bound run receipt must establish the common fit executable, compiler/features and thread environment; this comparator does not infer them from evaluation binaries.",
            "source_comparison_sha256":sha256_file(&out.join("source-comparison.json"))?,
            "prose_comparison_sha256":sha256_file(&out.join("prose-comparison.json"))?,
            "prose_quality":"NOT_AUTOMATICALLY_ADJUDICATED; apply the retained four-criterion rubric to actual text and preserve disagreements.",
            "parent_greedy":"Not required or regenerated; existing parent greedy evidence remains separately bound historical context."
        }),
    )
}

fn study_config(cfg: &Campaign, geometry: ReadGeometry) -> Result<()> {
    let expected = JointConfig {
        seed: 240924,
        read_geometry: geometry,
        ..JointConfig::default()
    };
    let initialization = if geometry == ReadGeometry::Dot {
        None
    } else {
        Some(ReadInitialization::UnitScale)
    };
    if cfg.model != expected
        || cfg.read_initialization != initialization
        || cfg.optimizer != AdamConfig::default()
        || cfg.data_seed != 240927
        || cfg.batch != 16
        || cfg.context != 256
        || cfg.cpu_gradient_shards != 2
        || cfg.total_steps != 1024
        || cfg.development_every_steps != 512
        || cfg.development_blocks != 64
        || cfg.checkpoint_steps != [512]
        || cfg.training_window_transition.is_some()
        || cfg.quantization_transition.is_some()
        || cfg.projection_transition.is_some()
    {
        return Err(invalid("reader campaign changes the fixed model, objective/optimizer, stream, windows, shards or endpoint"));
    }
    Ok(())
}

fn same_fields(left: &Value, right: &Value, fields: &[&str], label: &str) -> Result<()> {
    for field in fields {
        if left[*field] != right[*field] {
            return Err(invalid(format!("{label} differs in {field}")));
        }
    }
    Ok(())
}

fn matched_execution(read: &Value, no_read: &Value, dot: &Value) -> Result<()> {
    let batch = read["evaluation"]["requested_batch_size"].as_u64();
    if !batch.is_some_and(|size| (1..=32).contains(&size))
        || read["evaluation"]["requested_batch_size"]
            != no_read["evaluation"]["requested_batch_size"]
        || read["evaluation"]["requested_batch_size"] != dot["evaluation"]["requested_batch_size"]
    {
        return Err(invalid(
            "reader evaluations require the same valid batch shape",
        ));
    }
    same_fields(
        read,
        no_read,
        &[
            "campaign",
            "checkpoint_binding",
            "checkpoint",
            "source_commit",
            "executable_sha256",
            "requested_device",
            "mode",
            "artifact",
            "executed_quantization",
            "evaluation_quantization_strength",
            "cpu_accelerate_compiled",
            "candle_source",
            "transfer_receipt",
            "executed_numerical_contract",
        ],
        "Read/NoRead checkpoint pair",
    )?;
    same_fields(
        read,
        dot,
        &[
            "source_commit",
            "executable_sha256",
            "requested_device",
            "cpu_accelerate_compiled",
            "candle_source",
        ],
        "reader evaluation execution",
    )?;
    same_fields(
        &read["checkpoint_binding"],
        &dot["checkpoint_binding"],
        &[
            "source_commit",
            "data_seed",
            "data_sampler",
            "optimizer_step",
            "next_data_step",
            "training_batch",
            "training_context",
            "cpu_gradient_shards",
        ],
        "reader training source/stream",
    )
}

fn validate_study(reports: &[Value], evaluator: &Value) -> Result<Vec<TransferReceipt>> {
    if reports.len() != 7 {
        return Err(invalid("reader report count differs"));
    }
    let parent = &reports[0];
    let parent_cfg: Campaign = serde_json::from_value(parent["campaign"].clone())?;
    if parent_cfg.model
        != (JointConfig {
            seed: 240924,
            ..JointConfig::default()
        })
        || parent_cfg.shared_parameter_transfer.is_some()
        || parent_cfg.read_initialization.is_some()
        || parent["checkpoint_binding"]["model_sha256"] != PARENT_MODEL
        || parent["checkpoint_binding"]["source_commit"] != PARENT_SOURCE
        || parent["checkpoint_binding"]["optimizer_step"] != 15672
        || parent["checkpoint_binding"]["sampled_target_visits"] != 64192512
    {
        return Err(invalid(
            "parent context does not identify the selected continuous Dot parent",
        ));
    }
    let mut receipts = Vec::new();
    for (index, geometry) in [
        (1, ReadGeometry::Dot),
        (3, ReadGeometry::Lorentz),
        (5, ReadGeometry::LorentzAffine),
    ] {
        let read = &reports[index];
        let cfg: Campaign = serde_json::from_value(read["campaign"].clone())?;
        study_config(&cfg, geometry)?;
        let binding = &read["checkpoint_binding"];
        if binding["optimizer_step"] != 1024
            || binding["next_data_step"] != 1024
            || binding["sampled_target_visits"] != 4194304
            || binding["training_batch"] != 16
            || binding["training_context"] != 256
            || binding["sampled_targets_per_step"] != 4096
            || binding["cpu_gradient_shards"] != 2
        {
            return Err(invalid(
                "reader candidate is not the fixed1024-update endpoint",
            ));
        }
        matched_execution(read, &reports[index + 1], &reports[1])?;
        let receipt = crate::joint_transfer::validate_binding(&cfg, binding)?
            .ok_or_else(|| invalid("reader candidate has no parameter-transfer receipt"))?;
        let spec = &receipt.specification;
        let tokenizer = evaluator["reference_inputs"]
            .as_array()
            .and_then(|a| {
                a.iter().find(|v| {
                    v["path"]
                        .as_str()
                        .is_some_and(|p| p.ends_with("/tokenizer.json"))
                })
            })
            .ok_or_else(|| invalid("canonical tokenizer identity missing"))?;
        if spec.parent_checkpoint_sha256 != PARENT_CHECKPOINT
            || spec.parent_campaign_sha256 != PARENT_CAMPAIGN
            || spec.parent_optimizer_step != 15672
            || spec.parent_sampled_target_visits != 64192512
            || receipt.parent_model_sha256 != PARENT_MODEL
            || receipt.parent_source_commit != PARENT_SOURCE
            || receipt.shared_parameters_sha256 != SHARED_MANIFEST
            || receipt.copied_scalar_count != 1678466
            || receipt.copied_parameters.len() != 21
            || serde_json::to_value(&receipt.tokenizer)? != *tokenizer
        {
            return Err(invalid(
                "reader transfer differs from the selected shared parent/tokenizer",
            ));
        }
        if let Some(first) = receipts.first() {
            let normalized = |receipt: &TransferReceipt| -> Result<Value> {
                let mut value = serde_json::to_value(receipt)?;
                let object = value
                    .as_object_mut()
                    .ok_or_else(|| invalid("receipt object"))?;
                object.remove("target_read_geometry");
                object.remove("initial_radial_scalars");
                Ok(value)
            };
            if normalized(first)? != normalized(&receipt)? {
                return Err(invalid(
                    "reader arms do not share the same transferred arrays and reset lineage",
                ));
            }
        }
        receipts.push(receipt);
    }
    // Same parameter source, scalar initialization and optimizer are distinct
    // from equal subsequent trajectories; do not compare learned arrays here.
    if receipts[1].initial_radial_scalars != receipts[2].initial_radial_scalars {
        return Err(invalid("radial initial scalars differ"));
    }
    Ok(receipts)
}

#[derive(Clone, Copy, Default)]
struct Sum {
    total: f64,
    correction: f64,
}
impl Sum {
    fn add(&mut self, value: f64) {
        let adjusted = value - self.correction;
        let next = self.total + adjusted;
        self.correction = (next - self.total) - adjusted;
        self.total = next;
    }
}
#[derive(Default)]
struct Statistics {
    count: usize,
    sums: [Sum; 7],
    deltas: [Sum; 9],
    orders: [[usize; 3]; 9],
}
impl Statistics {
    fn add(&mut self, losses: [f64; 7]) {
        self.count += 1;
        for (sum, value) in self.sums.iter_mut().zip(losses) {
            sum.add(value);
        }
        for (p, &(candidate, control)) in PAIRS.iter().enumerate() {
            self.deltas[p].add(losses[candidate] - losses[control]);
            let order = if losses[candidate] < losses[control] {
                0
            } else if losses[candidate] == losses[control] {
                1
            } else {
                2
            };
            self.orders[p][order] += 1;
        }
    }
    fn report(&self) -> Value {
        let means: BTreeMap<_, _> = NAMES
            .iter()
            .zip(&self.sums)
            .map(|(name, sum)| (*name, sum.total / self.count as f64))
            .collect();
        let pairs:Vec<_>=PAIRS.iter().enumerate().map(|(p,&(candidate,control))|json!({
            "candidate":NAMES[candidate],"comparator":NAMES[control],"mean_nll_delta":self.deltas[p].total/self.count as f64,
            "lower_equal_higher_targets":self.orders[p]})).collect();
        json!({"targets":self.count,"mean_nll_nats":means,"paired":pairs})
    }
}

fn compare_population(
    roots: &[PathBuf],
    reports: &[Value],
    evaluator: &Value,
    out: &Path,
) -> Result<Value> {
    let tokens = read_tokens(&evaluator["dev_source"])?;
    if tokens.len() != 250000 {
        return Err(invalid("canonical development token count differs"));
    }
    let mut readers = roots
        .iter()
        .map(|root| {
            let path = root.join("targets.bin");
            if fs::metadata(&path)?.len() != (TARGETS * 44) as u64 {
                return Err(invalid("incomplete/extra reader target records"));
            }
            Ok(BufReader::new(File::open(path)?))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut partitions: [Statistics; 3] = std::array::from_fn(|_| Statistics::default());
    let mut blocks = BufWriter::new(File::create_new(out.join("comparison-blocks.jsonl"))?);
    for block in 0..BLOCKS {
        let mut statistics = Statistics::default();
        for position in 0..CONTEXT {
            let offset = block * CONTEXT + position;
            let mut losses = [0.0; 7];
            for index in 0..7 {
                losses[index] = joint_row(
                    &mut readers[index],
                    offset,
                    u32::from(tokens[offset + 1]),
                    mode(index),
                    NAMES[index],
                )?;
            }
            statistics.add(losses);
            partitions[usize::from(block >= 64)].add(losses);
            partitions[2].add(losses);
        }
        serde_json::to_writer(
            &mut blocks,
            &json!({"block":block,"partition":if block<64 {"tune"} else {"comparison"},"statistics":statistics.report()}),
        )?;
        writeln!(blocks)?;
    }
    blocks.flush()?;
    blocks.get_ref().sync_all()?;
    for reader in &mut readers {
        if reader.read(&mut [0u8; 1])? != 0 {
            return Err(invalid("extra target row after complete population"));
        }
    }
    for (partition, name) in partitions.iter().zip(["tune", "comparison", "full"]) {
        for (index, report) in reports.iter().enumerate() {
            let recorded = &report["evaluation"][name];
            let mean = recorded["mean_nll_nats"]
                .as_f64()
                .ok_or_else(|| invalid("missing evaluation mean"))?;
            if recorded["scored_targets"] != json!(partition.count)
                || !mean.is_finite()
                || (mean - partition.sums[index].total / partition.count as f64).abs() > 1e-10
            {
                return Err(invalid(format!(
                    "{} {name} row mean differs from its report",
                    NAMES[index]
                )));
            }
        }
    }
    Ok(
        json!({"tune":partitions[0].report(),"comparison":partitions[1].report(),"full":partitions[2].report(),
        "rows_verified":TARGETS,"partition_blocks":[64,912],"context":CONTEXT,
        "target_identity":"Every row offset and shifted target equals the verified canonical development store; native binary rows do not independently record input-token IDs.",
        "delta_convention":"Candidate minus comparator NLL; negative is lower. Exact lower/equal/higher counts describe dependent targets, not significance or equivalence.",
        "no_read_scope":"NoRead removes both read feedback and copying; its penalty does not isolate geometry."}),
    )
}

fn generation<'a>(
    value: &'a Value,
    prompt: &str,
    seed: Option<u64>,
    cap: usize,
    read_mode: ReadMode,
    tokenizer: &Value,
) -> Result<&'a Value> {
    let ids = value["generated_token_ids"]
        .as_array()
        .ok_or_else(|| invalid("generation token IDs missing"))?;
    let decisions = value["decisions"]
        .as_array()
        .ok_or_else(|| invalid("generation decisions missing"))?;
    let prompt_ids = value["prompt_token_ids"]
        .as_array()
        .ok_or_else(|| invalid("generation prompt IDs missing"))?;
    if value["schema"] != "uor-r4.joint-incremental-generation/1"
        || value["prompt"] != prompt
        || value["seed"] != json!(seed)
        || value["max_new_tokens"] != json!(cap)
        || value["mode"] != json!(read_mode)
        || value["tokenizer_cid"] != *tokenizer
        || value["sampler_policy"]
            != if seed.is_some() {
                SEEDED_POLICY
            } else {
                GREEDY_POLICY
            }
        || value["bos_policy"] != "prepend checkpoint BOS0 exactly once; EOS1 ends output"
        || value["context_capacity"] != 256
        || value["fresh_session"] != true
        || value["whole_prompt_read_mode"] != true
        || value["gradients_tracked"] != false
        || prompt_ids.first() != Some(&json!(0))
        || prompt_ids.len() + cap > CONTEXT
        || ids.is_empty()
        || ids.len() > cap
        || decisions.len() != ids.len()
        || ids
            .iter()
            .chain(prompt_ids)
            .any(|v| v.as_u64().is_none_or(|n| n >= 4096))
        || !value["response_text"].is_string()
        || !value["raw_decoded"].is_string()
        || !value["stop"].is_object()
        || decisions
            .iter()
            .zip(ids)
            .any(|(row, id)| row["selected_token"] != *id)
    {
        return Err(invalid(
            "generation prompt, policy, token or session contract differs",
        ));
    }
    Ok(value)
}

fn canonical_prompts(evaluator: &Value) -> Result<Value> {
    let document = read_json(&verify_identity(&evaluator["prompt_source"])?)?;
    if document["prompts"].as_array().map(Vec::len) != Some(5)
        || !document["tokenizer_cid"].is_string()
    {
        return Err(invalid("canonical five prompts missing"));
    }
    Ok(document)
}

fn join_source_rows(roots: &[PathBuf], evaluator: &Value) -> Result<Value> {
    let tokenizer = canonical_prompts(evaluator)?["tokenizer_cid"].clone();
    let expected = story_probes();
    let mut packets = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let document = read_json(&root.join("story-probes.json"))?;
        let mut packet = BTreeMap::new();
        for row in document
            .as_array()
            .ok_or_else(|| invalid("source probes not an array"))?
        {
            let id = row["probe"]["id"]
                .as_str()
                .ok_or_else(|| invalid("probe id missing"))?;
            if packet.insert(id.to_owned(), row.clone()).is_some() {
                return Err(invalid("duplicate source probe"));
            }
        }
        if packet.len() != expected.len() {
            return Err(invalid("source probe count differs"));
        }
        for probe in &expected {
            let row = packet
                .get(&probe.id)
                .ok_or_else(|| invalid("missing canonical source probe"))?;
            if row["probe"] != serde_json::to_value(probe)?
                || row["schema"] != "uor-r4.joint-story-source-edit/1"
                || row["scope"] != STORY_PROBE_SCOPE
                || row["stop_policy"] != STORY_STOP_POLICY
                || row["mode"] != json!(mode(index))
            {
                return Err(invalid("source probe identity, mode or policy differs"));
            }
            for (side, prompt) in [
                ("original", &probe.original.prompt),
                ("edited", &probe.edited.prompt),
            ] {
                generation(
                    &row[side]["generation"],
                    prompt,
                    None,
                    32,
                    mode(index),
                    &tokenizer,
                )?;
                if !row[side]["first_noun_correct"].is_boolean()
                    || !row[side]["complete_correct"].is_boolean()
                {
                    return Err(invalid("source verdict missing"));
                }
            }
            if row["pair_complete_correct"]
                != json!(
                    row["original"]["complete_correct"] == true
                        && row["edited"]["complete_correct"] == true
                )
                || row["outputs_differ"]
                    != json!(
                        row["original"]["generation"]["response_text"]
                            != row["edited"]["generation"]["response_text"]
                    )
            {
                return Err(invalid("source pair verdict differs from recorded sides"));
            }
        }
        packets.push(packet);
    }
    let mut rows = Vec::new();
    let mut pairs = Vec::new();
    let mut changes = Vec::new();
    for probe in &expected {
        let pair_outputs:BTreeMap<_,_>=NAMES.iter().enumerate().map(|(i,name)|(*name,json!({
            "pair_complete_correct":packets[i][&probe.id]["pair_complete_correct"],"outputs_differ":packets[i][&probe.id]["outputs_differ"]}))).collect();
        pairs.push(json!({"probe_id":probe.id,"arms":pair_outputs}));
        for side in ["original", "edited"] {
            let base_prompt_ids = &packets[0][&probe.id][side]["generation"]["prompt_token_ids"];
            if packets
                .iter()
                .any(|p| p[&probe.id][side]["generation"]["prompt_token_ids"] != *base_prompt_ids)
            {
                return Err(invalid("source prompt token IDs differ across arms"));
            }
            let outputs: BTreeMap<_, _> = NAMES
                .iter()
                .enumerate()
                .map(|(i, name)| (*name, packets[i][&probe.id][side].clone()))
                .collect();
            rows.push(json!({"probe_id":probe.id,"side":side,"arms":outputs}));
            for (candidate, control) in [(1, 0), (3, 0), (5, 0), (3, 1), (5, 1), (3, 5)] {
                for verdict in ["first_noun_correct", "complete_correct"] {
                    let c = packets[candidate][&probe.id][side][verdict] == true;
                    let p = packets[control][&probe.id][side][verdict] == true;
                    if c != p {
                        changes.push(json!({"probe_id":probe.id,"side":side,"criterion":verdict,
                        "candidate":NAMES[candidate],"comparator":NAMES[control],"change":if c {"gained"} else {"lost"}}));
                    }
                }
            }
        }
    }
    Ok(
        json!({"scope":STORY_PROBE_SCOPE,"stop_policy":STORY_STOP_POLICY,"rows":rows,"pairs":pairs,
        "gained_lost_rows":changes,"verdict_scope":"Existing sealed source-panel judgments; complete and first-noun results remain separate. No response repair or new acceptance rule."}),
    )
}

fn join_prose(roots: &[PathBuf], reports: &[Value], evaluator: &Value) -> Result<Value> {
    let prompts = canonical_prompts(evaluator)?;
    let prompt_array = prompts["prompts"]
        .as_array()
        .ok_or_else(|| invalid("prompts array"))?;
    let mut sampled = Vec::new();
    let mut greedy = BTreeMap::new();
    let mut identities = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let values = read_json(&root.join("generations.json"))?;
        validate_prose(
            &values,
            prompt_array,
            Some(2014),
            mode(index),
            &prompts["tokenizer_cid"],
        )?;
        sampled.push(values);
        if matches!(index, 1 | 3 | 5) {
            let path = root.join("greedy-generations.json");
            let packet = read_json(&path)?;
            if packet["schema"] != "uor-r4.joint-greedy-prose/1"
                || packet["checkpoint_binding"] != reports[index]["checkpoint_binding"]
                || packet["evaluator_sha256"] != EVALUATOR_SHA
                || packet["prompt_source"] != evaluator["prompt_source"]
                || packet["policy"] != "greedy"
                || packet["mode"] != json!(ReadMode::Enabled)
                || packet["operation"] != "joint-greedy-prose"
                || packet["status"] != "COMPLETE"
            {
                return Err(invalid(
                    "greedy prose supplement differs from its evaluation/checkpoint",
                ));
            }
            same_fields(
                &packet,
                &reports[index],
                &[
                    "campaign",
                    "checkpoint",
                    "source_commit",
                    "executable_sha256",
                    "requested_device",
                    "cpu_accelerate_compiled",
                ],
                "greedy supplement execution",
            )?;
            validate_prose(
                &packet["generations"],
                prompt_array,
                None,
                ReadMode::Enabled,
                &prompts["tokenizer_cid"],
            )?;
            identities.push(json!({"arm":NAMES[index],"sha256":sha256_file(&path)?}));
            greedy.insert(index, packet["generations"].clone());
        }
    }
    let rows: Vec<_> = prompt_array
        .iter()
        .enumerate()
        .map(|(i, prompt)| {
            let sampled_outputs: BTreeMap<_, _> = NAMES
                .iter()
                .enumerate()
                .map(|(arm, name)| (*name, sampled[arm][i].clone()))
                .collect();
            let greedy_outputs: BTreeMap<_, _> = greedy
                .iter()
                .map(|(&arm, values)| (NAMES[arm], values[i].clone()))
                .collect();
            json!({"prompt":prompt,"sampled":sampled_outputs,"greedy":greedy_outputs})
        })
        .collect();
    Ok(
        json!({"rows":rows,"greedy_supplements":identities,"quality_adjudication":"NOT_AUTOMATED; retain the same four binary prose criteria and textual evidence, separately by policy.",
        "parent_greedy":"Absent from this join; reuse the independently retained historical packet without a new replay."}),
    )
}

fn validate_prose(
    values: &Value,
    prompts: &[Value],
    seed: Option<u64>,
    read_mode: ReadMode,
    tokenizer: &Value,
) -> Result<()> {
    let values = values
        .as_array()
        .ok_or_else(|| invalid("prose generations are not an array"))?;
    if values.len() != 5 {
        return Err(invalid("prose count differs from frozen five prompts"));
    }
    for (index, (value, prompt)) in values.iter().zip(prompts).enumerate() {
        generation(
            value,
            prompt["text"]
                .as_str()
                .ok_or_else(|| invalid("canonical prompt text"))?,
            seed.map(|s| s + index as u64),
            128,
            read_mode,
            tokenizer,
        )?;
        let mut expected = vec![json!(0)];
        expected.extend(
            prompt["token_ids"]
                .as_array()
                .ok_or_else(|| invalid("canonical prompt IDs"))?
                .iter()
                .cloned(),
        );
        if value["prompt_token_ids"] != json!(expected) {
            return Err(invalid(
                "prose prompt token IDs differ from the canonical source",
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campaign() -> Result<Campaign> {
        Ok(serde_json::from_value(json!({
            "schema":"uor-r4.joint-recurrent-campaign/1","evaluator_path":"/canonical/evaluator.json",
            "model":JointConfig { seed:240924,..JointConfig::default() },"optimizer":AdamConfig::default(),
            "data_seed":240927,"batch":16,"context":256,"cpu_gradient_shards":2,"total_steps":1024,
            "development_every_steps":512,"development_blocks":64,"checkpoint_steps":[512],
            "max_process_seconds":3600,"trial_scope":"reader comparator fixture"
        }))?)
    }

    #[test]
    fn reader_comparison_matches_training_identity_separately_from_evaluation() -> Result<()> {
        let cfg = campaign()?;
        study_config(&cfg, ReadGeometry::Dot)?;
        for geometry in [ReadGeometry::Lorentz, ReadGeometry::LorentzAffine] {
            let mut radial = cfg.clone();
            radial.model.read_geometry = geometry;
            radial.read_initialization = Some(ReadInitialization::UnitScale);
            study_config(&radial, geometry)?;
            radial.read_initialization = None;
            assert!(study_config(&radial, geometry).is_err());
        }
        for field in ["data_seed", "cpu_gradient_shards", "total_steps", "context"] {
            let mut changed = serde_json::to_value(&cfg)?;
            changed[field] = json!(1);
            assert!(study_config(&serde_json::from_value(changed)?, ReadGeometry::Dot).is_err());
        }
        let mut changed_optimizer = cfg.clone();
        changed_optimizer.optimizer.learning_rate *= 2.0;
        assert!(study_config(&changed_optimizer, ReadGeometry::Dot).is_err());
        let dot = json!({"source_commit":"evaluation-build","executable_sha256":"evaluation-executable",
            "evaluation":{"requested_batch_size":4},
            "checkpoint_binding":{"source_commit":"training-build","model_sha256":"dot-weights","cpu_gradient_shards":2,
                "optimizer_step":1024,"next_data_step":1024,"data_seed":240927,"data_sampler":"counter",
                "training_batch":16,"training_context":256}});
        let mut radial = dot.clone();
        radial["checkpoint_binding"]["model_sha256"] = json!("radial-weights");
        matched_execution(&radial, &radial, &dot)?;
        let mut wrong_pair = radial.clone();
        wrong_pair["checkpoint_binding"]["model_sha256"] = json!("another-checkpoint");
        assert!(matched_execution(&radial, &wrong_pair, &dot).is_err());
        for batch in [json!(null), json!(0), json!(8), json!(33)] {
            let mut changed = radial.clone();
            changed["evaluation"]["requested_batch_size"] = batch;
            assert!(matched_execution(&radial, &changed, &dot).is_err());
            assert!(matched_execution(&changed, &changed, &dot).is_err());
        }
        // Equal evaluator executables cannot disguise a different fit source
        // or shard count, even when both modes share that changed checkpoint.
        for field in ["source_commit", "cpu_gradient_shards", "data_seed"] {
            let mut changed = radial.clone();
            changed["checkpoint_binding"][field] = json!("different-training-binding");
            assert!(matched_execution(&changed, &changed, &dot).is_err());
        }
        Ok(())
    }

    #[test]
    fn reader_comparison_checks_shifted_target_rows_and_paired_direction() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "uor-reader-rows-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| invalid(e.to_string()))?
                .as_nanos()
        ));
        report_output::claim(&root)?;
        let write_row = |name: &str, probability: f64, no_read: f32, top: i32| -> Result<PathBuf> {
            let path = root.join(name);
            let mut file = File::create_new(&path)?;
            file.write_all(&17u64.to_le_bytes())?;
            file.write_all(&37u32.to_le_bytes())?;
            file.write_all(&37u32.to_le_bytes())?;
            file.write_all(&probability.to_le_bytes())?;
            file.write_all(&(-probability.ln()).to_le_bytes())?;
            file.write_all(&no_read.to_le_bytes())?;
            file.write_all(&0f32.to_le_bytes())?;
            file.write_all(&top.to_le_bytes())?;
            Ok(path)
        };
        let valid = write_row("valid.bin", 0.25, 1.0, -1)?;
        let read = |path: &Path, offset, target, read_mode| -> Result<f64> {
            joint_row(
                &mut BufReader::new(File::open(path)?),
                offset,
                target,
                read_mode,
                "fixture",
            )
        };
        assert_eq!(read(&valid, 17, 37, ReadMode::NoRead)?, 4f64.ln());
        assert!(read(&valid, 18, 37, ReadMode::Enabled).is_err());
        assert!(read(&valid, 17, 38, ReadMode::Enabled).is_err());
        assert!(read(
            &write_row("nonfinite.bin", f64::NAN, 1.0, -1)?,
            17,
            37,
            ReadMode::Enabled
        )
        .is_err());
        assert!(read(
            &write_row("future.bin", 0.25, 0.5, 17)?,
            17,
            37,
            ReadMode::Enabled
        )
        .is_err());
        let enabled = write_row("enabled.bin", 0.25, 0.5, 16)?;
        read(&enabled, 17, 37, ReadMode::Enabled)?;
        assert!(read(&enabled, 17, 37, ReadMode::NoRead).is_err());
        let mut sums = Statistics::default();
        sums.add([2.0, 2.0, 3.0, 1.0, 2.0, 3.0, 4.0]);
        sums.add([2.0, 2.0, 3.0, 3.0, 4.0, 1.0, 2.0]);
        let report = sums.report();
        assert_eq!(report["paired"][0]["mean_nll_delta"], 0.0);
        assert_eq!(
            report["paired"][0]["lower_equal_higher_targets"],
            json!([1, 0, 1])
        );
        assert_eq!(report["paired"][6]["mean_nll_delta"], 1.0);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn reader_comparison_keeps_prompt_ids_and_sampler_policies_distinct() -> Result<()> {
        let prompts: Vec<_> = (0..5)
            .map(|i| json!({"text":format!("prompt {i}"),"token_ids":[40+i]}))
            .collect();
        let tokenizer = json!("fixture-tokenizer");
        let generations:Vec<_>=prompts.iter().enumerate().map(|(i,prompt)|json!({
            "schema":"uor-r4.joint-incremental-generation/1","prompt":prompt["text"],
            "seed":null,"max_new_tokens":128,"mode":"enabled","tokenizer_cid":tokenizer,
            "sampler_policy":GREEDY_POLICY,"bos_policy":"prepend checkpoint BOS0 exactly once; EOS1 ends output",
            "context_capacity":256,"fresh_session":true,"whole_prompt_read_mode":true,"gradients_tracked":false,
            "prompt_token_ids":[0,40+i],"generated_token_ids":[1],"decisions":[{"selected_token":1}],
            "response_text":"","raw_decoded":"","stop":{"reason":"eos"}
        })).collect();
        let values = json!(generations);
        validate_prose(&values, &prompts, None, ReadMode::Enabled, &tokenizer)?;
        assert!(
            validate_prose(&values, &prompts, Some(2014), ReadMode::Enabled, &tokenizer).is_err()
        );
        let mut changed = values.clone();
        changed[0]["prompt_token_ids"] = json!([0, 99]);
        assert!(validate_prose(&changed, &prompts, None, ReadMode::Enabled, &tokenizer).is_err());
        let mut changed = values;
        changed[0]["decisions"][0]["selected_token"] = json!(7);
        assert!(validate_prose(&changed, &prompts, None, ReadMode::Enabled, &tokenizer).is_err());
        Ok(())
    }
}
