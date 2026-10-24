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
use uor_r4_object_store::{Limits, Store};

const ITEMS_PER_SUBPANEL: usize = 64;
const PANEL_FILE: &str = "m3-durable-memory-panel-v1.jsonl";

/// Pinned whole-object SHA-256 of the canonical JSONL bytes (set by `write_frozen_panel`).
const FROZEN_SHA256: &str = "01cb94df8a969450f76b00c2191548cbc0eb0fd823d9e929a1c4086853b440ad";
/// Pinned `Manifest::version()` (hex) of the folded panel object.
const FROZEN_MANIFEST_VERSION: &str =
    "0b0616fa124b689f4ca95cb049237ed974a1bee0c3d0f395f6d3a2a9eb48fbee";

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
    let (other, other_city, _, _) = entity(i + 5);
    json!({
        "id": format!("m3dm-corr-{:03}", i + 1),
        "sub_panel": "correction",
        "scenario": "assert, revise, query current and superseded value",
        "turns": [
            line("user", format!("{name} lives in {city}.")),
            line("user", format!("Actually {other} lives in {other_city}.")),
        ],
        "queries": [
            {"kind": "current_value", "relation": "hometown", "owner": other, "expected_value": other_city, "source_turn": 1},
            {"kind": "prior_value", "relation": "hometown", "owner": name, "expected_value": city, "source_turn": 0},
        ],
    })
}

fn abstention_entry(i: usize) -> Value {
    let (name, city, _, _) = entity(i);
    let (other, _, _, _) = entity(i + 7);
    let mode = if i % 2 == 0 { "unstated" } else { "retracted" };
    let turns = if mode == "unstated" {
        json!([line("user", format!("{name} lives in {city}."))])
    } else {
        json!([
            line("user", format!("{name} lives in {city}.")),
            line("user", format!("Forget {other}'s phone number.")),
        ])
    };
    json!({
        "id": format!("m3dm-abs-{:03}", i + 1),
        "sub_panel": "abstention",
        "scenario": format!("{mode} facts must be refused, never guessed"),
        "turns": turns,
        "queries": [
            {"kind": if mode == "unstated" { "refuse_unstated" } else { "refuse_retracted" },
             "owner": other, "relation": "phone_number", "source_turn": 0},
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
