//! Frozen durable-memory panel (`m3-durable-memory-panel-v1`) for the M3 `m3-memory` bars.
//!
//! The pre-registration in `docs/labs/object-store-consumer-m3/README.md` names a 384-item
//! panel across six sub-panels. This target is the *instrument* that generates it deterministically,
//! writes the frozen JSONL artifact once (`--ignored write_frozen_panel`), and every normal run
//! asserts the committed artifact is byte-identical to a fresh generation and that folding it into
//! `uor_r4_object_store::Store` reproduces the pinned whole-object SHA-256 and `Manifest::version()`.
//!
//! The panel is text-only fixture data for finite CPU controls; it is not a model result. The sheet
//! stays DRAFT until a non-author FIT, an M1/M2 model selection and Casey's typed release.

use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use uor_r4_object_store::{
    bindings::PinnedTokenizer,
    durable_consumer::{DurableObjectMemoryStore, FactAction, FactReadOutcome, IdentityScope},
    Limits, Store,
};
use uor_r4_tokenizer::ByteBpeTokenizer;

const ITEMS_PER_SUBPANEL: usize = 64;
const PANEL_FILE: &str = "m3-durable-memory-panel-v1.jsonl";

/// Pinned whole-object SHA-256 of the canonical JSONL bytes (set by `write_frozen_panel`).
const FROZEN_SHA256: &str = "582e38798e66432a79fec340a9f87d6f4cb1bfd8c3d2ab594f8e6e305694f9d1";
/// Pinned `Manifest::version()` (hex) of the folded panel object.
const FROZEN_MANIFEST_VERSION: &str =
    "88c27b94526c32b0f06fed7fabeb7a2abcf0b31a2f2534517b8656240eddce70";

fn entity(i: usize) -> (&'static str, &'static str, &'static str, &'static str) {
    const PEOPLE: [(&str, &str, &str, &str); 16] = [
        ("Alice", "Lisbon", "port", "teal"),
        ("Bruno", "Porto", "canary", "amber"),
        ("Chen", "Taipei", "koi", "coral"),
        ("Diana", "Athens", "owl", "indigo"),
        ("Eitan", "Haifa", "cat", "olive"),
        ("Farah", "Cairo", "falcon", "saffron"),
        ("Gustav", "Vienna", "dachshund", "cream"),
        ("Hana", "Kyoto", "crane", "moss"),
        ("Ivan", "Tallinn", "lynx", "slate"),
        ("Juana", "Lima", "llama", "ochre"),
        ("Kofi", "Accra", "parrot", "kente"),
        ("Lena", "Berlin", "fox", "cobalt"),
        ("Mateo", "Bogota", "toucan", "ruby"),
        ("Nadia", "Tunis", "horse", "jade"),
        ("Oskar", "Oslo", "puffin", "ice"),
        ("Priya", "Jaipur", "peacock", "marigold"),
    ];
    PEOPLE[i % PEOPLE.len()]
}

fn line(role: &str, text: String) -> Value {
    json!({"role": role, "text": text})
}

fn provenance_entry(i: usize) -> Value {
    let (name, city, pet, color) = entity(i);
    let ask_direct = i % 2 == 0;
    json!({
        "id": format!("m3dm-prov-{:03}", i + 1),
        "sub_panel": "provenance",
        "scenario": "assert facts then quote them exactly",
        "turns": [
            line("user", format!("{name} lives in {city}. {name} keeps a {pet} and likes {color}.")),
            line("assistant", "Noted.".to_string()),
        ],
        "queries": [{
            "kind": "exact_quote",
            "relation": "hometown",
            "owner": name,
            "expected_value": city,
            "source_turn": 0,
            "ask": if ask_direct { format!("Where does {name} live?") } else { format!("Where did {name} say they live again?") },
        }],
    })
}

