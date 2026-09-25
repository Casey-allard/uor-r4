//! A declared new optimizer learns under the deployed bounded admission rule.
//! This is offline QAT with fixed grids, not an integer serving implementation.
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_core::report_output;

use crate::baseline_protocol::{load_evaluator, read_tokens, save_json};
use crate::joint_admission::AdmissionPolicy;
use crate::joint_campaign;
use crate::joint_model::{JointModel, ReadMode};
use crate::joint_optimizer::{AdamConfig, NamedAdamW};
use crate::{invalid, sha256_file, Result};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundedCampaign {
    pub schema: String,
    pub parent_packed: PathBuf,
    pub parent_manifest_sha256: String,
    pub evaluator_path: PathBuf,
    pub evaluator_sha256: String,
    pub admission: AdmissionPolicy,
    pub optimizer: AdamConfig,
    pub steps: usize,
    pub data_start_step: usize,
    pub data_seed: u64,
    pub batch: usize,
    pub context: usize,
    pub cpu_gradient_shards: usize,
    pub max_process_seconds: u64,
    pub stop_file: Option<PathBuf>,
    pub scope: String,
}

impl BoundedCampaign {
    fn validate(&self) -> Result<()> {
        self.optimizer.validate()?;
        if self.schema != "uor-r4.joint-bounded-campaign/1"
            || self.admission == AdmissionPolicy::Full
            || !(1..=100_000).contains(&self.steps)
            || self.batch != 16
            || self.context != 256
            || !matches!(self.cpu_gradient_shards, 1 | 2 | 4)
            || self.batch % self.cpu_gradient_shards != 0
            || !(1..=86400).contains(&self.max_process_seconds)
            || self.data_start_step.checked_add(self.steps).is_none()
            || self.scope.trim().is_empty()
            || [&self.parent_manifest_sha256, &self.evaluator_sha256]
                .iter()
                .any(|s| s.len() != 64 || !s.bytes().all(|c| c.is_ascii_hexdigit()))
        {
            return Err(invalid("invalid bounded continuation configuration"));
        }
        Ok(())
    }
}

