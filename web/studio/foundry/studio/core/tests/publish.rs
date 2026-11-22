//! Byte identity of the Publish rules on the corpus `tools/corpus_publish.py` writes to
//! `model/publish.json`, and PrismPM's vendored composer on the committed archive vectors.

use hologram_forge_core::holo::archive::{compose_application, validate_application, ApplicationArchiveInput, ArchiveProvenance};
use hologram_forge_core::{holoPath, publishDecision, publishPreimage, sourceManifest, Decision};
use serde_json::Value;

fn corpus() -> Vec<Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/publish.json");
    let text = std::fs::read_to_string(path).expect("model/publish.json; run python3 tools/corpus.py");
    serde_json::from_str(&text).expect("corpus is JSON")
}

fn decision_name(d: &Decision) -> &'static str {
    match d {
        Decision::Accept => "Accept",
        Decision::Refuse => "Refuse",
    }
}

fn s(v: &Value) -> String {
    v.as_str().unwrap_or("").to_owned()
}

#[test]
fn publish_bytes_and_decisions_match_the_rule_on_every_case() {
    let cases = corpus();
    assert!(cases.len() >= 12, "corpus has {} cases", cases.len());
    for case in &cases {
        let name = s(&case["name"]);
        let got = publishPreimage(s(&case["application"]), s(&case["version"]), s(&case["closure"]), s(&case["attestation"]));
        assert_eq!(got, s(&case["preimage"]), "model document bytes differ on case {name}");
        let i = &case["sourceInputs"];
        let manifest = sourceManifest(s(&i["leanManifest"]), s(&i["coverage"]), s(&i["kernel"]), s(&i["modelId"]), s(&i["semanticId"]), s(&i["sourceId"]));
        assert_eq!(manifest, s(&case["sourceManifest"]), "source manifest bytes differ on case {name}");
        let d = publishDecision(s(&case["closure"]), s(&case["built"]), case["attestation"].as_str().map(str::to_owned));
        assert_eq!(decision_name(&d), s(&case["decision"]), "publish decision differs on case {name}");
        assert_eq!(holoPath("blake3:".to_owned() + &"a1".repeat(32)), s(&case["holoPath"]), "holo path differs on case {name}");
    }
    println!("publish corpus: {} cases byte identical", cases.len());
}

fn fixture(name: &str) -> Vec<u8> {
    let path = format!("{}/../model/objects/publish/{}", env!("CARGO_MANIFEST_DIR"), name);
    std::fs::read(&path).unwrap_or_else(|_| panic!("fixture {path}"))
}

/// The archive the browser produced, committed as a vector: PrismPM's validator accepts it, its
/// address is the recorded one, its directory is the recorded one, and one flipped byte is refused.
#[test]
fn the_committed_archive_is_what_prismpm_accepts() {
    let bytes = fixture("hello.holo");
    validate_application(&bytes).expect("PrismPM accepts the committed archive");
    let record: Value = serde_json::from_slice(&fixture("hello.json")).expect("record is JSON");
    assert_eq!(hologram::space::address_bytes(&bytes).to_string(), s(&record["identities"]["archive_kappa"]), "archive address");
    assert_eq!(bytes.len() as u64, record["byteLength"].as_u64().unwrap(), "archive length");
    let mut flipped = bytes.clone();
    let middle = flipped.len() / 2;
    flipped[middle] ^= 1;
    assert!(validate_application(&flipped).is_err(), "a flipped byte must be refused");
}

/// Composing from the recorded inputs reproduces the committed archive byte for byte: the composer
/// is deterministic and the browser used the same inputs the record names.
#[test]
fn composing_the_recorded_inputs_reproduces_the_archive() {
    let record: Value = serde_json::from_slice(&fixture("hello.json")).expect("record is JSON");
    let provenance: ArchiveProvenance = serde_json::from_value(record["provenance"].clone()).expect("provenance");
    let holo = compose_application(&ApplicationArchiveInput {
        application_name: s(&record["application"]),
        guest_wasm: fixture("guest.wasm"),
        view_bundle: fixture("view.holoview"),
        model_document: fixture("model.json"),
        source_manifest: fixture("source-manifest.json"),
        provenance,
    })
    .expect("compose");
    // The browser's address of the archive it composed and kept; the same bytes, or a different address.
    assert_eq!(holo.identities.archive_kappa, s(&record["identities"]["archive_kappa"]), "the browser composed the same archive");
    assert_eq!(String::from_utf8_lossy(&holo.directory), s(&record["directory"]), "directory");
    let path = format!("{}/../model/objects/publish/hello.holo", env!("CARGO_MANIFEST_DIR"));
    if std::env::var("PUBLISH_WRITE").is_ok() && !std::path::Path::new(&path).exists() {
        std::fs::write(&path, &holo.bytes).expect("write hello.holo");
        println!("wrote {path}");
    }
    assert_eq!(holo.bytes, fixture("hello.holo"), "the recorded inputs compose to the committed archive");
    println!("composed {} bytes, archive {}", holo.bytes.len(), holo.identities.archive_kappa);
}
