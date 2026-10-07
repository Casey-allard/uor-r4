//! Eight-row retained-artifact comparison for the production native bank API.
//! No training or label file is loaded. Saved canonical prefixes are evaluator
//! conditions; free generation starts empty and feeds back actual output only.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};
use uor_r4_core::{
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
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    baseline: PathBuf,
    expected_baseline_report_sha256: String,
    expected_baseline_admission_sha256: String,
    inputs: PathBuf,
    output: PathBuf,
    pin_lineage: u64,
    pin_commit: u64,
    pin_scope: String,
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
fn snapshot(p: Packet, c: &Config) -> Result<PinnedBankSnapshot> {
    if !p.actual_prefix_ids.is_empty() {
        return Err(bad("panel free generation must start with empty prefix"));
    }
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: c.pin_lineage,
            commit: c.pin_commit,
            scope: c.pin_scope.as_bytes().to_vec(),
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
    report_output::verify(&c.baseline)?;
    if file_hash(&c.baseline.join("report.json"))? != c.expected_baseline_report_sha256
        || file_hash(&c.baseline.join("admission.json"))? != c.expected_baseline_admission_sha256
    {
        return Err(bad("sealed baseline exact identity differs"));
    }
    let admission = read(&c.baseline.join("admission.json"))?;
    let report = read(&c.baseline.join("report.json"))?;
    if report["status"] != "COMPLETED"
        || report["mode"] != "admission"
        || report["updates"] != 0
        || admission["source_commit"] != "f63ed81fd4f74d2501ba3d9888420662492416df"
    {
        return Err(bad(
            "baseline is not completed retained f63 zero-update admission",
        ));
    }
    let input_hash = file_hash(&c.inputs)?;
    if admission["training_input_sha256"] != input_hash {
        return Err(bad("panel differs from authenticated baseline"));
    }
    if let Some(parent) = c.inputs.parent() {
        report_output::verify(parent)?;
    } else {
        return Err(bad("panel has no sealed parent"));
    }
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    if panel.schema != "uor-r4.native-source-bank-probe-input/1" || panel.cases.len() != 512 {
        return Err(bad("expected complete retained 512-row input panel"));
    }
    let cp = c.baseline.join("checkpoint-0000");
    let receipt = read(&cp.join("receipt.json"))?;
    if receipt["step"] != 0 || receipt["native_independently_reloaded"] != true {
        return Err(bad(
            "baseline initialized native checkpoint receipt differs",
        ));
    }
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let joint_path = cp.join("cue/cue-joint-q4.bin");
    let joint = if joint_path.exists() {
        Some(bytes(&joint_path)?)
    } else {
        None
    };
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let native_dir = cp.join("native");
    let exp_hash = hash(&exp);
    let mut generator = NativeBankGenerator::load(NativeBankArtifacts {
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
    let development = read(&c.baseline.join("development-0000.json"))?;
    let saved_rows = development["rows"]
        .as_array()
        .ok_or_else(|| bad("baseline row index absent"))?;
    if saved_rows.len() != 512 {
        return Err(bad("baseline row coverage incomplete"));
    }
    let mut rows = Vec::new();
    for (index, packet) in panel.cases.into_iter().take(8).enumerate() {
        let rowref = &saved_rows[index];
        if rowref["id"] != packet.id {
            return Err(bad("input and baseline ordered row identity differs"));
        }
        let name = expected_string(&rowref["row_file"])?;
        if Path::new(name).components().count() != 1 {
            return Err(bad("baseline row reference is not a leaf"));
        }
        let saved_path = c.baseline.join(name);
        let saved_hash = file_hash(&saved_path)?;
        if rowref["row_sha256"] != saved_hash {
            return Err(bad("baseline row hash differs"));
        }
        let saved = read(&saved_path)?;
        if saved["id"] != packet.id {
            return Err(bad("saved row identity differs"));
        }
        let id = packet.id.clone();
        let bank = generator.admit_bank(snapshot(packet, c)?)?;
        let canonical = saved["canonical"]
            .as_array()
            .ok_or_else(|| bad("canonical step evidence absent"))?;
        let mut canonical_steps = Vec::new();
        for (position, evidence) in canonical.iter().enumerate() {
            // Prefix is a saved evaluator condition, never a selected source or
            // runtime target. No target_label_only field is read.
            let actual: Vec<u32> =
                serde_json::from_value(evidence["native"]["actual_prefix_ids"].clone())?;
            let step = generator.step(&bank, &actual)?;
            compare_step(&step, &evidence["native"])?;
            canonical_steps.push(json!({"position":position,"actual_prefix_ids":actual,"chosen_token_id":step.actions.summary.chosen_token_id,
                "generate_raw_scores_sha256":hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?)}));
        }
        let allowance = 32usize.min(128usize.saturating_sub(bank.base_tokens()));
        if saved["generation_allowance"] != allowance {
            return Err(bad("saved own-prefix context allowance differs"));
        }
        let generation = generator.generate(
            &bank,
            GenerationLimits {
                maximum_tokens: 32,
                maximum_retained_steps: 32,
            },
        )?;
        if saved["generated_ids"] != json!(generation.generated_ids)
            || saved["eos"] != json!(generation.stop == GenerationStop::Eos)
        {
            return Err(bad("actual own-prefix generation parity differs"));
        }
        let own = saved["generation"]
            .as_array()
            .ok_or_else(|| bad("saved own-prefix trace absent"))?;
        if own.len() != generation.steps.len() {
            return Err(bad("own-prefix step coverage differs"));
        }
        for (step, expected) in generation.steps.iter().zip(own) {
            compare_step(step, expected)?;
        }
        let cutoff = match generation.stop {
            GenerationStop::Eos => Value::Null,
            GenerationStop::ContextLimit => json!("context-cap"),
            GenerationStop::OutputLimit => json!("generation-cap"),
        };
        if saved["cutoff"] != cutoff {
            return Err(bad("own-prefix cutoff reason differs"));
        }
        rows.push(json!({"index":index,"id":id,"saved_row_file":name,"saved_row_sha256":saved_hash,
            "canonical_positions":canonical.len(),"canonical_steps":canonical_steps,"own_prefix_positions":generation.executed_steps,
            "generated_ids":generation.generated_ids,"eos":generation.stop==GenerationStop::Eos,"cutoff":cutoff,"parity":"PASS"}));
    }
    Ok(
        json!({"schema":"uor-r4.native-bank-generate-parity/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),
        "executable_sha256":file_hash(&std::env::current_exe()?)?,"baseline":c.baseline,"baseline_producer":admission["source_commit"],
        "baseline_report_sha256":c.expected_baseline_report_sha256,"baseline_admission_sha256":c.expected_baseline_admission_sha256,
        "inputs_sha256":input_hash,"checkpoint_receipt_sha256":file_hash(&cp.join("receipt.json"))?,
        "source_binding":generator.source_binding(),"generate_sha256":generator.generate_sha256(),"bridge_sha256":generator.bridge_sha256(),
        "row_indices":[0,1,2,3,4,5,6,7],"available_baseline_rows":512,"executed_rows":rows.len(),"not_executed_rows":504,"rows":rows,
        "pin":{"lineage":c.pin_lineage,"commit":c.pin_commit,"scope":c.pin_scope,"authority":"externally supplied panel snapshot; no store iterator/completeness claim"},
        "scope":"CPU integer artifact-only API; saved canonical-prefix replay plus independently actual own-prefix output; eight construction rows; no training, chat or generalization claim",
        "comparison":"complete Copy score/order/occurrence and components, poststate, all Generate scores via saved SHA, reducer summary and bridge; saved traces lack full action/token masses, so their per-atom parity is NOT_AVAILABLE"}),
    )
}
fn admit_paths(c: &mut Config) -> Result<()> {
    if [&c.baseline, &c.inputs, &c.output].iter().any(|p| {
        !p.is_absolute()
            || p.components()
                .any(|v| matches!(v, std::path::Component::ParentDir))
    }) || c.pin_scope.is_empty()
    {
        return Err(bad(
            "paths must be absolute without parent traversal; pin scope must be nonempty",
        ));
    }
    c.baseline = fs::canonicalize(&c.baseline)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    // Resolve symlinked existing parents and reject ANY sealed ancestor before
    // claim creates a missing parent or writes its attempt sentinel.
    c.output = output_support::prospective_output(&c.output)?;
    let input_parent = c
        .inputs
        .parent()
        .ok_or_else(|| bad("input parent absent"))?;
    if c.output.starts_with(&c.baseline)
        || c.baseline.starts_with(&c.output)
        || c.output.starts_with(input_parent)
        || input_parent.starts_with(&c.output)
    {
        return Err(bad("output overlaps canonical baseline/input ancestry"));
    }
    Ok(())
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        println!("{}", json!({"status":"VERIFIED","report_root":argv[2]}));
        return Ok(());
    }
    if argv.len() != 2 {
        return Err(bad("usage: native-bank-generate-parity CONFIG.json"));
    }
    let config_bytes = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&config_bytes)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    let result = run(&c);
    let report = match &result {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-bank-generate-parity/1","status":"FAILED","error":e.to_string(),
            "source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"execution/parity failure; no model quality verdict"})
        }
    };
    write(
        &c.output.join("config.json"),
        &serde_json::from_slice(&config_bytes)?,
    )?;
    write(&c.output.join("report.json"), &report)?;
    report_output::seal(&c.output)?;
    result.map(|_| ())
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    #[test]
    fn parity_paths_protect_sealed_input_ancestors_and_preserve_missing_parents() -> Result<()> {
        let root = std::env::temp_dir().join(format!(
            "native-bank-parity-paths-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
        ));
        fs::create_dir(&root)?;
        let result = (|| -> Result<()> {
            let baseline = root.join("baseline");
            fs::create_dir(&baseline)?;
            let inputs_root = root.join("inputs");
            fs::create_dir(&inputs_root)?;
            let inputs = inputs_root.join("inputs.json");
            fs::write(&inputs, b"{}")?;
            fs::write(inputs_root.join(report_output::MANIFEST_FILE), b"{}")?;
            let config = |output| Config {
                baseline: baseline.clone(),
                expected_baseline_report_sha256: String::new(),
                expected_baseline_admission_sha256: String::new(),
                inputs: inputs.clone(),
                output,
                pin_lineage: 0,
                pin_commit: 2,
                pin_scope: "compiler-probe".into(),
            };
            let mut nested = config(inputs_root.join("missing/attempt"));
            assert!(admit_paths(&mut nested).is_err());
            assert!(!inputs_root.join("missing").exists());
            #[cfg(unix)]
            {
                let alias = root.join("alias");
                std::os::unix::fs::symlink(&inputs_root, &alias)?;
                let mut linked = config(alias.join("missing/attempt"));
                assert!(admit_paths(&mut linked).is_err());
                assert!(!inputs_root.join("missing").exists());
            }
            let mut ancestor = config(root.clone());
            assert!(admit_paths(&mut ancestor).is_err());
            let mut safe = config(root.join("outputs/missing/attempt"));
            admit_paths(&mut safe)?;
            assert_eq!(
                safe.output,
                fs::canonicalize(&root)?.join("outputs/missing/attempt")
            );
            assert!(!root.join("outputs").exists());
            Ok(())
        })();
        fs::remove_dir_all(&root)?;
        result
    }
}
