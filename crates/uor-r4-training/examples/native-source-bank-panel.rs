//! Rust-only source-bank input preparation. Labels are authored after all inputs.
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::{
    geometric_source_actions::SourceActionBinding, geometric_source_realizer::NativeArtifactBinding,
};
use uor_r4_tokenizer::{dialogue::SCHEMA_V2, ByteBpeTokenizer};
use uor_r4_training::{geometric_source_emission_view::SourceEmissionCompiler, sha256_file};
#[path = "../../uor-r4-integer/examples/support/source_probe.rs"]
mod output_support;
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
const LIMIT: usize = 64 * 1024 * 1024;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    out: PathBuf,
    spec: PathBuf,
    tokenizer: PathBuf,
    trusted_binding: PathBuf,
    compiled_panel: PathBuf,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Spec {
    schema: String,
    status: String,
    scope: String,
    bound_compiled64: String,
    query: String,
    actual_prefix: Vec<u32>,
    maximum_generated_tokens: usize,
    input_preparation: String,
    label_separation: String,
    event_semantics: String,
    no_admission_from_labels: bool,
    rows: Vec<TextRow>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TextRow {
    id: String,
    segments: Vec<TextSegment>,
    expected_literal: String,
    #[serde(default)]
    single_record_exact_replay: bool,
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
enum TextSegment {
    Source {
        literal: String,
        entity: Vec<u32>,
        relation: u32,
        scope: String,
        record: u64,
        commit: u64,
        view: u32,
        event: u64,
    },
    Context {
        text: String,
        role: String,
        event: u64,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Packet {
    id: String,
    segments: Vec<Segment>,
    query_ids: Vec<u32>,
    actual_prefix_ids: Vec<u32>,
}
#[derive(Serialize, Deserialize)]
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
fn invalid(s: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, s.into())
}
fn read<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path)?;
    if bytes.len() > LIMIT {
        return Err(invalid("input file cap exceeded").into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}
fn write(root: &Path, name: &str, value: &Value) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    let current = fs::read_dir(root)?.try_fold(0usize, |sum, e| -> io::Result<usize> {
        Ok(sum + e?.metadata()?.len() as usize)
    })?;
    if current.saturating_add(bytes.len()) > LIMIT - 128 * 1024 {
        return Err(invalid("preparation report cap reached").into());
    }
    fs::write(root.join(name), bytes)?;
    Ok(())
}
fn horizon(projected: usize, query: usize, prefix: usize, generated: usize) -> Result<usize> {
    let n = projected
        .checked_add(query)
        .and_then(|n| n.checked_add(prefix))
        .and_then(|n| n.checked_add(generated))
        .ok_or_else(|| invalid("horizon overflow"))?;
    if generated != 32 || n > 128 {
        return Err(invalid("128 sequence horizon including32 generation exceeded").into());
    }
    Ok(n)
}
fn encode(tok: &ByteBpeTokenizer, text: &str, vocab: usize) -> Result<Vec<u32>> {
    let ids = tok.encode(text);
    if ids.is_empty() || ids.iter().any(|id| *id as usize >= vocab) || tok.decode(&ids) != text {
        return Err(invalid("tokenizer text roundtrip/admission failed").into());
    }
    Ok(ids)
}
fn args() -> Result<Args> {
    let mut argv = std::env::args().skip(1);
    let config = argv
        .next()
        .ok_or_else(|| invalid("one JSON config required"))?;
    if argv.next().is_some() {
        return Err(invalid("one JSON config only").into());
    }
    let a: Args = read(Path::new(&config))?;
    let output = output_support::prospective_output(&a.out)?;
    for p in [&a.spec, &a.tokenizer, &a.trusted_binding, &a.compiled_panel] {
        let p = fs::canonicalize(p)?;
        if output.starts_with(&p) {
            return Err(invalid("output beneath input").into());
        }
    }
    Ok(a)
}
fn prepare(a: &Args, start: Instant) -> Result<Value> {
    let input_paths = [&a.spec, &a.tokenizer, &a.trusted_binding, &a.compiled_panel];
    let input_hashes = input_paths
        .iter()
        .map(|p| Ok((p.to_string_lossy().into_owned(), sha256_file(p)?)))
        .collect::<Result<std::collections::BTreeMap<String, String>>>()?;
    let sealed = a
        .compiled_panel
        .ancestors()
        .skip(1)
        .find(|p| p.join(report_output::MANIFEST_FILE).is_file())
        .ok_or_else(|| invalid("compiled64 sealed parent absent"))?;
    report_output::verify(sealed)?;
    let spec: Spec = read(&a.spec)?;
    if spec.schema != "uor-r4.prospective-bank-development-slice/1"
        || spec.status != "TEXT_SPEC_FROZEN_BEFORE_PREPARATION_OR_PREDICTIONS"
        || !spec.no_admission_from_labels
        || spec.rows.len() != 6
        || !spec.actual_prefix.is_empty()
        || spec.maximum_generated_tokens != 32
    {
        return Err(invalid("prospective spec identity/bounds differ").into());
    }
    let parent: NativeArtifactBinding = read(&a.trusted_binding)?;
    let bytes = fs::read(&a.tokenizer)?;
    if bytes.len() > LIMIT || sha256_file(&a.tokenizer)? != parent.identity.tokenizer_sha256 {
        return Err(invalid("trusted tokenizer bytes differ").into());
    }
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("tokenizer JSON invalid"))?;
    let binding = SourceActionBinding::new(&bytes)?;
    if binding.protocol().schema != SCHEMA_V2 {
        return Err(invalid("literal-role dialogue2 required").into());
    }
    let compiler = SourceEmissionCompiler::new(&bytes)?;
    let query = encode(&tok, &spec.query, binding.vocab_size())?;
    let compiled: Value = read(&a.compiled_panel)?;
    let training = compiled["training64"]
        .as_array()
        .filter(|rows| rows.len() == 64)
        .ok_or_else(|| invalid("bound compiled64 rows absent"))?;
    let mut packets = Vec::new();
    let mut views = Vec::new();
    let mut ids = BTreeSet::new();
    let mut selected_parent = None;
    // This pass uses no expected_literal, answer set, model or support mass.
    for row in &spec.rows {
        if start.elapsed().as_secs() >= 297 {
            return Err(invalid("remaining preparation wall cap reached").into());
        }
        if row.id.is_empty() || !ids.insert(&row.id) || row.segments.is_empty() {
            return Err(invalid("row identity/segments invalid").into());
        }
        let mut segments = Vec::new();
        let mut projected = 0usize;
        let mut row_views = Vec::new();
        for (index, segment) in row.segments.iter().enumerate() {
            match segment {
                TextSegment::Source {
                    literal,
                    entity,
                    relation,
                    scope,
                    record,
                    commit,
                    view,
                    event,
                } => {
                    let original = encode(&tok, literal, binding.vocab_size())?;
                    if entity.is_empty()
                        || entity.iter().any(|id| *id as usize >= binding.vocab_size())
                        || scope.is_empty()
                    {
                        return Err(invalid("source frame invalid").into());
                    }
                    let emitted = compiler.compile(&original)?;
                    projected = projected
                        .checked_add(emitted.emitted_token_ids().len())
                        .ok_or_else(|| invalid("projected length overflow"))?;
                    row_views
                        .push(json!({"segment_index":index,"event":event,"source_view":emitted}));
                    segments.push(Segment::Source {
                        event: *event,
                        record: *record,
                        commit: *commit,
                        scope: scope.clone(),
                        entity: entity.clone(),
                        relation: *relation,
                        view: *view,
                        original_source_ids: original,
                    });
                }
                TextSegment::Context { text, role, event } => {
                    if role != "user" {
                        return Err(invalid("only declared opaque user role1 admitted").into());
                    }
                    let tokens = encode(&tok, text, binding.vocab_size())?;
                    projected = projected
                        .checked_add(tokens.len())
                        .ok_or_else(|| invalid("projected length overflow"))?;
                    segments.push(Segment::Context {
                        event: *event,
                        role: 1,
                        token_ids: tokens,
                    });
                }
            }
        }
        let admitted = horizon(
            projected,
            query.len(),
            spec.actual_prefix.len(),
            spec.maximum_generated_tokens,
        )?;
        let packet = Packet {
            id: row.id.clone(),
            segments,
            query_ids: query.clone(),
            actual_prefix_ids: spec.actual_prefix.clone(),
        };
        if row.single_record_exact_replay {
            if packet.segments.len() != 1 || selected_parent.is_some() {
                return Err(invalid("one declared single parent row required").into());
            }
            let Segment::Source {
                record,
                commit,
                scope,
                entity,
                relation,
                view,
                original_source_ids,
                ..
            } = &packet.segments[0]
            else {
                return Err(invalid("single parent must be Source").into());
            };
            if scope != "m-world-v2" || *view != 0 {
                return Err(invalid("single parent frame differs").into());
            }
            let matches = training
                .iter()
                .filter(|e| {
                    e["original_source_ids"] == json!(original_source_ids)
                        && e["query_ids"] == json!(packet.query_ids)
                        && e["record"] == *record
                        && e["commit"] == *commit
                        && e["entity"] == json!(entity)
                        && e["relation"] == *relation
                        && e["source_view"] == row_views[0]["source_view"]
                })
                .collect::<Vec<_>>();
            let matched = matches.first().ok_or_else(|| {
                invalid("single source/query/view does not match retained compiled64")
            })?;
            selected_parent = Some(
                json!({"prepared_id":row.id,"parent_id":matched["id"],"matching_row_count":matches.len(),"source_query_frame_view_exact":true}),
            );
        }
        views.push(json!({"id":row.id,"projected_context_tokens":projected,"query_tokens":query.len(),"actual_prefix_tokens":spec.actual_prefix.len(),"maximum_generated_tokens":32,"admitted_horizon":admitted,"source_views":row_views}));
        packets.push(packet);
    }
    let parent_row = selected_parent.ok_or_else(|| invalid("single exact parent row absent"))?;
    // All inputs and their projection were completed before label authoring.
    write(
        &a.out,
        "inputs.json",
        &json!({"schema":"uor-r4.native-source-bank-probe-input/1","cases":packets}),
    )?;
    write(
        &a.out,
        "source-views.json",
        &json!({"schema":"uor-r4.native-source-bank-projection/1","cases":views}),
    )?;
    let mut labels = Vec::new();
    for row in &spec.rows {
        let answers = FrozenAnswers {
            intent: RecordedValueIntent::Current,
            accepted: vec![format!("{}.", row.expected_literal)],
        };
        answers.validate()?;
        labels.push(json!({"id":row.id,"answers":answers}));
    }
    write(
        &a.out,
        "labels.json",
        &json!({"schema":"uor-r4.native-source-bank-labels/1","cases":labels,"protocol":SCHEMA_V2,"membership_only":true}),
    )?;
    for (path, expected) in &input_hashes {
        if sha256_file(Path::new(path))? != *expected {
            return Err(invalid("preparation input changed").into());
        }
    }
    Ok(
        json!({"schema":"uor-r4.native-source-bank-panel/1","input_files_sha256":input_hashes,"input_files_unchanged":true,"status":"completed","cases":6,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"trusted_binding":parent,"trusted_binding_sha256":sha256_file(&a.trusted_binding)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"compiled64_sha256":sha256_file(&a.compiled_panel)?,"text_spec_sha256":sha256_file(&a.spec)?,"inputs_sha256":sha256_file(&a.out.join("inputs.json"))?,"labels_sha256":sha256_file(&a.out.join("labels.json"))?,"source_views_sha256":sha256_file(&a.out.join("source-views.json"))?,"single_parent":parent_row,"segment_order_preserved":true,"events_are_opaque_not_sort_keys":true,"context_role_user":1,"context_role_does_not_synthesize_tokens":true,"expected_literals_excluded_from_inputs_projection_and_admission":true,"labels_authored_after_all_inputs":true,"model_calls":0,"target_support_filtering":false,"predictions":"NOT_RUN","maximum_generated_tokens":32,"maximum_sequence_horizon":128,"elapsed_seconds":start.elapsed().as_secs_f64(),"prospective_scope":spec.scope,"bound_compiled64_provenance":spec.bound_compiled64,"input_preparation":spec.input_preparation,"label_separation":spec.label_separation,"event_semantics":spec.event_semantics}),
    )
}
fn main() -> Result<()> {
    let a = args()?;
    report_output::claim(&a.out)?;
    let start = Instant::now();
    let result = prepare(&a, start);
    match &result {
        Ok(report) => write(&a.out, "report.json", report)?,
        Err(e) => write(
            &a.out,
            "failure.json",
            &json!({"error":e.to_string(),"elapsed_seconds":start.elapsed().as_secs_f64(),"model_calls":0}),
        )?,
    };
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result.map(|_| ())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizon_reserves_generation_and_rejects_overflow() -> Result<()> {
        assert_eq!(horizon(87, 9, 0, 32)?, 128);
        assert!(horizon(88, 9, 0, 32).is_err());
        assert!(horizon(87, 9, 1, 32).is_err());
        assert!(horizon(usize::MAX, 1, 0, 32).is_err());
        Ok(())
    }
    #[test]
    fn inference_packet_rejects_expected_literal_and_keeps_order() -> Result<()> {
        let packet = json!({"id":"x","segments":[{"kind":"Context","event":20,"role":1,"token_ids":[1]},{"kind":"Context","event":10,"role":1,"token_ids":[2]}],"query_ids":[3],"actual_prefix_ids":[]});
        let parsed: Packet = serde_json::from_value(packet.clone())?;
        let encoded = serde_json::to_value(parsed)?;
        assert_eq!(packet, encoded);
        let mut contaminated = packet;
        contaminated["expected_literal"] = json!("singer");
        assert!(serde_json::from_value::<Packet>(contaminated).is_err());
        Ok(())
    }
}
