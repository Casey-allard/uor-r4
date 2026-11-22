//! Byte identity of the generated core against the daemon's rule, on the
//! fixed corpus `tools/corpus.py` writes to `model/inference.json`.

use hologram_forge_core::{
    admitPage, done, encodeCompletion, encodeDelta, encodeError, encodeFinal, encodeModels, encodeOpenRouterRequest, encodeRole, endpointReady, expertPage, fetchSource, firstTokenReady, loaderStart, memoMatches, objEntry, packRank, packed, pageAction, poolAdmit, prefetchOrder, preimages, promote, rootPreimage, route, tablePage, Admission, Completion, Manifest, Memo, Message, Obj, PageAction, Priority, Request, Route, Section, Shard, Source, Staging, Start, Tier,
};
use serde_json::Value;

fn corpus() -> Vec<Value> {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/inference.json");
    let text = std::fs::read_to_string(path).expect("model/inference.json; run python3 tools/corpus.py");
    serde_json::from_str(&text).expect("corpus is JSON")
}

fn request(case: &Value) -> Request {
    Request {
        model: "webgpu:Qwen".to_owned(),
        messages: case["messages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|m| Message {
                role: m["role"].as_str().unwrap().to_owned(),
                content: m["content"].as_str().unwrap().to_owned(),
            })
            .collect(),
        maxTokens: case["max_tokens"].as_u64(),
        seed: case["seed"].as_u64(),
        // The model spells an absent temperature as the empty string.
        temperature: case["temperature"].as_str().unwrap_or("").to_owned(),
    }
}

#[test]
fn prompt_and_params_bytes_match_the_daemon_rule_on_every_case() {
    let cases = corpus();
    assert!(cases.len() >= 16, "corpus has {} cases", cases.len());
    let mut checked = 0;
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let got = preimages(&request(case));
        assert_eq!(
            String::from_utf8(got.prompt.clone()).unwrap(),
            case["prompt"].as_str().unwrap(),
            "prompt bytes differ on case {name}"
        );
        assert_eq!(
            String::from_utf8(got.params.clone()).unwrap(),
            case["params"].as_str().unwrap(),
            "params bytes differ on case {name}"
        );
        checked += 1;
    }
    println!("corpus: {checked} cases, prompt and params bytes identical");
}

#[test]
fn memo_matches_its_own_key_and_nothing_else() {
    let memo = Memo {
        model: vec!["did:holo:sha256:model".to_owned(), "webgpu:Qwen".to_owned()],
        engineKappa: "e".to_owned(),
        promptKappa: "blake3:p".to_owned(),
        paramsKappa: "blake3:q".to_owned(),
        outputKappa: "blake3:o".to_owned(),
        receipt: "blake3:r".to_owned(),
    };
    let by_name = vec!["webgpu:Qwen".to_owned()];
    let by_kappa = vec!["did:holo:sha256:model".to_owned()];
    let other = vec!["smollm2".to_owned()];
    assert!(memoMatches(&memo, &by_name, "blake3:p", "blake3:q"));
    assert!(memoMatches(&memo, &by_kappa, "blake3:p", "blake3:q"));
    assert!(!memoMatches(&memo, &other, "blake3:p", "blake3:q"));
    assert!(!memoMatches(&memo, &by_name, "blake3:x", "blake3:q"));
    assert!(!memoMatches(&memo, &by_name, "blake3:p", "blake3:x"));
    assert!(!memoMatches(&memo, &[], "blake3:p", "blake3:q"));
}

/// The wire vectors `tools/corpus.py` writes to `model/wire.json`: every response byte the
/// endpoint can emit, restated in Python, must come out of the generated encoders identically.
#[test]
fn wire_bytes_match_the_vectors_on_every_case() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/wire.json");
    let text = std::fs::read_to_string(path).expect("model/wire.json; run python3 tools/corpus.py");
    let vectors: Vec<Value> = serde_json::from_str(&text).expect("wire vectors are JSON");
    assert!(vectors.len() >= 5, "wire has {} vectors", vectors.len());
    let s = |v: &Value| v.as_str().unwrap().to_owned();
    let mut checked = 0;
    for v in &vectors {
        let name = v["name"].as_str().unwrap();
        let c = &v["completion"];
        let completion = Completion { id: s(&c["id"]), created: s(&c["created"]), model: s(&c["model"]), text: s(&c["text"]), fingerprint: s(&c["fingerprint"]), receipt: s(&c["receipt"]) };
        let ids: Vec<String> = v["ids"].as_array().unwrap().iter().map(s).collect();
        assert_eq!(encodeCompletion(&completion), s(&v["completion_bytes"]), "completion bytes differ on {name}");
        assert_eq!(encodeRole(&completion), s(&v["role_bytes"]), "role chunk differs on {name}");
        assert_eq!(encodeDelta(&completion, s(&v["delta"])), s(&v["delta_bytes"]), "delta chunk differs on {name}");
        assert_eq!(encodeFinal(&completion), s(&v["final_bytes"]), "final chunk differs on {name}");
        assert_eq!(encodeError(s(&v["error"]["message"]), s(&v["error"]["type"])), s(&v["error_bytes"]), "error envelope differs on {name}");
        assert_eq!(encodeModels(&ids), s(&v["models_bytes"]), "models list differs on {name}");
        assert_eq!(done(), s(&v["done_bytes"]), "done frame differs on {name}");
        checked += 7;
    }
    println!("wire: {} vectors, {checked} encodings byte identical", vectors.len());
}

