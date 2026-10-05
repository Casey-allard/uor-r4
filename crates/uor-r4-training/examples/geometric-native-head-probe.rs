//! Zero-update source-only native compiler head decomposition on opened panels.
//! Selection uses frozen source/label metadata; labels never enter inspect/predict.
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, OpenOptions},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_turn_compiler::NativeTurnCompiler,
    relation_compiler::{word_spans, Example},
    sha256_bytes,
    stack_grounded_session::{CompiledAction, SourceSpan},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROW_CAP: usize = 272;
const REPORT_CAP: u64 = 134_217_728;
const STEPS: [u64; 5] = [0, 16, 32, 48, 64];
fn invalid(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}
fn read(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn sha(path: &Path) -> Result<String> {
    Ok(sha256_bytes(&fs::read(path)?))
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
struct Args {
    native: PathBuf,
    binding: PathBuf,
    binding_sha: String,
    tokenizer: PathBuf,
    fit: PathBuf,
    out: PathBuf,
    max_seconds: u64,
}
fn args() -> Result<Args> {
    let mut f = BTreeMap::new();
    let mut it = std::env::args().skip(1);
    while let Some(k) = it.next() {
        let v = it
            .next()
            .ok_or_else(|| invalid("each flag needs a value"))?;
        if f.insert(k, v).is_some() {
            return Err(invalid("duplicate flag").into());
        }
    }
    let mut get = |k: &str| f.remove(k).ok_or_else(|| invalid(&format!("missing {k}")));
    let native = PathBuf::from(get("--native-artifact")?);
    let binding = PathBuf::from(get("--binding")?);
    let binding_sha = get("--binding-sha")?;
    let tokenizer = PathBuf::from(get("--tokenizer")?);
    let fit = PathBuf::from(get("--fit-root")?);
    let out = PathBuf::from(get("--output")?);
    let max_seconds = f
        .remove("--max-seconds")
        .map(|s| s.parse())
        .transpose()?
        .unwrap_or(300);
    if !f.is_empty()
        || max_seconds == 0
        || max_seconds > 300
        || binding_sha.len() != 64
        || !binding_sha.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("arguments/300-second ceiling/binding SHA").into());
    }
    for p in [&native, &binding, &tokenizer, &fit, &out] {
        if !p.is_absolute() {
            return Err(invalid("paths must be absolute").into());
        }
    }
    Ok(Args {
        native,
        binding,
        binding_sha,
        tokenizer,
        fit,
        out,
        max_seconds,
    })
}
fn example(v: &Value) -> Result<Example> {
    let act = match v["act"].as_str() {
        Some("assert") => "assert",
        Some("update") => "update",
        Some("query") => "query",
        Some("none") => "none",
        _ => return Err(invalid("invalid frozen act").into()),
    };
    let e = Example {
        text: v["text"]
            .as_str()
            .ok_or_else(|| invalid("source absent"))?
            .into(),
        relation: v["relation"]
            .as_str()
            .ok_or_else(|| invalid("relation absent"))?
            .into(),
        act,
        template: v["template"].as_str().map(str::to_owned),
    };
    if matches!(act, "assert" | "update") && serde_json::to_value(e.slot_span())? != v["gold_span"]
    {
        return Err(invalid("frozen original span receipt differs").into());
    }
    Ok(e)
}
#[derive(Clone)]
struct Row {
    e: Example,
    panel: String,
    index: usize,
    provenance: Vec<Value>,
}
fn row_value(r: &Row) -> Value {
    json!({"panel":r.panel,"source_index":r.index,"source":r.e.text,"relation_labels_only":r.e.relation,"act_labels_only":r.e.act,"template_matching_only":r.e.template,"expected_span_labels_only":r.e.slot_span(),"selection_provenance":r.provenance})
}
fn key(e: &Example) -> (String, String, Option<String>) {
    (e.relation.clone(), e.act.into(), e.template.clone())
}
fn rows(inputs: &Value) -> Result<(Vec<Row>, Value)> {
    let training: Vec<Example> = inputs["training"]
        .as_array()
        .ok_or_else(|| invalid("training absent"))?
        .iter()
        .map(example)
        .collect::<Result<_>>()?;
    if training.len() != 512 || inputs["development"].as_array().map(Vec::len) != Some(128) {
        return Err(invalid("crossed512/128 inputs required").into());
    }
    let mut selected = Vec::new();
    for (panel, list) in [
        ("development", &inputs["development"]),
        (
            "known_phrasing_new_values",
            &inputs["factor_inputs"]["known_phrasing_new_values"],
        ),
        (
            "new_phrasing_known_values",
            &inputs["factor_inputs"]["new_phrasing_known_values"],
        ),
        (
            "repeated_values",
            &inputs["factor_inputs"]["repeated_values"],
        ),
    ] {
        for (index, v) in list
            .as_array()
            .ok_or_else(|| invalid("frozen panel absent"))?
            .iter()
            .enumerate()
        {
            let e = example(v)?;
            if panel == "development" && !matches!(e.act, "assert" | "update") {
                continue;
            }
            selected.push(Row {
                e,
                panel: panel.into(),
                index,
                provenance: vec![
                    json!({"selection":"all frozen factor rows or development writes"}),
                ],
            });
        }
    }
    if selected.iter().filter(|r| r.panel == "development").count() != 64
        || selected.iter().filter(|r| r.panel != "development").count() != 144
    {
        return Err(invalid("exact64 development writes +144 factor rows required").into());
    }
    let mut frame_first = BTreeMap::new();
    for (i, e) in training.iter().enumerate() {
        if matches!(e.act, "assert" | "update") {
            frame_first.entry(key(e)).or_insert(i);
        }
    }
    if frame_first.len() != 32 {
        return Err(invalid("32 training write frames required").into());
    }
    let mut training_provenance:BTreeMap<usize,Vec<Value>>=frame_first.values().map(|&i|(i,vec![json!({"selection":"first original training row for each relation/act/template"})])).collect();
    let mut matches = Vec::new();
    for r in &selected {
        let first = frame_first.get(&key(&r.e)).copied();
        let mut base_match = None;
        if r.panel == "repeated_values" {
            let value =
                r.e.slot_value()
                    .ok_or_else(|| invalid("repetition value absent"))?;
            let words: Vec<_> = value.split_whitespace().collect();
            if words.len() % 2 == 0 && words[..words.len() / 2] == words[words.len() / 2..] {
                let base = words[..words.len() / 2].join(" ");
                let text =
                    r.e.template
                        .as_deref()
                        .ok_or_else(|| invalid("repetition frame absent"))?
                        .replace("{v}", &base);
                base_match = training
                    .iter()
                    .position(|e| key(e) == key(&r.e) && e.text == text);
                if let Some(i) = base_match {
                    training_provenance.entry(i).or_default().push(json!({"selection":"exact repeated-base training source counterpart","factor_panel":r.panel,"factor_index":r.index,"factor_source":r.e.text,"base_source":training[i].text}));
                }
            }
        }
        matches.push(json!({"panel":r.panel,"index":r.index,"source":r.e.text,"exact_frame_training_index":first,"exact_frame_training_source":first.map(|i|&training[i].text),"exact_repeated_base_training_index":base_match,"exact_repeated_base_training_source":base_match.map(|i|&training[i].text),"unmatched_exact_frame":first.is_none(),"matching_uses":"frozen relation/act/template/source only;never predictions;new phrasing may be unmatched"}));
    }
    if training_provenance.len() > 64 {
        return Err(invalid("training counterpart64 cap exceeded").into());
    }
    let counterpart_count = training_provenance.len();
    for (i, provenance) in training_provenance {
        selected.push(Row {
            e: training[i].clone(),
            panel: "training".into(),
            index: i,
            provenance,
        });
    }
    if selected.len() > ROW_CAP {
        return Err(invalid("probe272 row cap exceeded").into());
    }
    let mut dedup = BTreeSet::new();
    for r in &selected {
        if !dedup.insert((r.panel.clone(), r.index)) {
            return Err(invalid("duplicate original source row selection").into());
        }
    }
    let plan = json!({"schema":"uor-r4.native-head-probe-inputs/1","rows":selected.iter().map(row_value).collect::<Vec<_>>(),"cases_per_checkpoint":selected.len(),"training_counterparts":counterpart_count,"matches":matches,"checkpoint_steps":STEPS,"fresh_predictions":"NOT_RUN","updates":0,"selection":"all development writes/all opened factors;first training row perframe plus exact repeated-base counterpart;source-and-label-only fixed before model load"});
    Ok((selected, plan))
}
fn integers(v: &Value) -> Result<Vec<i64>> {
    v.as_array()
        .ok_or_else(|| invalid("score array absent"))?
        .iter()
        .map(|x| {
            x.as_i64()
                .ok_or_else(|| invalid("integer score absent"))
                .map_err(Into::into)
        })
        .collect()
}
fn head_check(head: &Value) -> Result<()> {
    let scores = integers(&head["scores"])?;
    let mut total = integers(&head["bias"])?;
    if total.len() != scores.len() {
        return Err(invalid("bias class shape").into());
    }
    for row in head["rows"]
        .as_array()
        .ok_or_else(|| invalid("head rows absent"))?
    {
        for group in row["groups"]
            .as_array()
            .ok_or_else(|| invalid("head groups absent"))?
        {
            let contribution = integers(&group["class_scores"])?;
            if contribution.len() != total.len() {
                return Err(invalid("group class shape").into());
            }
            for (a, b) in total.iter_mut().zip(contribution) {
                *a = a
                    .checked_add(b)
                    .ok_or_else(|| invalid("head contribution overflow"))?;
            }
        }
    }
    if total != scores {
        return Err(invalid("actual quarter head decomposition differs").into());
    }
    Ok(())
}
fn fields(a: &CompiledAction) -> (&str, Option<u32>, Option<SourceSpan>) {
    match a {
        CompiledAction::Assert { relation, span } => ("assert", Some(*relation), Some(*span)),
        CompiledAction::Correct { relation, span } => ("update", Some(*relation), Some(*span)),
        CompiledAction::QueryCurrent { relation } | CompiledAction::Query { relation, .. } => {
            ("query", Some(*relation), None)
        }
        CompiledAction::Unresolved { .. } => ("none", None, None),
    }
}
fn evaluate_inspection(r: &Row, inspect: &Value, prediction: &CompiledAction) -> Result<Value> {
    if inspect["source"] != r.e.text || inspect["action"] != serde_json::to_value(prediction)? {
        return Err(
            invalid("inspect hard action differs from independently called predict").into(),
        );
    }
    head_check(&inspect["act"])?;
    head_check(&inspect["relation"])?;
    let words = inspect["words"]
        .as_array()
        .ok_or_else(|| invalid("word trace absent"))?;
    let spans = inspect["span"]
        .as_array()
        .ok_or_else(|| invalid("span heads absent"))?;
    if words.len() != spans.len() || words.len() > 64 {
        return Err(invalid("word/span trace shape").into());
    }
    let mut margins = Vec::new();
    let mut inside_zero = 0;
    let mut inside_negative = 0;
    let gold = r.e.slot_span();
    for (i, (w, s)) in words.iter().zip(spans).enumerate() {
        head_check(s)?;
        let scores = integers(&s["scores"])?;
        if scores.len() != 2 {
            return Err(invalid("inside/outside class shape").into());
        }
        let margin = scores[1] - scores[0];
        if s["word_index"].as_u64() != Some(i as u64) || s["margin"].as_i64() != Some(margin) {
            return Err(invalid("actual span margin/index mismatch").into());
        }
        margins.push(margin);
        let start = w["span"]["start"]
            .as_u64()
            .ok_or_else(|| invalid("word start"))? as usize;
        let end = w["span"]["end"]
            .as_u64()
            .ok_or_else(|| invalid("word end"))? as usize;
        if gold.is_some_and(|(a, b)| start >= a && end <= b) {
            inside_zero += usize::from(margin == 0);
            inside_negative += usize::from(margin < 0);
        }
    }
    let mut best: Option<(i64, usize, usize)> = None;
    let mut top_ties = 0;
    for start in 0..margins.len() {
        let mut sum = 0i64;
        for (end, &m) in margins.iter().enumerate().skip(start).take(8) {
            sum = sum
                .checked_add(m)
                .ok_or_else(|| invalid("interval overflow"))?;
            if sum > 0 && best.is_none_or(|(b, _, _)| sum > b) {
                best = Some((sum, start, end + 1));
                top_ties = 1;
            } else if best.is_some_and(|(b, _, _)| sum == b) {
                top_ties += 1;
            }
        }
    }
    if let Some((sum, a, b)) = best {
        let chosen = &inspect["selected_span"];
        if chosen["margin_sum"].as_i64() != Some(sum)
            || chosen["word_start"].as_u64() != Some(a as u64)
            || chosen["word_end_exclusive"].as_u64() != Some(b as u64)
            || chosen["start"] != words[a]["span"]["start"]
            || chosen["end"] != words[b - 1]["span"]["end"]
        {
            return Err(invalid("native best interval/earliest-shortest tie differs").into());
        }
    } else if !inspect["selected_span"].is_null() {
        return Err(invalid("fabricated positive interval").into());
    }
    if inspect["span_selection"]["positive_maximum_ties"].as_u64() != Some(top_ties as u64)
        || inspect["span_selection"]["maximum_margin"].as_i64() != Some(best.map_or(0, |x| x.0))
    {
        return Err(invalid("native interval tie/margin summary differs").into());
    }
    let (act, relation, span) = fields(prediction);
    let expected_relation = match r.e.relation.as_str() {
        "job" => Some(1),
        "home" => Some(2),
        _ => None,
    };
    let expected = gold.map(|(start, end)| SourceSpan { start, end });
    Ok(
        json!({"act_correct":act==r.e.act,"relation_correct":relation==expected_relation,"span_exact":span==expected,"complete_exact":act==r.e.act&&relation==expected_relation&&span==expected,"predicted_span_start_correct":span.zip(expected).is_some_and(|(a,b)|a.start==b.start),"predicted_span_end_correct":span.zip(expected).is_some_and(|(a,b)|a.end==b.end),"payload_word_length":r.e.slot_value().map(|v|word_spans(v).len()),"inside_zero_margin_words":inside_zero,"inside_negative_margin_words":inside_negative,"positive_best_interval_ties":top_ties,"interval_reproduced_without_labels":true,"inspection_predict_equal":true}),
    )
}
fn fit_identity(report: &Value, inputs: &Value) -> Result<()> {
    if report["curriculum"] != "crossed-1"
        || inputs["curriculum"] != "crossed-1"
        || report["curriculum_sha256"] != inputs["curriculum_sha256"]
        || report["panel_sha256"] != inputs["panel_sha256"]
        || report["source_commit"]
            .as_str()
            .is_none_or(|x| x.len() != 40)
    {
        return Err(invalid("fit source/crossed curriculum identity differs").into());
    }
    Ok(())
}
fn run(a: &Args) -> Result<()> {
    let started = Instant::now();
    report_output::verify(&a.fit)?;
    let fit_manifest = sha(&a.fit.join("manifest.json"))?;
    let report = read(&a.fit.join("report.json"))?;
    let inputs = read(&a.fit.join("frozen-inputs.json"))?;
    fit_identity(&report, &inputs)?;
    let frozen_input_sha = sha(&a.fit.join("frozen-inputs.json"))?;
    if report["frozen_inputs_sha256"] != frozen_input_sha {
        return Err(invalid("fit frozen inputs SHA differs").into());
    }
    let (selected, plan) = rows(&inputs)?;
    write_json(&a.out.join("probe-inputs.json"), &plan)?;
    let plan_sha = sha(&a.out.join("probe-inputs.json"))?;
    // Everything above is metadata-only, frozen before loading any native model.
    if sha(&a.binding)? != a.binding_sha {
        return Err(invalid("trusted binding SHA differs").into());
    }
    let binding: NativeArtifactBinding = serde_json::from_slice(&fs::read(&a.binding)?)?;
    let native = NativeSourceRealizer::load_native(&a.native, &binding)?;
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("tokenizer invalid"))?;
    let tokenizer_sha = sha256_bytes(&tokenizer_bytes);
    if tokenizer_sha != binding.identity.tokenizer_sha256
        || report["tokenizer_sha256"] != tokenizer_sha
    {
        return Err(invalid("actual tokenizer/fit/native mismatch").into());
    }
    for row in &selected {
        if tokenizer.encode(&row.e.text).len() > 128 || word_spans(&row.e.text).len() > 64 {
            return Err(invalid("actual probe source bounds exceeded").into());
        }
    }
    let factors = read(&a.fit.join("checkpoint-factor-panels.json"))?;
    for checkpoints in [&factors, &report["checkpoints"]] {
        let list = checkpoints
            .as_array()
            .ok_or_else(|| invalid("checkpoint list absent"))?;
        let actual: BTreeSet<u64> = list
            .iter()
            .map(|v| {
                v["step"]
                    .as_u64()
                    .ok_or_else(|| invalid("checkpoint step absent"))
            })
            .collect::<std::result::Result<_, _>>()?;
        if list.len() != STEPS.len() || actual != STEPS.into_iter().collect() {
            return Err(invalid("exact five checkpoint identities required").into());
        }
    }
    let mut summaries = Vec::new();
    let mut written = 0u64;
    for step in STEPS {
        let original = factors
            .as_array()
            .ok_or_else(|| invalid("checkpoint panels absent"))?
            .iter()
            .find(|v| v["step"].as_u64() == Some(step))
            .ok_or_else(|| invalid("declared checkpoint absent"))?;
        let entry = report["checkpoints"]
            .as_array()
            .ok_or_else(|| invalid("fit checkpoints absent"))?
            .iter()
            .find(|v| v["step"].as_u64() == Some(step))
            .ok_or_else(|| invalid("report checkpoint absent"))?;
        let bytes = fs::read(
            a.fit
                .join(format!("checkpoint-{step:04}/native-compiler.json")),
        )?;
        let artifact_sha = sha256_bytes(&bytes);
        let artifact: Value = serde_json::from_slice(&bytes)?;
        if entry["artifact_sha256"] != artifact_sha
            || original["artifact_sha256"] != artifact_sha
            || artifact["feature_mode"] != report["feature_mode"]
            || artifact["initialization_seed"] != report["learning_seed"]
            || artifact["parent"] != serde_json::to_value(&binding)?
            || artifact["max_value_words"] != 8
            || artifact["max_words"] != 64
            || artifact["max_tokens"] != 128
        {
            return Err(invalid("loaded checkpoint hash/mode/seed/parent differs").into());
        }
        let compiler =
            NativeTurnCompiler::load(&bytes, &artifact_sha, &native, &tokenizer, &tokenizer_bytes)?;
        let path = a.out.join(format!("checkpoint-{step:04}.jsonl"));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        let mut writer = BufWriter::new(file);
        let mut counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for (i, row) in selected.iter().enumerate() {
            if started.elapsed().as_secs() > a.max_seconds {
                return Err(invalid("probe time ceiling reached;no fit performed").into());
            }
            let inspected = compiler.inspect(&row.e.text)?;
            let predicted = compiler.predict(&row.e.text)?;
            let diagnostic = evaluate_inspection(row, &inspected, &predicted)?;
            let saved = original[&row.panel]["rows"]
                .as_array()
                .ok_or_else(|| invalid("saved comparison panel absent"))?
                .get(row.index)
                .ok_or_else(|| invalid("saved comparison row absent"))?;
            let expected_relation = match row.e.relation.as_str() {
                "job" => 1u32,
                "home" => 2u32,
                _ => return Err(invalid("unknown source relation label").into()),
            };
            let expected_span = row
                .e
                .slot_span()
                .map(|(start, end)| SourceSpan { start, end });
            if saved["source"] != row.e.text
                || saved["gold_act"] != row.e.act
                || saved["gold_relation"] != expected_relation
                || saved["gold_span"] != serde_json::to_value(expected_span)?
                || saved["predicted"] != serde_json::to_value(&predicted)?
            {
                return Err(invalid(
                    "actual predicted action differs from retained samecheckpoint row",
                )
                .into());
            }
            let c = counts.entry(row.panel.clone()).or_default();
            *c.entry("rows".into()).or_default() += 1;
            for k in [
                "act_correct",
                "relation_correct",
                "span_exact",
                "complete_exact",
                "predicted_span_start_correct",
                "predicted_span_end_correct",
            ] {
                *c.entry(k.into()).or_default() += usize::from(diagnostic[k] == true);
            }
            let record = json!({"probe_row":i,"checkpoint_step":step,"checkpoint_artifact_sha256":artifact_sha,"input":row_value(row),"source_only_native_inspection":inspected,"diagnostics_labels_only_after_read":diagnostic,"retained_prediction_equal":true});
            let line = serde_json::to_vec(&record)?;
            written += line.len() as u64 + 1;
            if written > REPORT_CAP - 1_048_576 {
                return Err(invalid("probe128MiB report cap reached").into());
            }
            writer.write_all(&line)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;
        summaries.push(json!({"step":step,"artifact_sha256":artifact_sha,"rows":selected.len(),"jsonl_sha256":sha(&path)?,"panels":counts}));
    }
    if sha(&a.fit.join("manifest.json"))? != fit_manifest
        || sha(&a.binding)? != a.binding_sha
        || sha(&a.tokenizer)? != tokenizer_sha
    {
        return Err(invalid("input identity changed during probe").into());
    }
    report_output::verify(&a.fit)?;
    write_json(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.native-head-probe/1","status":"completed","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"fit_source_commit":report["source_commit"],"fit_manifest_sha256":fit_manifest,"fit_frozen_inputs_sha256":frozen_input_sha,"probe_inputs_sha256":plan_sha,"parent_binding":binding,"binding_sha256":a.binding_sha,"tokenizer_sha256":tokenizer_sha,"feature_mode":report["feature_mode"],"learning_seed":report["learning_seed"],"panel_sha256":report["panel_sha256"],"curriculum_sha256":report["curriculum_sha256"],"checkpoint_summaries":summaries,"updates":0,"fresh_predictions":"NOT_RUN","checkpoint_selection":"unchanged original fit;allfive retained checkpoints inspected","scope":"actual native headscore decomposition and interval/action reproduction on opened development/factor/trainingcounterpart source rows;labels onlypostprediction diagnostics","elapsed_seconds":started.elapsed().as_secs_f64(),"jsonl_payload_bytes":written,"report_cap_bytes":REPORT_CAP,"maximum_seconds":a.max_seconds}),
    )?;
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    Ok(())
}
fn main() -> Result<()> {
    let a = args()?;
    let parent = fs::canonicalize(
        a.out
            .parent()
            .ok_or_else(|| invalid("output parent absent"))?,
    )?;
    let prospective = parent.join(
        a.out
            .file_name()
            .ok_or_else(|| invalid("output basename absent"))?,
    );
    for anc in prospective.ancestors() {
        if anc.join(report_output::MANIFEST_FILE).exists() {
            return Err(invalid("output under sealed report").into());
        }
    }
    for input in [&a.native, &a.binding, &a.tokenizer, &a.fit] {
        let input = fs::canonicalize(input)?;
        if prospective.starts_with(&input) {
            return Err(invalid("output beneath input").into());
        }
        for anc in input.ancestors() {
            if anc.join(report_output::MANIFEST_FILE).exists() && prospective.starts_with(anc) {
                return Err(invalid("output beneath sealed input").into());
            }
        }
    }
    report_output::claim(&a.out)?;
    if let Err(e) = run(&a) {
        write_json(
            &a.out.join("failure.json"),
            &json!({"status":"execution-failure","error":e.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"updates":0,"fresh_predictions":"NOT_RUN","model_quality_evidence":false}),
        )?;
        report_output::seal(&a.out)?;
        report_output::verify(&a.out)?;
        return Err(e);
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_label_identity_rejects_legacy_or_changed_curriculum() -> Result<()> {
        let r = json!({"curriculum":"crossed-1","curriculum_sha256":"a","panel_sha256":"p","source_commit":"0".repeat(40)});
        let mut i = json!({"curriculum":"crossed-1","curriculum_sha256":"a","panel_sha256":"p"});
        fit_identity(&r, &i)?;
        i["curriculum"] = json!("legacy");
        assert!(fit_identity(&r, &i).is_err());
        i["curriculum"] = json!("crossed-1");
        i["panel_sha256"] = json!("changed");
        assert!(fit_identity(&r, &i).is_err());
        Ok(())
    }
    #[test]
    fn original_span_receipt_cannot_be_changed() -> Result<()> {
        let v = json!({"text":"My job is singer.","act":"assert","relation":"job","template":"My job is {v}.","gold_span":[10,16]});
        example(&v)?;
        let mut wrong = v;
        wrong["gold_span"] = json!([10, 15]);
        assert!(example(&wrong).is_err());
        Ok(())
    }
}