fn fit(cfg: &BoundedCampaign, out: &Path, resume: Option<&Path>) -> Result<()> {
    let start = Instant::now();
    let checkpoint = out.join("checkpoint-final");
    let packed = out.join("packed-model");
    report_output::claim(&checkpoint)?;
    report_output::claim(&packed)?;
    let evaluator = load_evaluator(&cfg.evaluator_path)?;
    if evaluator.sha256 != cfg.evaluator_sha256
        || sha256_file(&cfg.parent_packed.join("hard-model.json"))? != cfg.parent_manifest_sha256
    {
        return Err(invalid("bounded parent/evaluator identity mismatch"));
    }
    let mut parent = joint_campaign::load_hard_export(&cfg.parent_packed, &evaluator)?;
    if parent.campaign.batch != cfg.batch
        || parent.model.config.context != cfg.context
        || parent.campaign.data_seed != cfg.data_seed
    {
        return Err(invalid(
            "bounded continuation changes retained data dimensions or seed",
        ));
    }
    let prior_counter = parent.binding["rounding_calibration"]["checkpoint"]["next_data_step"]
        .as_u64()
        .ok_or_else(|| invalid("bounded continuation requires retained learned-code lineage"))?;
    if prior_counter != cfg.data_start_step as u64 {
        return Err(invalid(
            "bounded continuation skips/repeats retained sampler",
        ));
    }
    let sources = evaluator.document["train_sources"]
        .as_array()
        .ok_or_else(|| invalid("training sources"))?;
    let stores = sources
        .iter()
        .map(read_tokens)
        .collect::<Result<Vec<_>>>()?;
    let mut sample_cfg = parent.campaign.clone();
    sample_cfg.cpu_gradient_shards = cfg.cpu_gradient_shards;
    let (mut model, mut optimizer, begin, resume_binding) = if let Some(path) = resume {
        report_output::verify(path)?;
        let old: BoundedCampaign =
            serde_json::from_slice(&fs::read(path.join("bounded-campaign.json"))?)?;
        let binding: Value = serde_json::from_slice(&fs::read(path.join("checkpoint.json"))?)?;
        if old != *cfg
            || binding["schema"] != "uor-r4.joint-bounded-checkpoint/1"
            || binding["model_sha256"] != sha256_file(&path.join("model.safetensors"))?
            || binding["model_config_sha256"] != sha256_file(&path.join("config.json"))?
            || binding["bounded_campaign_sha256"]
                != sha256_file(&path.join("bounded-campaign.json"))?
        {
            return Err(invalid(
                "bounded resume differs from its immutable configuration/bindings",
            ));
        }
        let complete = binding["completed_updates"]
            .as_u64()
            .and_then(|s| usize::try_from(s).ok())
            .ok_or_else(|| invalid("bounded resume clock"))?;
        if complete > cfg.steps
            || binding["next_data_step"] != json!(cfg.data_start_step + complete)
        {
            return Err(invalid("bounded sampler clock"));
        }
        let model = JointModel::load(path, &Device::Cpu)?;
        if model.config != parent.model.config
            || model.quantization() != parent.model.quantization()
            || model.admission_policy() != cfg.admission
        {
            return Err(invalid(
                "bounded resume changes frozen grids/model/admission",
            ));
        }
        let optimizer = NamedAdamW::load(path, model.variables(), &cfg.optimizer)?;
        if optimizer.step_count() != complete as u64 {
            return Err(invalid("bounded optimizer clock"));
        }
        (
            model,
            optimizer,
            complete,
            json!({"path":path,"binding":binding,"checkpoint_sha256":sha256_file(&path.join("checkpoint.json"))?}),
        )
    } else {
        let mut model = parent.model.packed_training_start()?;
        model.set_admission_policy(cfg.admission)?;
        let optimizer = NamedAdamW::new(model.variables(), cfg.optimizer.clone())?;
        (model, optimizer, 0, Value::Null)
    };
    let mut report = joint_campaign::metadata(&parent.campaign, "joint-bound-fit", "cpu")?;
    report["bounded_campaign"] = json!(cfg);
    report["parent_artifact"] = parent.artifact.clone();
    report["parent_checkpoint_binding"] = parent.binding.clone();
    report["resume"] = resume_binding;
    report["optimizer_scope"] = json!("New declared AdamW from retained packed code values, fixed quantizers, full-strength STE, clamp shadows to representable range after each update. No alpha optimization or scale search. Retained parent model/quantizer clock is separate from new optimizer/data clocks.");
    save_json(&out.join("bounded-campaign.json"), cfg)?;
    save_json(&out.join("evaluator.json"), &evaluator.document)?;
    let mut curve = BufWriter::new(File::create_new(out.join("learning-curve.jsonl"))?);
    let mut samples = Sha256::new();
    let mut complete = begin;
    for step in begin..cfg.steps {
        if start.elapsed().as_secs() >= cfg.max_process_seconds
            || cfg.stop_file.as_ref().is_some_and(|p| p.exists())
        {
            break;
        }
        let timer = Instant::now();
        let (inputs, targets) =
            joint_campaign::training_batch(&stores, &sample_cfg, cfg.data_start_step + step)?;
        for token in inputs.iter().chain(&targets) {
            samples.update(token.to_le_bytes());
        }
        let gradients = crate::joint_parallel::batch_gradients(
            &model,
            &inputs,
            &targets,
            cfg.batch,
            cfg.context,
            cfg.cpu_gradient_shards,
        )?;
        let update = optimizer.step(model.variables(), &gradients.gradients)?;
        let projection = model
            .quantization()
            .ok_or_else(|| invalid("bounded learner lost frozen grids"))?
            .spec
            .project_parameters(model.variables())?;
        complete = step + 1;
        let row = json!({"completed_updates":complete,"data_step":cfg.data_start_step+step,"batch_mean_nll":gradients.mean_nll,"optimizer":update,"projection":projection,"step_seconds":timer.elapsed().as_secs_f64()});
        serde_json::to_writer(&mut curve, &row)?;
        writeln!(curve)?;
        curve.flush()?;
        if step == begin || complete % 16 == 0 {
            eprintln!(
                "bounded {complete}/{} NLL {:.6} {:.3}s",
                cfg.steps,
                gradients.mean_nll,
                timer.elapsed().as_secs_f64()
            );
        }
    }
    curve.flush()?;
    curve.get_ref().sync_all()?;
    model.save(&checkpoint)?;
    optimizer.save(&checkpoint)?;
    save_json(&checkpoint.join("campaign.json"), &parent.campaign)?;
    save_json(&checkpoint.join("bounded-campaign.json"), cfg)?;
    let binding = json!({"schema":"uor-r4.joint-bounded-checkpoint/1","completed_updates":complete,"next_data_step":cfg.data_start_step+complete,
        "model_sha256":sha256_file(&checkpoint.join("model.safetensors"))?,"model_config_sha256":sha256_file(&checkpoint.join("config.json"))?,
        "bounded_campaign_sha256":sha256_file(&checkpoint.join("bounded-campaign.json"))?,"sampled_input_target_sha256":format!("{:x}",samples.finalize()),
        "new_updates_this_attempt":complete-begin,"new_target_visits_this_attempt":(complete-begin)*cfg.batch*cfg.context,"source_commit":report["source_commit"],
        "parent_quantization_clock":model.quantization(),"scope":"Bounded continuation optimizer clock is distinct from the retained historical quantizer clock. Legacy campaign.json is parent lineage; bounded-campaign.json is actual training recipe."});
    save_json(&checkpoint.join("checkpoint.json"), &binding)?;
    report_output::seal(&checkpoint)?;
    report_output::verify(&checkpoint)?;
    report["final_checkpoint"] = binding.clone();
    report["status"] = json!(if complete == cfg.steps {
        "COMPLETE_FIXED_RECIPE"
    } else {
        "STOPPED_CHECKPOINTED"
    });
    report["completed_updates"] = json!(complete);
    report["elapsed_seconds"] = json!(start.elapsed().as_secs_f64());
    if complete == cfg.steps {
        let lineage = json!({"schema":"uor-r4.joint-bounded-lineage/1","campaign":cfg,"checkpoint":binding,"checkpoint_path":checkpoint,
            "checkpoint_sha256":sha256_file(&checkpoint.join("checkpoint.json"))?,"source_commit":report["source_commit"]});
        parent.model = model;
        parent.binding["bounded_continuation"] = lineage.clone();
        parent.artifact["bounded_continuation"] = lineage;
        let result = joint_campaign::write_hard_export(&parent, &checkpoint, &evaluator, &packed);
        joint_campaign::finish_attempt(&packed, result)?;
        report["packed_model"] = json!(packed);
    } else {
        save_json(
            &packed.join("not-run.json"),
            &json!({"status":"NOT_RUN","reason":"Learning checkpointed before fixed dose completed"}),
        )?;
        report_output::seal(&packed)?;
        report_output::verify(&packed)?;
    }
    save_json(&out.join("bounded-fit-report.json"), &report)?;
    Ok(())
}

