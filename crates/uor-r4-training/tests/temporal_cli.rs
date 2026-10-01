//! Actual CLI construction witness; initialized emitter, authored training rows.
//! This test does not qualify unseen temporal language or complete answers.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use candle_core::Device;
use serde_json::{json, Value};
use uor_r4_core::native_geometric::learner::realtext_support::sha256_hex;
use uor_r4_core::report_output;
use uor_r4_tokenizer::ByteBpeTokenizer;
use uor_r4_training::geometric_stack::{
    PointerConfig, ReadScore, StackArch, StackConfig, StackModel, TransportSnap,
};
use uor_r4_training::relation_compiler::{CompilerSettings, Example, SavedCompiler};
use uor_r4_training::stack_checkpoint::{
    save_checkpoint, sealed_manifest_sha256, CheckpointIdentity, DataIdentity,
};
use uor_r4_training::stack_grounded_session::{
    CompiledAction, ContextPolicy, GroundedSession, MemoryEffect, SessionLimits, SessionScope,
};
use uor_r4_training::stack_store::{HistoryView, StackStore, StoreRead};
use uor_r4_training::temporal_compiler::GroundedCompiler;

fn tokenizer_bytes() -> Vec<u8> {
    let mut printable: Vec<u32> = (u32::from(b'!')..=u32::from(b'~')).collect();
    printable.extend(0xA1..=0xAC);
    printable.extend(0xAE..=0xFF);
    let mut vocab = serde_json::Map::new();
    let specials = ["<|bos|>", "<|eos|>", "<|unk|>"];
    for (id, token) in specials.iter().enumerate() {
        vocab.insert((*token).into(), json!(id));
    }
    let mut extra = 0;
    for byte in 0u32..256 {
        let code = if printable.contains(&byte) {
            byte
        } else {
            extra += 1;
            255 + extra
        };
        vocab.insert(
            char::from_u32(code).expect("alphabet").to_string(),
            json!(byte + 3),
        );
    }
    let added: Vec<Value> = specials
        .iter()
        .enumerate()
        .map(|(id, token)| json!({"id":id,"content":token}))
        .collect();
    serde_json::to_vec(&json!({
        "pre_tokenizer":{"type":"ByteLevel","add_prefix_space":false},
        "added_tokens":added,
        "model":{"type":"BPE","vocab":vocab,"merges":[]}
    }))
    .expect("tokenizer")
}

fn invoke(executable: &str, arguments: &[String]) -> Value {
    let output = Command::new(executable)
        .args(arguments)
        .output()
        .expect("spawn actual CLI");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI JSON")
}

fn pair(key: &str, path: &Path) -> String {
    format!("{key}={}", path.display())
}

