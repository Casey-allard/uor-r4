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
    fs::remove_dir_all(base).expect("clean generated test fixture");
}
