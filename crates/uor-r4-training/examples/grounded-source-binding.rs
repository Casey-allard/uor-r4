//! Open development source-binding diagnostic. No fit or native-serving claim.
//! Each arm changes user source text and generates its own preceding replies.
//! Evaluation relation/answers never enter the compiler, store or emitter.
use candle_core::Device;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    relation_compiler::{OpPolicy, SavedCompiler, Trunk},
    sha256_file,
    stack_checkpoint::sealed_manifest_sha256,
    stack_grounded_session::{
        CompiledAction, GroundedSession, MemoryEffect, RecallDisposition, SessionLimits,
        SessionScope, TurnCompiler, TurnControls, TurnOutcome,
    },
    stack_store::StoreRead,
    temporal_compiler::GroundedCompiler,
    Result, TrainingError,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    mode: String,
    out: PathBuf,
    retained_report: Option<PathBuf>,
    suite: Option<PathBuf>,
    checkpoint: Option<PathBuf>,
    tokenizer: Option<PathBuf>,
    compiler: Option<PathBuf>,
    trunk: Option<PathBuf>,
    maximum_seconds: u64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source_group: String,
    selected_changed: bool,
    distractor_changed: bool,
    source: Vec<String>,
    relation: u32,
    selected: String,
    distractor: String,
    answers: FrozenAnswers,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    schema: String,
    retained_report_sha256: String,
    cases: Vec<Case>,
}
fn invalid(s: impl Into<String>) -> TrainingError {
    TrainingError::Invalid(s.into())
}
fn write(p: &Path, v: &impl Serialize) -> Result<()> {
    fs::write(p, serde_json::to_vec_pretty(v)?)?;
    Ok(())
}
fn need<'a>(p: &'a Option<PathBuf>, name: &str) -> Result<&'a Path> {
    p.as_deref()
        .ok_or_else(|| invalid(format!("missing {name}")))
}
fn text<'a>(v: &'a Value, k: &str) -> Result<&'a str> {
    v[k].as_str()
        .ok_or_else(|| invalid(format!("missing string {k}")))
}
// Authored once from the typed query, before any candidate generation.
fn answers(relation: u32, value: &str) -> Result<FrozenAnswers> {
    let forms: &[&str] = match relation {
        4 => &["Your hometown is {v}.", "You grew up in {v}."],
        7 => &["You work as a {v}.", "Your job is {v}."],
        _ => {
            return Err(invalid(
                "only the frozen hometown/job diagnostic intents are admitted",
            ))
        }
    };
    let mut accepted = vec![
        format!("{value}."),
        format!("It's {value}."),
        format!("It is {value}."),
    ];
    accepted.extend(forms.iter().map(|s| s.replace("{v}", value)));
    let a = FrozenAnswers {
        intent: RecordedValueIntent::Current,
        accepted,
    };
    a.validate().map_err(|e| invalid(e.to_string()))?;
    Ok(a)
}
fn span_value(turn: &Value) -> Result<(usize, usize, String)> {
    let s = text(turn, "user")?;
    let span = &turn["gold_action"]["span"];
    let start = span["start"]
        .as_u64()
        .and_then(|x| usize::try_from(x).ok())
        .ok_or_else(|| invalid("bad start"))?;
    let end = span["end"]
        .as_u64()
        .and_then(|x| usize::try_from(x).ok())
        .ok_or_else(|| invalid("bad end"))?;
    let value = s
        .get(start..end)
        .ok_or_else(|| invalid("span outside original UTF-8 source"))?;
    Ok((start, end, value.to_owned()))
}
fn replace_span(turn: &Value, value: &str) -> Result<String> {
    let s = text(turn, "user")?;
    let (start, end, _) = span_value(turn)?;
    Ok(format!("{}{value}{}", &s[..start], &s[end..]))
}
fn prepare(report: &Path, out: &Path) -> Result<()> {
    let d: Value = serde_json::from_slice(&fs::read(report)?)?;
    let conversations = d["arms"]["default"]["conversations"]
        .as_array()
        .ok_or_else(|| invalid("missing retained conversations"))?;
    // Predetermined exposed failures and same-relation successes. No fresh draw.
    let seeds = [
        ("mw2-0246", "dancer", "Oslo"),
        ("mw2-0146", "dancer", "Oslo"),
        ("mw2-0288", "Brimfold", "Nourkterk"),
        ("mw2-0124", "Brimfold", "Nourkterk"),
    ];
    let mut cases = Vec::new();
    for (id, selected_alt, distractor_alt) in seeds {
        let c = conversations
            .iter()
            .find(|c| c["id"] == id)
            .ok_or_else(|| invalid(format!("missing {id}")))?;
        let ts = c["turns"]
            .as_array()
            .ok_or_else(|| invalid("missing turns"))?;
        let last = ts.last().ok_or_else(|| invalid("empty turns"))?;
        let relation = last["gold_action"]["relation"]
            .as_u64()
            .and_then(|x| u32::try_from(x).ok())
            .ok_or_else(|| invalid("bad query relation"))?;
        if text(&last["gold_action"], "kind")? != "query_current" {
            return Err(invalid("not a current query"));
        }
        let writes: Vec<usize> = ts
            .iter()
            .enumerate()
            .take(ts.len() - 1)
            .filter_map(|(i, t)| {
                matches!(
                    t["gold_action"]["kind"].as_str(),
                    Some("assert" | "correct")
                )
                .then_some(i)
            })
            .collect();
        let selected_index = writes
            .iter()
            .copied()
            .rev()
            .find(|&i| ts[i]["gold_action"]["relation"] == relation)
            .ok_or_else(|| invalid("missing selected write"))?;
        let distractor_index = writes
            .iter()
            .copied()
            .rev()
            .find(|&i| ts[i]["gold_action"]["relation"] != relation)
            .ok_or_else(|| invalid("missing companion write"))?;
        let (_, _, selected) = span_value(&ts[selected_index])?;
        let (_, _, distractor) = span_value(&ts[distractor_index])?;
        if selected == selected_alt || distractor == distractor_alt {
            return Err(invalid("counterfactual replacement is unchanged"));
        }
        for selected_changed in [false, true] {
            for distractor_changed in [false, true] {
                let value = if selected_changed {
                    selected_alt
                } else {
                    &selected
                };
                let other = if distractor_changed {
                    distractor_alt
                } else {
                    &distractor
                };
                let mut source: Vec<String> = ts
                    .iter()
                    .map(|t| text(t, "user").map(str::to_owned))
                    .collect::<Result<_>>()?;
                source[selected_index] = replace_span(&ts[selected_index], value)?;
                source[distractor_index] = replace_span(&ts[distractor_index], other)?;
                cases.push(Case {
                    id: format!(
                        "{id}-s{}-d{}",
                        u8::from(selected_changed),
                        u8::from(distractor_changed)
                    ),
                    source_group: id.into(),
                    selected_changed,
                    distractor_changed,
                    source,
                    relation,
                    selected: value.into(),
                    distractor: other.into(),
                    answers: answers(relation, value)?,
                });
            }
        }
    }
    // Authored same-type city/city control: frame selection cannot be replaced
    // by selecting an obvious payload type. This is not a retained/fresh row.
    for selected_changed in [false, true] {
        for distractor_changed in [false, true] {
            let selected = if selected_changed {
                "Brimfold"
            } else {
                "Louston"
            };
            let distractor = if distractor_changed { "Oslo" } else { "Warsaw" };
            cases.push(Case {
                id: format!(
                    "authored-city-city-s{}-d{}",
                    u8::from(selected_changed),
                    u8::from(distractor_changed)
                ),
                source_group: "authored-city-city".into(),
                selected_changed,
                distractor_changed,
                source: vec![
                    format!("I was raised in {selected}."),
                    format!("I reside in {distractor}."),
                    "Remind me where I am from.".into(),
                ],
                relation: 4,
                selected: selected.into(),
                distractor: distractor.into(),
                answers: answers(4, selected)?,
            });
        }
    }
    write(
        &out.join("suite.json"),
        &Suite {
            schema: "uor-r4.source-binding-suite/1".into(),
            retained_report_sha256: sha256_file(report)?,
            cases,
        },
    )
}
fn validate_suite(s: &Suite) -> Result<()> {
    if s.schema != "uor-r4.source-binding-suite/1" || s.cases.is_empty() || s.cases.len() > 32 {
        return Err(invalid("invalid suite/schema/count"));
    }
    if s.retained_report_sha256.len() != 64
        || !s
            .retained_report_sha256
            .bytes()
            .all(|b| b.is_ascii_hexdigit())
    {
        return Err(invalid("bad retained report digest"));
    }
    let mut ids = BTreeSet::new();
    let mut turns = 0;
    for c in &s.cases {
        if !ids.insert(&c.id)
            || c.id.is_empty()
            || c.source_group.is_empty()
            || c.source.len() < 2
            || c.source.len() > 16
            || c.selected.is_empty()
            || c.distractor.is_empty()
            || !matches!(c.relation, 4 | 7)
            || c.source.iter().any(|x| x.is_empty() || x.len() > 65536)
        {
            return Err(invalid("malformed or duplicate case"));
        }
        c.answers.validate().map_err(|e| invalid(e.to_string()))?;
        if c.answers != answers(c.relation, &c.selected)? {
            return Err(invalid("answers differ from frozen typed authoring rule"));
        }
        turns += c.source.len() * 2;
    }
    if turns > 512 {
        return Err(invalid("turn budget exceeded"));
    }
    Ok(())
}
fn stage(
    outcome: &TurnOutcome,
    relation: u32,
    selected: &str,
    tokenizer: &ByteBpeTokenizer,
    read: bool,
) -> &'static str {
    let query_matches =
        matches!(outcome.action, CompiledAction::QueryCurrent{relation:r} if r==relation);
    if !query_matches {
        return "compiler_query";
    }
    let found = match &outcome.memory {
        MemoryEffect::Read {
            read: StoreRead::Found(v),
        } => v,
        _ => return "store_status",
    };
    if found.conflict {
        return "store_conflict";
    }
    if tokenizer.decode_bytes(&found.tokens) != selected.as_bytes() {
        return "store_value";
    }
    if !read {
        return "read_disabled_control";
    }
    if outcome.recall != RecallDisposition::Value {
        return "recall_interface";
    }
    let input = tokenizer.decode_bytes(&outcome.emitter_input_ids);
    if !String::from_utf8_lossy(&input)
        .ends_with(&format!("\nSystem: Memory: {selected}.\nAssistant:"))
    {
        return "input_payload_unverified";
    }
    "selected_payload_delivered"
}
fn evaluate(a: &Args, suite: Suite) -> Result<()> {
    let started = Instant::now();
    let tokenizer_path = need(&a.tokenizer, "tokenizer")?;
    let tokenizer_bytes = fs::read(tokenizer_path)?;
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| invalid("unreadable tokenizer"))?;
    let compiler_path = need(&a.compiler, "compiler")?;
    let compiler_bytes = fs::read(compiler_path)?;
    let checkpoint = need(&a.checkpoint, "checkpoint")?;
    let retained_root = need(&a.retained_report, "retained_report")?
        .parent()
        .ok_or_else(|| invalid("retained report has no parent"))?;
    report_output::verify(retained_root)?;
    if sealed_manifest_sha256(checkpoint).map_err(|e| invalid(e.to_string()))?
        != sealed_manifest_sha256(&retained_root.join("checkpoint"))
            .map_err(|e| invalid(e.to_string()))?
    {
        return Err(invalid(
            "checkpoint differs from retained configuration/protocol/store",
        ));
    }
    let retained: Value =
        serde_json::from_slice(&fs::read(need(&a.retained_report, "retained_report")?)?)?;
    if sha256_file(need(&a.retained_report, "retained_report")?)? != suite.retained_report_sha256 {
        return Err(invalid("suite/report source identity differs"));
    }
    if sha256_file(&checkpoint.join("model.safetensors"))?
        != text(
            &retained["model_identity"]["files_sha256"],
            "model.safetensors",
        )?
        || sha256_file(tokenizer_path)? != text(&retained, "tokenizer_sha256")?
    {
        return Err(invalid(
            "model/tokenizer differs from selected retained artifact",
        ));
    }
    let compiler = GroundedCompiler::Legacy(
        SavedCompiler::load(
            compiler_bytes,
            Some(Trunk::load(
                need(&a.trunk, "trunk")?,
                &tokenizer_bytes,
                &Device::Cpu,
            )?),
        )?
        .with_op_policy(OpPolicy::UnlessQuery)?,
    );
    let limits = SessionLimits {
        max_new_tokens: 64,
        max_turns: 64,
        max_source_bytes: 65536,
        max_history_tokens: 1 << 20,
        max_store_records: 4096,
        context_policy: uor_r4_training::stack_grounded_session::ContextPolicy::WholeCompletedTurns,
    };
    if serde_json::to_value(compiler.identity())? != retained["compiler_identity"]
        || serde_json::to_value(&limits)? != retained["limits"]
        || retained["op_policy"] != "UnlessQuery"
        || retained["log_recall"] != "sieve"
    {
        return Err(invalid(
            "compiler/policy/provider/limits differ from retained artifact",
        ));
    }
    let scope = SessionScope {
        scope: b"m-world-v2".to_vec(),
        entity: tokenizer.encode("user"),
    };
    let recall: uor_r4_training::stack_grounded_session::LogRecall =
        Arc::new(|log: &[&str], q: &str| {
            uor_r4_training::milestone_world_v2::sieve_value_text(log, q)
        });
    let mut rows = Vec::new();
    for (case_index, c) in suite.cases.iter().enumerate() {
        for read in [true, false] {
            write(
                &a.out.join("active.json"),
                &json!({"case_index":case_index,"case":c,"read":read,"stage":"opening_model","completed_outcomes":[]}),
            )?;
            if started.elapsed().as_secs() >= a.maximum_seconds {
                return Err(invalid(
                    "configured wall budget reached before next conversation",
                ));
            }
            let mut session = GroundedSession::from_checkpoint_path(
                checkpoint,
                tokenizer_bytes.clone(),
                compiler.clone(),
                scope.clone(),
                limits.clone(),
                &Device::Cpu,
            )
            .map_err(|e| invalid(e.to_string()))?
            .with_log_recall("sieve", recall.clone());
            let mut outcomes = Vec::new();
            for (turn_index, source) in c.source.iter().enumerate() {
                write(
                    &a.out.join("active.json"),
                    &json!({"case_index":case_index,"case":c,"read":read,"stage":"before_turn","turn_index":turn_index,"source":source,"completed_outcomes":outcomes}),
                )?;
                if started.elapsed().as_secs() >= a.maximum_seconds {
                    return Err(invalid(
                        "configured wall budget reached before next bounded turn",
                    ));
                }
                outcomes.push(
                    session
                        .turn_with_controls(source, TurnControls { read, write: true })
                        .map_err(|e| invalid(e.to_string()))?,
                );
            }
            let last = outcomes.last().ok_or_else(|| invalid("empty outcome"))?;
            let classification = stage(last, c.relation, &c.selected, &tokenizer, read);
            let complete_answer = c.answers.accepts(&last.reply_text);
            let row = json!({"case":c,"read":read,"stage":classification,"complete_answer":complete_answer,"mentions_selected_diagnostic_only":last.reply_text.contains(&c.selected),"mentions_distractor_diagnostic_only":last.reply_text.contains(&c.distractor),"source_bpe_lengths":c.source.iter().map(|s|tokenizer.encode(s).len()).collect::<Vec<_>>(),"emitter_input_decoded":String::from_utf8_lossy(&tokenizer.decode_bytes(&last.emitter_input_ids)),"outcomes":outcomes});
            write(
                &a.out.join(format!("case-{case_index:02}-read-{read}.json")),
                &row,
            )?;
            write(
                &a.out.join("active.json"),
                &json!({"case_index":case_index,"case_id":c.id,"read":read,"stage":"completed","completed_turns":c.source.len()}),
            )?;
            rows.push(row);
            println!(
                "{} read={read} stage={classification} answer={complete_answer}",
                c.id
            );
        }
    }
    write(
        &a.out.join("report.json"),
        &json!({"schema":"uor-r4.source-binding-diagnostic/1","scope":"exposed development source counterfactuals, actual generated histories; floating development emitter, no native serving or general-chat qualification","suite_sha256":sha256_file(need(&a.suite,"suite")?)?,"checkpoint_manifest_sha256":sealed_manifest_sha256(checkpoint).map_err(|e|invalid(e.to_string()))?,"compiler":compiler.identity(),"op_policy":"unless_query","log_recall":"sieve","tokenizer_sha256":sha256_file(tokenizer_path)?,"executable_sha256":sha256_file(&std::env::current_exe()?)?,"limits":limits,"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"optimizer_updates":0,"wall_seconds":started.elapsed().as_secs_f64(),"rows":rows}),
    )
}
fn main() -> Result<()> {
    let args_path = std::env::args()
        .nth(1)
        .ok_or_else(|| invalid("usage: grounded-source-binding args.json"))?;
    let a: Args = serde_json::from_slice(&fs::read(args_path)?)?;
    if !matches!(a.mode.as_str(), "prepare" | "evaluate") || !(1..=900).contains(&a.maximum_seconds)
    {
        return Err(invalid("unknown mode or wall limit outside1..900"));
    }
    let suite = if a.mode == "evaluate" {
        need(&a.retained_report, "retained_report")?;
        need(&a.checkpoint, "checkpoint")?;
        need(&a.tokenizer, "tokenizer")?;
        need(&a.compiler, "compiler")?;
        need(&a.trunk, "trunk")?;
        let s: Suite = serde_json::from_slice(&fs::read(need(&a.suite, "suite")?)?)?;
        validate_suite(&s)?;
        Some(s)
    } else {
        need(&a.retained_report, "retained_report")?;
        None
    };
    report_output::claim(&a.out)?;
    let result = match suite {
        Some(s) => evaluate(&a, s),
        None => prepare(need(&a.retained_report, "retained_report")?, &a.out),
    };
    if let Err(e) = &result {
        write(&a.out.join("error.json"), &json!({"error":e.to_string()}))?;
    }
    report_output::seal(&a.out)?;
    report_output::verify(&a.out)?;
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_binding_replaces_exact_utf8_span_only() {
        let t =
            json!({"user":"I was raised in Évreux.","gold_action":{"span":{"start":16,"end":23}}});
        assert_eq!(replace_span(&t, "Oslo").unwrap(), "I was raised in Oslo.");
        let bad = json!({"user":"Évreux","gold_action":{"span":{"start":1,"end":3}}});
        assert!(span_value(&bad).is_err());
    }
    #[test]
    fn source_binding_scores_complete_forms_not_presence() {
        let a = answers(7, "singer").unwrap();
        assert!(a.accepts("Your job is singer."));
        assert!(!a.accepts("Your job is singer. You live in Lisbon."));
        assert!(!a.accepts("singer singer singer"));
    }
}
