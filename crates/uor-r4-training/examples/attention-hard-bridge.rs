//! Saved-model counterfactual hard capture/hold; no fitting or serving claim.
use candle_core::{Device, D};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_stack::{
    ReadBinding, ReadBindingTarget, ReadIdentityLatch, ReadScore, StackArch, StackModel,
};
use uor_r4_training::{sha256_file, Result, TrainingError};
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
#[derive(Deserialize)]
struct Episode {
    ids: Vec<u32>,
    roles: Vec<String>,
    source: usize,
    query: usize,
    answer: u32,
    facts: usize,
    pair: usize,
    condition: String,
    target_write_gap: usize,
    query_gap: usize,
}
fn score(
    model: &StackModel,
    episodes: &[Episode],
    hard: bool,
    start: Instant,
    limit: u64,
) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for group in episodes.chunks(32) {
        if start.elapsed().as_secs() >= limit {
            return Err(invalid("evaluation wall ceiling reached"));
        }
        let time = group
            .iter()
            .map(|e| e.ids.len())
            .max()
            .ok_or_else(|| invalid("empty batch"))?;
        let mut ids = vec![38; group.len() * time];
        for (i, e) in group.iter().enumerate() {
            ids[i * time..i * time + e.ids.len()].copy_from_slice(&e.ids);
        }
        let logits = if hard {
            model.forward_hard_read_identity_latch(&ids, group.len(), time)?
        } else {
            model.forward(&ids, group.len(), time)?
        };
        let predictions = logits.argmax(D::Minus1)?.to_vec1::<u32>()?;
        let values = logits.to_vec2::<f32>()?;
        let mut masses = Vec::new();
        for head in [0, 1] {
            let target = ReadBindingTarget {
                layer: 2,
                head,
                rows: group
                    .iter()
                    .enumerate()
                    .map(|(batch, e)| ReadBinding {
                        batch,
                        query: e.query,
                        sources: vec![e.source],
                    })
                    .collect(),
            };
            let mass = if hard {
                model.read_binding_masses_hard_read_identity_latch(
                    &ids,
                    group.len(),
                    time,
                    &target,
                )?
            } else {
                model.read_binding_masses(&ids, group.len(), time, &target)?
            };
            masses.push(mass.to_vec1::<f32>()?);
        }
        let actions = if hard {
            Some(
                model
                    .read_identity_latch_hard_gates(&ids, group.len(), time, 2)?
                    .to_vec2::<f32>()?,
            )
        } else {
            None
        };
        let gates = model
            .read_identity_latch_gate_logits(&ids, group.len(), time, 2)?
            .to_vec2::<f32>()?;
        for (i, e) in group.iter().enumerate() {
            if let Some(actions) = &actions {
                for (z, b) in gates[i].iter().zip(&actions[i]) {
                    if *b != f32::from(u8::from(*z >= 0.)) {
                        return Err(invalid("hard actions differ from final-rra gate logits"));
                    }
                }
            }
            let prediction = predictions[i * time + e.query];
            let answer_logits = &values[i * time + e.query];
            let mut top = answer_logits.iter().enumerate().collect::<Vec<_>>();
            top.sort_by(|a, b| b.1.total_cmp(a.1));
            let gate_rows = e
                .roles
                .iter()
                .zip(&gates[i])
                .enumerate()
                .map(|(t, (role, z))| json!({"position":t,"role":role,"logit":z,"capture":*z>=0.}))
                .collect::<Vec<_>>();
            rows.push(json!({"pair":e.pair,"condition":e.condition,"facts":e.facts,"source":e.source,"query":e.query,"answer":e.answer,"prediction":prediction,"answer_correct":prediction==e.answer,"answer_logits":answer_logits,"top_logit_margin":top[0].1-top[1].1,"source_masses_head0_head1":[masses[0][i],masses[1][i]],"source_majorities_head0_head1":[masses[0][i]>0.5,masses[1][i]>0.5],"target_write_gap":e.target_write_gap,"query_gap":e.query_gap,"positions":e.ids.len(),"gate_logits":gate_rows}));
        }
    }
    Ok(rows)
}
fn run(out: &Path, parent: &Path, limit: u64) -> Result<Value> {
    let start = Instant::now();
    let mut panels = Vec::new();
    for file in ["evaluation.json", "stress.json"] {
        let bytes = fs::read(parent.join(file))?;
        let episodes: Vec<Episode> = serde_json::from_slice(&bytes)?;
        if episodes.len() != 128
            || episodes.iter().any(|e| {
                e.ids.is_empty()
                    || e.ids.len() > 128
                    || e.roles.len() != e.ids.len()
                    || e.query >= e.ids.len()
                    || e.source >= e.query
                    || e.answer >= 40
            })
        {
            return Err(invalid("invalid fixed episode panel"));
        }
        fs::write(out.join(file), &bytes)?;
        panels.push((file, episodes));
    }
    let mut reports = Vec::new();
    for name in ["Held-s1", "Held-s2", "Local-s1", "Local-s2"] {
        let model_root = parent.join(name).join("model");
        let identity = sha256_file(&model_root.join("model.safetensors"))?;
        let model = StackModel::load(&model_root, &Device::Cpu)?;
        if model.config.arch != StackArch::Geometric
            || model.config.vocab_size != 40
            || model.config.read != ReadScore::Lorentz
            || !model.config.rotation
            || model.config.pointer.is_some()
            || model.config.select.is_some()
            || model.config.memory.is_some()
            || model.config.pattern != "rra"
            || model.config.width != 32
            || model.config.context != 128
            || model.config.heads != 2
            || model.read_identity_latch()
                != Some(if name.starts_with("Held") {
                    ReadIdentityLatch::Held
                } else {
                    ReadIdentityLatch::Local
                })
        {
            return Err(invalid("unexpected model contract"));
        }
        let mut measurements = Vec::new();
        for (file, episodes) in &panels {
            let soft = score(&model, episodes, false, start, limit)?;
            let hard = score(&model, episodes, true, start, limit)?;
            let soft_again = score(&model, episodes, false, start, limit)?;
            if soft != soft_again {
                return Err(invalid("hard diagnostic changed ordinary soft outputs"));
            }
            measurements.push(json!({"panel":file,"input_sha256":sha256_file(&parent.join(file))?,"soft":soft,"hard":hard,"soft_after_identical":true}));
        }
        if sha256_file(&model_root.join("model.safetensors"))? != identity {
            return Err(invalid("saved model changed"));
        }
        reports.push(json!({"name":name,"model_root":model_root,"model_sha256":identity,"measurements":measurements}));
    }
    Ok(
        json!({"schema":"uor-r4.attention-hard-bridge/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"reports":reports,"elapsed_seconds":start.elapsed().as_secs_f64(),"training_updates":0,"threshold":"learned logit >=0 CAPTURE, otherwise HOLD; Local clears on noncapture","scope":"saved synthetic development models; gate discretization only, remaining computation float/full causal scan; no natural-language, integer, energy or geometric advantage qualification"}),
    )
}
fn main() -> Result<()> {
    let (mut out, mut parent) = (None, None);
    let mut limit = 900u64;
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "out" => out = Some(std::path::PathBuf::from(v)),
            "parent" => parent = Some(std::path::PathBuf::from(v)),
            "max_seconds" => limit = v.parse().map_err(|_| invalid("max_seconds"))?,
            _ => return Err(invalid("unknown argument")),
        }
    }
    let out = out.ok_or_else(|| invalid("out required"))?;
    let parent = parent.ok_or_else(|| invalid("parent required"))?;
    if limit == 0 || limit > 900 {
        return Err(invalid("max_seconds1..900"));
    }
    report_output::claim(&out)?;
    let result = run(&out, &parent, limit);
    match &result {
        Ok(r) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(r)?)?,
        Err(e) => fs::write(
            out.join("error.json"),
            serde_json::to_vec_pretty(&json!({"error":e.to_string()}))?,
        )?,
    }
    report_output::seal(&out)?;
    report_output::verify(&out)?;
    result?;
    Ok(())
}