fn correction_entry(i: usize) -> Value {
    let (name, city, _, _) = entity(i);
    let (_, other_city, _, _) = entity(i + 5);
    json!({
        "id": format!("m3dm-corr-{:03}", i + 1),
        "sub_panel": "correction",
        "scenario": "assert, revise, query current and superseded value",
        "turns": [
            line("user", format!("{name} lives in {city}.")),
            line("user", format!("Actually {name} lives in {other_city}.")),
        ],
        "queries": [
            {"kind": "current_value", "relation": "hometown", "owner": name, "expected_value": other_city, "source_turn": 1},
            {"kind": "prior_value", "relation": "hometown", "owner": name, "expected_value": city, "source_turn": 0},
        ],
    })
}

fn abstention_entry(i: usize) -> Value {
    let (name, city, _, _) = entity(i);
    let (other, _, _, _) = entity(i + 7);
    let mode = if i % 2 == 0 { "unstated" } else { "retracted" };
    let (turns, q0) = if mode == "unstated" {
        (
            json!([line("user", format!("{name} lives in {city}."))]),
            json!({
                "kind": "refuse_unstated",
                "owner": other,
                "relation": "phone_number",
                "source_turn": 0
            }),
        )
    } else {
        (
            json!([
                line("user", format!("{name} lives in {city}.")),
                line("user", format!("Forget where {name} lives.")),
            ]),
            json!({
                "kind": "refuse_retracted",
                "owner": name,
                "relation": "hometown",
                "source_turn": 1
            }),
        )
    };
    json!({
        "id": format!("m3dm-abs-{:03}", i + 1),
        "sub_panel": "abstention",
        "scenario": format!("{mode} facts must be refused, never guessed"),
        "turns": turns,
        "queries": [
            q0,
            {"kind": "refuse_absent_relation", "owner": name, "relation": "shoe_size", "source_turn": 0},
        ],
    })
}

fn non_memory_entry(i: usize) -> Value {
    let (name, city, _, _) = entity(i);
    let (other, _, pet, _) = entity(i + 3);
    let distractors = 4 + (i % 5);
    let mut turns = vec![line("user", format!("{name} lives in {city}."))];
    for d in 0..distractors {
        let topic = if d % 2 == 0 {
            "the weather"
        } else {
            "the bus timetable"
        };
        turns.push(line("user", format!("Let us set that aside; {other} was talking about {topic} and their {pet} yesterday.")));
    }
    json!({
        "id": format!("m3dm-prose-{:03}", i + 1),
        "sub_panel": "non_memory_prose",
        "scenario": "distractor prose must never overwrite the stored fact",
        "turns": turns,
        "queries": [{"kind": "exact_quote", "relation": "hometown", "owner": name, "expected_value": city, "source_turn": 0}],
    })
}

fn reset_isolation_entry(i: usize) -> Value {
    let (name, city, _, _) = entity(i);
    let (other, other_city, _, _) = entity(i + 9);
    if i % 2 == 0 {
        json!({
            "id": format!("m3dm-reset-{:03}", i + 1),
            "sub_panel": "reset_isolation",
            "scenario": "explicit reset wipes the scope; post-reset queries are refused",
            "turns": [line("user", format!("{name} lives in {city}."))],
            "queries": [{"kind": "reset_then_refuse", "owner": name, "relation": "hometown"}],
        })
    } else {
        json!({
            "id": format!("m3dm-scope-{:03}", i + 1),
            "sub_panel": "reset_isolation",
            "scenario": "distinct identity scopes never leak into each other",
            "turns": [
                line("user", format!("{name} lives in {city}.")),
                line("user", format!("{other} lives in {other_city}.")),
            ],
            "queries": [{"kind": "cross_scope_refuse", "owner": other, "relation": "hometown"}],
        })
    }
}

