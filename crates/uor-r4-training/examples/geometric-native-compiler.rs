//! Bounded source-only learned native turn compiler and exact-store integration.
//! Exact addressed admission and all-bank native reads are reported separately.
#[path = "geometric_native_compiler/reader.rs"]
mod reader;
use serde_json::{json, Value};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use uor_r4_core::{
    answer_oracle::{FrozenAnswers, RecordedValueIntent},
    report_output,
};
use uor_r4_integer::geometric_source_realizer::{NativeArtifactBinding, NativeSourceRealizer};
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::{
    geometric_turn_compiler::{fit_examples, FitConfig, NativeTurnCompiler},
    relation_compiler::{Example, NONE},
    sha256_bytes,
    stack_grounded_session::{
        CompiledAction, CompilerIdentity, GroundedSessionError, RelationLabel, SourceSpan,
        TurnCompiler,
    },
    stack_store::{HistoryView, StackStore, Update},
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fail(message: &str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, message)
}
fn write_json(path: &Path, value: &Value) -> Result<()> {
    fs::write(path, serde_json::to_vec_pretty(value)?)?;
    Ok(())
}
struct Args {
    native: PathBuf,
    binding: PathBuf,
    tokenizer: PathBuf,
    output: PathBuf,
    binding_sha: Option<String>,
    cue: PathBuf,
    prefix: PathBuf,
    end: PathBuf,
    audit_artifacts: Option<PathBuf>,
}
fn args() -> Result<Args> {
    let mut flags = BTreeMap::new();
    let mut args = std::env::args().skip(1);
    while let Some(key) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| fail("every flag needs a value"))?;
        if flags.insert(key, value).is_some() {
            return Err(fail("duplicate flag").into());
        }
    }
    let mut get = |key: &str| {
        flags
            .remove(key)
            .map(PathBuf::from)
            .ok_or_else(|| fail(&format!("missing {key}")))
    };
    let (native, binding, tokenizer, output) = (
        get("--native-artifact")?,
        get("--binding")?,
        get("--tokenizer")?,
        get("--output")?,
    );
    let (cue, prefix, end) = (get("--cue")?, get("--prefix")?, get("--source-end")?);
    let binding_sha = flags.remove("--binding-sha");
    let audit_artifacts = flags.remove("--audit-artifacts").map(PathBuf::from);
    if !flags.is_empty() {
        return Err(fail("unknown arguments").into());
    }
    Ok(Args {
        native,
        binding,
        tokenizer,
        output,
        binding_sha,
        cue,
        prefix,
        end,
        audit_artifacts,
    })
}
fn example(relation: &str, act: &'static str, template: &str, value: &str) -> Example {
    Example {
        text: template.replace("{v}", value),
        relation: relation.into(),
        act,
        template: Some(template.into()),
    }
}
fn pools(
    split: &str,
) -> (
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
    Vec<&'static str>,
) {
    match split {
        "training" => (
            vec![
                "engineer",
                "green valley",
                "librarian",
                "red meadow",
                "architect",
                "blue harbor",
                "singer",
                "silver hill",
            ],
            vec![
                "My {r} is {v}.",
                "For my {r}, record {v}.",
                "I have {v} as my {r}.",
                "The value of my {r} is {v}.",
            ],
            vec![
                "My {r} is now {v}.",
                "Correct my {r} to {v}.",
                "Update my {r}: {v}.",
                "Replace my {r} with {v}.",
            ],
            vec![
                "What is my {r}?",
                "Tell me my {r}.",
                "Recall my {r}.",
                "Which {r} did I state?",
            ],
        ),
        "development" => (
            vec!["potter", "amber lake", "dancer", "cedar grove"],
            vec![
                "Regarding my {r}, it is {v}.",
                "Please remember {v} for my {r}.",
            ],
            vec![
                "Revise the saved {r} to {v}.",
                "The old {r} should become {v}.",
            ],
            vec![
                "Can you recall the saved {r}?",
                "What did I give as my {r}?",
            ],
        ),
        _ => (
            vec!["weaver", "orchid bay", "painter", "birch ridge"],
            vec![
                "The {r} I want remembered is {v}.",
                "Keep this as my {r}: {v}.",
            ],
            vec![
                "Change the remembered {r} into {v}.",
                "Use {v} instead for my {r}.",
            ],
            vec![
                "Give back the {r} I recorded.",
                "Which value is saved for my {r}?",
            ],
        ),
    }
}
fn panel(split: &str) -> Vec<Example> {
    let (values, asserts, updates, queries) = pools(split);
    let mut rows = Vec::new();
    for relation in ["job", "home"] {
        for (act, templates) in [("assert", &asserts), ("update", &updates)] {
            for template in templates {
                let t = template.replace("{r}", relation);
                for value in &values {
                    rows.push(example(relation, act, &t, value));
                }
            }
        }
        for template in &queries {
            rows.push(example(
                relation,
                "query",
                &template.replace("{r}", relation),
                "",
            ));
        }
    }
    let negatives = match split {
        "training" => vec![
            "Hello there.",
            "The weather seems pleasant.",
            "I enjoy stories.",
            "Let us think carefully.",
            "A job can require patience.",
            "A home can be welcoming.",
            "What a lovely evening.",
            "Thanks for listening.",
            "Words have many meanings.",
            "We should be curious.",
            "The river flows quietly.",
            "I like reading fiction.",
            "It might rain tomorrow.",
            "Geometry is interesting.",
            "Good morning everyone.",
            "Please continue talking.",
        ],
        "development" => vec![
            "I am happy to chat.",
            "The garden looks beautiful.",
            "Learning takes time.",
            "That was an interesting thought.",
            "People discuss job satisfaction.",
            "A home needs care.",
            "The sunset was bright.",
            "Thank you very much.",
        ],
        _ => vec![
            "We can discuss ideas.",
            "A quiet walk is pleasant.",
            "The moon is visible.",
            "I appreciate your help.",
            "Job interviews can be difficult.",
            "Home design is fascinating.",
            "That story made me smile.",
            "Good afternoon to you.",
        ],
    };
    for text in negatives {
        rows.push(Example {
            text: text.into(),
            relation: NONE.into(),
            act: NONE,
            template: None,
        });
    }
    rows
}
fn input_rows(rows: &[Example]) -> Value {
    json!(rows.iter().map(|e|json!({"text":e.text,"relation":e.relation,"act":e.act,"template":e.template,"gold_span":e.slot_span()})).collect::<Vec<_>>())
}
fn action_fields(action: &CompiledAction) -> (&str, Option<u32>, Option<SourceSpan>) {
    match action {
        CompiledAction::Assert { relation, span } => ("assert", Some(*relation), Some(*span)),
        CompiledAction::Correct { relation, span } => ("update", Some(*relation), Some(*span)),
        CompiledAction::QueryCurrent { relation } => ("query", Some(*relation), None),
        CompiledAction::Query { relation, .. } => ("query", Some(*relation), None),
        CompiledAction::Unresolved { .. } => (NONE, None, None),
    }
}
fn evaluate(compiler: &dyn TurnCompiler, rows: &[Example]) -> Result<Value> {
    let mut records = Vec::new();
    let (
        mut act_correct,
        mut relation_correct,
        mut span_correct,
        mut exact,
        mut prose,
        mut false_writes,
    ) = (0, 0, 0, 0, 0, 0);
    let (
        mut writes,
        mut write_exact,
        mut write_act,
        mut write_relation,
        mut write_span,
        mut queries,
        mut query_exact,
    ) = (0, 0, 0, 0, 0, 0, 0);
    for e in rows {
        let action = compiler.compile(&e.text)?;
        let (act, relation, span) = action_fields(&action);
        let gold_relation = match e.relation.as_str() {
            "job" => Some(1),
            "home" => Some(2),
            _ => None,
        };
        let gold_span = e.slot_span().map(|(start, end)| SourceSpan { start, end });
        let ac = act == e.act;
        let rc = relation == gold_relation;
        let sc = span == gold_span;
        act_correct += usize::from(ac);
        relation_correct += usize::from(rc);
        span_correct += usize::from(sc);
        exact += usize::from(ac && rc && sc);
        if matches!(e.act, "assert" | "update") {
            writes += 1;
            write_exact += usize::from(ac && rc && sc);
            write_act += usize::from(ac);
            write_relation += usize::from(rc);
            write_span += usize::from(sc);
        }
        if e.act == "query" {
            queries += 1;
            query_exact += usize::from(ac && rc && sc);
        }
        if e.act == NONE {
            prose += 1;
            false_writes += usize::from(matches!(
                &action,
                CompiledAction::Assert { .. } | CompiledAction::Correct { .. }
            ));
        }
        records.push(json!({"source":e.text,"gold_act":e.act,"gold_relation":gold_relation,"gold_span":gold_span,"predicted":action,"act_correct":ac,"relation_correct":rc,"span_exact":sc,"complete_exact":ac&&rc&&sc}));
    }
    Ok(
        json!({"rows":records,"examples":rows.len(),"act_correct":act_correct,"relation_correct":relation_correct,"span_exact":span_correct,"complete_exact":exact,"prose_rows":prose,"prose_false_writes":false_writes,"write_rows":writes,"write_complete_exact":write_exact,"write_act_correct":write_act,"write_relation_correct":write_relation,"write_span_exact":write_span,"query_rows":queries,"query_complete_exact":query_exact}),
    )
}
fn store_episodes(
    compiler: &dyn TurnCompiler,
    tokenizer: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
    split: &str,
    a: &Args,
) -> Result<Value> {
    let (values, asserts, updates, queries) = pools(split);
    let mut rows = Vec::new();
    let (mut correct, mut queries_total, mut selected_complete, mut bank_complete) = (0, 0, 0, 0);
    let mut reader_calls = 0;
    for (episode, relation) in ["job", "home"].iter().enumerate() {
        let mut store = StackStore::new(100 + episode as u64, 16)?;
        let mut journal = BTreeMap::<u64, (u64, Vec<u32>)>::new();
        let mut steps = Vec::new();
        let distractor = if *relation == "job" { "home" } else { "job" };
        let turns = vec![
            example(
                distractor,
                "assert",
                &asserts[0].replace("{r}", distractor),
                values[2],
            ),
            example(
                relation,
                "assert",
                &asserts[0].replace("{r}", relation),
                values[0],
            ),
            Example {
                text: "I appreciate our conversation.".into(),
                relation: NONE.into(),
                act: NONE,
                template: None,
            },
            example(relation, "query", &queries[0].replace("{r}", relation), ""),
            example(
                relation,
                "update",
                &updates[0].replace("{r}", relation),
                values[1],
            ),
            example(relation, "query", &queries[1].replace("{r}", relation), ""),
        ];
        for (index, e) in turns.iter().enumerate() {
            let expected = if index < 4 { values[0] } else { values[1] };
            let answers = FrozenAnswers {
                intent: RecordedValueIntent::Current,
                accepted: vec![expected.to_owned(), format!("{expected}.")],
            };
            answers.validate()?;
            let action = compiler.compile(&e.text)?;
            let mut read: Option<Value> = None;
            let mut write = None;
            match &action {
                CompiledAction::Assert { relation, span }
                | CompiledAction::Correct { relation, span } => {
                    let text = e
                        .text
                        .get(span.start..span.end)
                        .ok_or_else(|| fail("predicted span is not UTF8 source range"))?;
                    let tokens = tokenizer.encode(text);
                    let update = if matches!(&action, CompiledAction::Assert { .. }) {
                        Update::Assert
                    } else {
                        Update::Correct
                    };
                    let written = store.write_from(
                        b"compiler-probe",
                        &[1],
                        *relation,
                        &tokens,
                        update,
                        index as u64,
                    )?;
                    journal.insert(written.id, (index as u64, tokenizer.encode(&e.text)));
                    write = Some(written);
                    let bytes = store.to_bytes()?;
                    let history = store.history_sha256()?;
                    fs::write(
                        a.output
                            .join(format!("{split}-episode-{episode}-step-{index}-store.bin")),
                        &bytes,
                    )?;
                    store = StackStore::from_bytes(&bytes, 100 + episode as u64)?;
                    if store.history_sha256()? != history {
                        return Err(fail("store reload changed history").into());
                    }
                }
                CompiledAction::QueryCurrent {
                    relation: predicted_relation,
                } => {
                    let result = store.read(
                        b"compiler-probe",
                        &[1],
                        *predicted_relation,
                        HistoryView::Current,
                    )?;
                    let tokens = result.value().map(|v| v.tokens.clone());
                    let expected = if index < 4 { values[0] } else { values[1] };
                    let hit = tokens
                        .as_ref()
                        .is_some_and(|t| *t == tokenizer.encode(expected));
                    let mut generated = Vec::new();
                    // Admission uses actual predicted store addresses, never the expected label.
                    let mut bank = Vec::new();
                    for address in store.addresses()? {
                        if address.scope != b"compiler-probe" || address.entity != vec![1] {
                            continue;
                        }
                        let actual = store.read(
                            &address.scope,
                            &address.entity,
                            address.relation,
                            HistoryView::Current,
                        )?;
                        if let Some(v) = actual.value() {
                            let (event, cue) = journal.get(&v.record).ok_or_else(|| {
                                fail("actual record has no original statement cue")
                            })?;
                            bank.push(reader::Record {
                                event: *event,
                                record: v.record,
                                commit: v.commit,
                                relation: address.relation,
                                cue: cue.clone(),
                                value: v.tokens.clone(),
                            });
                        }
                    }
                    bank.sort_by_key(|r| (r.event, r.commit, r.record));
                    for (lane, records) in [
                        (
                            "selected-record",
                            bank.iter()
                                .filter(|r| Some(r.record) == result.value().map(|v| v.record))
                                .collect::<Vec<_>>(),
                        ),
                        ("all-bank", bank.iter().collect::<Vec<_>>()),
                    ] {
                        let owned = records
                            .into_iter()
                            .map(|r| reader::Record {
                                event: r.event,
                                record: r.record,
                                commit: r.commit,
                                relation: r.relation,
                                cue: r.cue.clone(),
                                value: r.value.clone(),
                            })
                            .collect::<Vec<_>>();
                        if owned.is_empty() {
                            generated
                                .push(json!({"lane":lane,"status":"no actual admitted records"}));
                            continue;
                        }
                        reader_calls += 1;
                        let mut output = reader::generate(
                            native,
                            &a.cue,
                            &a.prefix,
                            &a.end,
                            &tokenizer.encode(&e.text),
                            &owned,
                            32,
                        )?;
                        let ids: Vec<u32> =
                            serde_json::from_value(output["generated_ids_including_eos"].clone())?;
                        let eos = output["eos"] == true;
                        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
                        let bytes = tokenizer.decode_bytes(plain);
                        let raw = String::from_utf8_lossy(&bytes);
                        let text = raw.strip_prefix(' ').unwrap_or(&raw);
                        let complete = eos
                            && String::from_utf8(bytes.clone()).is_ok()
                            && answers.accepts(text);
                        if e.act == "query" {
                            if lane == "selected-record" {
                                selected_complete += usize::from(complete);
                            } else {
                                bank_complete += usize::from(complete);
                            }
                        }
                        output["lane"] = json!(lane);
                        output["reply_text"] = json!(text);
                        output["complete_answer"] = json!(complete);
                        output["frozen_answers_evaluation_only"] = json!(answers);
                        generated.push(output);
                    }
                    if e.act == "query" {
                        queries_total += 1;
                        correct += usize::from(hit);
                    }
                    read = Some(
                        json!({"store_read":result,"decoded":tokens.map(|t|tokenizer.decode(&t)),"expected_evaluation_only":expected,"exact_store_answer":hit,"native_generation":generated}),
                    );
                }
                _ => {}
            }
            if e.act == "query" && !matches!(&action, CompiledAction::QueryCurrent { .. }) {
                queries_total += 1;
            }
            steps.push(json!({"source":e.text,"predicted":action,"write":write,"read":read}));
        }
        rows.push(json!({"episode":episode,"relation_evaluation_label":relation,"steps":steps,"final_store_history_sha256":store.history_sha256()?}));
    }
    Ok(
        json!({"episodes":rows,"query_rows":queries_total,"native_reader_calls":reader_calls,"exact_store_answers":correct,"selected_record_complete":selected_complete,"all_bank_complete":bank_complete,"policy":"predicted-address-and-source-span-only;serialize-reload-after-writes;original-statement-cues;gold-used-only-to-score"}),
    )
}
fn run(a: &Args) -> Result<()> {
    let started = Instant::now();
    let (training, development, fresh) = (panel("training"), panel("development"), panel("fresh"));
    if training.len() > 512 || development.len() > 128 || fresh.len() > 64 {
        return Err(fail("panel cap exceeded").into());
    }
    let config = FitConfig {
        steps: 64,
        learning_rate: 0.03,
        max_tokens: 128,
        max_words: 64,
        max_value_words: 8,
    };
    let inputs = json!({"schema":"native-turn-frozen-inputs/1","training":input_rows(&training),"development":input_rows(&development),"fresh":input_rows(&fresh),"config":config,"limitations":"authored supervised panels;no general-language qualification"});
    let input_bytes = serde_json::to_vec_pretty(&inputs)?;
    fs::write(a.output.join("frozen-inputs.json"), &input_bytes)?;
    let binding_bytes = fs::read(&a.binding)?;
    let binding_sha = sha256_bytes(&binding_bytes);
    if a.binding_sha
        .as_ref()
        .is_some_and(|sha| sha != &binding_sha)
    {
        return Err(fail("trusted binding receipt mismatch").into());
    }
    let binding: NativeArtifactBinding = serde_json::from_slice(&binding_bytes)?;
    let native = NativeSourceRealizer::load_native(&a.native, &binding)?;
    let tokenizer_bytes = fs::read(&a.tokenizer)?;
    let tokenizer_sha = sha256_bytes(&tokenizer_bytes);
    let tokenizer = ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_bytes)
        .ok_or_else(|| fail("invalid tokenizer"))?;
    if let Some(root) = &a.audit_artifacts {
        return audit(a, root, &native, &tokenizer, &tokenizer_bytes);
    }
    let fit = fit_examples(
        &native,
        &tokenizer,
        &tokenizer_bytes,
        &["job".into(), "home".into()],
        &training,
        &development,
        config,
    )?;
    for checkpoint in &fit.checkpoint_artifacts {
        let dir = a.output.join(format!("checkpoint-{:04}", checkpoint.step));
        fs::create_dir(&dir)?;
        fs::write(
            dir.join("native-compiler.json"),
            &checkpoint.native_artifact,
        )?;
        fs::write(
            dir.join("source-parameters.json"),
            &checkpoint.source_parameters,
        )?;
    }
    fs::write(
        a.output.join("selected-native-compiler.json"),
        &fit.native_artifact,
    )?;
    fs::write(
        a.output.join("source-parameters.json"),
        &fit.source_parameters,
    )?;
    let selected = fs::read(a.output.join("selected-native-compiler.json"))?;
    let selected_sha = sha256_bytes(&selected);
    let compiler = NativeTurnCompiler::load(
        &selected,
        &selected_sha,
        &native,
        &tokenizer,
        &tokenizer_bytes,
    )?;
    let dev = evaluate(&compiler, &development)?;
    let fresh_eval = evaluate(&compiler, &fresh)?;
    let dev_store = store_episodes(&compiler, &tokenizer, &native, "development", &a)?;
    let fresh_store = store_episodes(&compiler, &tokenizer, &native, "fresh", &a)?;
    write_json(
        &a.output.join("report.json"),
        &json!({"schema":"geometric-native-compiler-report/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"host":std::env::consts::ARCH,"elapsed_seconds":started.elapsed().as_secs_f64(),"binding_receipt_sha256":binding_sha,"tokenizer_sha256":tokenizer_sha,"binding_trust":if a.binding_sha.is_some(){"explicit-pinned-sha"}else{"caller-supplied-trusted-receipt"},"frozen_inputs_sha256":sha256_bytes(&input_bytes),"selected_step":fit.selected_step,"selected_artifact_sha256":selected_sha,"compiler_identity":compiler.identity(),"checkpoints":fit.checkpoints,"diagnostics":fit.diagnostics,"development":dev,"fresh":fresh_eval,"development_store":dev_store,"fresh_store":fresh_store,"geometric_reader_available":"selected-record-and-all-bank","native_reader_calls":dev_store["native_reader_calls"].as_u64().unwrap_or(0)+fresh_store["native_reader_calls"].as_u64().unwrap_or(0),"native_emission_status":if dev_store["native_reader_calls"]==0 && fresh_store["native_reader_calls"]==0 {"NOT_RUN"}else{"executed"},"claims":"authored native compiler/store/reader episodes;encoder frozen;authored panels;no general prose/reasoning/energy qualification"}),
    )?;
    report_output::seal(&a.output)?;
    report_output::verify(&a.output)?;
    println!("sealed {}", a.output.display());
    Ok(())
}

// Deliberately explicit instrumentation control, not a learned model or serving fallback.
struct ReferenceRule {
    identity: CompilerIdentity,
    bytes: Vec<u8>,
    writes: Vec<(String, String, u32, bool)>,
    queries: Vec<(String, u32)>,
}
impl ReferenceRule {
    fn new(tokenizer_sha: &str) -> Result<Self> {
        let mut writes = Vec::new();
        let mut queries = Vec::new();
        for update in [true, false] {
            for split in ["training", "development"] {
                let (_, asserts, updates, qs) = pools(split);
                for (id, rel) in [(1, "job"), (2, "home")] {
                    for frame in if update { &updates } else { &asserts } {
                        let frame = frame.replace("{r}", rel);
                        let (prefix, suffix) = frame
                            .split_once("{v}")
                            .ok_or_else(|| fail("reference frame lacks value"))?;
                        writes.push((prefix.into(), suffix.into(), id, update));
                    }
                    if update {
                        for q in &qs {
                            queries.push((q.replace("{r}", rel), id));
                        }
                    }
                }
            }
        }
        let bytes = serde_json::to_vec(
            &json!({"schema":"authored-exact-template-instrument-control/1","writes":writes,"queries":queries}),
        )?;
        let identity = CompilerIdentity {
            schema: "authored-exact-template-instrument-control/1".into(),
            artifact_sha256: sha256_bytes(&bytes),
            tokenizer_sha256: tokenizer_sha.into(),
            label_schema: "job1-home2".into(),
            relations: vec![
                RelationLabel {
                    id: 1,
                    name: "job".into(),
                },
                RelationLabel {
                    id: 2,
                    name: "home".into(),
                },
            ],
            encoder: None,
        };
        Ok(Self {
            identity,
            bytes,
            writes,
            queries,
        })
    }
}
impl TurnCompiler for ReferenceRule {
    fn identity(&self) -> &CompilerIdentity {
        &self.identity
    }
    fn artifact_bytes(&self) -> &[u8] {
        &self.bytes
    }
    fn compile(&self, source: &str) -> std::result::Result<CompiledAction, GroundedSessionError> {
        for (q, relation) in &self.queries {
            if source == q {
                return Ok(CompiledAction::QueryCurrent {
                    relation: *relation,
                });
            }
        }
        for (prefix, suffix, relation, update) in &self.writes {
            if let Some(value) = source
                .strip_prefix(prefix)
                .and_then(|s| s.strip_suffix(suffix))
            {
                if !value.is_empty() {
                    let span = SourceSpan {
                        start: prefix.len(),
                        end: source.len() - suffix.len(),
                    };
                    return Ok(if *update {
                        CompiledAction::Correct {
                            relation: *relation,
                            span,
                        }
                    } else {
                        CompiledAction::Assert {
                            relation: *relation,
                            span,
                        }
                    });
                }
            }
        }
        Ok(CompiledAction::Unresolved {
            reason: "outside declared exact-template instrument control".into(),
        })
    }
}
fn panel_cross(phrasing: &str, value_split: &str) -> Vec<Example> {
    let mut rows = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let (values, _, _, _) = pools(value_split);
    for e in panel(phrasing) {
        if e.act == "assert" || e.act == "update" {
            if let Some(t) = &e.template {
                if seen.insert((e.relation.clone(), e.act, t.clone())) {
                    for v in &values {
                        rows.push(example(&e.relation, e.act, t, v));
                    }
                }
            }
        } else {
            rows.push(e);
        }
    }
    rows
}
fn audit(
    a: &Args,
    root: &Path,
    native: &NativeSourceRealizer,
    tokenizer: &ByteBpeTokenizer,
    tokenizer_bytes: &[u8],
) -> Result<()> {
    let parent_report_bytes = fs::read(root.join("report.json"))?;
    let parent_report: Value = serde_json::from_slice(&parent_report_bytes)?;
    let parent_manifest_sha = sha256_bytes(&fs::read(root.join(report_output::MANIFEST_FILE))?);
    let original_inputs: Value =
        serde_json::from_slice(&fs::read(root.join("frozen-inputs.json"))?)?;
    if original_inputs["training"] != input_rows(&panel("training"))
        || original_inputs["development"] != input_rows(&panel("development"))
    {
        return Err(fail("audit panels differ from actual fitted inputs").into());
    }
    let panels = [
        ("training", panel("training")),
        ("development", panel("development")),
        (
            "known_phrasing_new_values",
            panel_cross("training", "development"),
        ),
        (
            "new_phrasing_known_values",
            panel_cross("development", "training"),
        ),
    ];
    let frozen = json!(panels
        .iter()
        .map(|(name, rows)| json!({"name":name,"rows":input_rows(rows)}))
        .collect::<Vec<_>>());
    write_json(&a.output.join("audit-inputs.json"), &frozen)?;
    let mut results = Vec::new();
    for step in [0, 16, 32, 48, 64] {
        let bytes = fs::read(root.join(format!("checkpoint-{step:04}/native-compiler.json")))?;
        let sha = sha256_bytes(&bytes);
        let declared = parent_report["checkpoints"]
            .as_array()
            .ok_or_else(|| fail("parent lacks checkpoints"))?
            .iter()
            .filter(|c| c["step"] == step)
            .collect::<Vec<_>>();
        if declared.len() != 1 || declared[0]["artifact_sha256"] != sha {
            return Err(fail("saved checkpoint step/hash mismatches parent fit report").into());
        }
        let compiler = NativeTurnCompiler::load(&bytes, &sha, native, tokenizer, tokenizer_bytes)?;
        let meta: Value = serde_json::from_slice(&bytes)?;
        let mut activation = serde_json::Map::new();
        for (key, classes, slots) in [
            (
                "act_packed",
                4,
                meta["lanes"]
                    .as_u64()
                    .ok_or_else(|| fail("missing lanes"))? as usize,
            ),
            (
                "relation_packed",
                3,
                meta["lanes"]
                    .as_u64()
                    .ok_or_else(|| fail("missing lanes"))? as usize,
            ),
            (
                "span_packed",
                2,
                3 * meta["lanes"]
                    .as_u64()
                    .ok_or_else(|| fail("missing lanes"))? as usize,
            ),
        ] {
            let packed: Vec<u8> = serde_json::from_value(meta[key].clone())?;
            let coefficients = uor_r4_integer::geometric_potential_q4::unpack_coefficients(
                classes * (1 + slots * 120),
                &packed,
            )?;
            activation.insert(key.into(),json!({"nonzero":coefficients.iter().filter(|q|**q!=0).count(),"coefficients":coefficients.len()}));
        }
        let mut measurements = serde_json::Map::new();
        for (name, rows) in &panels {
            measurements.insert((*name).into(), evaluate(&compiler, rows)?);
        }
        results.push(json!({"step":step,"artifact_sha256":sha,"activation":activation,"panels":measurements}));
    }
    write_json(
        &a.output.join("checkpoint-measurements.json"),
        &json!(results),
    )?;
    let control = ReferenceRule::new(&sha256_bytes(tokenizer_bytes))?;
    let mut control_panels = serde_json::Map::new();
    for (name, rows) in &panels {
        control_panels.insert((*name).into(), evaluate(&control, rows)?);
    }
    let control_store = store_episodes(&control, tokenizer, native, "development", a)?;
    write_json(
        &a.output.join("report.json"),
        &json!({"schema":"native-compiler-zero-update-audit/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"parent_fit":root,"parent_report_sha256":sha256_bytes(&parent_report_bytes),"parent_manifest_sha256":parent_manifest_sha,"checkpoint_step_hashes_matched_parent":true,"fitted_input_rows_exactly_matched":true,"updates":0,"selection_changes":false,"fresh_used_for_design":false,"checkpoint_panels":results,"positive_control":{"label":"exact-template instrumentation control;not a learned language result","identity":control.identity(),"panels":control_panels,"store_reader":control_store},"original_report_erratum":"Fit-1 made zero store writes, reads and native reader calls. Mechanism-available strings in its sealed report did not mean execution; reader NOT_RUN. Original bytes retained.","scope":"candidate bound;one deterministic readout fit on one frozen carrier;not multi-seed architecture verdict"}),
    )?;
    report_output::seal(&a.output)?;
    report_output::verify(&a.output)?;
    println!("sealed zero-update audit {}", a.output.display());
    Ok(())
}

fn main() -> Result<()> {
    let a = args()?;
    if !a.output.is_absolute() {
        return Err(fail("output must be absolute").into());
    }
    let parent = a
        .output
        .parent()
        .ok_or_else(|| fail("output has no parent"))?;
    let output = fs::canonicalize(parent)?.join(
        a.output
            .file_name()
            .ok_or_else(|| fail("output needs basename"))?,
    );
    for input in [
        &a.native,
        &a.binding,
        &a.tokenizer,
        &a.cue,
        &a.prefix,
        &a.end,
    ] {
        let input = fs::canonicalize(input)?;
        if output.starts_with(&input) {
            return Err(fail("output is beneath input").into());
        }
        for ancestor in input.ancestors() {
            if ancestor.join(report_output::MANIFEST_FILE).is_file() && output.starts_with(ancestor)
            {
                return Err(fail("output is beneath sealed input").into());
            }
        }
    }
    if let Some(root) = &a.audit_artifacts {
        report_output::verify(root)?;
        let root = fs::canonicalize(root)?;
        if output.starts_with(root) {
            return Err(fail("audit output beneath sealed artifact").into());
        }
    }
    report_output::claim(&a.output)?;
    match run(&a) {
        Ok(()) => Ok(()),
        Err(error) => {
            write_json(
                &a.output.join("failure.json"),
                &json!({"status":"execution-failure","error":error.to_string(),"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"model_quality_evidence":false}),
            )?;
            report_output::seal(&a.output)?;
            report_output::verify(&a.output)?;
            Err(error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_template_control_passes_the_same_action_span_instrument() -> Result<()> {
        let control = ReferenceRule::new(&"0".repeat(64))?;
        for rows in [
            panel("training"),
            panel("development"),
            panel_cross("training", "development"),
            panel_cross("development", "training"),
        ] {
            let report = evaluate(&control, &rows)?;
            assert_eq!(report["complete_exact"], json!(rows.len()));
            assert_eq!(report["prose_false_writes"], json!(0));
        }
        Ok(())
    }
}
