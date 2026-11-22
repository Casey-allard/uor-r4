//! Byte identity of the generated core against the host's rule, on the fixed corpus
//! `tools/corpus.py` writes to `model/corpus.json`.

use hologram_forge_core::{headOf, networkDecision, previewPath, restoreDecision, snapshotPreimage, Capabilities, Decision, Entry, Grant, Project, Ref};
use serde_json::Value;

fn corpus() -> Vec<Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/corpus.json");
    let text = std::fs::read_to_string(path).expect("model/corpus.json; run python3 tools/corpus.py");
    serde_json::from_str(&text).expect("corpus is JSON")
}

fn decision_name(d: &Decision) -> &'static str {
    match d {
        Decision::Accept => "Accept",
        Decision::Refuse => "Refuse",
    }
}

#[test]
fn preimage_and_decisions_match_the_rule_on_every_case() {
    let cases = corpus();
    assert!(cases.len() >= 16, "corpus has {} cases", cases.len());
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let project = Project {
            label: case["label"].as_str().unwrap().to_owned(),
            entries: case["entries"]
                .as_array()
                .unwrap()
                .iter()
                .map(|e| Entry {
                    path: e["path"].as_str().unwrap().to_owned(),
                    kappa: e["kappa"].as_str().unwrap().to_owned(),
                    bytes: e["bytes"].as_u64().unwrap(),
                })
                .collect(),
        };
        let got = snapshotPreimage(&project, case["parent"].as_str().unwrap().to_owned());
        assert_eq!(got, case["preimage"].as_str().unwrap(), "preimage bytes differ on case {name}");

        let caps = Capabilities {
            endpoints: case["endpoints"].as_array().unwrap().iter().map(|e| Grant { endpoint: e.as_str().unwrap().to_owned() }).collect(),
        };
        let d = networkDecision(&caps, case["origin"].as_str().unwrap().to_owned());
        assert_eq!(decision_name(&d), case["decision"].as_str().unwrap(), "network decision differs on case {name}");

        let r = &case["restore"];
        let rd = restoreDecision(r["derived"].as_str().unwrap().to_owned(), r["expected"].as_str().unwrap().to_owned());
        assert_eq!(decision_name(&rd), r["decision"].as_str().unwrap(), "restore decision differs on case {name}");
    }
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let refs: Vec<Ref> = case["refs"].as_array().unwrap().iter().map(|r| Ref { branch: r["branch"].as_str().unwrap().to_owned(), kappa: r["kappa"].as_str().unwrap().to_owned() }).collect();
        let head = headOf(&refs, case["branch"].as_str().unwrap().to_owned());
        assert_eq!(head.as_deref(), case["head"].as_str(), "head differs on case {name}");
        assert_eq!(previewPath("blake3:".to_owned() + &"a1".repeat(32)), case["previewPath"].as_str().unwrap(), "preview path differs on case {name}");
    }
    println!("corpus: {} cases byte identical", cases.len());
}