/// The bytes OpenRouter receives, for every corpus case, streamed and plain: the model's encoder
/// against the Python restatement, and never a key.
#[test]
fn openrouter_request_bytes_match_on_every_case() {
    let cases = corpus();
    let mut checked = 0;
    for case in &cases {
        let name = case["name"].as_str().unwrap();
        let model = case["openrouter_model"].as_str().unwrap().to_owned();
        let got_stream = encodeOpenRouterRequest(model.clone(), &request(case), true);
        let got_plain = encodeOpenRouterRequest(model, &request(case), false);
        assert_eq!(got_stream, case["openrouter_stream"].as_str().unwrap(), "streamed request bytes differ on {name}");
        assert_eq!(got_plain, case["openrouter_plain"].as_str().unwrap(), "plain request bytes differ on {name}");
        assert!(!got_stream.contains("sk-or-"), "a key in the request bytes on {name}");
        checked += 2;
    }
    println!("openrouter: {checked} request encodings byte identical");
}

/// The route table: every row, as the theorems state it.
#[test]
fn route_serves_hits_and_never_pays_without_a_key() {
    assert_eq!(route(true, false), Route::Serve);
    assert_eq!(route(false, true), Route::Local);
    assert_eq!(route(false, false), Route::NoGpu);
}

/// The endpoint readiness table: a resident model is enough; a key needs the network; nothing else is ready.
#[test]
fn endpoint_is_ready_with_a_resident_model_or_a_key_online() {
    assert!(endpointReady(true, false, false));
    assert!(endpointReady(true, true, true));
    assert!(endpointReady(false, true, true));
    assert!(!endpointReady(false, true, false));
    assert!(!endpointReady(false, false, true));
    assert!(!endpointReady(false, false, false));
}

fn text(v: &Value) -> String {
    v.as_str().unwrap_or("").to_owned()
}

/// The real edge0-8b κ object (1,550 tensors, 26,496 expert pages, derived off the wire): the
/// generated root preimage equals the Python restatement byte for byte, every object line of every
/// shard equals the generated `objEntry`, and the admission rows hold.
#[test]
fn kappa_object_root_preimage_and_object_lines_match_on_the_real_manifest() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../model/objects/");
    // The second object, Qwen3.8-Flash-Next (360 GB, 131 shards, 5,051,898 κ), carries only its manifest here;
    // its 856 MB of object lists live outside git. Its preimage is checked the same way.
    for name in ["qwen38-flash-next"] {
        let m: Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}{name}.manifest.json")).expect("manifest")).unwrap();
        let want = std::fs::read_to_string(format!("{dir}{name}.preimage.json")).expect("preimage");
        let manifest = Manifest {
            spec: text(&m["spec"]), repo: text(&m["repo"]), revision: text(&m["revision"]),
            experts: m["experts"].as_u64().unwrap(), tableRows: m["table_rows"].as_u64().unwrap(),
            shards: m["shards"].as_array().unwrap().iter().map(|sh| Shard {
                label: text(&sh["name"]), bytes: sh["bytes"].as_u64().unwrap(), sha256: text(&sh["sha256"]), kappa: text(&sh["kappa"]), objects: text(&sh["objects"]),
            }).collect(),
        };
        let got = rootPreimage(&manifest);
        assert!(got == want, "{name}: root preimage differs from the Python restatement ({} vs {} bytes)", got.len(), want.len());
        println!("kappa object {name}: preimage {} bytes identical, {} shards", got.len(), manifest.shards.len());
    }
    let m: Value = serde_json::from_str(&std::fs::read_to_string(format!("{dir}edge0-8b.manifest.json")).expect("manifest")).unwrap();
    let want = std::fs::read_to_string(format!("{dir}edge0-8b.preimage.json")).expect("preimage");
    let manifest = Manifest {
        spec: text(&m["spec"]), repo: text(&m["repo"]), revision: text(&m["revision"]),
        experts: m["experts"].as_u64().unwrap(), tableRows: m["table_rows"].as_u64().unwrap(),
        shards: m["shards"].as_array().unwrap().iter().map(|sh| Shard {
            label: text(&sh["name"]), bytes: sh["bytes"].as_u64().unwrap(), sha256: text(&sh["sha256"]), kappa: text(&sh["kappa"]), objects: text(&sh["objects"]),
        }).collect(),
    };
    let got = rootPreimage(&manifest);
    assert!(got == want, "root preimage differs from the Python restatement ({} vs {} bytes)", got.len(), want.len());
    let mut lines = 0; let mut listed = Vec::new();
    for sh in &manifest.shards {
        let body = std::fs::read_to_string(format!("{dir}edge0-8b/{}.objects.jsonl", sh.label.replace('/', "_"))).expect("object list");
        for line in body.split('\n') {
            let o: Value = serde_json::from_str(line).unwrap();
            let obj = Obj { kind: text(&o[0]), label: text(&o[1]), kappa: text(&o[2]), bytes: o[3].as_u64().unwrap() };
            assert_eq!(objEntry(&obj), line.to_owned(), "object line differs");
            listed.push(obj.kappa); lines += 1;
        }
    }
    let first = listed[0].clone();
    assert!(admitPage(&listed, &first, &first));
    assert!(!admitPage(&listed, &first, "blake3:0"));
    assert!(!admitPage(&listed, "blake3:0", "blake3:0"));
    println!("kappa object: preimage {} bytes identical, {lines} object lines identical, admit rows hold", got.len());
}

