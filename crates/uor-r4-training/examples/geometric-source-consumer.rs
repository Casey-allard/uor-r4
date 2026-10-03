//! Connected actual-BPE construction check, not training or native-chat qualification.
//! Source frames come only from a retained predicted StoreRead::Found. Answers
//! are teacher-forced labels; evaluator relation/selected fields never enter forward.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::report_output;
use uor_r4_integer::geometric_occurrence_read::{
    FrameMetadata, FrameStatus, SelectedRecordFrame, SourceIdentity,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_occurrence_consumer::{
    mix_scores, uniform_scores, ConsumerIdentity, ConsumerWeights, NativeConsumerArtifact,
};
use uor_r4_training::{
    sha256_file,
    stack_checkpoint::{load_checkpoint, sealed_manifest_sha256},
    stack_grounded_session::{CompiledAction, MemoryEffect, TurnOutcome},
    stack_store::StoreRead,
    Result, TrainingError,
};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Args {
    out: PathBuf,
    retained_report: PathBuf,
    tokenizer: PathBuf,
    checkpoint: PathBuf,
    maximum_cases: usize,
    maximum_batches: usize,
    maximum_seconds: u64,
    seed: u64,
}
#[derive(Deserialize)]
struct RetainedReport {
    schema: String,
    tokenizer_sha256: String,
    checkpoint_manifest_sha256: String,
    rows: Vec<RetainedRow>,
}
#[derive(Deserialize)]
struct RetainedRow {
    case: CaseLabels,
    read: bool,
    outcomes: Vec<TurnOutcome>,
}
#[derive(Deserialize)]
struct CaseLabels {
    id: String,
    answers: AnswerLabels,
}
#[derive(Deserialize)]
struct AnswerLabels {
    accepted: Vec<String>,
}
struct PreparedCase {
    id: String,
    query: Vec<u32>,
    parent_input: Vec<u32>,
    tokens: Vec<u32>,
    record: u64,
    commit: u64,
    relation: u32,
    entity: Vec<u32>,
    response: Vec<u32>,
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn write(path: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn score_hash(scores: &[f32]) -> String {
    let mut h = Sha256::new();
    for x in scores {
        h.update(x.to_bits().to_le_bytes());
    }
    hex::encode(h.finalize())
}

fn prepare(
    d: RetainedReport,
    tok: &ByteBpeTokenizer,
    eos: u32,
    a: &Args,
) -> Result<Vec<PreparedCase>> {
    let mut out = Vec::new();
    let mut steps = 0usize;
    for row in d.rows.into_iter().filter(|r| r.read).take(a.maximum_cases) {
        let last = row
            .outcomes
            .last()
            .ok_or_else(|| invalid("empty retained outcome"))?;
        if !last.controls.read || !last.controls.write {
            return Err(invalid("unexpected intervention in read-enabled source"));
        }
        let found = match &last.memory {
            MemoryEffect::Read {
                read: StoreRead::Found(x),
            } => x,
            _ => return Err(invalid("last predicted source is not Found")),
        };
        if found.conflict {
            return Err(invalid("conflicted selected source is not admitted"));
        }
        let relation = match last.action {
            CompiledAction::QueryCurrent { relation } => relation,
            _ => return Err(invalid("construction requires predicted current query")),
        };
        let query = tok.encode(&last.source);
        let answer = row
            .case
            .answers
            .accepted
            .first()
            .ok_or_else(|| invalid("missing complete accepted training label"))?;
        let mut response = tok.encode(answer);
        response.push(eos);
        if response.is_empty() || response.len() > 64 || query.is_empty() || found.tokens.is_empty()
        {
            return Err(invalid("empty or oversized admitted sequence"));
        }
        // The whole selected source, query and possible response must fit. No truncation.
        if query.len() + found.tokens.len() + response.len() > 128
            || last.emitter_input_ids.len() + response.len() > 384
        {
            return Err(invalid(
                "whole case exceeds consumer128/parent384 admission",
            ));
        }
        steps = steps
            .checked_add(response.len())
            .ok_or_else(|| invalid("batch count overflow"))?;
        if steps > a.maximum_batches {
            return Err(invalid(
                "whole response exceeds declared construction batch budget",
            ));
        }
        for ids in [&query, &response, &found.tokens, &last.emitter_input_ids] {
            if ids.iter().any(|x| *x as usize >= 4096) {
                return Err(invalid("token outside actual BPE vocabulary"));
            }
        }
        out.push(PreparedCase {
            id: row.case.id,
            query,
            parent_input: last.emitter_input_ids.clone(),
            tokens: found.tokens.clone(),
            record: found.record,
            commit: found.commit,
            relation,
            entity: tok.encode("user"),
            response,
        });
    }
    if out.is_empty() {
        return Err(invalid("no read-enabled construction cases"));
    }
    Ok(out)
}
fn load_inputs(
    a: &Args,
) -> Result<(
    uor_r4_training::stack_checkpoint::StackCheckpoint,
    Vec<PreparedCase>,
)> {
    let containing = a
        .retained_report
        .parent()
        .ok_or_else(|| invalid("report has no envelope"))?;
    report_output::verify(containing)?;
    let d: RetainedReport = serde_json::from_slice(&fs::read(&a.retained_report)?)?;
    if d.schema != "uor-r4.source-binding-diagnostic/1" {
        return Err(invalid("unexpected retained schema"));
    }
    if sha256_file(&a.tokenizer)? != d.tokenizer_sha256
        || sealed_manifest_sha256(&a.checkpoint).map_err(|e| invalid(e.to_string()))?
            != d.checkpoint_manifest_sha256
    {
        return Err(invalid("retained tokenizer/checkpoint identity mismatch"));
    }
    let bytes = fs::read(&a.tokenizer)?;
    let tok = ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes)
        .ok_or_else(|| invalid("unreadable tokenizer"))?;
    let checkpoint =
        load_checkpoint(&a.checkpoint, &Device::Cpu).map_err(|e| invalid(e.to_string()))?;
    checkpoint
        .identity()
        .check_tokenizer(&bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if tok.vocab_size() != 4096
        || checkpoint.model.config.vocab_size != 4096
        || checkpoint.model.config.context != 384
    {
        return Err(invalid(
            "only bound V4096 context384 development parent admitted",
        ));
    }
    let cases = prepare(d, &tok, checkpoint.identity().protocol.eos_id, a)?;
    Ok((checkpoint, cases))
}
fn main() -> Result<()> {
    let mut argv = std::env::args().skip(1);
    let args_file = argv
        .next()
        .ok_or_else(|| invalid("usage: geometric-source-consumer ARGS.json"))?;
    if argv.next().is_some() {
        return Err(invalid("one JSON argument expected"));
    }
    let a: Args = serde_json::from_slice(&fs::read(args_file)?)?;
    if !(1..=2).contains(&a.maximum_cases)
        || !(1..=8).contains(&a.maximum_batches)
        || !(1..=900).contains(&a.maximum_seconds)
    {
        return Err(invalid(
            "construction limits: cases1..2 batches1..8 seconds1..900",
        ));
    }
    report_output::claim(&a.out)?;
    write(&a.out.join("args.json"), &a)?;
    let result = run(&a);
    if let Err(e) = &result {
        write(
            &a.out.join("error.json"),
            &json!({"error":e.to_string(),"optimizer_updates":0}),
        )?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
fn run(a: &Args) -> Result<()> {
    let started = Instant::now();
    let source_commit = option_env!("UOR_BUILD_SOURCE_COMMIT")
        .filter(|s| s.len() == 40 && s.bytes().all(|b| b.is_ascii_hexdigit()))
        .ok_or_else(|| {
            invalid("build must bind a full UOR_BUILD_SOURCE_COMMIT before construction")
        })?;
    let (checkpoint, cases) = load_inputs(a)?;
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    let identity = ConsumerIdentity::new(&tokenizer_bytes, &a.checkpoint)?;
    let weights = ConsumerWeights::initialize(
        &a.out.join("initializer"),
        &tokenizer_bytes,
        4096,
        2,
        4,
        a.seed,
    )?;
    weights.save_source(&a.out.join("consumer-source"))?;
    let native = weights.compile(identity.clone())?;
    native.save(&a.out.join("consumer-native"))?;
    let reloaded_source = ConsumerWeights::load_source(&a.out.join("consumer-source"))?;
    let loaded =
        NativeConsumerArtifact::load(&a.out.join("consumer-native"), &reloaded_source, &identity)?;
    // Preserve a separate deliberately changed payload. This checks artifact
    // rejection without re-executing any model or backward construction batch.
    let corrupt_root = a.out.join("corrupt-native-negative");
    native.save(&corrupt_root)?;
    let corrupt_file = corrupt_root.join("context-q4.bin");
    let mut corrupt_bytes = fs::read(&corrupt_file)?;
    let first = corrupt_bytes
        .first_mut()
        .ok_or_else(|| invalid("empty context payload"))?;
    *first ^= 1;
    fs::write(&corrupt_file, &corrupt_bytes)?;
    let corrupt_error =
        match NativeConsumerArtifact::load(&corrupt_root, &reloaded_source, &identity) {
            Err(e) => e.to_string(),
            Ok(_) => return Err(invalid("changed context native payload was admitted")),
        };
    write(
        &a.out.join("artifact-negative.json"),
        &json!({
            "changed_payload_rejected":true,
            "change":"one context-q4.bin byte flipped; metadata and source unchanged",
            "error":corrupt_error,
            "corrupt_payload_sha256":sha256_file(&corrupt_file)?,
            "source_commit":source_commit
        }),
    )?;
    let mut rows = Vec::<Value>::new();
    let mut family_nonzero = std::collections::BTreeSet::<String>::new();
    for case in cases {
        let frame = SelectedRecordFrame {
            identity: SourceIdentity {
                record: case.record,
                commit: case.commit,
            },
            metadata: FrameMetadata {
                scope: b"m-world-v2",
                entity: &case.entity,
                relation: case.relation,
                view: 0,
                status: FrameStatus::Found,
            },
            token_ids: &case.tokens,
        };
        for (step, &target) in case.response.iter().enumerate() {
            if started.elapsed().as_secs() >= a.maximum_seconds {
                return Err(invalid("declared wall budget reached"));
            }
            let prefix = &case.response[..step];
            let mut parent_ids = case.parent_input.clone();
            parent_ids.extend_from_slice(prefix);
            let parent_scores = checkpoint.model.next_scores(&parent_ids)?;
            if parent_scores.len() != 4096 || parent_scores.iter().any(|x| !x.is_finite()) {
                return Err(invalid("invalid full parent distribution"));
            }
            // Only query, selected exact source, and preceding teacher-forced tokens
            // are admitted to the consumer. Current target enters only the loss.
            let traced = native.read(frame.clone(), &case.query, prefix)?;
            let replayed = loaded.read(frame.clone(), &case.query, prefix)?;
            if traced != replayed {
                return Err(invalid(
                    "independent native source-bound reload trace mismatch",
                ));
            }
            let hard = mix_scores(&parent_scores, &traced, true)?;
            let replay_hard = mix_scores(&parent_scores, &replayed, true)?;
            if score_hash(&hard) != score_hash(&replay_hard) {
                return Err(invalid("reload hard score mismatch"));
            }
            let disabled = mix_scores(&parent_scores, &traced, false)?;
            if score_hash(&disabled) != score_hash(&parent_scores) {
                return Err(invalid("disabled parent score identity broken"));
            }
            let ordinary = uniform_scores(&parent_scores, &traced)?;
            let out = weights.loss(
                frame.clone(),
                &case.query,
                prefix,
                &parent_scores,
                target,
                &native,
            )?;
            if out.trace != traced || score_hash(&out.mixed_scores) != score_hash(&hard) {
                return Err(invalid(
                    "connected hard/relaxed bridge differs from native forward",
                ));
            }
            let loss = out.loss.to_scalar::<f32>()?;
            if !loss.is_finite() {
                return Err(invalid("nonfinite ordinary-answer credit"));
            }
            let native_target_nll = -*hard
                .get(target as usize)
                .ok_or_else(|| invalid("target outside full native score distribution"))?;
            let loss_tolerance = 1e-4_f32 + 1e-5_f32 * native_target_nll.abs();
            let loss_native_error = (loss - native_target_nll).abs();
            if !native_target_nll.is_finite() || loss_native_error > loss_tolerance {
                return Err(invalid(format!("loss/native target NLL mismatch: loss={loss}, native={native_target_nll}, tolerance={loss_tolerance}")));
            }
            let grads = out.loss.backward()?;
            let mut gradient_rows = Vec::new();
            for (name, var) in weights.parameters() {
                let Some(g) = grads.get(var.as_tensor()) else {
                    continue;
                };
                let values = g.flatten_all()?.to_vec1::<f32>()?;
                if values.iter().any(|x| !x.is_finite()) {
                    return Err(invalid(format!("nonfinite gradient {name}")));
                }
                let l1: f64 = values.iter().map(|x| f64::from(x.abs())).sum();
                if l1 > 0. {
                    family_nonzero
                        .insert(name.split('.').next().unwrap_or(name.as_str()).to_owned());
                }
                gradient_rows.push(json!({"name":name,"elements":values.len(),"l1":l1,"nonzero_elements":values.iter().filter(|x|**x!=0.).count()}));
            }
            rows.push(json!({"case":case.id,"step":step,"target_label_only":target,"teacher_forced_prefix_ids":prefix,"query_ids":case.query,"selected_record_frame":{"record":case.record,"commit":case.commit,"tokens":case.tokens,"scope_bytes":b"m-world-v2".as_slice(),"entity_ids":case.entity,"relation":case.relation,"view":"current","status":"Found","metadata_role":"exact caller identity, not geometric scoring"},"parent_input_ids":parent_ids,"parent_pointer_aware_scores_sha256":score_hash(&parent_scores),"parent_score_count":parent_scores.len(),"native_trace":traced,"source_reload_equal":true,"hard_scores_sha256":score_hash(&hard),"disabled_identity":true,"uniform_same_information_scores_sha256":score_hash(&ordinary),"ordinary_answer_nll":loss,"native_target_nll":native_target_nll,"loss_matches_native_target":true,"loss_native_absolute_error":loss_native_error,"loss_tolerance":loss_tolerance,"gradients":gradient_rows}));
            write(
                &a.out.join("progress.json"),
                &json!({"completed_batches":rows.len(),"rows":rows,"nonzero_gradient_families":family_nonzero,"wall_seconds":started.elapsed().as_secs_f64(),"optimizer_updates":0}),
            )?;
        }
    }
    for family in ["context", "potential", "no_read"] {
        if !family_nonzero.contains(family) {
            return Err(invalid(format!(
                "missing active {family} language-loss gradient"
            )));
        }
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.geometric-source-consumer-construction/1","scope":"actual-BPE selected-record teacher-forced zero-update construction; newly initialized geometric consumer, floating pointer-aware development parent; no language, native-chat, or quality qualification","optimizer_updates":0,"initializer_language_weights":false,"parent_model_sha256":checkpoint.record.model_sha256,"parent_checkpoint_manifest_sha256":sealed_manifest_sha256(&a.checkpoint).map_err(|e|invalid(e.to_string()))?,"retained_report_sha256":sha256_file(&a.retained_report)?,"tokenizer_sha256":sha256_file(&a.tokenizer)?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"source_commit":source_commit,"changed_payload_rejected":true,"loss_matches_native_target":true,"consumer_positions":128,"parent_context":384,"native_stats":native.stats(),"wall_seconds":started.elapsed().as_secs_f64(),"nonzero_gradient_families":family_nonzero,"uniform_control_scope":"same source information zero/uniform diagnostic; not trained or capacity matched","rows":rows}),
    )?;
    Ok(())
}