fn restart_entry(i: usize) -> Value {
    let (name, city, _, color) = entity(i);
    json!({
        "id": format!("m3dm-restart-{:03}", i + 1),
        "sub_panel": "restart_restore",
        "scenario": "export, reload from a fresh process, continue token-for-token",
        "turns": [
            line("user", format!("{name} lives in {city} and likes {color}.")),
            line("user", "Tell me a short sentence about that.".to_string()),
        ],
        "restart": {"after_turn": 1, "continue_tokens": 32, "expected": "identical_tokens"},
    })
}

fn generate() -> Vec<Value> {
    let builders: [fn(usize) -> Value; 6] = [
        provenance_entry,
        correction_entry,
        abstention_entry,
        non_memory_entry,
        reset_isolation_entry,
        restart_entry,
    ];
    let mut out = Vec::with_capacity(builders.len() * ITEMS_PER_SUBPANEL);
    for build in builders {
        for i in 0..ITEMS_PER_SUBPANEL {
            out.push(build(i));
        }
    }
    out
}

fn canonical_bytes() -> Vec<u8> {
    let mut out = Vec::new();
    for item in generate() {
        out.extend_from_slice(
            serde_json::to_string(&item)
                .expect("panel item serializes")
                .as_bytes(),
        );
        out.push(b'\n');
    }
    out
}

fn panel_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/labs/object-store-consumer-m3")
        .join(PANEL_FILE)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fold_panel(bytes: &[u8]) -> (String, String) {
    let mut store = Store::new(Limits::default());
    let folded = store.fold(bytes).expect("panel folds");
    let version = folded
        .manifest
        .version()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let whole_sha = folded
        .manifest
        .whole_sha256()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    (whole_sha, version)
}

#[test]
#[ignore = "writes the frozen panel artifact once; run explicitly"]
fn write_frozen_panel() {
    let bytes = canonical_bytes();
    std::fs::write(panel_path(), &bytes).expect("write frozen panel");
    let (whole_sha, version) = fold_panel(&bytes);
    println!("panel bytes: {}", bytes.len());
    println!("FROZEN_SHA256: {whole_sha}");
    println!("FROZEN_MANIFEST_VERSION: {version}");
}

#[test]
fn frozen_panel_matches_generation_and_manifest_pin() {
    let generated = canonical_bytes();
    let committed = std::fs::read(panel_path()).expect("frozen panel artifact is committed");
    assert_eq!(
        committed, generated,
        "committed panel must equal a fresh deterministic generation"
    );
    assert_eq!(committed.iter().filter(|b| **b == b'\n').count(), 384);
    assert_eq!(
        sha256_hex(&committed),
        FROZEN_SHA256,
        "pinned panel SHA-256 drifted"
    );
    let (whole_sha, version) = fold_panel(&committed);
    assert_eq!(
        whole_sha, FROZEN_SHA256,
        "folded whole-object SHA-256 must match the pin"
    );
    assert_eq!(
        version, FROZEN_MANIFEST_VERSION,
        "pinned Manifest::version() drifted"
    );
    assert!(!committed.is_empty());
}

fn byte_level_ascii_tokenizer() -> ByteBpeTokenizer {
    let mut vocab = serde_json::Map::new();
    vocab.insert("Ġ".to_string(), json!(0));
    let mut next_id = 1u32;
    for b in 0x21u8..=0x7eu8 {
        vocab.insert((b as char).to_string(), json!(next_id));
        next_id += 1;
    }
    let spec = json!({
        "pre_tokenizer": {"type": "ByteLevel", "add_prefix_space": false},
        "added_tokens": [],
        "model": {
            "type": "BPE",
            "vocab": vocab,
            "merges": []
        }
    });
    ByteBpeTokenizer::from_tokenizer_json_bytes(
        serde_json::to_string(&spec)
            .expect("serialize tokenizer json")
            .as_bytes(),
    )
    .expect("valid byte-level ascii tokenizer")
}