#[test]
fn expert_and_table_pages_tile_their_tensors() {
    // 4 experts in 80 bytes from offset 100: stride 20.
    for e in 0..4u64 {
        let r = expertPage(100, 80, 4, e).unwrap();
        assert_eq!((r.start, r.stop), (100 + 20 * e, 120 + 20 * e));
    }
    // rows of 2 bytes, 3 rows per page, a tensor of 20 bytes from offset 7: pages 6,6,6,2.
    let ends: Vec<(u64, u64)> = (0..4u64).map(|i| { let r = tablePage(7, 27, 2, 3, i).unwrap(); (r.start, r.stop) }).collect();
    assert_eq!(ends, vec![(7, 13), (13, 19), (19, 25), (25, 27)]);
}

#[test]
fn page_arithmetic_refuses_overflow_instead_of_wrapping() {
    assert!(expertPage(u64::MAX - 1, 80, 4, 3).is_err());
    assert!(tablePage(u64::MAX - 1, u64::MAX, 2, 3, 1).is_err());
}

/// The pool and stage tables, every row, as the theorems state them.
#[test]
fn pool_and_stage_tables_hold_on_every_row() {
    assert_eq!(pageAction(true, Staging::TrueRouting), PageAction::Bind);
    assert_eq!(pageAction(true, Staging::StagedReplace), PageAction::Bind);
    assert_eq!(pageAction(false, Staging::TrueRouting), PageAction::Fetch);
    assert_eq!(pageAction(false, Staging::StagedReplace), PageAction::Drop);
    assert_eq!(poolAdmit(true, false), Admission::Touch);
    assert_eq!(poolAdmit(true, true), Admission::Touch);
    assert_eq!(poolAdmit(false, true), Admission::Insert);
    assert_eq!(poolAdmit(false, false), Admission::EvictThenInsert);
    assert_eq!(fetchSource(true, true, true), Source::Device);
    assert_eq!(fetchSource(false, true, true), Source::Peer);
    assert_eq!(fetchSource(false, false, true), Source::Peer);
    assert_eq!(fetchSource(false, true, false), Source::Mirror);
    assert_eq!(fetchSource(false, false, false), Source::Nowhere);
    assert_eq!(prefetchOrder(true, true), Priority::First);
    assert_eq!(prefetchOrder(true, false), Priority::First);
    assert_eq!(prefetchOrder(false, true), Priority::Fill);
    assert_eq!(prefetchOrder(false, false), Priority::Skip);
}

/// Pack, Ladder and Loader, every row, as the theorems state them; promotion is monotone.
#[test]
fn pack_ladder_and_loader_tables_hold_on_every_row() {
    assert_eq!((packRank(Section::Header), packRank(Section::Tokenizer), packRank(Section::Spine), packRank(Section::Expert), packRank(Section::Table)), (0, 1, 2, 3, 4));
    assert!(packed(Section::Header) && packed(Section::Tokenizer) && packed(Section::Spine) && packed(Section::Expert) && !packed(Section::Table));
    assert!(firstTokenReady(true, true) && !firstTokenReady(true, false) && !firstTokenReady(false, true) && !firstTokenReady(false, false));
    for resident in [false, true] { for fast in [false, true] { assert_eq!(promote(Tier::Large, resident, fast), Tier::Large); } }
    assert_eq!(promote(Tier::Small, true, true), Tier::Large);
    assert_eq!(promote(Tier::Small, false, true), Tier::Small);
    assert_eq!(promote(Tier::Small, true, false), Tier::Small);
    assert_eq!(promote(Tier::Small, false, false), Tier::Small);
    assert_eq!(loaderStart(true, true), Start::Resume);
    assert_eq!(loaderStart(false, true), Start::Resume);
    assert_eq!(loaderStart(true, false), Start::Warm);
    assert_eq!(loaderStart(false, false), Start::Cold);
}
