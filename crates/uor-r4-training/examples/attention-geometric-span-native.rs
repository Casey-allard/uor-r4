//! Saved-model retention through exported token actions and an integer span register.
//! No training; controller, reconstruction and surrounding reader remain float.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::Path, time::Instant};
use uor_r4_core::report_output;
use uor_r4_training::geometric_span_native::{
    compare_reference, trace_native, CompiledSpanActions, SpanSourceBinding,
};
use uor_r4_training::geometric_stack::{ReadBinding, ReadBindingTarget, StackModel};
use uor_r4_training::{sha256_file, Result, TrainingError};

fn invalid(message: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(message.into())
}
#[derive(Clone, Deserialize)]
struct Episode {
    ids: Vec<u32>,
    source: usize,
    query: usize,
    answer: u32,
    pair: usize,
    condition: String,
}
#[derive(Serialize)]
struct Row {
    pair: usize,
    condition: String,
    source: usize,
    query: usize,
    answer: u32,
    baseline_prediction: u32,
    native_prediction: u32,
    baseline_answer_logits: Vec<f32>,
    native_answer_logits: Vec<f32>,
    baseline_source_masses: [f32; 2],
    native_source_masses: [f32; 2],
    answer_logits_bit_identical: bool,
    source_masses_bit_identical: bool,
    historical_answer_logits_bit_identical: bool,
    historical_source_masses_bit_identical: bool,
}
fn batch(es: &[Episode]) -> Result<(Vec<u32>, usize)> {
    let time = es
        .iter()
        .map(|e| e.ids.len())
        .max()
        .ok_or_else(|| invalid("empty panel chunk"))?;
    let count = es
        .len()
        .checked_mul(time)
        .ok_or_else(|| invalid("batch overflow"))?;
    let mut ids = vec![38; count];
    for (b, e) in es.iter().enumerate() {
        if e.query >= e.ids.len() || e.source > e.query || e.ids.get(e.source) != Some(&e.answer) {
            return Err(invalid("saved source/query/value contract differs"));
        }
        ids[b * time..b * time + e.ids.len()].copy_from_slice(&e.ids);
    }
    Ok((ids, time))
}
fn target(es: &[Episode], head: usize) -> ReadBindingTarget {
    ReadBindingTarget {
        layer: 2,
        head,
        rows: es
            .iter()
            .enumerate()
            .map(|(batch, e)| ReadBinding {
                batch,
                query: e.query,
                sources: vec![e.source],
            })
            .collect(),
    }
}
fn bits_equal(a: &[f32], b: &[f32]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.to_bits() == y.to_bits())
}
fn floats(value: &Value, key: &str) -> Result<Vec<f32>> {
    value
        .get(key)
        .and_then(Value::as_array)
        .ok_or_else(|| invalid(format!("missing {key}")))?
        .iter()
        .map(|v| {
            v.as_f64()
                .map(|x| x as f32)
                .ok_or_else(|| invalid("non-numeric reference"))
        })
        .collect()
}
fn registry() -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(
        &json!({"schema":"uor-r4.authored-span-token-registry/1",
        "vocabulary":40,"key_atoms":[0,15],"values":[16,31],"write_open":32,
        "query_open":33,"answer_marker":34,"value_marker":35,"bos":36,
        "noise_marker":37,"padding":38,"commit":39,
        "meaning":"exact authored registry of attention-geometric-span.rs; not a general tokenizer"}),
    )?)
}
fn compare_panel(
    model: &StackModel,
    compiled: &CompiledSpanActions,
    es: &[Episode],
    historical: &[Value],
    directory: &Path,
    deadline: Instant,
) -> Result<Value> {
    if es.len() != 128 || historical.len() != es.len() {
        return Err(invalid("fixed panel size differs"));
    }
    fs::create_dir(directory)?;
    let variables = model.variables();
    let embedding = variables
        .get("embedding.weight")
        .ok_or_else(|| invalid("embedding absent"))?
        .as_tensor();
    let span = model
        .geometric_span()
        .ok_or_else(|| invalid("span mode absent"))?;
    let mut all_rows = Vec::new();
    let mut positions = 0;
    let mut unpadded_positions = 0;
    let mut present = 0;
    let mut presence_mismatches = 0;
    let mut code_mismatches = 0;
    let mut reconstruction_gap = 0f32;
    let mut full_logit_bit_mismatches = 0;
    for (chunk, group) in es.chunks(32).enumerate() {
        if Instant::now() >= deadline {
            return Err(invalid("replay deadline; completed chunks preserved"));
        }
        let (ids, time) = batch(group)?;
        let events = model.geometric_span_control_logits(&ids, group.len(), time)?;
        let comparison =
            compare_reference(&ids, group.len(), time, &events, embedding, span, compiled)?;
        positions += comparison.positions;
        present += comparison.reference_present_positions;
        unpadded_positions += group.iter().map(|e| e.ids.len()).sum::<usize>();
        presence_mismatches += comparison.presence_mismatches;
        code_mismatches += comparison.present_lane_code_mismatches;
        reconstruction_gap = reconstruction_gap.max(comparison.max_absolute_reconstruction_error);
        let trace = trace_native(&ids, group.len(), time, &events, compiled)?;
        let baseline = model.forward(&ids, group.len(), time)?.to_vec2::<f32>()?;
        let native = model
            .forward_geometric_span_native(&ids, group.len(), time, compiled)?
            .to_vec2::<f32>()?;
        if baseline.len() != ids.len()
            || native.len() != baseline.len()
            || baseline
                .iter()
                .zip(&native)
                .any(|(a, b)| a.len() != model.config.vocab_size || b.len() != a.len())
        {
            return Err(invalid("full-prefix output shape differs"));
        }
        for (a, b) in baseline.iter().zip(&native) {
            full_logit_bit_mismatches += a
                .iter()
                .zip(b)
                .filter(|(x, y)| x.to_bits() != y.to_bits())
                .count();
        }
        let mut base_mass = [Vec::new(), Vec::new()];
        let mut native_mass = [Vec::new(), Vec::new()];
        for head in 0..2 {
            base_mass[head] = model
                .read_binding_masses(&ids, group.len(), time, &target(group, head))?
                .to_vec1::<f32>()?;
            native_mass[head] = model
                .read_binding_masses_geometric_span_native(
                    &ids,
                    group.len(),
                    time,
                    &target(group, head),
                    compiled,
                )?
                .to_vec1::<f32>()?;
        }
        let mut rows = Vec::new();
        for (i, e) in group.iter().enumerate() {
            let at = i * time + e.query;
            let old = &historical[chunk * 32 + i];
            if old["pair"].as_u64() != Some(e.pair as u64)
                || old["condition"].as_str() != Some(e.condition.as_str())
                || old["source"].as_u64() != Some(e.source as u64)
                || old["query"].as_u64() != Some(e.query as u64)
                || old["answer"].as_u64() != Some(e.answer as u64)
            {
                return Err(invalid("historical row identity differs"));
            }
            let old_logits = floats(old, "answer_logits")?;
            let old_mass = floats(old, "source_masses_head0_head1")?;
            let base = [base_mass[0][i], base_mass[1][i]];
            let nm = [native_mass[0][i], native_mass[1][i]];
            let bp = argmax(&baseline[at])?;
            let np = argmax(&native[at])?;
            rows.push(Row {
                pair: e.pair,
                condition: e.condition.clone(),
                source: e.source,
                query: e.query,
                answer: e.answer,
                baseline_prediction: bp,
                native_prediction: np,
                baseline_answer_logits: baseline[at].clone(),
                native_answer_logits: native[at].clone(),
                baseline_source_masses: base,
                native_source_masses: nm,
                answer_logits_bit_identical: bits_equal(&baseline[at], &native[at]),
                source_masses_bit_identical: bits_equal(&base, &nm),
                historical_answer_logits_bit_identical: bits_equal(&baseline[at], &old_logits),
                historical_source_masses_bit_identical: bits_equal(&base, &old_mass),
            });
        }
        // Write each completed chunk before proceeding; a stopped run retains its actual work.
        fs::write(
            directory.join(format!("chunk-{chunk}.json")),
            serde_json::to_vec_pretty(&json!({
            "chunk":chunk,"time":time,"batch":group.len(),"comparison":comparison,"trace":trace,"rows":rows}))?,
        )?;
        all_rows.extend(rows);
    }
    let answer_matches = all_rows
        .iter()
        .filter(|r| r.answer_logits_bit_identical)
        .count();
    let source_matches = all_rows
        .iter()
        .filter(|r| r.source_masses_bit_identical)
        .count();
    let historical_answer_matches = all_rows
        .iter()
        .filter(|r| r.historical_answer_logits_bit_identical)
        .count();
    let historical_source_matches = all_rows
        .iter()
        .filter(|r| r.historical_source_masses_bit_identical)
        .count();
    let result = json!({"rows":all_rows.len(),"padded_positions":positions,"unpadded_positions":unpadded_positions,
        "present_positions":present,"presence_mismatches":presence_mismatches,"present_lane_code_mismatches":code_mismatches,
        "max_absolute_reconstruction_error":reconstruction_gap,"full_logit_bit_mismatches":full_logit_bit_mismatches,
        "identical_answer_logit_rows":answer_matches,"identical_source_mass_rows":source_matches,
        "historical_identical_answer_logit_rows":historical_answer_matches,"historical_identical_source_mass_rows":historical_source_matches,
        "baseline_answers":all_rows.iter().filter(|r|r.baseline_prediction==r.answer).count(),
        "native_answers":all_rows.iter().filter(|r|r.native_prediction==r.answer).count(),
        "native_head0_source_majorities":all_rows.iter().filter(|r|r.native_source_masses[0]>0.5).count(),
        "rows_detail":all_rows});
    fs::write(
        directory.join("summary.json"),
        serde_json::to_vec_pretty(&result)?,
    )?;
    Ok(result)
}
fn argmax(row: &[f32]) -> Result<u32> {
    if row.is_empty() || row.iter().any(|x| !x.is_finite()) {
        return Err(invalid("invalid answer logits"));
    }
    let mut selected = 0;
    for i in 1..row.len() {
        if row[i] > row[selected] {
            selected = i;
        }
    }
    Ok(selected as u32)
}
fn run(parent: &Path, out: &Path, seconds: u64) -> Result<Value> {
    let start = Instant::now();
    let deadline = start + std::time::Duration::from_secs(seconds);
    for (file, expected) in [
        (
            "report.json",
            "60f78851dd86b0063cca537eabd50aa2a3c092bd25b29de4f6db7ae5d2ef3db5",
        ),
        (
            "evaluation.json",
            "dabdaa1a2a8dcf23e1cfe5164ef00b97b923c3b74dc09dcd1d4a307d1ff64915",
        ),
        (
            "stress.json",
            "e3c81984b7b41e0fcb329fde026318d067b6397e93832508c3ee60180a5a9d63",
        ),
    ] {
        if sha256_file(&parent.join("run-1").join(file))? != expected {
            return Err(invalid(format!("fixed parent {file} identity differs")));
        }
    }
    let prior: Value = serde_json::from_slice(&fs::read(parent.join("run-1/report.json"))?)?;
    if prior["source_commit"].as_str() != Some("cfdf535a800e343873699ca4693f7b11f9a2e32d") {
        return Err(invalid("reference source differs"));
    }
    let evaluation: Vec<Episode> =
        serde_json::from_slice(&fs::read(parent.join("run-1/evaluation.json"))?)?;
    let stress: Vec<Episode> =
        serde_json::from_slice(&fs::read(parent.join("run-1/stress.json"))?)?;
    let identity = registry()?;
    let mut results = Vec::new();
    for seed in prior["reports"]
        .as_array()
        .ok_or_else(|| invalid("reference reports absent"))?
    {
        if Instant::now() >= deadline {
            return Err(invalid("deadline before next saved seed"));
        }
        let name = seed["name"]
            .as_str()
            .ok_or_else(|| invalid("seed name absent"))?;
        if !matches!(
            name,
            "GeometricSpan-Ordered-s1" | "GeometricSpan-Ordered-s2"
        ) {
            return Err(invalid("unknown seed path"));
        }
        let directory = parent.join("run-1").join(name).join("model");
        if Some(sha256_file(&directory.join("model.safetensors"))?.as_str())
            != seed["model_sha256"].as_str()
        {
            return Err(invalid("saved model hash differs"));
        }
        let model = StackModel::load(&directory, &Device::Cpu)?;
        if serde_json::to_value(&model.config)? != seed["config"]
            || serde_json::to_value(model.geometric_span())? != seed["geometric_span_config"]
        {
            return Err(invalid("fixed saved model/span configuration differs"));
        }
        let source = SpanSourceBinding::from_files(
            &directory.join("model.safetensors"),
            &directory.join("config.json"),
            &identity,
        )?;
        let vars = model.variables();
        let embedding = vars
            .get("embedding.weight")
            .ok_or_else(|| invalid("embedding absent"))?
            .as_tensor();
        let span = model
            .geometric_span()
            .ok_or_else(|| invalid("saved span absent"))?;
        let compiled = CompiledSpanActions::compile(embedding, span, &source)?;
        let attempt = out.join(name);
        fs::create_dir(&attempt)?;
        compiled.save(&attempt.join("native-actions"))?;
        let loaded = CompiledSpanActions::load(&attempt.join("native-actions"), &source, span)?;
        if loaded.metadata() != compiled.metadata() {
            return Err(invalid("native action reload differs"));
        }
        let measured = &seed["measured"];
        let a = compare_panel(
            &model,
            &loaded,
            &evaluation,
            measured["rows"]
                .as_array()
                .ok_or_else(|| invalid("reference rows absent"))?,
            &attempt.join("original"),
            deadline,
        )?;
        let b = compare_panel(
            &model,
            &loaded,
            &stress,
            measured["stress_rows"]
                .as_array()
                .ok_or_else(|| invalid("stress reference absent"))?,
            &attempt.join("stress"),
            deadline,
        )?;
        let decision = if [&a, &b].iter().all(|p| {
            p["presence_mismatches"] == 0
                && p["present_lane_code_mismatches"] == 0
                && p["max_absolute_reconstruction_error"] == 0.
                && p["full_logit_bit_mismatches"] == 0
                && p["identical_answer_logit_rows"] == 128
                && p["identical_source_mass_rows"] == 128
                && p["historical_identical_answer_logit_rows"] == 128
                && p["historical_identical_source_mass_rows"] == 128
        }) {
            "EXACT_FIXED_REPLAY"
        } else {
            "MEASURED_MISMATCH_RETAIN_TRACES"
        };
        let r = json!({"name":name,"source_model_sha256":seed["model_sha256"],"native_metadata":loaded.metadata(),"original":a,"stress":b,"decision":decision});
        fs::write(attempt.join("result.json"), serde_json::to_vec_pretty(&r)?)?;
        results.push(r);
    }
    if results.len() != 2 {
        return Err(invalid("fixed two-model scope incomplete"));
    }
    Ok(
        json!({"schema":"uor-r4.geometric-span-native-replay/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT").unwrap_or("UNAVAILABLE"),
        "parent_report_sha256":sha256_file(&parent.join("run-1/report.json"))?,"evaluation_sha256":sha256_file(&parent.join("run-1/evaluation.json"))?,
        "stress_sha256":sha256_file(&parent.join("run-1/stress.json"))?,"elapsed_seconds":start.elapsed().as_secs_f64(),"maximum_seconds":seconds,
        "training_updates":0,"results":results,"scope":"native integer dictionary/register component with float predicted controller, reconstruction and surrounding reader; no full serving/language/algebra superiority/energy qualification"}),
    )
}
fn main() -> Result<()> {
    let mut parent = None;
    let mut out = None;
    let mut seconds = 900;
    for arg in std::env::args().skip(1) {
        let (k, v) = arg.split_once('=').ok_or_else(|| invalid("key=value"))?;
        match k {
            "parent" => parent = Some(std::path::PathBuf::from(v)),
            "out" => out = Some(std::path::PathBuf::from(v)),
            "max_seconds" => seconds = v.parse().map_err(|_| invalid("seconds"))?,
            _ => return Err(invalid("unknown option")),
        }
    }
    if seconds == 0 || seconds > 900 {
        return Err(invalid("max_seconds1..900"));
    }
    let parent = parent.ok_or_else(|| invalid("parent required"))?;
    let out = out.ok_or_else(|| invalid("out required"))?;
    report_output::claim(&out)?;
    let result = run(&parent, &out, seconds);
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_geometric_span_evaluator_observes_occurrences_without_action_labels() -> Result<()> {
        let e = Episode {
            ids: vec![36, 32, 1, 2, 39, 35, 17, 34],
            source: 6,
            query: 7,
            answer: 17,
            pair: 1,
            condition: "test".into(),
        };
        let (ids, t) = batch(&[e.clone()])?;
        assert_eq!(ids, e.ids);
        assert_eq!(t, 8);
        assert_eq!(target(&[e], 0).rows[0].sources, vec![6]);
        assert_eq!(argmax(&[1., 1., 0.])?, 0);
        assert!(!bits_equal(&[0.], &[-0.]));
        assert!(argmax(&[f32::NAN]).is_err());
        Ok(())
    }
}