fn load_ur4sess1_fixture() -> Vec<u8> {
    let hex_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/integer_ur4sess1_v1.hex");
    let text = std::fs::read_to_string(hex_path).expect("read integer_ur4sess1_v1.hex");
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        assert_eq!(trimmed.len() % 2, 0);
        for chunk in trimmed.as_bytes().chunks(2) {
            let s = std::str::from_utf8(chunk).expect("ascii hex");
            out.push(u8::from_str_radix(s, 16).expect("hex byte"));
        }
    }
    out
}

fn find_value_range(turn: &str, value: &str) -> std::ops::Range<u64> {
    let pos = turn
        .find(value)
        .unwrap_or_else(|| panic!("value {value:?} not found in turn {turn:?}"));
    (pos as u64)..((pos + value.len()) as u64)
}

#[test]
fn panel_executes_through_durable_consumer_and_beats_no_memory_control() {
    let tok = byte_level_ascii_tokenizer();
    let pinned = PinnedTokenizer::new(&tok).expect("pinned byte-level ascii tokenizer");
    let ur4sess1 = load_ur4sess1_fixture();
    let committed = std::fs::read_to_string(panel_path()).expect("read frozen panel");
    let items: Vec<Value> = committed
        .lines()
        .map(|l| serde_json::from_str(l).expect("valid jsonl item"))
        .collect();
    assert_eq!(items.len(), 384);

    let mut prov_hits = 0usize;
    let mut prov_no_mem_hits = 0usize;
    let mut corr_curr_hits = 0usize;
    let mut corr_prior_hits = 0usize;
    let mut corr_no_mem_hits = 0usize;
    let mut abs_refusals = 0usize;
    let mut prose_hits = 0usize;
    let mut reset_iso_hits = 0usize;
    let mut restart_hits = 0usize;

    for (idx, item) in items.iter().enumerate() {
        let sub_panel = item["sub_panel"].as_str().expect("sub_panel");
        let mut store = Store::new(Limits::default());
        let mut memory = DurableObjectMemoryStore::new();
        let no_memory = DurableObjectMemoryStore::new();
        let scope = IdentityScope::new("user-m3", "proj-m3", format!("item-{idx}")).expect("scope");

        match sub_panel {
            "provenance" => {
                let turn0 = item["turns"][0]["text"].as_str().expect("turn0");
                let q = &item["queries"][0];
                let owner = q["owner"].as_str().expect("owner");
                let relation = q["relation"].as_str().expect("relation");
                let expected = q["expected_value"].as_str().expect("expected_value");
                let range = find_value_range(turn0, expected);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner,
                        relation,
                        turn0.as_bytes(),
                        range,
                    )
                    .expect("assert provenance fact");
                if let FactReadOutcome::Quoted { value_bytes, .. } = memory
                    .read_fact(&store, &pinned, &scope, owner, relation)
                    .expect("read provenance fact")
                {
                    if value_bytes == expected.as_bytes() {
                        prov_hits += 1;
                    }
                }
                if matches!(
                    no_memory
                        .read_fact(&store, &pinned, &scope, owner, relation)
                        .expect("read no-memory"),
                    FactReadOutcome::Quoted { .. }
                ) {
                    prov_no_mem_hits += 1;
                }
            }
            "correction" => {
                let turn0 = item["turns"][0]["text"].as_str().expect("turn0");
                let turn1 = item["turns"][1]["text"].as_str().expect("turn1");
                let q_curr = &item["queries"][0];
                let q_prior = &item["queries"][1];
                let owner0 = q_prior["owner"].as_str().expect("prior owner");
                let val0 = q_prior["expected_value"].as_str().expect("prior value");
                let owner1 = q_curr["owner"].as_str().expect("curr owner");
                let val1 = q_curr["expected_value"].as_str().expect("curr value");
                let relation = q_curr["relation"].as_str().expect("relation");

                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner0,
                        relation,
                        turn0.as_bytes(),
                        find_value_range(turn0, val0),
                    )
                    .expect("assert initial fact");
                memory
                    .revise_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner1,
                        relation,
                        turn1.as_bytes(),
                        find_value_range(turn1, val1),
                    )
                    .expect("revise fact");

                if let FactReadOutcome::Quoted {
                    value_bytes,
                    previous_value_bytes,
                    action,
                    conflict,
                    ..
                } = memory
                    .read_fact(&store, &pinned, &scope, owner1, relation)
                    .expect("read revised fact")
                {
                    if value_bytes == val1.as_bytes() && action == FactAction::Revise && !conflict {
                        corr_curr_hits += 1;
                    }
                    if previous_value_bytes.as_deref() == Some(val0.as_bytes()) {
                        corr_prior_hits += 1;
                    }
                }
                if matches!(
                    no_memory
                        .read_fact(&store, &pinned, &scope, owner1, relation)
                        .expect("read no-memory"),
                    FactReadOutcome::Quoted { .. }
                ) {
                    corr_no_mem_hits += 1;
                }
            }
            "abstention" => {
                let turns = item["turns"].as_array().expect("turns");
                let turn0 = turns[0]["text"].as_str().expect("turn0");
                let (name, city, _, _) = entity(idx % ITEMS_PER_SUBPANEL);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city),
                    )
                    .expect("assert baseline fact");

                let q0 = &item["queries"][0];
                let q0_kind = q0["kind"].as_str().expect("q0 kind");
                let q0_owner = q0["owner"].as_str().expect("q0 owner");
                let q0_rel = q0["relation"].as_str().expect("q0 relation");
                if q0_kind == "refuse_retracted" {
                    memory
                        .retract_fact(&scope, q0_owner, q0_rel)
                        .expect("retract stated fact");
                    if matches!(
                        memory
                            .read_fact(&store, &pinned, &scope, q0_owner, q0_rel)
                            .expect("read retracted"),
                        FactReadOutcome::RefusedRetracted { .. }
                    ) {
                        abs_refusals += 1;
                    }
                } else if matches!(
                    memory
                        .read_fact(&store, &pinned, &scope, q0_owner, q0_rel)
                        .expect("read unstated"),
                    FactReadOutcome::RefusedUnstated
                ) {
                    abs_refusals += 1;
                }

                let q1 = &item["queries"][1];
                let q1_owner = q1["owner"].as_str().expect("q1 owner");
                let q1_rel = q1["relation"].as_str().expect("q1 relation");
                if matches!(
                    memory
                        .read_fact(&store, &pinned, &scope, q1_owner, q1_rel)
                        .expect("read absent relation"),
                    FactReadOutcome::RefusedUnstated
                ) {
                    abs_refusals += 1;
                }
            }
            "non_memory_prose" => {
                let turns = item["turns"].as_array().expect("turns");
                let turn0 = turns[0]["text"].as_str().expect("turn0");
                let q = &item["queries"][0];
                let owner = q["owner"].as_str().expect("owner");
                let relation = q["relation"].as_str().expect("relation");
                let expected = q["expected_value"].as_str().expect("expected_value");
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        owner,
                        relation,
                        turn0.as_bytes(),
                        find_value_range(turn0, expected),
                    )
                    .expect("assert fact before distractor prose");
                for distractor in &turns[1..] {
                    let text = distractor["text"].as_str().expect("distractor text");
                    memory
                        .ingest_non_memory_prose(&mut store, &pinned, &scope, text.as_bytes())
                        .expect("ingest non-memory prose");
                }
                if let FactReadOutcome::Quoted {
                    value_bytes,
                    version,
                    conflict,
                    ..
                } = memory
                    .read_fact(&store, &pinned, &scope, owner, relation)
                    .expect("read fact after distractor prose")
                {
                    if value_bytes == expected.as_bytes() && version == 1 && !conflict {
                        prose_hits += 1;
                    }
                }
            }
            "reset_isolation" => {
                let turns = item["turns"].as_array().expect("turns");
                let turn0 = turns[0]["text"].as_str().expect("turn0");
                let (name, city, _, _) = entity(idx % ITEMS_PER_SUBPANEL);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city),
                    )
                    .expect("assert primary scope fact");
                let q = &item["queries"][0];
                let kind = q["kind"].as_str().expect("kind");
                let q_owner = q["owner"].as_str().expect("owner");
                let q_rel = q["relation"].as_str().expect("relation");
                if kind == "reset_then_refuse" {
                    memory.reset_scope(&scope).expect("reset scope");
                    if matches!(
                        memory
                            .read_fact(&store, &pinned, &scope, q_owner, q_rel)
                            .expect("read after reset"),
                        FactReadOutcome::RefusedUnstated
                    ) {
                        reset_iso_hits += 1;
                    }
                } else {
                    let other_scope =
                        IdentityScope::new("user-other", "proj-m3", format!("other-{idx}"))
                            .expect("other scope");
                    let turn1 = turns[1]["text"].as_str().expect("turn1");
                    let (_, other_city, _, _) = entity((idx % ITEMS_PER_SUBPANEL) + 9);
                    memory
                        .assert_fact(
                            &mut store,
                            &pinned,
                            &other_scope,
                            q_owner,
                            q_rel,
                            turn1.as_bytes(),
                            find_value_range(turn1, other_city),
                        )
                        .expect("assert other scope fact");
                    if matches!(
                        memory
                            .read_fact(&store, &pinned, &scope, q_owner, q_rel)
                            .expect("cross-scope query"),
                        FactReadOutcome::RefusedUnstated
                    ) {
                        reset_iso_hits += 1;
                    }
                }
            }
            "restart_restore" => {
                let turn0 = item["turns"][0]["text"].as_str().expect("turn0");
                let (name, city, _, _) = entity(idx % ITEMS_PER_SUBPANEL);
                memory
                    .assert_fact(
                        &mut store,
                        &pinned,
                        &scope,
                        name,
                        "hometown",
                        turn0.as_bytes(),
                        find_value_range(turn0, city),
                    )
                    .expect("assert restart fact");
                memory
                    .set_ur4sess1_snapshot(&scope, &ur4sess1)
                    .expect("attach ur4sess1 snapshot");
                let manifest = memory
                    .export_scope(&mut store, &scope)
                    .expect("export scope");
                let mut restored = DurableObjectMemoryStore::new();
                restored
                    .import_scope(&store, &manifest)
                    .expect("import scope after restart");
                let snap_ok = restored.ur4sess1_snapshot(&scope) == Some(ur4sess1.as_slice());
                let fact_ok = matches!(
                    restored
                        .read_fact(&store, &pinned, &scope, name, "hometown")
                        .expect("read restored fact"),
                    FactReadOutcome::Quoted { ref value_bytes, .. } if value_bytes == city.as_bytes()
                );
                if snap_ok && fact_ok {
                    restart_hits += 1;
                }
            }
            other => panic!("unexpected sub_panel {other}"),
        }
    }

    assert_eq!(prov_hits, 64, "provenance sub-panel must score 64/64");
    assert_eq!(
        prov_no_mem_hits, 0,
        "no-memory control must score 0/64 on provenance (L-010)"
    );
    assert_eq!(
        corr_curr_hits, 64,
        "correction current_value must score 64/64"
    );
    assert_eq!(
        corr_prior_hits, 64,
        "correction prior_value lineage must score 64/64"
    );
    assert_eq!(
        corr_no_mem_hits, 0,
        "no-memory control must score 0/64 on correction (L-010)"
    );
    assert_eq!(
        abs_refusals, 128,
        "abstention refusal must score 128/128 across 64 dialogues"
    );
    assert_eq!(
        prose_hits, 64,
        "#1718 non-memory prose retention must score 64/64"
    );
    assert_eq!(
        reset_iso_hits, 64,
        "reset and scope isolation must score 64/64"
    );
    assert_eq!(
        restart_hits, 64,
        "UR4SESS1 restart/restore must score 64/64"
    );
}
