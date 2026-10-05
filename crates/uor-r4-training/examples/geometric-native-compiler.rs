//! Bounded source-only learned native turn compiler and exact-store integration.
//! Exact addressed admission and all-bank native reads are reported separately.
#[path = "geometric_native_compiler/curriculum.rs"]
mod curriculum;
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
    geometric_turn_compiler::{
        fit_examples, FeatureMode, FitConfig, NativeTurnCompiler, SpanObjective,
    },
    relation_compiler::{word_spans, Example, NONE},
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
    feature_mode: FeatureMode,
    span_objective: SpanObjective,
    seed: Option<u64>,
    fresh_split: String,
    compact_trace: bool,
    curriculum: Option<String>,
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
    let curriculum = flags.remove("--curriculum");
    validate_curriculum_override(curriculum.as_deref(), audit_artifacts.is_some())?;
    if curriculum.is_some() && flags.contains_key("--fresh-profile") {
        return Err(fail("crossed curriculum owns its frozen fresh split").into());
    }
    let feature_mode = match flags.remove("--feature-mode").as_deref() {
        None | Some("endpoint") => FeatureMode::Endpoint,
        Some("local-relative") => FeatureMode::LocalRelative,
        Some("local-product") => FeatureMode::LocalProduct,
        Some("ordered-prefix-carrier") => FeatureMode::OrderedPrefixCarrier,
        Some("ordered-prefix-transport") => FeatureMode::OrderedPrefixTransport,
        _ => return Err(fail("unknown feature mode").into()),
    };
    let span_objective = parse_span_objective(flags.remove("--span-objective").as_deref())?;
    let seed = flags
        .remove("--seed")
        .map(|s| s.parse::<u64>())
        .transpose()?;
    let fresh_split = match flags.remove("--fresh-profile").as_deref() {
        None | Some("legacy") => "fresh",
        Some("local-1") => "fresh-local-1",
        _ => return Err(fail("unknown fresh profile").into()),
    }
    .to_owned();
    let compact_trace = flags
        .remove("--compact-trace")
        .map(|s| s.parse::<bool>())
        .transpose()?
        .unwrap_or(false);
    if audit_artifacts.is_some()
        && (feature_mode != FeatureMode::Endpoint
            || span_objective != SpanObjective::AllRows
            || seed.is_some()
            || fresh_split != "fresh")
    {
        return Err(
            fail("legacy checkpoint audit does not accept fit mode/seed/fresh overrides").into(),
        );
    }
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
        feature_mode,
        span_objective,
        seed,
        fresh_split,
        compact_trace,
        curriculum,
    })
}
fn parse_span_objective(value: Option<&str>) -> Result<SpanObjective> {
    match value {
        None | Some("all-rows") => Ok(SpanObjective::AllRows),
        Some("conditional-write") => Ok(SpanObjective::ConditionalWrite),
        _ => Err(fail("unknown span objective").into()),
    }
}
fn validate_curriculum_override(curriculum: Option<&str>, audit: bool) -> Result<()> {
    if curriculum.is_some_and(|v| !matches!(v, "crossed-1" | "crossed-2" | "crossed-3")) {
        return Err(fail("unknown curriculum").into());
    }
    if audit && curriculum.is_some() {
        return Err(fail("legacy checkpoint audit rejects curriculum override").into());
    }
    Ok(())
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
        "fresh-local-1" => (
            vec!["Velorin", "Nareth Cove", "Tarnbridge", "Opal Terrace"],
            vec!["For my {r}, please save {v}.", "My recorded {r} is {v}."],
            vec!["Please update my {r} to {v}.", "My {r} has changed to {v}."],
            vec![
                "Please tell me the {r} I saved.",
                "What is the current value of my {r}?",
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
        "fresh-local-1" => vec![
            "I enjoy quiet mornings.",
            "A job title can change.",
            "Many people share a home.",
            "The clouds moved slowly.",
            "Thanks for explaining that.",
            "There are several ways to think.",
            "I am reading a long novel.",
            "We can talk about another idea.",
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
fn verify_prospective_panel(rows: &[Example]) -> Result<()> {
    let mut exposed = std::collections::BTreeSet::new();
    let mut exposed_values = std::collections::BTreeSet::new();
    for split in ["training", "development", "fresh"] {
        for row in panel(split) {
            exposed.insert(row.text);
        }
        exposed_values.extend(pools(split).0);
    }
    let mut seen = std::collections::BTreeSet::new();
    if rows
        .iter()
        .any(|row| exposed.contains(&row.text) || !seen.insert(row.text.clone()))
        || pools("fresh-local-1")
            .0
            .iter()
            .any(|value| exposed_values.contains(value))
    {
        return Err(
            fail("prospective source rows or values overlap legacy compiler panels").into(),
        );
    }
    Ok(())
}
fn verify_crossed_fresh(rows: &[Example]) -> Result<()> {
    let mut exposed = std::collections::BTreeSet::new();
    let mut exposed_values = std::collections::BTreeSet::new();
    for split in ["training", "development", "fresh", "fresh-local-1"] {
        for e in panel(split) {
            if matches!(e.act, "assert" | "update") {
                if let Some(value) = e.slot_value() {
                    exposed_values.insert(value.to_owned());
                }
            }
            exposed.insert(e.text);
        }
    }
    exposed.extend(
        panel_cross("training", "development")
            .into_iter()
            .map(|e| e.text),
    );
    exposed.extend(
        panel_cross("development", "training")
            .into_iter()
            .map(|e| e.text),
    );
    exposed.extend(repeated_value_panel().into_iter().map(|e| e.text));
    let mut seen = std::collections::BTreeSet::new();
    if rows.iter().any(|e| {
        exposed.contains(&e.text)
            || !seen.insert(&e.text)
            || matches!(e.act, "assert" | "update")
                && e.slot_value().is_some_and(|v| exposed_values.contains(v))
    }) {
        return Err(
            fail("crossed fresh source overlaps an exposed compiler panel or itself").into(),
        );
    }
    Ok(())
}
fn source_admission(rows: &[Example], tokenizer: &ByteBpeTokenizer) -> Result<Value> {
    let mut receipts = Vec::new();
    for e in rows {
        let tokens = tokenizer.encode(&e.text);
        let words = word_spans(&e.text);
        if tokens.len() > 128 || words.is_empty() || words.len() > 64 {
            return Err(fail("actual curriculum BPE/word bound exceeded").into());
        }
        for word in &words {
            for source in [
                &e.text[..word.start],
                &e.text[..word.end],
                &e.text[word.start..word.end],
            ] {
                if tokenizer.encode(source).len() > 128 {
                    return Err(
                        fail("actual geometric feature substring exceeds token bound").into(),
                    );
                }
            }
        }
        let span = if matches!(e.act, "assert" | "update") {
            let (start, end) = e
                .slot_span()
                .ok_or_else(|| fail("write label needs original exact span"))?;
            let inside: Vec<_> = words
                .iter()
                .filter(|w| w.start < end && start < w.end)
                .collect();
            if inside.is_empty()
                || inside.len() > 8
                || inside[0].start != start
                || inside[inside.len() - 1].end != end
            {
                return Err(fail("write span cuts word edges or exceeds value8 bound").into());
            }
            Some((start, end))
        } else {
            None
        };
        receipts.push(json!({"source":e.text,"actual_source_token_ids":tokens,"words":words.len(),"write_span_labels_only":span}));
    }
    Ok(
        json!({"cases":rows.len(),"max_source_tokens":128,"max_source_words":64,"max_value_words":8,"model_calls":0,"rows":receipts}),
    )
}
fn repeated_value_panel() -> Vec<Example> {
    let (_, asserts, updates, _) = pools("training");
    let mut rows = Vec::new();
    for relation in ["job", "home"] {
        for (act, frame) in [("assert", asserts[0]), ("update", updates[0])] {
            for value in ["engineer engineer", "green valley green valley"] {
                rows.push(example(
                    relation,
                    act,
                    &frame.replace("{r}", relation),
                    value,
                ));
            }
        }
    }
    rows
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
        let gold_span = if matches!(e.act, "assert" | "update") {
            e.slot_span().map(|(start, end)| SourceSpan { start, end })
        } else {
            None
        };
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
fn compact_generation(output: &mut Value) -> Result<()> {
    let tokens = output["tokens"]
        .as_array_mut()
        .ok_or_else(|| fail("reader lacks token traces"))?;
    for token in tokens {
        let native = &token["native"];
        let bank = &native["prefix_bank"]["cue_bank"]["bank"];
        let end = &native["source_end"];
        let compact = json!({
            "actions":native["actions"],
            "bank_binding_sha256":bank["bank_binding_sha256"],
            "segments":bank["segments"],"candidates":bank["candidates"],
            "context":bank["context"],
            "cue_angular_indices":native["prefix_bank"]["cue_bank"]["carrier"]["angular_indices"],
            "prefix_relative_roots":native["prefix_bank"]["prefix"]["relative_roots"],
            "source_end":{"selected_bank_index":end["selected_bank_index"],"selected_source_index":end["selected_source_index"],"angular_indices":end["angular_indices"],"relative_roots":end["relative_roots"],"period_q24":end["period_q24"],"stop_q24":end["stop_q24"],"factual_joint_copy_q24":end["factual_joint_copy_q24"]}
        });
        if compact["actions"]["chosen_token_id"].is_null() || compact["segments"].is_null() {
            return Err(
                fail("compact trace would lose authoritative actions or source identity").into(),
            );
        }
        token["native"] = compact;
    }
    output["trace_scope"] = json!("authoritative final actions/aliases, actual own-prefix, admitted source/occurrence identities and selected geometry indices; duplicated sidecar metadata omitted; donor identities bound once in report");
    Ok(())
}
fn store_episodes(
    compiler: &dyn TurnCompiler,
    tokenizer: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
    split: &str,
    artifact_prefix: &str,
    a: &Args,
) -> Result<Value> {
    let (values, asserts, updates, queries) = pools(split);
    let mut episodes = Vec::new();
    for (episode, relation) in ["job", "home"].iter().enumerate() {
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
        let query_expected = turns
            .iter()
            .enumerate()
            .map(|(i, e)| {
                (e.act == "query").then(|| if i < 4 { values[0] } else { values[1] }.to_owned())
            })
            .collect();
        episodes.push(curriculum::StoreEpisode {
            id: format!("legacy-{split}-{episode}"),
            turns,
            query_expected,
        });
    }
    store_authored_episodes(
        compiler,
        tokenizer,
        native,
        split,
        artifact_prefix,
        a,
        &episodes,
    )
}
fn store_authored_episodes(
    compiler: &dyn TurnCompiler,
    tokenizer: &ByteBpeTokenizer,
    native: &NativeSourceRealizer,
    split: &str,
    artifact_prefix: &str,
    a: &Args,
    episodes: &[curriculum::StoreEpisode],
) -> Result<Value> {
    let mut rows = Vec::new();
    let (mut correct, mut queries_total, mut selected_complete, mut bank_complete) = (0, 0, 0, 0);
    let mut reader_calls = 0;
    for (episode, authored) in episodes.iter().enumerate() {
        if authored.turns.len() != authored.query_expected.len() || authored.turns.len() > 32 {
            return Err(fail("authored episode/expectation alignment or bound").into());
        }
        let mut store = StackStore::new(100 + episode as u64, 16)?;
        let mut journal = BTreeMap::<u64, (u64, Vec<u32>)>::new();
        let mut steps = Vec::new();
        for (index, e) in authored.turns.iter().enumerate() {
            let expected = authored.query_expected[index].as_deref();
            if (e.act == "query") != expected.is_some() {
                return Err(fail("query-only authored expected value contract").into());
            }
            let answers = expected.map(|v| FrozenAnswers {
                intent: RecordedValueIntent::Current,
                accepted: vec![v.to_owned(), format!("{v}.")],
            });
            if let Some(answers) = &answers {
                answers.validate()?;
            }
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
                        a.output.join(format!(
                            "{artifact_prefix}-{split}-episode-{episode}-step-{index}-store.bin"
                        )),
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
                    let hit = expected.is_some_and(|expected| {
                        tokens
                            .as_ref()
                            .is_some_and(|t| *t == tokenizer.encode(expected))
                    });
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
                        if a.compact_trace {
                            compact_generation(&mut output)?;
                        }
                        let ids: Vec<u32> =
                            serde_json::from_value(output["generated_ids_including_eos"].clone())?;
                        let eos = output["eos"] == true;
                        let plain = if eos { &ids[..ids.len() - 1] } else { &ids[..] };
                        let bytes = tokenizer.decode_bytes(plain);
                        let raw = String::from_utf8_lossy(&bytes);
                        let text = raw.strip_prefix(' ').unwrap_or(&raw);
                        let complete = eos
                            && String::from_utf8(bytes.clone()).is_ok()
                            && answers
                                .as_ref()
                                .is_some_and(|answers| answers.accepts(text));
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
        rows.push(json!({"episode":episode,"episode_id":authored.id,"relation_evaluation_label":authored.turns.iter().find(|e|e.act=="query").map(|e|&e.relation),"steps":steps,"final_store_history_sha256":store.history_sha256()?}));
    }
    Ok(
        json!({"episodes":rows,"query_rows":queries_total,"native_reader_calls":reader_calls,"exact_store_answers":correct,"selected_record_complete":selected_complete,"all_bank_complete":bank_complete,"policy":"predicted-address-and-source-span-only;serialize-reload-after-writes;original-statement-cues;gold-used-only-to-score"}),
    )
}
fn run(a: &Args) -> Result<()> {
    let started = Instant::now();
    let crossed = a
        .curriculum
        .as_deref()
        .map(curriculum::build_profile)
        .transpose()?;
    let (training, development, fresh) = if let Some(c) = &crossed {
        (c.training.clone(), c.development.clone(), c.fresh.clone())
    } else {
        (
            panel("training"),
            panel("development"),
            panel(&a.fresh_split),
        )
    };
    if crossed.is_none() && a.fresh_split == "fresh-local-1" {
        verify_prospective_panel(&fresh)?;
    }
    if training.len() > 512 || development.len() > 128 || fresh.len() > 64 {
        return Err(fail("panel cap exceeded").into());
    }
    if crossed.is_some() && (training.len() != 512 || development.len() != 128) {
        return Err(fail("crossed curriculum requires exact512 training/128 development").into());
    }
    let known_phrasing_new_values = crossed
        .as_ref()
        .map(|c| c.known_phrasing_new_values.clone())
        .unwrap_or_else(|| panel_cross("training", "development"));
    let new_phrasing_known_values = crossed
        .as_ref()
        .map(|c| c.new_phrasing_known_values.clone())
        .unwrap_or_else(|| panel_cross("development", "training"));
    let repeated_values = crossed
        .as_ref()
        .map(|c| c.repeated_values.clone())
        .unwrap_or_else(repeated_value_panel);
    let curriculum_receipt = crossed
        .as_ref()
        .map(|c| c.manifest.clone())
        .unwrap_or(json!({"mode":"legacy"}));
    let curriculum_sha = sha256_bytes(&serde_json::to_vec(&curriculum_receipt)?);
    let episode_inputs = |episodes: &[curriculum::StoreEpisode]| {
        json!(episodes.iter().map(|e|json!({"id":e.id,"turns":input_rows(&e.turns),"query_expected_evaluation_only":e.query_expected})).collect::<Vec<_>>())
    };
    let frozen_curriculum = crossed.as_ref().map(|c|json!({"manifest":c.manifest,"reference_templates_control_only":input_rows(&c.reference_templates),"development_store_episodes":episode_inputs(&c.development_episodes),"fresh_store_episodes":episode_inputs(&c.fresh_episodes)}));
    if crossed.is_some() {
        verify_crossed_fresh(&fresh)?;
    }
    let config = FitConfig {
        steps: 64,
        learning_rate: 0.03,
        max_tokens: 128,
        max_words: 64,
        max_value_words: 8,
        feature_mode: a.feature_mode,
        span_objective: a.span_objective,
        seed: a.seed,
    };
    let panel_bytes = serde_json::to_vec(
        &json!({"training":input_rows(&training),"development":input_rows(&development),"fresh":input_rows(&fresh),"curriculum":frozen_curriculum,"factors":{"known_phrasing_new_values":input_rows(&known_phrasing_new_values),"new_phrasing_known_values":input_rows(&new_phrasing_known_values),"repeated_values":input_rows(&repeated_values)}}),
    )?;
    let panel_sha = sha256_bytes(&panel_bytes);
    let inputs = json!({"schema":"native-turn-frozen-inputs/1","training":input_rows(&training),"development":input_rows(&development),"fresh":input_rows(&fresh),"config":config,"panel_sha256":panel_sha,"fresh_profile":a.curriculum.as_deref().unwrap_or(&a.fresh_split),"curriculum":a.curriculum,"curriculum_sha256":curriculum_sha,"curriculum_frozen":frozen_curriculum,"factor_inputs":{"known_phrasing_new_values":input_rows(&known_phrasing_new_values),"new_phrasing_known_values":input_rows(&new_phrasing_known_values),"repeated_values":input_rows(&repeated_values)},"fresh_exclusions":if crossed.is_some(){"crossed curriculum exclusion/provenance is bound in curriculum_frozen.manifest; no global encoder-corpus exclusion claim"}else{"local-1 excludes exact source rows and literal values from legacy compiler training/development/fresh;no claim of exclusion from every historical encoder corpus"},"limitations":"authored supervised panels;three seeded readouts on one frozen carrier are not independent encoder or architecture replications"});
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
    let mut admitted_rows = Vec::new();
    for rows in [
        &training,
        &development,
        &fresh,
        &known_phrasing_new_values,
        &new_phrasing_known_values,
        &repeated_values,
    ] {
        admitted_rows.extend(rows.iter().cloned());
    }
    if let Some(c) = &crossed {
        for episode in c.development_episodes.iter().chain(&c.fresh_episodes) {
            admitted_rows.extend(episode.turns.iter().cloned());
        }
    }
    let token_admission = source_admission(&admitted_rows, &tokenizer)?;
    let mut expected_value_admission = Vec::new();
    if let Some(c) = &crossed {
        for episode in c.development_episodes.iter().chain(&c.fresh_episodes) {
            if episode.turns.len() != episode.query_expected.len() {
                return Err(fail("frozen episode expectations not aligned").into());
            }
            for (step, (turn, expected)) in episode
                .turns
                .iter()
                .zip(&episode.query_expected)
                .enumerate()
            {
                if (turn.act == "query") != expected.is_some() {
                    return Err(fail("frozen episode query expectation contract").into());
                }
                if let Some(value) = expected {
                    let value_ids = tokenizer.encode(value);
                    let answer_ids = tokenizer.encode(&format!(" {value}."));
                    if value_ids.len() > 30 || answer_ids.len() + 1 > 32 {
                        return Err(fail(
                            "source-only expected value/Period/EOS exceeds declared generation32",
                        )
                        .into());
                    }
                    expected_value_admission.push(json!({"episode":episode.id,"step":step,"value_labels_only":value,"value_bpe_ids":value_ids,"complete_literal_period_bpe_ids_labels_only":answer_ids,"required_tokens_including_eos":answer_ids.len()+1,"generation_cap":32}));
                }
            }
        }
    }
    let reference = if let Some(c) = &crossed {
        ReferenceRule::new_with_templates(&tokenizer_sha, &c.reference_templates)?
    } else {
        ReferenceRule::new_with_splits(
            &tokenizer_sha,
            &["training", "development", &a.fresh_split],
        )?
    };
    let mut reference_admission = Vec::new();
    if crossed.is_some() {
        for (name, rows) in [
            ("training", &training),
            ("development", &development),
            ("fresh", &fresh),
            ("known_phrasing_new_values", &known_phrasing_new_values),
            ("new_phrasing_known_values", &new_phrasing_known_values),
            ("repeated_values", &repeated_values),
        ] {
            let evaluated = evaluate(&reference, rows)?;
            if evaluated["complete_exact"] != json!(rows.len())
                || evaluated["prose_false_writes"] != json!(0)
            {
                return Err(fail("reference compilation instrument failed before fit").into());
            }
            reference_admission.push(json!({"panel":name,"result":evaluated}));
        }
        if let Some(c) = &crossed {
            for episode in c.development_episodes.iter().chain(&c.fresh_episodes) {
                let evaluated = evaluate(&reference, &episode.turns)?;
                if evaluated["complete_exact"] != json!(episode.turns.len())
                    || evaluated["prose_false_writes"] != json!(0)
                {
                    return Err(fail("reference episode instrument failed before fit").into());
                }
                reference_admission.push(json!({"episode":episode.id,"result":evaluated}));
            }
        }
    }
    write_json(
        &a.output.join("curriculum-admission.json"),
        &json!({"source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"frozen_inputs_sha256":sha256_bytes(&input_bytes),"curriculum_sha256":curriculum_sha,"tokenizer_sha256":tokenizer_sha,"source_token_admission":token_admission,"expected_value_generation_admission_labels_only":expected_value_admission,"reference_compilation_admission":reference_admission,"reference_store_execution":"NOT_RUN at prefit admission","fit_updates":0}),
    )?;
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
    let mut factors = Vec::new();
    for checkpoint in &fit.checkpoint_artifacts {
        let c = NativeTurnCompiler::load(
            &checkpoint.native_artifact,
            &sha256_bytes(&checkpoint.native_artifact),
            &native,
            &tokenizer,
            &tokenizer_bytes,
        )?;
        factors.push(json!({"step":checkpoint.step,"artifact_sha256":sha256_bytes(&checkpoint.native_artifact),"training":evaluate(&c,&training)?,"development":evaluate(&c,&development)?,"known_phrasing_new_values":evaluate(&c,&known_phrasing_new_values)?,"new_phrasing_known_values":evaluate(&c,&new_phrasing_known_values)?,"repeated_values":evaluate(&c,&repeated_values)?}));
    }
    write_json(
        &a.output.join("checkpoint-factor-panels.json"),
        &json!(factors),
    )?;
    let dev = evaluate(&compiler, &development)?;
    let fresh_eval = evaluate(&compiler, &fresh)?;
    let (dev_store, fresh_store) = if let Some(c) = &crossed {
        (
            store_authored_episodes(
                &compiler,
                &tokenizer,
                &native,
                "development",
                "learned",
                a,
                &c.development_episodes,
            )?,
            store_authored_episodes(
                &compiler,
                &tokenizer,
                &native,
                "fresh",
                "learned",
                a,
                &c.fresh_episodes,
            )?,
        )
    } else {
        (
            store_episodes(&compiler, &tokenizer, &native, "development", "learned", a)?,
            store_episodes(&compiler, &tokenizer, &native, &a.fresh_split, "learned", a)?,
        )
    };
    let reference_store = if let Some(c) = &crossed {
        json!({"development":store_authored_episodes(&reference, &tokenizer, &native, "development", "control", a, &c.development_episodes)?,
               "fresh":store_authored_episodes(&reference, &tokenizer, &native, "fresh", "control", a, &c.fresh_episodes)?})
    } else {
        json!({"status":"NOT_RUN","scope":"legacy fit preserves prior reference compilation-only control"})
    };
    let positive = json!({"label":"authored exact-template instrumentation control;not learned model or serving fallback","identity":reference.identity(),"training":evaluate(&reference,&training)?,"development":evaluate(&reference,&development)?,"fresh":evaluate(&reference,&fresh)?});
    let donor_identities = json!({"cue":serde_json::from_slice::<Value>(&fs::read(a.cue.join("native-metadata.json"))?)?,"prefix":serde_json::from_slice::<Value>(&fs::read(a.prefix.join("native-metadata.json"))?)?,"source_end":serde_json::from_slice::<Value>(&fs::read(a.end.join("native-metadata.json"))?)?});
    write_json(
        &a.output.join("report.json"),
        &json!({"schema":"geometric-native-compiler-report/1","source_commit":option_env!("UOR_BUILD_SOURCE_COMMIT"),"host":std::env::consts::ARCH,"elapsed_seconds":started.elapsed().as_secs_f64(),"binding_receipt_sha256":binding_sha,"tokenizer_sha256":tokenizer_sha,"binding_trust":if a.binding_sha.is_some(){"explicit-pinned-sha"}else{"caller-supplied-trusted-receipt"},"frozen_inputs_sha256":sha256_bytes(&input_bytes),"feature_mode":a.feature_mode,"span_objective":a.span_objective,"learning_seed":a.seed,"seed_scope":"one frozen learned encoder;independent initialized Q4 readouts only","panel_sha256":panel_sha,"fresh_profile":a.curriculum.as_deref().unwrap_or(&a.fresh_split),"curriculum":a.curriculum,"curriculum_sha256":curriculum_sha,"curriculum_manifest":curriculum_receipt,"compact_trace":a.compact_trace,"reference_store_execution":reference_store,"reference_native_reader_calls":reference_store["development"]["native_reader_calls"].as_u64().unwrap_or(0)+reference_store["fresh"]["native_reader_calls"].as_u64().unwrap_or(0),"instrument_positive_control":positive,"reader_donors":donor_identities,"transport_cost":"local-relative inverse+compose two group table operations per present lane;local-product compose one;ordered-prefix modes retain local-product base plus one compose/lane/word;carrier adds unary prefix read,transport adds inverse+compose/present prefix lane;same paired information/slots,not identical group-operation count","selected_step":fit.selected_step,"selected_artifact_sha256":selected_sha,"compiler_identity":compiler.identity(),"checkpoints":fit.checkpoints,"diagnostics":fit.diagnostics,"development":dev,"fresh":fresh_eval,"development_store":dev_store,"fresh_store":fresh_store,"geometric_reader_available":"selected-record-and-all-bank","native_reader_calls":dev_store["native_reader_calls"].as_u64().unwrap_or(0)+fresh_store["native_reader_calls"].as_u64().unwrap_or(0),"native_emission_status":if dev_store["native_reader_calls"]==0 && fresh_store["native_reader_calls"]==0 {"NOT_RUN"}else{"executed"},"claims":"authored native compiler/store/reader episodes;encoder frozen;authored panels;no general prose/reasoning/energy qualification"}),
    )?;
    report_output::seal(&a.output)?;
    report_output::verify(&a.output)?;
    println!("sealed {}", a.output.display());
    Ok(())
}

// Deliberately explicit instrumentation control, not a learned model or serving fallback.
pub(super) struct ReferenceRule {
    identity: CompilerIdentity,
    bytes: Vec<u8>,
    writes: Vec<(String, String, u32, bool)>,
    queries: Vec<(String, u32)>,
}
impl ReferenceRule {
    fn new(tokenizer_sha: &str) -> Result<Self> {
        Self::new_with_splits(tokenizer_sha, &["training", "development"])
    }
    fn new_with_splits(tokenizer_sha: &str, splits: &[&str]) -> Result<Self> {
        let mut writes = Vec::new();
        let mut queries = Vec::new();
        for update in [true, false] {
            for split in splits {
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
        Self::from_templates(tokenizer_sha, writes, queries)
    }
    pub(super) fn new_with_templates(tokenizer_sha: &str, examples: &[Example]) -> Result<Self> {
        let mut writes = Vec::new();
        let mut queries = Vec::new();
        for e in examples {
            let relation = match e.relation.as_str() {
                "job" => 1,
                "home" => 2,
                NONE => continue,
                _ => return Err(fail("reference template unknown relation").into()),
            };
            if e.act == "query" {
                let row = (e.text.clone(), relation);
                if !queries.contains(&row) {
                    queries.push(row);
                }
            } else if e.act == "assert" || e.act == "update" {
                let template = e
                    .template
                    .as_deref()
                    .ok_or_else(|| fail("reference write lacks explicit frame"))?;
                let (prefix, suffix) = template
                    .split_once("{v}")
                    .ok_or_else(|| fail("reference frame lacks value"))?;
                if suffix.contains("{v}") {
                    return Err(fail("reference frame must have one exact source span").into());
                }
                let row = (
                    prefix.to_owned(),
                    suffix.to_owned(),
                    relation,
                    e.act == "update",
                );
                if !writes.contains(&row) {
                    writes.push(row);
                }
            }
        }
        // Explicit instrumentation prefers the most specific authored frame.
        // It is never consulted by the learned compiler or native reader.
        writes.sort_by(|a, b| {
            (b.0.len() + b.1.len())
                .cmp(&(a.0.len() + a.1.len()))
                .then_with(|| b.3.cmp(&a.3))
        });
        Self::from_templates(tokenizer_sha, writes, queries)
    }
    fn from_templates(
        tokenizer_sha: &str,
        writes: Vec<(String, String, u32, bool)>,
        queries: Vec<(String, u32)>,
    ) -> Result<Self> {
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
    let control_store = store_episodes(&control, tokenizer, native, "development", "control", a)?;
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
    for ancestor in output.ancestors() {
        if ancestor.join(report_output::MANIFEST_FILE).is_file() {
            return Err(fail("output is beneath sealed report").into());
        }
    }
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
    fn explicit_span_objective_and_crossed2_profile_admit() -> Result<()> {
        assert_eq!(parse_span_objective(None)?, SpanObjective::AllRows);
        assert_eq!(
            parse_span_objective(Some("conditional-write"))?,
            SpanObjective::ConditionalWrite
        );
        assert!(parse_span_objective(Some("predicted-write")).is_err());
        validate_curriculum_override(Some("crossed-2"), false)?;
        assert!(validate_curriculum_override(Some("crossed-2"), true).is_err());
        let c = curriculum::build_profile("crossed-2")?;
        verify_crossed_fresh(&c.fresh)?;
        let reference = ReferenceRule::new_with_templates(&"0".repeat(64), &c.reference_templates)?;
        for rows in [
            &c.training,
            &c.development,
            &c.fresh,
            &c.known_phrasing_new_values,
            &c.new_phrasing_known_values,
            &c.repeated_values,
        ] {
            assert_eq!(
                evaluate(&reference, rows)?["complete_exact"],
                json!(rows.len())
            );
        }
        for episode in c.development_episodes.iter().chain(&c.fresh_episodes) {
            assert_eq!(
                evaluate(&reference, &episode.turns)?["complete_exact"],
                json!(episode.turns.len())
            );
        }
        Ok(())
    }
    #[test]
    fn crossed_fresh_exclusion_rejects_actual_legacy_sources() -> Result<()> {
        let c = curriculum::build()?;
        verify_crossed_fresh(&c.fresh)?;
        assert!(verify_crossed_fresh(&panel("fresh-local-1")).is_err());
        Ok(())
    }
    #[test]
    fn crossed_override_rejects_checkpoint_audit_and_unknown_curriculum() -> Result<()> {
        validate_curriculum_override(None, true)?;
        validate_curriculum_override(Some("crossed-1"), false)?;
        assert!(validate_curriculum_override(Some("crossed-1"), true).is_err());
        assert!(validate_curriculum_override(Some("unknown"), false).is_err());
        Ok(())
    }
    #[test]
    fn none_value_provenance_is_not_a_gold_write_span_or_control_rule() -> Result<()> {
        let none = example(
            NONE,
            NONE,
            "I mentioned {v} while thinking.",
            "singer singer",
        );
        assert!(none.slot_span().is_some());
        let control = ReferenceRule::new_with_templates(&"0".repeat(64), &[none.clone()])?;
        let result = evaluate(&control, &[none])?;
        assert_eq!(result["complete_exact"], json!(1));
        assert_eq!(result["prose_false_writes"], json!(0));
        Ok(())
    }
    #[test]
    fn explicit_control_uses_original_exact_span_and_full_template() -> Result<()> {
        let assertion = example(
            "job",
            "assert",
            "Please save {v} as my job.",
            "singer singer",
        );
        let correction = example(
            "job",
            "update",
            "For my job instead record {v}.",
            "dancer dancer",
        );
        let query = example("job", "query", "What job do I currently have?", "");
        let rows = vec![assertion, correction, query];
        let control = ReferenceRule::new_with_templates(&"0".repeat(64), &rows)?;
        let result = evaluate(&control, &rows)?;
        assert_eq!(result["complete_exact"], json!(3));
        assert!(matches!(
            control.compile("Other job singer.")?,
            CompiledAction::Unresolved { .. }
        ));
        Ok(())
    }
    #[test]
    fn prospective_local_panel_is_excluded_and_reference_solvable() -> Result<()> {
        let rows = panel("fresh-local-1");
        verify_prospective_panel(&rows)?;
        assert_eq!(rows.len(), 44);
        let control = ReferenceRule::new_with_splits(
            &"0".repeat(64),
            &["training", "development", "fresh-local-1"],
        )?;
        let report = evaluate(&control, &rows)?;
        assert_eq!(report["complete_exact"], json!(44));
        assert_eq!(report["write_complete_exact"], json!(32));
        assert_eq!(report["prose_false_writes"], json!(0));
        assert!(verify_prospective_panel(&panel("fresh")).is_err());
        Ok(())
    }
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
