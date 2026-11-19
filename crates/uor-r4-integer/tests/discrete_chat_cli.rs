use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use uor_r4_integer::discrete_d11::{IntegerDiscreteModel, DISCRETE_SCHEMA, SNAPSHOT_SCHEMA};

const A: [i32; 8] = [2, 2, 0, 0, 0, 0, 0, 0];
const NEG_A: [i32; 8] = [-2, -2, 0, 0, 0, 0, 0, 0];
const B: [i32; 8] = [0, 0, 2, 0, 2, 0, 0, 0];

struct ToyArtifact {
    directory: PathBuf,
    path: PathBuf,
    bytes: Vec<u8>,
}

impl Drop for ToyArtifact {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

fn push_section(blob: &mut Vec<u8>, bytes: &[u8]) -> Value {
    let pad = (64 - blob.len() % 64) % 64;
    blob.resize(blob.len() + pad, 0);
    let offset = blob.len() as u64;
    blob.extend_from_slice(bytes);
    serde_json::json!({"offset": offset, "bytes": bytes.len()})
}

fn toy_bytes() -> Vec<u8> {
    artifact_bytes(&[A, NEG_A, B], &[0, 17, 238], 3, None)
}

fn artifact_bytes(
    roots: &[[i32; 8]],
    cells: &[u32],
    context: usize,
    tokenizer: Option<&str>,
) -> Vec<u8> {
    let stages = context.ilog(3) as usize + 1;
    let shape = serde_json::json!({
        "vocabulary": roots.len(),
        "blocks": 2,
        "context": context,
        "kept_cells": 3,
        "stages": stages,
        "read_mode": "route",
        "read_kept": 0,
        "read_sweeps": 0,
        "objective_policy": "nearest_address_margin_v1",
        "read_initialization": "zero_gates_v1",
        "seed": 7,
        "epoch": 0,
    });
    let source = serde_json::json!({
        "snapshot_schema": SNAPSHOT_SCHEMA,
        "snapshot_sha256": "00".repeat(32),
        "operator_kappa": "blake3:a0ea3a10dd10c5f4856b0d881d33d94c3293bad0e68bd07a47918e2bb82c819b",
        "composition_kappa": "blake3:f2986c8e68dcb30a9cb511a42547179a87b66ef212a87b728765d03e70e640b0",
        "kernel_sha256": "11".repeat(32),
        "source_sha256": "22".repeat(32),
        "geometry_sha256": "33".repeat(32),
    });
    let table: Vec<i32> = roots
        .iter()
        .flat_map(|root| [*root, *root])
        .flatten()
        .collect();
    let routes = vec![0u32; 2 * stages];
    let gates = vec![0i32; 2 * stages];
    let table_bytes: Vec<u8> = table.iter().flat_map(|x| x.to_le_bytes()).collect();
    let cells_bytes: Vec<u8> = cells.iter().flat_map(|x| x.to_le_bytes()).collect();
    let routes_bytes: Vec<u8> = routes.iter().flat_map(|x| x.to_le_bytes()).collect();
    let gates_bytes: Vec<u8> = gates.iter().flat_map(|x| x.to_le_bytes()).collect();
    let mut blob = Vec::new();
    let table_span = push_section(&mut blob, &table_bytes);
    let cells_span = push_section(&mut blob, &cells_bytes);
    let routes_span = push_section(&mut blob, &routes_bytes);
    let gates_span = push_section(&mut blob, &gates_bytes);
    let header = serde_json::json!({
        "schema": DISCRETE_SCHEMA,
        "tokenizer_sha256": tokenizer,
        "shape": shape,
        "matrices": [],
        "tables": [
            {"name": "discrete.table", "kind": "i32", "len": table.len(), "span": table_span},
            {"name": "discrete.cells", "kind": "u32", "len": cells.len(), "span": cells_span},
            {"name": "discrete.routes", "kind": "u32", "len": routes.len(), "span": routes_span},
            {"name": "discrete.gates", "kind": "i32", "len": gates.len(), "span": gates_span},
        ],
        "source": source,
    });
    let json = serde_json::to_vec(&header).unwrap();
    let mut out = Vec::new();
    out.extend_from_slice(b"UORLUT01");
    out.extend_from_slice(&(json.len() as u64).to_le_bytes());
    out.extend_from_slice(&json);
    let pad = (64 - out.len() % 64) % 64;
    out.resize(out.len() + pad, 0);
    out.extend_from_slice(&blob);
    out
}

fn write_artifact(tag: &str, bytes: &[u8]) -> ToyArtifact {
    let directory = std::env::temp_dir().join(format!(
        "uor-r4-discrete-chat-cli-{tag}-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).unwrap();
    let path = directory.join("model.lut");
    fs::write(&path, bytes).unwrap();
    ToyArtifact {
        directory,
        path,
        bytes: bytes.to_vec(),
    }
}

fn run_discrete(path: &Path, prompt: &str, max_tokens: usize, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--discrete")
        .arg(path)
        .arg("--prompt-ids")
        .arg(prompt)
        .arg("--max-tokens")
        .arg(max_tokens.to_string())
        .args(extra)
        .output()
        .unwrap()
}

fn successful_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "CLI failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "CLI wrote stderr on success: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn expected_ids(model: &IntegerDiscreteModel, prompt: &[u32], max_tokens: usize) -> Vec<u32> {
    let mut context = prompt.to_vec();
    let mut generated = Vec::with_capacity(max_tokens);
    for _ in 0..max_tokens {
        let prediction = model.evaluate_prefix(&context).unwrap().prediction;
        generated.push(prediction);
        if context.len() == model.shape().context as usize {
            context.remove(0);
        }
        context.push(prediction);
    }
    generated
}

#[test]
fn fresh_processes_match_library_and_each_other() {
    let artifact = write_artifact("reload", &toy_bytes());
    let model = IntegerDiscreteModel::from_bytes(&artifact.bytes).unwrap();
    let prompt = [0u32, 1, 2];
    let expected = expected_ids(&model, &prompt, 8);
    assert_eq!(expected.len(), 8);
    assert_ne!(expected[0], prompt[prompt.len() - 1]);

    let first = run_discrete(&artifact.path, "0,1,2", 8, &[]);
    let second = run_discrete(&artifact.path, "0,1,2", 8, &[]);
    let first_json = successful_json(&first);
    let second_json = successful_json(&second);

    assert_eq!(first.stdout, second.stdout);
    assert_eq!(second_json, first_json);
    assert_eq!(first_json["schema"], "uor-r4.uor-chat-discrete-parity/1");
    assert_eq!(first_json["artifact_sha256"], model.artifact_sha256());
    assert_eq!(
        first_json["generated_ids"],
        serde_json::to_value(expected).unwrap()
    );
}

#[test]
fn wrong_schema_is_refused() {
    let mut bytes = toy_bytes();
    let schema = b"uor-r4.lut-discrete/1";
    let at = bytes
        .windows(schema.len())
        .position(|window| window == schema)
        .unwrap();
    bytes[at + schema.len() - 1] = b'2';
    let artifact = write_artifact("wrong-schema", &bytes);

    let output = run_discrete(&artifact.path, "0,1,2", 8, &[]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("schema uor-r4.lut-discrete/1"));
}

#[test]
fn unsupported_modes_and_invalid_prompts_are_refused() {
    let artifact = write_artifact("refusals", &toy_bytes());

    let sampled = run_discrete(&artifact.path, "0,1,2", 8, &["--temperature", "0.5"]);
    assert!(!sampled.status.success());
    assert!(String::from_utf8_lossy(&sampled.stderr).contains("greedy"));

    let text = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--discrete")
        .arg(&artifact.path)
        .arg("--say")
        .arg("hello")
        .output()
        .unwrap();
    assert!(!text.status.success());
    assert!(String::from_utf8_lossy(&text.stderr).contains("--prompt-ids"));

    let repl = Command::new(env!("CARGO_BIN_EXE_uor-chat"))
        .arg("--discrete")
        .arg(&artifact.path)
        .output()
        .unwrap();
    assert!(!repl.status.success());
    assert!(String::from_utf8_lossy(&repl.stderr).contains("--prompt-ids"));

    let session = run_discrete(
        &artifact.path,
        "0,1,2",
        8,
        &["--save-session", "unused.session"],
    );
    assert!(!session.status.success());
    assert!(String::from_utf8_lossy(&session.stderr).contains("--stack"));

    let bad_token = run_discrete(&artifact.path, "0,1,3", 8, &[]);
    assert!(!bad_token.status.success());
    assert!(String::from_utf8_lossy(&bad_token.stderr).contains("vocabulary"));

    let over_context = run_discrete(&artifact.path, "0,1,2,0", 8, &[]);
    assert!(!over_context.status.success());
    assert!(String::from_utf8_lossy(&over_context.stderr).contains("context"));
}

#[test]
fn bound_text_uses_literal_roles_and_stops_on_the_committed_eos() {
    // Give every ID the same geometry so the lower-ID tie-break emits EOS (0).
    let pieces = [
        "<|eos|>", "<|bos|>", "<|unk|>", "a", "A", "s", "i", "t", "n", "U", "e", "r", ":", "Ġ",
        "Ċ", "h",
    ];
    let vocab: serde_json::Map<_, _> = pieces
        .iter()
        .enumerate()
        .map(|(id, surface)| (surface.to_string(), serde_json::json!(id)))
        .collect();
    let bytes = serde_json::to_vec(&serde_json::json!({
        "model":{"type":"BPE", "vocab":vocab, "merges":[]},
        "pre_tokenizer":{"type":"ByteLevel", "add_prefix_space":false},
        "added_tokens":[{"id":0,"content":"<|eos|>"},{"id":1,"content":"<|bos|>"},{"id":2,"content":"<|unk|>"}]
    })).unwrap();
    let digest = hex::encode(Sha256::digest(&bytes));
    let fixture = artifact_bytes(&[A; 16], &[0; 16], 64, Some(&digest));
    let artifact = write_artifact("text", &fixture);
    let path = artifact.directory.join("tokenizer.json");
    fs::write(&path, &bytes).unwrap();
    let run = || {
        Command::new(env!("CARGO_BIN_EXE_uor-chat"))
            .arg("--discrete")
            .arg(&artifact.path)
            .arg("--tokenizer")
            .arg(&path)
            .args(["--say", "hi", "--max-tokens", "4"])
            .output()
            .unwrap()
    };
    let record = successful_json(&run());
    let tokenizer = uor_r4_tokenizer::ByteBpeTokenizer::from_tokenizer_json_bytes(&bytes).unwrap();
    let protocol =
        uor_r4_tokenizer::dialogue::DialogueProtocol::literal_roles_v2(&tokenizer).unwrap();
    let expected = protocol
        .bind(&tokenizer)
        .unwrap()
        .encode_assistant_prefix(&[uor_r4_tokenizer::dialogue::Message {
            role: "user",
            content: "hi",
        }])
        .tokens;
    assert_eq!(expected.first(), Some(&protocol.bos_id));
    assert_eq!(record["prompt_ids"], serde_json::json!(expected));
    assert_eq!(record["generated_ids"], serde_json::json!([0]));
    assert_eq!(record["stop"], "eos");
    assert_eq!(record["text"], "");
    assert_eq!(record["tokenizer_sha256"], digest);
    let prompt = expected
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    assert_eq!(
        successful_json(&run_discrete(&artifact.path, &prompt, 4, &[]))["generated_ids"],
        serde_json::json!([0, 0, 0, 0])
    );
    // Same tokenizer contents and IDs, different file digest.
    fs::write(&path, [bytes.as_slice(), b" "].concat()).unwrap();
    assert!(String::from_utf8_lossy(&run().stderr).contains("SHA-256"));
}
