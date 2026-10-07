//! Frozen integer-only deployment and source-value substitution evaluation.
//! Labels are evaluator-only; native generation sees an owned complete bank.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::{
    answer_oracle::FrozenAnswers,
    native_geometric::learner::native_bank_generate::{
        BankPin, BoundNativeBytes, GenerationLimits, GenerationStop, NativeBankArtifacts,
        NativeBankGenerateStep, NativeBankGenerator, OwnedBankSegment, OwnedBankSource,
        PinnedBankSnapshot, SnapshotSourceStatus,
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata, geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_realizer::NativeArtifactBinding,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
use uor_r4_tokenizer::ByteBpeTokenizer;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    model_root: PathBuf,
    expected_model_report_sha256: String,
    expected_model_manifest_sha256: String,
    inputs: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    output: PathBuf,
    #[serde(default = "default_report_bytes")]
    maximum_report_bytes: u64,
    #[serde(default)]
    baseline_parity: bool,
}
fn default_report_bytes() -> u64 {
    // Full traces for 32 cases at the 32-token limit can exceed 2 GiB.
    3 << 30
}
fn write_row(c: &Config, name: &str, row: &Value) -> Result<()> {
    let used = fs::read_dir(&c.output)?.try_fold(0u64, |sum, e| -> Result<u64> {
        Ok(sum
            .checked_add(e?.metadata()?.len())
            .ok_or_else(|| bad("output size overflow"))?)
    })?;
    let bytes = serde_json::to_vec_pretty(row)?;
    if used.saturating_add(bytes.len() as u64) > c.maximum_report_bytes - (1 << 20) {
        return Err(bad(
            "declared report storage boundary reached; no rows truncated",
        ));
    }
    Ok(fs::write(c.output.join(name), bytes)?)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Panel {
    schema: String,
    cases: Vec<Packet>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum Segment {
    Source {
        event: u64,
        record: u64,
        commit: u64,
        scope: String,
        entity: Vec<u32>,
        relation: u32,
        view: u32,
        original_source_ids: Vec<u32>,
    },
    Context {
        event: u64,
        role: u32,
        token_ids: Vec<u32>,
    },
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Labels {
    schema: String,
    protocol: String,
    membership_only: bool,
    cases: Vec<Label>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Label {
    id: String,
    answers: FrozenAnswers,
    #[serde(default)]
    pair_id: Option<String>,
}
fn bad(s: &str) -> Box<dyn std::error::Error> {
    std::io::Error::other(s).into()
}
fn bytes(p: &Path) -> Result<Vec<u8>> {
    Ok(fs::read(p)?)
}
fn hash(b: &[u8]) -> String {
    hex::encode(Sha256::digest(b))
}
fn file_hash(p: &Path) -> Result<String> {
    Ok(hash(&bytes(p)?))
}
fn read(p: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&bytes(p)?)?)
}
fn write(p: &Path, v: &Value) -> Result<()> {
    Ok(fs::write(p, serde_json::to_vec_pretty(v)?)?)
}
fn expected_string(v: &Value) -> Result<&str> {
    v.as_str()
        .ok_or_else(|| bad("expected digest/string absent"))
}
fn seal_for(path: &Path) -> Result<PathBuf> {
    path.ancestors()
        .find(|p| p.join("manifest.json").is_file())
        .map(Path::to_path_buf)
        .ok_or_else(|| bad("input has no enclosing report seal"))
}
fn snapshot(p: Packet) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("generation must start with empty actual prefix"));
    }
    let mut scope = None;
    let mut commit = 0;
    for s in &p.segments {
        if let Segment::Source {
            scope: q,
            commit: c,
            ..
        } = s
        {
            if q.is_empty() || scope.as_ref().is_some_and(|old| old != q) {
                return Err(bad("panel bank has empty/mixed Source scope"));
            }
            scope = Some(q.clone());
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("Source substitution panel requires Source records"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.into_bytes(),
        },
        query_ids: p.query_ids,
        segments: p
            .segments
            .into_iter()
            .map(|s| match s {
                Segment::Source {
                    event,
                    record,
                    commit,
                    scope,
                    entity,
                    relation,
                    view,
                    original_source_ids,
                } => OwnedBankSegment::Source(OwnedBankSource {
                    event,
                    record,
                    commit,
                    scope: scope.into_bytes(),
                    entity,
                    relation,
                    view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids,
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event,
                    role,
                    token_ids,
                },
            })
            .collect(),
    })
}
fn serve(
    g: &mut NativeBankGenerator,
    p: Packet,
) -> Result<uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGeneration> {
    let bank = g.admit_bank(snapshot(p)?)?;
    Ok(g.generate(
        &bank,
        GenerationLimits {
            maximum_tokens: 32,
            maximum_retained_steps: 32,
        },
    )?)
}
fn compare_step(actual: &NativeBankGenerateStep, expected: &Value) -> Result<()> {
    if expected["actual_prefix_ids"] != json!(actual.actual_prefix_ids)
        || expected["copy_token_ids"] != json!(actual.copy_token_ids)
        || expected["copy_raw_scores_q24"] != json!(actual.copy_raw_scores_q24)
        || expected["retained_state_codes"]
            != json!(actual
                .post_state
                .iter()
                .map(|c| c.index())
                .collect::<Vec<_>>())
        || expected["generate_raw_scores_sha256"]
            != hash(&serde_json::to_vec(&actual.generate_raw_scores_q24)?)
        || expected["pool"]["summary"] != serde_json::to_value(&actual.actions.summary)?
        || expected["generate_costs"] != serde_json::to_value(&actual.generate_counts)?
    {
        return Err(bad(
            "native prefix/Copy/poststate/Generate/reducer summary parity differs",
        ));
    }
    if let Some(trace) = &actual.bank_trace {
        let bank = &trace.cue_bank.bank;
        if expected["source_provenance"]["candidates"] != serde_json::to_value(&bank.candidates)?
            || expected["source_provenance"]["causal_tokens"] != json!(bank.context.tokens)
            || expected["source_provenance"]["legacy_terminal_actions_discarded"] != true
        {
            return Err(bad("full occurrence chronology/provenance parity differs"));
        }
        let count = actual.copy_token_ids.len();
        let sum = |heads: &[Vec<i64>]| -> Result<Vec<i64>> {
            let mut result = vec![0i64; count];
            for head in heads {
                if head.len() != count {
                    return Err(bad("component candidate count differs"));
                }
                for (i, &score) in head.iter().enumerate() {
                    result[i] = result[i]
                        .checked_add(score)
                        .ok_or_else(|| bad("component overflow"))?;
                }
            }
            Ok(result)
        };
        let cue = sum(&trace.cue_bank.carrier.copy_q24)?;
        let prefix = sum(&trace.prefix.copy_q24)?;
        let context = actual
            .copy_raw_scores_q24
            .iter()
            .enumerate()
            .map(|(i, &score)| {
                score
                    .checked_sub(cue[i])
                    .and_then(|v| v.checked_sub(prefix[i]))
                    .ok_or_else(|| bad("attribution overflow"))
            })
            .collect::<Result<Vec<_>>>()?;
        if expected["source_provenance"]["copy_components_q24"]
            != json!({"cue":cue,"prefix":prefix,"contextual":context})
        {
            return Err(bad("Copy component attribution parity differs"));
        }
    } else if expected["source_provenance"]["no_source"] != true {
        return Err(bad("source availability parity differs"));
    }
    match &actual.bridge {
        Some(b) => {
            let saved = &expected["source_provenance"]["read_state_bridge"];
            if saved["selected_ordinal"] != json!(b.selected_ordinal)
                || saved["selected_candidate"] != serde_json::to_value(&b.selected_candidate)?
                || saved["query_state_codes"]
                    != json!(b.query_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["selected_source_state_codes"]
                    != json!(b.source_state.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_codes"]
                    != json!(b.action_codes.iter().map(|c| c.index()).collect::<Vec<_>>())
                || saved["action_scores_q24"] != json!(b.action_scores_q24)
                || saved["native_costs"] != serde_json::to_value(&b.counts)?
            {
                return Err(bad("raw-selected categorical bridge parity differs"));
            }
        }
        None if !expected["source_provenance"]["read_state_bridge"].is_null() => {
            return Err(bad("unexpected saved bridge"))
        }
        None => {}
    }
    Ok(())
}
fn run(c: &Config) -> Result<Value> {
    report_output::verify(&c.model_root)?;
    if file_hash(&c.model_root.join("report.json"))? != c.expected_model_report_sha256
        || file_hash(&c.model_root.join("manifest.json"))? != c.expected_model_manifest_sha256
    {
        return Err(bad("frozen model report/seal identity differs"));
    }
    let r = read(&c.model_root.join("report.json"))?;
    let admission = read(&c.model_root.join("admission.json"))?;
    if r["schema"] != "uor-r4.geometric-prediction-control/1"
        || r["status"] != "COMPLETED"
        || r["updates"] != 0
        || r["native_prediction_control_win"] != true
        || r["resume"]["component_recomposition"] != true
        || r["resume"]["source_step"] != 48
        || r["resume"]["generate_step"] != 64
        || r["resume"]["producer_source_commit"] != "b451571aa638d1f93e3ccf25f718f27ed2ea731f"
    {
        return Err(bad(
            "model is not the completed successful zero-update48/64 recomposition",
        ));
    }
    let cp = c.model_root.join("checkpoint-0000");
    let receipt = read(&cp.join("receipt.json"))?;
    if receipt != r["final_receipt"]
        || receipt != r["initial_receipt"]
        || receipt["step"] != 0
        || receipt["native_independently_reloaded"] != true
        || receipt["masters_independently_reloaded"] != true
    {
        return Err(bad("coherent frozen native checkpoint receipt differs"));
    }
    // This is an external authored panel, not a store enumeration or lineage proof.
    let input_root = seal_for(&c.inputs)?;
    let label_root = seal_for(&c.labels)?;
    report_output::verify(&input_root)?;
    report_output::verify(&label_root)?;
    if file_hash(&c.inputs)? != c.expected_inputs_sha256
        || file_hash(&c.labels)? != c.expected_labels_sha256
    {
        return Err(bad("panel identity differs"));
    }
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    let labels: Labels = serde_json::from_slice(&bytes(&c.labels)?)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1"
        || labels.schema != "uor-r4.native-source-bank-labels/1"
        || labels.protocol != "uor-r4.literal-role-dialogue/2"
        || !labels.membership_only
        || panel.cases.is_empty()
        || panel.cases.len() > 512
        || panel.cases.len() != labels.cases.len()
    {
        return Err(bad("bounded complete input/label schema coverage differs"));
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").is_file() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let native_dir = cp.join("native");
    let mut g = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &native_dir,
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen,
            sha256: expected_string(&receipt["generate_sha256"])?,
        },
        bridge: Some(BoundNativeBytes {
            bytes: &bridge,
            sha256: expected_string(&receipt["categorical_sha256"])?,
        }),
        cue_packed: &cue,
        cue_joint_packed: joint.as_deref(),
        cue_metadata: &cue_metadata,
        prefix_packed: &prefix,
        prefix_metadata: &prefix_metadata,
        exp: BoundNativeBytes {
            bytes: &exp,
            sha256: &exp_hash,
        },
    })?;
    let tok =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes(&native_dir.join("tokenizer.json"))?)
            .ok_or_else(|| bad("ByteBPE unavailable"))?;
    let saved = read(&c.model_root.join("prediction-0000.json"))?;
    let saved_refs = saved["rows"]
        .as_array()
        .ok_or_else(|| bad("saved model generation rows absent"))?;
    if c.baseline_parity && (panel.cases.len() != 8 || saved_refs.len() != 8) {
        return Err(bad("baseline parity requires exactly original eight cases"));
    }
    let mut seen = BTreeSet::new();
    let mut rows = Vec::new();
    let mut complete = 0;
    let mut entry = 0;
    let mut pair_groups: BTreeMap<String, Vec<(String, bool, Vec<u32>)>> = BTreeMap::new();
    for (index, (p, l)) in panel.cases.into_iter().zip(labels.cases).enumerate() {
        if p.id.is_empty() || p.id != l.id || !seen.insert(p.id.clone()) {
            return Err(bad("input/label IDs mismatched or duplicate"));
        }
        l.answers.validate()?;
        let id = p.id.clone();
        let generated = serve(&mut g, p)?; // No label, target or answer reaches this call.
        if generated
            .steps
            .iter()
            .any(|step| step.actions.summary.legal_generate_actions != 4096)
        {
            return Err(bad(
                "frozen model full 4096 legal Generate vocabulary differs",
            ));
        }
        let eos = generated.stop == GenerationStop::Eos;
        let output_ids = if eos {
            &generated.generated_ids[..generated.generated_ids.len() - 1]
        } else {
            &generated.generated_ids[..]
        };
        let decoded_bytes = tok.decode_bytes(output_ids);
        let valid_text = std::str::from_utf8(&decoded_bytes).ok();
        let accepted = eos && valid_text.is_some_and(|text| l.answers.accepts(text));
        let decoded = String::from_utf8_lossy(&decoded_bytes).into_owned();
        complete += usize::from(accepted);
        let target = tok.encode(&l.answers.accepted[0]);
        if tok.decode_bytes(&target) != l.answers.accepted[0].as_bytes() {
            return Err(bad("evaluator accepted form does not roundtrip tokenizer"));
        }
        let entry_correct = generated.generated_ids.first() == target.first();
        entry += usize::from(entry_correct);
        let mut parity = Value::Null;
        if c.baseline_parity {
            let reference = &saved_refs[index];
            if reference["id"] != id {
                return Err(bad("original baseline row order differs"));
            }
            let name = expected_string(&reference["row_file"])?;
            if Path::new(name)
                .components()
                .any(|p| !matches!(p, std::path::Component::Normal(_)))
            {
                return Err(bad("unsafe saved row path"));
            }
            let path = c.model_root.join(name);
            if file_hash(&path)? != reference["row_sha256"] {
                return Err(bad("saved row hash differs"));
            }
            let row = read(&path)?;
            let steps = row["generation"]
                .as_array()
                .ok_or_else(|| bad("saved actual-prefix steps absent"))?;
            if row["id"] != id
                || row["generated_ids"] != json!(generated.generated_ids)
                || row["eos"] != eos
                || steps.len() != generated.steps.len()
            {
                return Err(bad("original actual-prefix generation differs"));
            }
            for (step, old) in generated.steps.iter().zip(steps) {
                compare_step(step, old)?;
            }
            parity =
                json!({"status":"PASS","saved_row_file":name,"saved_row_sha256":file_hash(&path)?});
        }
        if let Some(pair) = l.pair_id {
            if pair.is_empty() {
                return Err(bad("empty pair metadata"));
            }
            pair_groups.entry(pair).or_default().push((
                id.clone(),
                accepted,
                generated.generated_ids.clone(),
            ));
        }
        let traces=generated.steps.iter().map(|s|Ok(json!({"actual_prefix_ids":s.actual_prefix_ids,"post_state_codes":s.post_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
            "copy_token_ids":s.copy_token_ids,"copy_raw_scores_q24":s.copy_raw_scores_q24,"generate_raw_scores_q24":s.generate_raw_scores_q24,
            "actions":s.actions,"bridge":s.bridge.as_ref().map(|b|json!({"selected_ordinal":b.selected_ordinal,"selected_candidate":b.selected_candidate,
                "query_state_codes":b.query_state.iter().map(|v|v.index()).collect::<Vec<_>>(),"source_state_codes":b.source_state.iter().map(|v|v.index()).collect::<Vec<_>>(),
                "action_codes":b.action_codes.iter().map(|v|v.index()).collect::<Vec<_>>()}))}))).collect::<Result<Vec<Value>>>()?;
        let row = json!({"id":id,"generated_ids":generated.generated_ids,"decoded":decoded,"decoded_utf8_valid":valid_text.is_some(),"decoded_bytes_sha256":hash(&decoded_bytes),"eos":eos,"complete":accepted,"entry_correct":entry_correct,
            "stop":format!("{:?}",generated.stop),"executed_steps":generated.executed_steps,"pin": {"lineage":generated.pin.lineage,"commit":generated.pin.commit,"scope":String::from_utf8(generated.pin.scope)?},"steps":traces,"baseline_parity":parity});
        let filename = format!("row-{index:04}.json");
        write_row(c, &filename, &row)?;
        rows.push(json!({"id":l.id,"row_file":filename,"row_bytes":fs::metadata(c.output.join(&filename))?.len(),"row_sha256":file_hash(&c.output.join(&filename))?,"complete":accepted,"entry_correct":entry_correct}));
    }
    let mut pair_complete = 0;
    let mut pairs = Vec::new();
    for (id, arms) in pair_groups {
        if arms.len() != 2 {
            return Err(bad("each evaluator pair must have two cases"));
        }
        let both = arms.iter().all(|v| v.1);
        let distinct = arms[0].2 != arms[1].2;
        pair_complete += usize::from(both && distinct);
        pairs.push(json!({"pair_id":id,"case_ids":arms.iter().map(|v|&v.0).collect::<Vec<_>>(),"both_complete":both,"outputs_distinct":distinct}));
    }
    Ok(
        json!({"schema":"uor-r4.native-bank-generalization/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"executable_sha256":file_hash(&std::env::current_exe()?)?,
        "model_report_sha256":c.expected_model_report_sha256,"model_manifest_sha256":c.expected_model_manifest_sha256,"model_source_commit":r["source_commit"],"component_provenance":r["resume"],
        "checkpoint_receipt_sha256":file_hash(&cp.join("receipt.json"))?,"source_binding":g.source_binding(),"generate_sha256":g.generate_sha256(),"bridge_sha256":g.bridge_sha256(),"exp_sha256":exp_hash,
        "inputs_sha256":c.expected_inputs_sha256,"labels_sha256":c.expected_labels_sha256,"input_seal_sha256":file_hash(&input_root.join("manifest.json"))?,"label_seal_sha256":file_hash(&label_root.join("manifest.json"))?,
        "config_sha256":file_hash(&c.output.join("config.json"))?,"cases":rows.len(),"complete":complete,"entry_correct":entry,"pairs":pairs,"pairs_both_complete_distinct":pair_complete,"rows":rows,
        "baseline_parity":c.baseline_parity,"runtime":"CPU bounded integer/table native generator; no training/CUDA/optimizer","pin_scope":"Source-metadata-derived authored panel pin, lineage0; no store enumeration/lineage authenticity claim",
        "scope":"frozen source-value substitution evaluation in declared grammar; labels after actual-feedback generation; no general chat/held-out whole-program claim","model_admission":admission["resume"]}),
    )
}
fn admit_paths(c: &mut Config) -> Result<()> {
    if c.maximum_report_bytes < 8 << 20 || c.maximum_report_bytes > 4 << 30 {
        return Err(bad("report storage limit must be8MiB..4GiB"));
    }
    for p in [&c.model_root, &c.inputs, &c.labels, &c.output] {
        if !p.is_absolute()
            || p.components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
        {
            return Err(bad("absolute nontraversing paths required"));
        }
    }
    c.model_root = fs::canonicalize(&c.model_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.labels = fs::canonicalize(&c.labels)?;
    c.output = output_support::prospective_output(&c.output)?;
    for p in [&c.model_root, &c.inputs, &c.labels] {
        let seal = seal_for(p)?;
        if c.output.starts_with(p) || p.starts_with(&c.output) || c.output.starts_with(seal) {
            return Err(bad("output/input or sealed ancestry overlap"));
        }
    }
    Ok(())
}
fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() == 3 && args[1] == "verify-report" {
        report_output::verify(Path::new(&args[2]))?;
        return Ok(());
    }
    if args.len() != 2 {
        return Err(bad("usage: native-bank-generalization CONFIG.json"));
    }
    let raw = bytes(Path::new(&args[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&raw)?,
    )?;
    let result = run(&c);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-bank-generalization/1","status":"FAILED","error":e.to_string(),"scope":"execution/instrument failure; no model-quality verdict"})
        }
    };
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn packet() -> Value {
        json!({"id":"fixture","actual_prefix_ids":[],"query_ids":[7],"segments":[
        {"kind":"Context","event":1,"role":1,"token_ids":[4]},
        {"kind":"Source","event":2,"record":9,"commit":3,"scope":"fixture-scope","entity":[4],"relation":2,"view":1,"original_source_ids":[5]}]})
    }
    #[test]
    fn snapshot_preserves_source_occurrences_and_query_with_metadata_only_pin() -> Result<()> {
        let input: Packet = serde_json::from_value(packet())?;
        let bank = snapshot(input)?;
        assert_eq!(bank.query_ids, vec![7]);
        assert_eq!(bank.pin.commit, 3);
        assert_eq!(bank.pin.scope, b"fixture-scope");
        match &bank.segments[1] {
            OwnedBankSegment::Source(s) => {
                assert_eq!(s.original_token_ids, vec![5]);
                assert_eq!(s.record, 9);
            }
            _ => return Err(bad("source absent")),
        }
        let mut malicious = packet();
        malicious["selected_record"] = json!(9);
        assert!(serde_json::from_value::<Packet>(malicious).is_err());
        let mut prefix = packet();
        prefix["actual_prefix_ids"] = json!([2997]);
        assert!(snapshot(serde_json::from_value(prefix)?).is_err());
        Ok(())
    }
    #[test]
    fn different_evaluator_answers_cannot_change_native_snapshot_inputs() -> Result<()> {
        let mut first =
            json!({"id":"fixture","answers":{"intent":"current","accepted":[" first."]}});
        let a: Label = serde_json::from_value(first.clone())?;
        a.answers.validate()?;
        first["answers"]["accepted"] = json!([" second."]);
        let b: Label = serde_json::from_value(first)?;
        b.answers.validate()?;
        assert_ne!(a.answers, b.answers);
        let x = snapshot(serde_json::from_value(packet())?)?;
        let y = snapshot(serde_json::from_value(packet())?)?;
        assert_eq!(x.pin, y.pin);
        assert_eq!(x.query_ids, y.query_ids);
        assert_eq!(x.segments.len(), y.segments.len());
        // Packet deserialization rejects evaluator labels in the runtime input.
        let mut p = packet();
        p["answers"] = json!([" second."]);
        assert!(serde_json::from_value::<Packet>(p).is_err());
        Ok(())
    }
    #[test]
    fn snapshot_rejects_mixed_scope_and_missing_source() -> Result<()> {
        let mut p = packet();
        let mut other = p["segments"][1].clone();
        other["scope"] = json!("other");
        p["segments"]
            .as_array_mut()
            .ok_or_else(|| bad("fixture segments"))?
            .push(other);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        let mut p = packet();
        p["segments"] = json!([{ "kind":"Context","event":1,"role":1,"token_ids":[4]}]);
        assert!(snapshot(serde_json::from_value(p)?).is_err());
        Ok(())
    }
}