pub fn run_cli(args: &[String]) -> Result<()> {
    if args.first().map(String::as_str) == Some("joint-bound-fit") {
        if !(args.len() == 3 || args.len() == 4) {
            return Err(invalid(
                "joint-bound-fit CONFIG NEW_ROOT [SEALED_BOUNDED_CHECKPOINT]",
            ));
        }
        let cfg: BoundedCampaign = serde_json::from_slice(&fs::read(&args[1])?)?;
        cfg.validate()?;
        let out = Path::new(&args[2]);
        report_output::claim(out)?;
        return joint_campaign::finish_attempt(out, fit(&cfg, out, args.get(3).map(Path::new)));
    }
    if args.len() != 8 || args[0] != "joint-evaluate-admission" || args[4] != "cpu" {
        return Err(invalid("joint-evaluate-admission PACKED EVALUATOR NEW_ROOT cpu {read|no-read} BATCH {full|recent64|orthant64|exact_cache64}"));
    }
    let policy = AdmissionPolicy::parse(&args[7])?;
    let mode = match args[5].as_str() {
        "read" => ReadMode::Enabled,
        "no-read" => ReadMode::NoRead,
        _ => return Err(invalid("read mode")),
    };
    let batch: usize = args[6].parse().map_err(|_| invalid("evaluation batch"))?;
    if !(1..=crate::joint_evaluation::MAX_EVALUATION_BATCH).contains(&batch) {
        return Err(invalid("evaluation batch limit"));
    }
    let out = Path::new(&args[3]);
    report_output::claim(out)?;
    let result = (|| {
        let evaluator = load_evaluator(Path::new(&args[2]))?;
        let source = Path::new(&args[1]);
        let mut input = joint_campaign::load_hard_export(source, &evaluator)?;
        let stored = input.model.admission_policy();
        input.model.set_admission_policy(policy)?;
        input.model.enable_admission_audit();
        input.artifact["admission_intervention"] = json!({"stored":stored,"executed":policy,"same_weights":true,"diagnostic_override":stored!=policy});
        joint_campaign::evaluate_loaded(
            &input, &evaluator, source, out, &args[0], "cpu", mode, batch,
        )
    })();
    joint_campaign::finish_attempt(out, result)
}