#[test]
fn fitted_adapter_drives_actual_clis_and_each_fresh_process_matches_generation() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let base =
        std::env::temp_dir().join(format!("uor-temporal-cli-{}-{nonce}", std::process::id()));
    fs::create_dir(&base).expect("exclusive fixture");
    let tokenizer_json = tokenizer_bytes();
    let tokenizer =
        ByteBpeTokenizer::from_tokenizer_json_bytes(&tokenizer_json).expect("tokenizer");
    let tokenizer_path = base.join("tokenizer.json");
    fs::write(&tokenizer_path, &tokenizer_json).expect("tokenizer file");
    let queries = [
        ("What is my name now?", "current"),
        ("What was the first name I told you?", "initial"),
        (
            "What name did I tell you immediately before?",
            "previous_assertion",
        ),
        (
            "What different name did I tell you before?",
            "previous_distinct_value",
        ),
        ("What could my name have been?", "unresolved"),
    ];
    let mut train = Vec::new();
    for (template, act) in [
        ("My name is {v}.", "assert"),
        ("Actually, my name is {v}.", "update"),
    ] {
        for value in ["Zorvak", "Plimbo", "Dunmere"] {
            train.push(Example {
                text: template.replace("{v}", value),
                relation: "user_name".into(),
                act,
                template: Some(template.into()),
            });
        }
    }
    for (text, _) in queries {
        train.push(Example {
            text: text.into(),
            relation: "user_name".into(),
            act: "query",
            template: Some(text.into()),
        });
    }
    for value in ["Zorvak", "Plimbo", "Dunmere"] {
        train.push(Example {
            text: format!("I live in {value}."),
            relation: "home".into(),
            act: "assert",
            template: Some("I live in {v}.".into()),
        });
    }
    let frozen = SavedCompiler::fit(
        &train,
        &sha256_hex(&tokenizer_json),
        json!({"fixture":"authored CLI construction"}),
        CompilerSettings::default(),
    )
    .expect("base fit");
    let frozen_path = base.join("base.json");
    fs::write(&frozen_path, frozen.bytes()).expect("base file");
    let rows = queries
        .iter()
        .map(|(text, view)| {
            json!({
                "text":text,"relation":if *view == "unresolved" {None} else {Some("user_name")},
                "view":view,"source_group":"authored-cli-construction"
            })
            .to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let rows_path = base.join("train.jsonl");
    fs::write(&rows_path, rows).expect("rows");
    let fit = base.join("fit");
    invoke(
        env!("CARGO_BIN_EXE_temporal-compiler"),
        &[
            "fit".into(),
            pair("out", &fit),
            pair("base", &frozen_path),
            pair("train", &rows_path),
        ],
    );
    report_output::verify(&fit).expect("sealed fit");
    let composite_path = fit.join("compiler.json");
    let compiler = GroundedCompiler::from_bytes(fs::read(&composite_path).expect("artifact"))
        .expect("independent load");
    assert_eq!(compiler.base().bytes(), frozen.bytes());
    // A wrong base relation remains a composite miss even when the view is right.
    let evaluation = base.join("evaluation.jsonl");
    fs::write(&evaluation, json!({"text":queries[0].0,"relation":"home","view":"current","source_group":"intentional-wrong-relation-control"}).to_string()).expect("evaluation");
    let score = base.join("score");
    invoke(
        env!("CARGO_BIN_EXE_temporal-compiler"),
        &[
            "score".into(),
            pair("out", &score),
            pair("compiler", &composite_path),
            pair("data", &evaluation),
        ],
    );
    report_output::verify(&score).expect("sealed score");
    let report: Value =
        serde_json::from_slice(&fs::read(score.join("report.json")).expect("report"))
            .expect("report JSON");
    assert_eq!(report["score"]["rows"][0]["joint_pass"], false);
    let name_id = frozen.relation_id("user_name").expect("name relation");
    assert_eq!(
        report["score"]["rows"][0]["base_action"],
        json!({"kind":"query_current","relation":name_id})
    );
    assert_eq!(report["score"]["rows"][0]["view_prediction"], "current");
    assert_eq!(
        report["score"]["view_head_diagnostic"],
        json!({"pass":1,"of":1})
    );
    assert_eq!(report["score"]["joint"], json!({"pass":0,"of":1}));

    let training_root = base.join("model-provenance");
    report_output::claim(&training_root).expect("claim provenance");
    fs::write(training_root.join("fixture.json"), b"{}").expect("provenance");
    report_output::seal(&training_root).expect("seal provenance");
    let identity = CheckpointIdentity::from_tokenizer(
        &tokenizer_json,
        vec![DataIdentity {
            label: "TEST_ONLY".into(),
            bytes: 7,
            sha256: sha256_hex(b"fixture"),
        }],
        sealed_manifest_sha256(&training_root).expect("manifest"),
    )
    .expect("identity");
    let mut model = StackModel::new(
        StackConfig {
            arch: StackArch::Geometric,
            vocab_size: 259,
            width: 8,
            heads: 2,
            mlp_hidden: 16,
            context: 256,
            pattern: "ra".into(),
            read: ReadScore::Lorentz,
            rotation: true,
            seed: 7,
            memory: None,
            select: None,
            pointer: Some(PointerConfig::new(4)),
        },
        &Device::Cpu,
    )
    .expect("model");
    model
        .set_transport_snap(Some(TransportSnap::Icosian))
        .expect("snap");
    let checkpoint = base.join("checkpoint");
    save_checkpoint(
        &checkpoint,
        &model,
        &identity,
        Some(&StackStore::new(41, 8).expect("store")),
    )
    .expect("checkpoint");
    let limits = SessionLimits {
        max_new_tokens: 2,
        max_turns: 32,
        max_source_bytes: 4096,
        max_history_tokens: 4096,
        max_store_records: 16,
        context_policy: ContextPolicy::WholeCompletedTurns,
    };
    let mut expected = GroundedSession::from_checkpoint_path(
        &checkpoint,
        tokenizer_json.clone(),
        compiler,
        SessionScope {
            scope: b"cli-test".to_vec(),
            entity: tokenizer.encode("user"),
        },
        limits,
        &Device::Cpu,
    )
    .expect("session");
    let mut root = base.join("session-0");
    invoke(
        env!("CARGO_BIN_EXE_grounded-session"),
        &[
            "init".into(),
            pair("out", &root),
            pair("checkpoint", &checkpoint),
            pair("tokenizer", &tokenizer_path),
            pair("compiler", &composite_path),
            "scope=cli-test".into(),
            "max_new_tokens=2".into(),
            "max_turns=32".into(),
            "max_source_bytes=4096".into(),
            "max_history_tokens=4096".into(),
            "max_store_records=16".into(),
        ],
    );
    let sources = [
        "My name is Zorvak.",
        "Actually, my name is Plimbo.",
        "Actually, my name is Dunmere.",
        "My name is Dunmere.",
        queries[0].0,
        queries[1].0,
        queries[2].0,
        queries[3].0,
        queries[4].0,
    ];
    for (index, source) in sources.iter().enumerate() {
        let outcome = expected.turn(source).expect("actual in-process generation");
        if (4..8).contains(&index) {
            let view = [
                HistoryView::Current,
                HistoryView::Initial,
                HistoryView::PreviousAssertion,
                HistoryView::PreviousDistinctValue,
            ][index - 4];
            let wanted = if view == HistoryView::Current {
                CompiledAction::QueryCurrent { relation: name_id }
            } else {
                CompiledAction::Query {
                    relation: name_id,
                    view,
                }
            };
            assert_eq!(outcome.action, wanted);
            let record = [4, 1, 3, 2][index - 4];
            let value = ["Dunmere", "Zorvak", "Dunmere", "Plimbo"][index - 4];
            assert!(
                matches!(&outcome.memory, MemoryEffect::Read {read:StoreRead::Found(actual)}
                if actual.record == record && actual.tokens == tokenizer.encode(value) && !actual.conflict)
            );
        } else if index == 8 {
            assert!(matches!(outcome.action, CompiledAction::Unresolved { .. }));
            assert_eq!(outcome.memory, MemoryEffect::Unresolved);
        }
        let next = base.join(format!("session-{}", index + 1));
        let actual = invoke(
            env!("CARGO_BIN_EXE_grounded-session"),
            &[
                "turn".into(),
                pair("session", &root),
                pair("out", &next),
                format!("text={source}"),
            ],
        );
        assert_eq!(
            actual["action"],
            serde_json::to_value(&outcome.action).expect("action")
        );
        assert_eq!(
            actual["memory"],
            serde_json::to_value(&outcome.memory).expect("memory")
        );
        assert_eq!(actual["reply"], outcome.reply_text);
        assert_eq!(
            actual["recall"],
            serde_json::to_value(&outcome.recall).expect("recall")
        );
        let persisted: Value = serde_json::from_slice(
            &fs::read(next.join("session.json")).expect("saved CLI envelope"),
        )
        .expect("saved envelope JSON");
        assert_eq!(
            persisted["turns"][index],
            serde_json::to_value(&outcome).expect("complete turn including exact generated IDs")
        );
        let loaded = GroundedSession::load(
            &next,
            GroundedCompiler::from_bytes(
                fs::read(next.join("compiler.bin")).expect("saved composite"),
            )
            .expect("load composite"),
            &Device::Cpu,
        )
        .expect("validate saved envelope");
        assert_eq!(loaded.turns(), expected.turns());
        assert_eq!(loaded.history_ids(), expected.history_ids());
        root = next;
    }
    // Every turn was a different process, loading the composite from compiler.bin.
    let parent = root.clone();
    let parent_manifest = fs::read(parent.join(report_output::MANIFEST_FILE)).expect("parent seal");
    let retained_store = expected.store().to_bytes().expect("retained store");
    // Exact scope and entity vary independently. Every restart uses the actual
    // composite compiler and model; none receives a gold action or query view.
    for (index, (scope_text, entity_text, wanted_value)) in [
        ("cli-test", "user", Some("Dunmere")),
        ("other-project", "user", None),
        ("cli-test", "other-user", None),
        ("other-project", "other-user", None),
        ("cli-test", "user", Some("Dunmere")),
    ]
    .iter()
    .enumerate()
    {
        let scope = SessionScope {
            scope: scope_text.as_bytes().to_vec(),
            entity: tokenizer.encode(entity_text),
        };
        expected.start_conversation(scope.clone()).expect("restart");
        assert!(expected.turns().is_empty());
        assert_eq!(expected.history_ids(), &[0]);
        assert_eq!(expected.store().to_bytes().expect("store"), retained_store);
        let restarted = base.join(format!("restarted-{index}"));
        let receipt = invoke(
            env!("CARGO_BIN_EXE_grounded-session"),
            &[
                "restart".into(),
                pair("session", &root),
                pair("out", &restarted),
                format!("scope={scope_text}"),
                format!("entity={entity_text}"),
            ],
        );
        assert_eq!(
            receipt["scope"],
            serde_json::to_value(scope).expect("scope")
        );
        assert_eq!(receipt["turns"], 0);
        assert_eq!(receipt["memory_commit"], 4);
        let loaded = load_session(&restarted);
        assert!(loaded.turns().is_empty());
        assert_eq!(loaded.history_ids(), &[0]);
        assert_eq!(
            loaded.store().to_bytes().expect("loaded store"),
            retained_store
        );
        let outcome = expected
            .turn(queries[0].0)
            .expect("actual restarted emission");
        match wanted_value {
            Some(value) => assert!(matches!(&outcome.memory,
                MemoryEffect::Read { read: StoreRead::Found(found) }
                if found.record == 4 && found.tokens == tokenizer.encode(value))),
            None => assert_eq!(
                outcome.memory,
                MemoryEffect::Read {
                    read: StoreRead::Absent
                }
            ),
        }
        // The first input of each conversation excludes old user sources,
        // old recall values and replies, even when durable memory supplies A.
        let input = tokenizer.decode(&outcome.emitter_input_ids);
        assert!(!input.contains("Zorvak"));
        assert!(!input.contains("Plimbo"));
        assert!(!input.contains("My name is"));
        assert!(!input.contains("What was the first"));
        let continued = base.join(format!("restarted-turn-{index}"));
        invoke(
            env!("CARGO_BIN_EXE_grounded-session"),
            &[
                "turn".into(),
                pair("session", &restarted),
                pair("out", &continued),
                format!("text={}", queries[0].0),
            ],
        );
        let loaded = load_session(&continued);
        assert_eq!(loaded.turns(), expected.turns());
        assert_eq!(loaded.history_ids(), expected.history_ids());
        root = continued;
    }
    // Default identity is retained; attempts to overwrite an envelope or use
    // an invalid identity fail without changing the sealed parent.
    let reset_default = base.join("restart-default");
    invoke(
        env!("CARGO_BIN_EXE_grounded-session"),
        &[
            "restart".into(),
            pair("session", &root),
            pair("out", &reset_default),
        ],
    );
    assert_eq!(load_session(&reset_default).scope(), expected.scope());
    for (out, extra) in [
        (base.join("invalid-scope"), "scope="),
        (base.join("invalid-entity"), "entity="),
        (root.clone(), "scope=cli-test"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_grounded-session"))
            .args([
                "restart".into(),
                pair("session", &root),
                pair("out", &out),
                extra.into(),
            ])
            .output()
            .expect("invalid restart process");
        assert!(!output.status.success());
        report_output::verify(&root).expect("parent unchanged");
    }
    assert_eq!(
        fs::read(parent.join(report_output::MANIFEST_FILE)).expect("parent seal"),
        parent_manifest
    );
    report_output::verify(&parent).expect("original parent preserved");
    scripted_evaluation_witness(&base, &base.join("session-0"), &queries);
    fs::remove_dir_all(base).expect("clean generated test fixture");
}

fn load_session(root: &Path) -> GroundedSession<GroundedCompiler> {
    GroundedSession::load(
        root,
        GroundedCompiler::from_bytes(fs::read(root.join("compiler.bin")).expect("compiler"))
            .expect("saved compiler"),
        &Device::Cpu,
    )
    .expect("validate restarted session")
}

/// Golden annotations are authored from this event sequence before execution.
/// The initialized emitter has a two-token cap; complete names cannot fit.
/// Answer membership must therefore report misses, independently of correct
/// compilation/reads. A miss here is intentional scorer-construction evidence.
fn scripted_evaluation_witness(base: &Path, baseline: &Path, queries: &[(&str, &str)]) {
    let scope = json!({"scope":"cli-test","entity":"user"});
    let restart = || json!({"kind":"start_conversation","scope":scope});
    let write = |id: &str,
                 text: &str,
                 kind: &str,
                 value: &str,
                 record: u64,
                 enabled: bool,
                 conflict: bool| {
        let start = text.find(value).expect("authored value role");
        json!({
            "kind":"user","id":id,"text":text,
            "controls":{"read":true,"write":enabled},
            "expected":{
                "action":{"kind":kind,"relation":"user_name","span":{"start":start,"end":start+value.len()}},
                "memory":if enabled {
                    json!({"kind":"write","value":value,"id":record,"commit":record,"conflict":conflict})
                } else {json!({"kind":"write_disabled","value":value})},
                "answers":null,"intervention_answers":null
            }
        })
    };
    let query = |id: &str, index: usize, status: Value, answer: &str, enabled: bool| {
        let view = [
            "Current",
            "Initial",
            "PreviousAssertion",
            "PreviousDistinctValue",
        ][index];
        json!({
            "kind":"user","id":id,"text":queries[index].0,
            "controls":{"read":enabled,"write":true},
            "expected":{
                "action":{"kind":"query","relation":"user_name","view":view},
                "memory":{"kind":"read","status":status},
                "answers":{"intent":queries[index].1,"accepted":[format!("{answer}.")]},
                "intervention_answers":null
            }
        })
    };
    let found = |value: &str, record: u64, conflict: bool| json!({"kind":"found","value":value,"record":record,"commit":record,"conflict":conflict});
    let case = |id: &str, condition: &str, events: Vec<Value>, cuts: Vec<&str>| {
        json!({
            "id":id,"source_group":"authored-script-construction","condition":condition,
            "scope":scope,"baseline_commit":0,"baseline_records":0,
            "events":events,"reload_after":cuts
        })
    };
    let prefix = vec![
        write(
            "a",
            "My name is Zorvak.",
            "assert",
            "Zorvak",
            1,
            true,
            false,
        ),
        write(
            "b",
            "Actually, my name is Plimbo.",
            "correct",
            "Plimbo",
            2,
            true,
            false,
        ),
        write(
            "b-again",
            "My name is Plimbo.",
            "assert",
            "Plimbo",
            3,
            true,
            false,
        ),
        write(
            "c",
            "Actually, my name is Dunmere.",
            "correct",
            "Dunmere",
            4,
            true,
            false,
        ),
        write(
            "c-again",
            "My name is Dunmere.",
            "assert",
            "Dunmere",
            5,
            true,
            false,
        ),
    ];
    let mut cases = Vec::new();
    for (id, clean, read) in [
        ("temporal-continuous", false, true),
        ("temporal-clean", true, true),
        ("temporal-no-read-clean", true, false),
    ] {
        let mut events = prefix.clone();
        for (index, (value, record)) in
            [("Dunmere", 5), ("Zorvak", 1), ("Dunmere", 4), ("Plimbo", 3)]
                .into_iter()
                .enumerate()
        {
            if clean {
                events.push(restart());
            }
            events.push(query(
                &format!("q{index}"),
                index,
                found(value, record, false),
                value,
                read,
            ));
        }
        cases.push(case(id, id, events, vec!["b-again", "q1"]));
    }
    let mut suppressed = vec![
        write(
            "a",
            "My name is Zorvak.",
            "assert",
            "Zorvak",
            1,
            true,
            false,
        ),
        write(
            "b",
            "Actually, my name is Plimbo.",
            "correct",
            "Plimbo",
            2,
            true,
            false,
        ),
        write(
            "c-disabled",
            "Actually, my name is Dunmere.",
            "correct",
            "Dunmere",
            0,
            false,
            false,
        ),
        restart(),
    ];
    let mut probe = query("q", 0, found("Plimbo", 2, false), "Dunmere", true);
    probe["expected"]["intervention_answers"] = json!({"intent":"current","accepted":["Plimbo."]});
    suppressed.push(probe);
    cases.push(case(
        "selected-update-disabled",
        "selected_write_disabled_clean",
        suppressed,
        vec!["c-disabled"],
    ));
    let mut identity_events = vec![
        write(
            "a",
            "My name is Zorvak.",
            "assert",
            "Zorvak",
            1,
            true,
            false,
        ),
        json!({"kind":"start_conversation","scope":{"scope":"other-project","entity":"user"}}),
        write(
            "b",
            "My name is Plimbo.",
            "assert",
            "Plimbo",
            2,
            true,
            false,
        ),
        query("b-read", 0, found("Plimbo", 2, false), "Plimbo", true),
        json!({"kind":"start_conversation","scope":{"scope":"cli-test","entity":"other-user"}}),
        write(
            "c",
            "My name is Dunmere.",
            "assert",
            "Dunmere",
            3,
            true,
            false,
        ),
        query("c-read", 0, found("Dunmere", 3, false), "Dunmere", true),
        restart(),
        query("a-return", 0, found("Zorvak", 1, false), "Zorvak", true),
    ];
    cases.push(case(
        "scope-entity-return",
        "explicit_caller_identities",
        std::mem::take(&mut identity_events),
        vec!["c"],
    ));
    let mut absent = query("absent", 0, json!({"kind":"absent"}), "Unknown", true);
    absent["expected"]["answers"]["intent"] = json!("absent");
    cases.push(case("absent", "clean_absence", vec![absent], vec![]));
    let mut no_history = query(
        "no-history",
        2,
        json!({"kind":"no_history"}),
        "Unknown",
        true,
    );
    no_history["expected"]["answers"]["intent"] = json!("no_history");
    cases.push(case(
        "no-history",
        "unsupported_history",
        vec![
            write(
                "a",
                "My name is Zorvak.",
                "assert",
                "Zorvak",
                1,
                true,
                false,
            ),
            restart(),
            no_history,
        ],
        vec![],
    ));
    cases.push(case(
        "conflict",
        "unsupported_conflict",
        vec![
            write(
                "a",
                "My name is Zorvak.",
                "assert",
                "Zorvak",
                1,
                true,
                false,
            ),
            write("b", "My name is Plimbo.", "assert", "Plimbo", 2, true, true),
            restart(),
            query("conflict", 0, found("Plimbo", 2, true), "Plimbo", true),
        ],
        vec![],
    ));
    let mut eviction = vec![write(
        "a",
        "My name is Zorvak.",
        "assert",
        "Zorvak",
        1,
        true,
        false,
    )];
    for record in 2..=9 {
        eviction.push(write(
            &format!("b{record}"),
            "My name is Plimbo.",
            "assert",
            "Plimbo",
            record,
            true,
            record == 2,
        ));
    }
    eviction.push(restart());
    let mut evicted = query("evicted", 1, json!({"kind":"evicted"}), "Unknown", true);
    evicted["expected"]["answers"]["intent"] = json!("evicted");
    eviction.push(evicted);
    cases.push(case(
        "evicted",
        "unsupported_eviction_capacity8",
        eviction,
        vec!["b9"],
    ));
    let mut wrong = query(
        "wrong-relation",
        0,
        json!({"kind":"absent"}),
        "Unknown",
        true,
    );
    wrong["expected"]["action"]["relation"] = json!("home");
    cases.push(case(
        "wrong-relation-control",
        "wrong_annotation_sentinel",
        vec![wrong],
        vec![],
    ));
    cases.push(case(
        "failed-turn-reload",
        "execution_failure_sentinel",
        vec![json!({
            "kind":"user","id":"error","text":"x".repeat(5000),
            "expected":{"action":{"kind":"unresolved"},"memory":{"kind":"unresolved"},
                        "answers":{"intent":"unresolved","accepted":["Unknown."]},
                        "intervention_answers":null}
        })],
        vec!["error"],
    ));
    let input = base.join("script.json");
    let script = json!({"schema":"uor-r4.grounded-script/1","cases":cases});
    fs::write(&input, serde_json::to_vec(&script).expect("frozen script")).expect("case file");
    let out = base.join("script-report");
    invoke(
        env!("CARGO_BIN_EXE_grounded-session"),
        &[
            "evaluate".into(),
            pair("session", baseline),
            pair("cases", &input),
            pair("out", &out),
        ],
    );
    report_output::verify(&out).expect("sealed script evaluation");
    let report: Value = serde_json::from_slice(&fs::read(out.join("report.json")).expect("report"))
        .expect("report JSON");
    assert_eq!(
        report["inputs"]["cases_sha256"],
        sha256_hex(&fs::read(&input).expect("input"))
    );
    assert_eq!(
        fs::read(out.join("cases.json")).expect("frozen retained cases"),
        fs::read(&input).expect("input")
    );
    assert_eq!(report["scores"]["action"], json!({"pass":53,"of":55}));
    assert_eq!(report["scores"]["memory"], json!({"pass":54,"of":55}));
    assert_eq!(
        report["scores"]["frozen_complete_answer_membership"],
        json!({"pass":0,"of":22})
    );
    assert_eq!(
        report["scores"]["intervention_complete_answer_membership"],
        json!({"pass":0,"of":1})
    );
    assert_eq!(report["scores"]["joint"], json!({"pass":33,"of":55}));
    let case = |id: &str| {
        report["cases"]
            .as_array()
            .expect("cases")
            .iter()
            .find(|case| case["id"] == id)
            .expect("case")
    };
    for id in [
        "temporal-continuous",
        "temporal-clean",
        "temporal-no-read-clean",
        "selected-update-disabled",
        "scope-entity-return",
        "evicted",
    ] {
        assert_eq!(case(id)["reload"]["pass"], true, "{id}");
    }
    assert_eq!(case("failed-turn-reload")["reload"]["pass"], false);
    assert_eq!(case("failed-turn-reload")["reload"]["failed_pairs"], 1);
    assert_eq!(
        case("absent")["rows"][0]["outcome"],
        case("wrong-relation-control")["rows"][0]["outcome"]
    );
    assert_eq!(case("absent")["rows"][0]["scores"]["action"], true);
    assert_eq!(
        case("wrong-relation-control")["rows"][0]["scores"]["action"],
        false
    );
    for row in &case("temporal-no-read-clean")["rows"]
        .as_array()
        .expect("rows")[5..]
    {
        assert_eq!(row["scores"]["memory"], true);
        assert_eq!(row["outcome"]["recall"]["kind"], "disabled");
    }
    assert_eq!(
        case("no-history")["rows"][1]["outcome"]["memory"]["read"],
        "NoHistory"
    );
    assert_eq!(
        case("evicted")["rows"][9]["outcome"]["memory"]["read"],
        "Evicted"
    );
    let invalid = base.join("script-invalid.json");
    let mut invalid_script = script.clone();
    invalid_script["cases"][0]["baseline_commit"] = json!(1);
    fs::write(
        &invalid,
        serde_json::to_vec(&invalid_script).expect("invalid"),
    )
    .expect("invalid input");
    let invalid_out = base.join("script-precondition-refusal");
    let refused = Command::new(env!("CARGO_BIN_EXE_grounded-session"))
        .args([
            "evaluate".into(),
            pair("session", baseline),
            pair("cases", &invalid),
            pair("out", &invalid_out),
        ])
        .output()
        .expect("refusal CLI");
    assert!(!refused.status.success());
    report_output::verify(&invalid_out).expect("sealed failed attempt");
    assert!(invalid_out.join("error.json").is_file());
    let duplicate_out = Command::new(env!("CARGO_BIN_EXE_grounded-session"))
        .args([
            "evaluate".into(),
            pair("session", baseline),
            pair("cases", &input),
            pair("out", &out),
        ])
        .output()
        .expect("exclusive report CLI");
    assert!(!duplicate_out.status.success());
    report_output::verify(&out).expect("original report intact");
}
