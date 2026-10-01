//! Actual integer gate/retained-state replay in the saved floating reader.
use candle_core::{Device, Tensor, D};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_integer::{
    codec::Grouped4BitRow,
    identity_latch::{DyadicBias, DyadicVector, IdentityGate, IdentityLatch, LatchMode},
};
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
#[derive(Serialize, Deserialize)]
struct GateArtifact {
    schema: String,
    model_sha256: String,
    current: Vec<i8>,
    current_exp: i32,
    previous: Vec<i8>,
    previous_exp: i32,
    bias: i64,
    bias_exp: i32,
    max_weight_error: f32,
    policy: String,
}
impl GateArtifact {
    fn gate(&self) -> Result<IdentityGate> {
        if self.schema != "uor-r4.integer-identity-gate/1" {
            return Err(invalid("gate schema"));
        }
        IdentityGate::new(
            self.current.clone(),
            self.current_exp,
            self.previous.clone(),
            self.previous_exp,
            DyadicBias {
                mantissa: self.bias,
                exponent: self.bias_exp,
            },
        )
        .map_err(|e| invalid(e.to_string()))
    }
}
fn export(model: &StackModel, model_sha256: String) -> Result<GateArtifact> {
    let w = model
        .variables()
        .get("layers.02.read.identity_gate.weight")
        .ok_or_else(|| invalid("gateweight"))?
        .flatten_all()?
        .to_vec1::<f32>()?;
    if w.len() != 64 {
        return Err(invalid("gatewidth"));
    }
    let mut halves = Vec::new();
    let mut max_weight_error = 0f32;
    for half in w.chunks(32) {
        let row = Grouped4BitRow::encode_f32(half, 32).map_err(|e| invalid(e.to_string()))?;
        let codes = (0..32)
            .map(|i| {
                let n = (row.packed_codes[i >> 1] >> ((i & 1) << 2)) & 15;
                (n as i8) << 4 >> 4
            })
            .collect::<Vec<_>>();
        for (a, b) in half.iter().zip(row.dequantize_f32()) {
            max_weight_error = max_weight_error.max((a - b).abs());
        }
        halves.push((codes, i32::from(row.group_exponents[0])));
    }
    let b = model
        .variables()
        .get("layers.02.read.identity_gate.bias")
        .ok_or_else(|| invalid("gatebias"))?
        .to_vec1::<f32>()?[0];
    let scaled = f64::from(b) * 2f64.powi(24);
    if !scaled.is_finite() || scaled.abs() >= i64::MAX as f64 {
        return Err(invalid("biasrange"));
    }
    let (previous, previous_exp) = halves.pop().ok_or_else(|| invalid("previoushalf"))?;
    let (current, current_exp) = halves.pop().ok_or_else(|| invalid("currenthalf"))?;
    Ok(GateArtifact{schema:"uor-r4.integer-identity-gate/1".into(),model_sha256,current,current_exp,previous,previous_exp,bias:scaled.round() as i64,bias_exp:-24,max_weight_error,policy:"existing Grouped4BitRow group32 signed[-7,7] dyadic maxabs-only exp[-24,16]; weights/input/bias nearest tiesaway; fixed Q24bias; gained inputs; distinct from stack_export grid".into()})
}
fn quantize(u: &[f32]) -> Result<(Vec<i16>, i32)> {
    if u.iter().any(|x| !x.is_finite()) {
        return Err(invalid("nonfiniteinput"));
    }
    let max = u.iter().fold(0f64, |a, x| a.max(f64::from(x.abs())));
    let exp = if max == 0. {
        0
    } else {
        (max / 32767.).log2().ceil() as i32
    };
    if !(-64..=64).contains(&exp) {
        return Err(invalid("input exponent"));
    }
    let scale = 2f64.powi(exp);
    let mut out = Vec::new();
    for x in u {
        let a = (f64::from(*x) / scale).round();
        if !(-32767. ..=32767.).contains(&a) {
            return Err(invalid("input clipping"));
        }
        out.push(a as i16);
    }
    Ok((out, exp))
}
fn score(
    model: &StackModel,
    es: &[Episode],
    gate: Option<&IdentityGate>,
    start: Instant,
    limit: u64,
) -> Result<Vec<Value>> {
    let mut rows = Vec::new();
    for group in es.chunks(32) {
        if start.elapsed().as_secs() >= limit {
            return Err(invalid("evaluation limit"));
        }
        let time = group
            .iter()
            .map(|e| e.ids.len())
            .max()
            .ok_or_else(|| invalid("emptybatch"))?;
        let mut ids = vec![38; group.len() * time];
        for (i, e) in group.iter().enumerate() {
            ids[i * time..i * time + e.ids.len()].copy_from_slice(&e.ids);
        }
        let input = model
            .read_identity_latch_replay_input(&ids, group.len(), time)?
            .to_vec3::<f32>()?;
        let z = model
            .read_identity_latch_gate_logits(&ids, group.len(), time, 2)?
            .to_vec2::<f32>()?;
        let mut diagnostic = Vec::new();
        let mut prior_values = Vec::new();
        if let Some(gate) = gate {
            for (b, sequence) in input.iter().enumerate() {
                let mode = if model.read_identity_latch() == Some(ReadIdentityLatch::Held) {
                    LatchMode::Held
                } else {
                    LatchMode::Local
                };
                let mut latch =
                    IdentityLatch::new(gate.clone(), mode).map_err(|e| invalid(e.to_string()))?;
                let mut reference = vec![0f32; 32];
                let mut events = Vec::new();
                for (t, u) in sequence.iter().enumerate() {
                    let held = latch.identity();
                    let prior = held
                        .mantissas
                        .iter()
                        .map(|x| (f64::from(*x) * 2f64.powi(held.exponent)) as f32)
                        .collect::<Vec<_>>();
                    let error = prior
                        .iter()
                        .zip(&reference)
                        .fold(0f32, |a, (x, y)| a.max((x - y).abs()));
                    prior_values.extend(prior);
                    let held_exp = held.exponent;
                    let (q, exp) = quantize(u)?;
                    let decision = latch
                        .step(DyadicVector {
                            mantissas: &q,
                            exponent: exp,
                        })
                        .map_err(|e| invalid(e.to_string()))?;
                    let iz = decision.logit.mantissa as f64 * 2f64.powi(decision.logit.exponent);
                    let capture = z[b][t] >= 0.;
                    if capture {
                        reference.copy_from_slice(u);
                    } else if mode == LatchMode::Local {
                        reference.fill(0.);
                    }
                    events.push(json!({"position":t,"role":group[b].roles.get(t),"reference_capture":capture,"integer_capture":decision.capture,"action_agrees":capture==decision.capture,"float_logit":z[b][t],"integer_logit":iz,"absolute_logit_error":(iz-f64::from(z[b][t])).abs(),"prior_content_max_error":error,"prior_exponent":held_exp,"input_exponent":exp,"integer_logit_mantissa":decision.logit.mantissa.to_string(),"integer_logit_exponent":decision.logit.exponent}));
                }
                diagnostic.push(events);
            }
        }
        let prior = if gate.is_some() {
            Some(Tensor::from_vec(
                prior_values,
                (group.len(), time, 32),
                model.device(),
            )?)
        } else {
            None
        };
        let logits = match &prior {
            Some(p) => model.forward_read_identity_latch_replay(&ids, group.len(), time, p)?,
            None => model.forward_hard_read_identity_latch(&ids, group.len(), time)?,
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
            let a = match &prior {
                Some(p) => model.read_binding_masses_identity_latch_replay(
                    &ids,
                    group.len(),
                    time,
                    &target,
                    p,
                )?,
                None => model.read_binding_masses_hard_read_identity_latch(
                    &ids,
                    group.len(),
                    time,
                    &target,
                )?,
            };
            masses.push(a.to_vec1::<f32>()?);
        }
        for (i, e) in group.iter().enumerate() {
            let prediction = predictions[i * time + e.query];
            rows.push(json!({"pair":e.pair,"condition":e.condition,"facts":e.facts,"source":e.source,"query":e.query,"answer":e.answer,"prediction":prediction,"answer_correct":prediction==e.answer,"answer_logits":values[i*time+e.query],"source_masses_head0_head1":[masses[0][i],masses[1][i]],"source_majorities_head0_head1":[masses[0][i]>0.5,masses[1][i]>0.5],"target_write_gap":e.target_write_gap,"query_gap":e.query_gap,"positions":e.ids.len(),"events":if gate.is_some(){diagnostic[i][..e.ids.len()].to_vec()}else{Vec::new()}}));
        }
    }
    Ok(rows)
}
fn run(out: &Path, parent: &Path, limit: u64) -> Result<Value> {
    let start = Instant::now();
    let mut panels = Vec::new();
    for file in ["evaluation.json", "stress.json"] {
        let bytes = fs::read(parent.join(file))?;
        let es: Vec<Episode> = serde_json::from_slice(&bytes)?;
        if es.len() != 128
            || es.iter().any(|e| {
                e.ids.is_empty()
                    || e.ids.len() > 128
                    || e.roles.len() != e.ids.len()
                    || e.source >= e.query
                    || e.query >= e.ids.len()
                    || e.answer >= 40
            })
        {
            return Err(invalid("panel"));
        }
        fs::write(out.join(file), bytes)?;
        panels.push((file, es));
    }
    let mut reports = Vec::new();
    for name in ["Held-s1", "Held-s2", "Local-s1", "Local-s2"] {
        let path = parent.join(name).join("model");
        let model_sha256 = sha256_file(&path.join("model.safetensors"))?;
        let model = StackModel::load(&path, &Device::Cpu)?;
        if model.config.arch != StackArch::Geometric
            || model.config.pattern != "rra"
            || model.config.width != 32
            || model.config.context != 128
            || model.config.heads != 2
            || model.config.vocab_size != 40
            || model.config.read != ReadScore::Lorentz
            || !model.config.rotation
            || model.config.pointer.is_some()
            || model.config.memory.is_some()
            || model.config.select.is_some()
            || model.read_identity_latch()
                != Some(if name.starts_with("Held") {
                    ReadIdentityLatch::Held
                } else {
                    ReadIdentityLatch::Local
                })
        {
            return Err(invalid("modelcontract"));
        }
        let a = export(&model, model_sha256.clone())?;
        let file = out.join(format!("{name}-gate.json"));
        fs::write(&file, serde_json::to_vec_pretty(&a)?)?;
        let reloaded: GateArtifact = serde_json::from_slice(&fs::read(&file)?)?;
        if reloaded.model_sha256 != model_sha256 {
            return Err(invalid("gatebinding"));
        }
        let gate = reloaded.gate()?;
        let mut measured = Vec::new();
        for (file, es) in &panels {
            let reference = score(&model, es, None, start, limit)?;
            let integer = score(&model, es, Some(&gate), start, limit)?;
            let after = score(&model, es, None, start, limit)?;
            if reference != after {
                return Err(invalid("replay altered model"));
            }
            measured.push(json!({"panel":file,"input_sha256":sha256_file(&parent.join(file))?,"hard_reference":reference,"integer_component":integer,"reference_after_identical":true}));
        }
        reports.push(json!({"name":name,"model_sha256":model_sha256,"gate_artifact":a,"measurements":measured}));
    }
    Ok(
        json!({"schema":"uor-r4.integer-identity-replay/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),"reports":reports,"elapsed_seconds":start.elapsed().as_secs_f64(),"training_updates":0,"scope":"integergate/16bitgainedretainedidentity; floattrunk/qk/value/scorer/head; authored development/fullprefix; no whole-serving/language/energy qualification"}),
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
            "max_seconds" => limit = v.parse().map_err(|_| invalid("limit"))?,
            _ => return Err(invalid("argument")),
        }
    }
    let out = out.ok_or_else(|| invalid("out"))?;
    let parent = parent.ok_or_else(|| invalid("parent"))?;
    if limit == 0 || limit > 900 {
        return Err(invalid("limit1..900"));
    }
    report_output::claim(&out)?;
    let result = run(&out, &parent, limit);
    match &result {
        Ok(v) => fs::write(out.join("report.json"), serde_json::to_vec_pretty(v)?)?,
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
