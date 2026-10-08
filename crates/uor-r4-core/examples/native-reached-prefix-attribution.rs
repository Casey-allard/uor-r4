//! Target-free attribution of three authenticated actual reached-prefix frames.
//! Offline saved witnesses only; no fit, intervention or model promotion.
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    native_geometric::learner::{
        geometric_continuation_field::NativeContinuationField,
        geometric_generate::GenerateReadCounts,
        native_bank_generate::{
            BankPin, BoundNativeBytes, NativeBankArtifacts, NativeBankGenerator, OwnedBankSegment,
            OwnedBankSource, PinnedBankSnapshot, SnapshotSourceStatus,
        },
    },
    report_output,
};
use uor_r4_integer::{
    geometric_cue_carrier::CueCarrierMetadata, geometric_prefix_transport::PrefixTransportMetadata,
    geometric_source_actions::SourceActionBinding,
    geometric_source_realizer::NativeArtifactBinding,
    geometric_vocabulary_actions::NativeVocabularyActions,
};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    compensation_root: PathBuf,
    expected_report_sha256: String,
    expected_manifest_sha256: String,
    expected_source_metadata_sha256: String,
    expected_generate_sha256: String,
    expected_continuation_sha256: String,
    inputs: PathBuf,
    inputs_seal_root: PathBuf,
    expected_inputs_sha256: String,
    labels: PathBuf,
    expected_labels_sha256: String,
    output: PathBuf,
    maximum_cache_bytes: u64,
    maximum_report_bytes: u64,
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
fn require(ok: bool, s: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(bad(s))
    }
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
fn text(v: &Value) -> Result<&str> {
    v.as_str().ok_or_else(|| bad("required string absent"))
}
fn write(c: &Config, name: &str, value: &Value, written: &mut u64) -> Result<()> {
    let b = serde_json::to_vec(value)?;
    *written = written
        .checked_add(b.len() as u64)
        .ok_or_else(|| bad("report byte overflow"))?;
    require(
        *written <= c.maximum_report_bytes,
        "report byte cap exhausted",
    )?;
    fs::write(c.output.join(name), b)?;
    Ok(())
}
fn snapshot(p: &Packet) -> Result<PinnedBankSnapshot> {
    require(
        p.actual_prefix_ids.is_empty(),
        "supplied input prefix excluded",
    )?;
    let mut scope = None;
    let mut commit = 0;
    for segment in &p.segments {
        if let Segment::Source {
            scope: s,
            commit: c,
            ..
        } = segment
        {
            require(
                !s.is_empty() && scope.map_or(true, |old: &String| old == s),
                "mixed/empty bank scope",
            )?;
            scope = Some(s);
            commit = commit.max(*c);
        }
    }
    let scope = scope.ok_or_else(|| bad("required Source bank absent"))?;
    Ok(PinnedBankSnapshot {
        pin: BankPin {
            lineage: 0,
            commit,
            scope: scope.as_bytes().to_vec(),
        },
        query_ids: p.query_ids.clone(),
        segments: p
            .segments
            .iter()
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
                    event: *event,
                    record: *record,
                    commit: *commit,
                    scope: scope.as_bytes().to_vec(),
                    entity: entity.clone(),
                    relation: *relation,
                    view: *view,
                    status: SnapshotSourceStatus::Found,
                    original_token_ids: original_source_ids.clone(),
                }),
                Segment::Context {
                    event,
                    role,
                    token_ids,
                } => OwnedBankSegment::Context {
                    event: *event,
                    role: *role,
                    token_ids: token_ids.clone(),
                },
            })
            .collect(),
    })
}
fn admit_paths(c: &mut Config) -> Result<()> {
    for p in [
        &c.compensation_root,
        &c.inputs,
        &c.inputs_seal_root,
        &c.labels,
        &c.output,
    ] {
        require(
            p.is_absolute()
                && !p
                    .components()
                    .any(|x| matches!(x, std::path::Component::ParentDir)),
            "absolute paths without traversal required",
        )?;
    }
    require(
        c.maximum_cache_bytes > 0
            && c.maximum_cache_bytes <= 128 * 1024 * 1024
            && c.maximum_report_bytes >= 1024 * 1024
            && c.maximum_report_bytes <= 128 * 1024 * 1024,
        "diagnostic caps exceed admitted128MiB",
    )?;
    c.compensation_root = fs::canonicalize(&c.compensation_root)?;
    c.inputs = fs::canonicalize(&c.inputs)?;
    c.inputs_seal_root = fs::canonicalize(&c.inputs_seal_root)?;
    c.labels = fs::canonicalize(&c.labels)?;
    c.output = output_support::prospective_output(&c.output)?;
    require(
        c.inputs.starts_with(&c.inputs_seal_root) && c.labels.starts_with(&c.inputs_seal_root),
        "inputs/labels outside declared seal",
    )?;
    for p in [&c.compensation_root, &c.inputs_seal_root] {
        require(
            !c.output.starts_with(p) && !p.starts_with(&c.output),
            "output overlaps sealed authority",
        )?;
    }
    Ok(())
}
fn authenticate_reached_prefix(saved: &Value) -> Result<()> {
    require(
        saved["generated_ids"][0] == 617
            && saved["generation"][1]["actual_prefix_ids"] == json!([617])
            && saved["canonical"][1]["native"]["pool"]["summary"]
                == saved["generation"][1]["pool"]["summary"],
        "saved position1 is not matching actual/canonical reached prefix",
    )
}
fn check_saved(
    step: &uor_r4_core::native_geometric::learner::native_bank_generate::NativeBankGenerateStep,
    saved: &Value,
) -> Result<()> {
    let actual = &saved["generation"][1];
    let canonical = &saved["canonical"][1]["native"];
    authenticate_reached_prefix(saved)?;
    require(
        actual["pool"]["summary"] == serde_json::to_value(&step.actions.summary)?
            && canonical["generate_raw_scores_sha256"]
                == hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?)
            && canonical["post_state_codes"]
                == json!(step
                    .post_state
                    .iter()
                    .map(|x| x.index())
                    .collect::<Vec<_>>())
            && canonical["copy_token_ids"] == json!(step.copy_token_ids),
        "saved native position1 parity differs",
    )?;
    let u = step
        .continuation
        .as_ref()
        .ok_or_else(|| bad("U witness missing"))?;
    require(
        canonical["continuation"] == actual["continuation"]
            && actual["continuation"]["state_codes"]
                == json!(u.state_codes.iter().map(|x| x.index()).collect::<Vec<_>>())
            && actual["continuation"]["delta_scores_q24_sha256"]
                == hash(&serde_json::to_vec(&u.delta_scores_q24)?),
        "actual U state/delta digest parity differs",
    )?;
    Ok(())
}
fn run(c: &Config, written: &mut u64) -> Result<Value> {
    let clock = Instant::now();
    report_output::verify(&c.compensation_root)?;
    report_output::verify(&c.inputs_seal_root)?;
    require(
        file_hash(&c.compensation_root.join("report.json"))? == c.expected_report_sha256
            && file_hash(&c.compensation_root.join("manifest.json"))? == c.expected_manifest_sha256
            && file_hash(&c.inputs)? == c.expected_inputs_sha256
            && file_hash(&c.labels)? == c.expected_labels_sha256,
        "sealed authority pins differ",
    )?;
    let report = read(&c.compensation_root.join("report.json"))?;
    let cp = c.compensation_root.join("checkpoint-0001");
    let receipt = read(&cp.join("receipt.json"))?;
    require(
        report["status"] == "COMPLETED"
            && report["mode"] == "prototype_compensation"
            && report["native_code_proposals"]["winner"] == 0
            && report["final_receipt"] == receipt
            && receipt["step"] == 1,
        "selected compensation endpoint differs",
    )?;
    let binding: NativeArtifactBinding = serde_json::from_value(receipt["parent"].clone())?;
    require(
        binding.metadata_sha256 == c.expected_source_metadata_sha256
            && c.expected_source_metadata_sha256
                == "9f0b272e7852a47bbbad0f549e8157a44ff3212af86905907501ec11f3e7989b"
            && c.expected_generate_sha256
                == "4248245471db609b1fc19482e8f180380b292c5832fc90c81ce69947bc4b7737"
            && c.expected_continuation_sha256
                == "a42cc8d9a9d04bdcddb9f1a513128f66361611556f0e6d768de37da0bc9d1030",
        "fixed successor artifact pins differ",
    )?;
    let gen = bytes(&cp.join("generate.bin"))?;
    let bridge = bytes(&cp.join("read-state-bridge-categorical.bin"))?;
    let exp = bytes(&cp.join("native/consumer/exp-q31.bin"))?;
    let exp_hash = hash(&exp);
    let cue = bytes(&cp.join("cue/cue-q4.bin"))?;
    let prefix = bytes(&cp.join("prefix/prefix-q4.bin"))?;
    let joint = if cp.join("cue/cue-joint-q4.bin").exists() {
        Some(bytes(&cp.join("cue/cue-joint-q4.bin"))?)
    } else {
        None
    };
    let cue_metadata: CueCarrierMetadata =
        serde_json::from_value(read(&cp.join("cue/native-metadata.json"))?)?;
    let prefix_metadata: PrefixTransportMetadata =
        serde_json::from_value(read(&cp.join("prefix/native-metadata.json"))?)?;
    require(
        hash(&gen) == c.expected_generate_sha256
            && receipt["generate_sha256"] == c.expected_generate_sha256,
        "Generate endpoint pin differs",
    )?;
    let mut generator = NativeBankGenerator::load(NativeBankArtifacts {
        native_directory: &cp.join("native"),
        source_binding: &binding,
        generate: BoundNativeBytes {
            bytes: &gen,
            sha256: &c.expected_generate_sha256,
        },
        bridge: Some(BoundNativeBytes {
            bytes: &bridge,
            sha256: text(&receipt["categorical_sha256"])?,
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
    require(
        generator.generate_model().lanes() == 8 && generator.generate_model().vocab_size() == 4096,
        "retained Generate shape differs",
    )?;
    let u = bytes(&cp.join("continuation-field.bin"))?;
    require(
        hash(&u) == c.expected_continuation_sha256
            && receipt["continuation_sha256"] == c.expected_continuation_sha256,
        "U endpoint pin differs",
    )?;
    let field = NativeContinuationField::from_bytes(&u, &binding, generator.generate_model())?;
    for lane in 0..8 {
        for relative in 0..120 {
            require(
                field.coefficient_unary(lane, relative)? == 0,
                "prototype-dependent U is nonzero",
            )?;
        }
    }
    generator = generator.with_continuation_field(BoundNativeBytes {
        bytes: &u,
        sha256: &c.expected_continuation_sha256,
    })?;
    let action_binding = SourceActionBinding::new(&bytes(&cp.join("native/tokenizer.json"))?)?;
    let mut reducer = NativeVocabularyActions::new(action_binding, &exp)?;
    require(
        reducer.legal_token_ids().iter().copied().eq(0u32..4096),
        "full legal vocabulary differs",
    )?;
    let panel: Panel = serde_json::from_slice(&bytes(&c.inputs)?)?;
    require(
        panel.schema == "uor-r4.native-source-bank-probe-input/1" && panel.cases.len() == 512,
        "frozen input panel differs",
    )?;
    // Selection is fixed by the prospective card, not by runtime scores or labels.
    let mut captured = Vec::new();
    for index in [399usize, 5, 13] {
        let packet = &panel.cases[index];
        let row = &report["final_evaluation"]["rows"][index];
        require(row["id"] == packet.id, "report/input case identity differs")?;
        let row_name = text(&row["row_file"])?;
        require(
            row_name == format!("development-0001-row-{index:04}.json"),
            "saved row path differs",
        )?;
        let saved_bytes = bytes(&c.compensation_root.join(row_name))?;
        require(
            hash(&saved_bytes) == text(&row["row_sha256"])?,
            "saved actual row digest differs",
        )?;
        let saved: Value = serde_json::from_slice(&saved_bytes)?;
        require(
            saved["id"] == packet.id
                && saved["generated_ids"] == row["generated_ids"]
                && saved["complete"] == row["complete"]
                && saved["continuation_sha256"] == c.expected_continuation_sha256,
            "saved actual row/report binding differs",
        )?;
        let bank = generator.admit_bank(snapshot(packet)?)?;
        let step = generator.step(&bank, &[617])?;
        check_saved(&step, &saved)?;
        let replay = reducer.reduce_trace(
            &step.generate_raw_scores_q24,
            &step.copy_token_ids,
            &step.copy_raw_scores_q24,
        )?;
        require(
            replay == step.actions,
            "full production reducer replay differs",
        )?;
        let trace = step
            .bank_trace
            .as_ref()
            .ok_or_else(|| bad("bank trace absent"))?;
        let bridge = step
            .bridge
            .as_ref()
            .ok_or_else(|| bad("bridge witness absent"))?;
        let mut summed_copy = vec![0i64; step.copy_token_ids.len()];
        for head in &trace.cue_bank.bank.heads {
            require(
                head.scores_q24.len() == summed_copy.len(),
                "physical Copy head shape differs",
            )?;
            for (sum, value) in summed_copy.iter_mut().zip(&head.scores_q24) {
                *sum = sum
                    .checked_add(*value)
                    .ok_or_else(|| bad("physical Copy sum overflow"))?;
            }
        }
        require(
            summed_copy == step.copy_raw_scores_q24,
            "physical Copy differs from already combined bank heads",
        )?;
        let selected = summed_copy
            .iter()
            .enumerate()
            .max_by(|(ia, a), (ib, b)| a.cmp(b).then_with(|| ib.cmp(ia)))
            .map(|(i, _)| i)
            .ok_or_else(|| bad("physical Copy candidates absent"))?;
        require(
            selected == bridge.selected_ordinal
                && trace.cue_bank.bank.candidates.get(selected) == Some(&bridge.selected_candidate),
            "bridge differs from earliest physical raw maximum",
        )?;
        let continuation = step
            .continuation
            .as_ref()
            .ok_or_else(|| bad("continuation witness absent"))?;
        require(
            continuation
                .state_codes
                .iter()
                .map(|x| x.index())
                .collect::<Vec<_>>()
                == [20, 37, 45, 72, 64, 108, 17, 26]
                && continuation.delta_scores_q24.iter().all(|x| *x == 0),
            "fixed U encoding or numeric0 differs",
        )?;
        let model = generator.generate_model();
        let mut factor_counts = GenerateReadCounts::default();
        let mut factors = Vec::with_capacity(4096);
        for token in 0..4096usize {
            let mut relative = Vec::with_capacity(8);
            let mut unary = Vec::with_capacity(8);
            let mut keys = vec![0u32; 8 + model.energy().edges().len()];
            model.factor_incidence_into(&step.post_state, token, &mut keys, &mut factor_counts)?;
            for lane in 0..8usize {
                let code = model.algebra().compose(
                    model.algebra().inverse(step.post_state[lane].index())?,
                    model.prototypes()[token * 8 + lane],
                )?;
                require(
                    keys[lane] == (lane * 120 + usize::from(code)) as u32,
                    "unary logical/finite algebra incidence differs",
                )?;
                relative.push(code);
                unary.push(model.energy().get_unary(lane as u8, code)?);
            }
            let mut pairs = Vec::new();
            for (edge_index, edge) in model.energy().edges().iter().enumerate() {
                let left = relative[usize::from(edge.left)];
                let right = relative[usize::from(edge.right)];
                require(
                    keys[8 + edge_index]
                        == (960 + edge_index * 14400 + usize::from(left) * 120 + usize::from(right))
                            as u32,
                    "pair logical/finite algebra incidence differs",
                )?;
                pairs.push(model.energy().get_pair(edge_index, left, right)?);
            }
            let bias = model.token_bias(token)?;
            let total = component_sum_q24(&unary, &pairs, bias)?;
            require(
                total == step.generate_raw_scores_q24[token],
                "factor decomposition does not equal complete Generate score",
            )?;
            factors.push(json!({"token_id":token,"prototype_codes":&model.prototypes()[token*8..(token+1)*8],"relative_codes":relative,"logical_factor_keys":keys,"unary_codes":unary,"pair_codes":pairs,"bias_code":bias,"score_shift":20,"total_q24":total}));
        }
        let frame = json!({"input_index":index,"id":packet.id,"actual_prefix_ids":[617],"saved_row_sha256":hash(&saved_bytes),
          "capture_target_free":true,"label_access_before_capture":false,"query_ids":packet.query_ids,"snapshot_pin":{"lineage":0,"commit":packet.segments.iter().filter_map(|s|if let Segment::Source{commit,..}=s{Some(*commit)}else{None}).max(),"scope":packet.segments.iter().find_map(|s|if let Segment::Source{scope,..}=s{Some(scope)}else{None})},"bank_trace":trace,
          "bridge":{"selected_ordinal":bridge.selected_ordinal,"selected_candidate":bridge.selected_candidate,
            "state_roles":{"query_state":"complete bank final prebridge state; not query-only","source_state":"cumulative bank replay state at selected physical occurrence; not isolated source embedding","post_state":"actual Generate scorer input after categorical bridge","continuation_state":"separate query+actualprefix encoding; U numeric0"},"selection_policy":"default earliest PHYSICAL raw Copy maximum before token alias pooling","physical_copy_sum_verified":true,"query_state":bridge.query_state.iter().map(|x|x.index()).collect::<Vec<_>>(),"source_state":bridge.source_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
            "action_codes":bridge.action_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"action_scores_q24":bridge.action_scores_q24,"counts":bridge.counts},
          "post_state":step.post_state.iter().map(|x|x.index()).collect::<Vec<_>>(),
          "continuation":{"query_tokens":continuation.query_tokens,"actual_prefix_tokens":continuation.actual_prefix_tokens,"state_codes":continuation.state_codes.iter().map(|x|x.index()).collect::<Vec<_>>(),"delta_scores_q24":continuation.delta_scores_q24,"encoding_coefficient_reads":continuation.encoding_coefficient_reads,"counts":continuation.counts},
          "generate_q24":step.generate_raw_scores_q24,"generate_sha256":hash(&serde_json::to_vec(&step.generate_raw_scores_q24)?),"copy_ids":step.copy_token_ids,"copy_q24":step.copy_raw_scores_q24,
          "pool":step.actions,"generate_counts":step.generate_counts,"factor_attribution_counts":factor_counts,"declared_pair_edges":model.energy().edges(),"factors":factors,
          "saved_actual_pos1":saved["generation"][1],"saved_canonical_native_pos1":saved["canonical"][1]["native"]});
        captured.push((index, frame));
        let retained = captured.iter().try_fold(0u64, |n, (_, v)| -> Result<u64> {
            Ok(n.checked_add(serde_json::to_vec(v)?.len() as u64)
                .ok_or_else(|| bad("cache size overflow"))?)
        })?;
        require(
            retained <= c.maximum_cache_bytes,
            "target-free saved frame cache exceeds cap",
        )?;
    }
    // Labels are parsed only after all three production captures and all-token attribution.
    let labels = read(&c.labels)?;
    require(
        labels["schema"] == "uor-r4.native-source-bank-labels/1"
            && labels["cases"].as_array().map_or(false, |x| x.len() == 512),
        "label authority shape differs",
    )?;
    let mut summaries = Vec::new();
    for (index, mut frame) in captured {
        let saved = read(
            &c.compensation_root
                .join(format!("development-0001-row-{index:04}.json")),
        )?;
        let target_ids = &saved["canonical_target_ids_labels_only"];
        require(
            labels["cases"][index]["id"] == frame["id"],
            "postcapture label case differs",
        )?;
        frame["posthoc_labels"] = json!({"authority":labels["cases"][index],"saved_canonical_target_ids":target_ids,"attached_after_all_three_captures":true});
        let name = format!("frame-{index:04}.json");
        let digest = hash(&serde_json::to_vec(&frame)?);
        summaries.push(json!({"input_index":index,"id":frame["id"],"file":name,"sha256":digest,"winner":frame["pool"]["summary"]["chosen_token_id"],"post_state":frame["post_state"]}));
        write(c, &name, &frame, written)?;
    }
    Ok(
        json!({"schema":"uor-r4.native-reached-prefix-attribution/1","status":"COMPLETED","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"runtime":"production native bank generator; exactly3 steps at authenticated actualprefix617; full Copy+Generate reducer","source_binding":binding,"generate_sha256":c.expected_generate_sha256,"continuation_sha256":c.expected_continuation_sha256,"compensation_report_sha256":c.expected_report_sha256,"compensation_manifest_sha256":c.expected_manifest_sha256,"inputs_sha256":c.expected_inputs_sha256,"labels_sha256":c.expected_labels_sha256,"categorical_sha256":receipt["categorical_sha256"],"exp_sha256":exp_hash,"frames":summaries,"elapsed_seconds":clock.elapsed().as_secs_f64(),"scope":"exposed reached-prefix attribution; source role unresolved; no counterfactual, fit, gradient, model promotion, transfer or chat claim"}),
    )
}
fn component_sum_q24(unary: &[i8], pair: &[i8], bias: i8) -> Result<i64> {
    let sum = unary.iter().chain(pair).try_fold(i64::from(bias), |n, x| {
        n.checked_add(i64::from(*x))
            .ok_or_else(|| bad("factor sum overflow"))
    })?;
    sum.checked_mul(1i64 << 20)
        .ok_or_else(|| bad("Q24 attribution overflow"))
}
fn main() -> Result<()> {
    let argv = std::env::args().collect::<Vec<_>>();
    if argv.len() == 3 && argv[1] == "verify-report" {
        report_output::verify(Path::new(&argv[2]))?;
        return Ok(());
    }
    require(
        argv.len() == 2,
        "usage: native-reached-prefix-attribution CONFIG.json",
    )?;
    let raw = bytes(Path::new(&argv[1]))?;
    let mut c: Config = serde_json::from_slice(&raw)?;
    admit_paths(&mut c)?;
    report_output::claim(&c.output)?;
    let mut written = 0u64;
    let outcome = (|| -> Result<Value> {
        write(
            &c,
            "config.json",
            &serde_json::from_slice(&raw)?,
            &mut written,
        )?;
        run(&c, &mut written)
    })();
    let mut report = match &outcome {
        Ok(v) => v.clone(),
        Err(e) => {
            json!({"schema":"uor-r4.native-reached-prefix-attribution/1","status":"FAILED","error":e.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"scope":"execution failure; no model-quality verdict"})
        }
    };
    let mut data = serde_json::to_vec(&report)?;
    let exceeded = written
        .checked_add(data.len() as u64)
        .map_or(true, |n| n > c.maximum_report_bytes);
    if exceeded {
        report = json!({"status":"FAILED","error":"final report exceeds byte cap","scope":"execution failure"});
        data = serde_json::to_vec(&report)?;
    }
    fs::write(c.output.join("report.json"), data)?;
    report_output::seal(&c.output)?;
    report_output::verify(&c.output)?;
    if exceeded {
        return Err(bad("final report exceeds byte cap"));
    }
    outcome.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_q4_factor_contributions_preserve_native_q24_sum() -> Result<()> {
        // Opposite extremes, cancellation and nonzero bias catch unsigned nibble/scaling drift.
        assert_eq!(component_sum_q24(&[-7, 7, -1], &[7, -7], 3)?, 2 << 20);
        assert_eq!(component_sum_q24(&[-7; 8], &[-7; 4], -7)?, -91 << 20);
        Ok(())
    }
    #[test]
    fn padded_pair_and_signed_nibbles_preserve_ordered_contributions() -> Result<()> {
        use uor_r4_core::native_geometric::learner::integrated_attention::geometry::{
            EnergyReadCounts, EnergyTables, LanePair,
        };
        let mut energy = EnergyTables::zeroed(2, vec![LanePair { left: 0, right: 1 }])?;
        energy.set_unary(0, 119, -7)?;
        energy.set_unary(1, 1, 7)?;
        energy.set_pair(0, 119, 1, -3)?;
        energy.set_pair(0, 1, 119, 6)?;
        let unary = [energy.get_unary(0, 119)?, energy.get_unary(1, 1)?];
        let pair = [energy.get_pair(0, 119, 1)?];
        assert_eq!(pair, [-3]);
        assert_eq!(energy.get_pair(0, 1, 119)?, 6);
        let mut counts = EnergyReadCounts::default();
        assert_eq!(
            component_sum_q24(&unary, &pair, 2)?,
            (i64::from(energy.score(&[119, 1], &mut counts)?) + 2) << 20
        );
        assert_eq!(energy.get_unary(0, 118)?, 0);
        Ok(())
    }
    #[test]
    fn teacher_only_prefix_cannot_authorize_actual_frame() -> Result<()> {
        // Reject actual-prefix mismatch before accessing any native arrays.
        let saved = json!({"generated_ids":[617],"generation":[{}, {"actual_prefix_ids":[2997]}],"canonical":[{}, {"native":{}}]});
        assert!(authenticate_reached_prefix(&saved).is_err());
        let missing_actual =
            json!({"generated_ids":[617],"canonical":[{}, {"native":{"actual_prefix_ids":[617]}}]});
        assert!(authenticate_reached_prefix(&missing_actual).is_err());
        Ok(())
    }
}
